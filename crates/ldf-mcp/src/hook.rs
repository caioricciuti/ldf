//! `ldf-mcp hook claude-code`: a Claude Code command hook that re-encodes
//! JSON datasets in Bash stdout as LDF before the model sees them.
//!
//! One command serves three events, told apart by `hook_event_name`:
//!
//! * `PostToolUse` on `Bash`: when stdout is one complete JSON dataset and
//!   LDF is shorter (and the selector agrees for the session's model), the
//!   hook prints `hookSpecificOutput.updatedToolOutput`, the original
//!   `tool_response` with `stdout` replaced and every other field kept. In
//!   every other case it prints nothing, which leaves the original output:
//!   Claude Code runs sibling hooks in parallel on the original output and
//!   the last rewrite wins, so an identity rewrite could undo another hook's
//!   redaction.
//! * `SessionStart` records the session's model for the selector (the only
//!   event whose input names it) and, after `/compact` or `/clear`, forgets
//!   that the primer was sent.
//! * `PostModelSwitch` records the new model.
//!
//! Claude Code only fires `PostToolUse` for a command that succeeded; a
//! non-zero exit goes to `PostToolUseFailure`, which cannot replace output.
//! Output Claude Code saved to a file because it was too large to show
//! inline arrives here incomplete, so it is left alone. The hook never
//! fails the tool call: every error, and every panic, ends in exit 0 with
//! no output, which is the original result. It makes no network calls and
//! starts no processes.
//!
//! Per-session state is two small files under `claude-code/sessions` in
//! [`crate::stats::ldf_state_dir`]: `<id>.model` and `<id>.primed`.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use serde_json::{Value as Json, json};

use crate::rewrite::{self, Kept, PRIMER, Selector};
use crate::stats::Recorder;

/// Hook input larger than this is not read (Claude Code shows at most
/// 128 000 characters of Bash output inline).
pub const MAX_INPUT: u64 = 16 * 1024 * 1024;

/// Session files older than this are removed at the next `SessionStart`.
const SESSION_TTL: Duration = Duration::from_secs(30 * 24 * 3600);

#[derive(Debug, Clone)]
pub struct Config {
    pub min_bytes: usize,
    /// `--model`: used instead of the model Claude Code reports.
    pub model: Option<String>,
    pub evidence: Option<Vec<ldf_select::Evidence>>,
    /// Where `<id>.model` and `<id>.primed` live; `None` keeps no state.
    pub sessions: Option<PathBuf>,
}

/// What one invocation decided. `output` is what to print on stdout.
#[derive(Debug, Default)]
pub struct Outcome {
    pub output: Option<String>,
    /// Set for `PostToolUse` only: (in bytes, out bytes, reason kept).
    pub event: Option<(usize, usize, Option<Kept>)>,
    /// The model the selector was asked about, for the statistics.
    pub model: Option<String>,
}

/// Runs one hook invocation on `input`, the JSON Claude Code wrote to
/// stdin.
pub fn handle(input: &[u8], cfg: &Config) -> Outcome {
    let Ok(Json::Object(ev)) = serde_json::from_slice::<Json>(input) else {
        return Outcome {
            output: None,
            event: Some((input.len(), input.len(), Some(Kept::Hook("bad_input")))),
            model: None,
        };
    };
    let session = ev
        .get("session_id")
        .and_then(Json::as_str)
        .filter(|s| valid_id(s));
    let files = cfg
        .sessions
        .as_deref()
        .zip(session)
        .map(|(dir, id)| SessionFiles::new(dir, id));
    match ev.get("hook_event_name").and_then(Json::as_str) {
        Some("SessionStart") => {
            if let Some(f) = &files {
                f.start(
                    ev.get("model").and_then(Json::as_str),
                    ev.get("source").and_then(Json::as_str),
                );
            }
            Outcome::default()
        }
        Some("PostModelSwitch") => {
            if let (Some(f), Some(m)) = (&files, ev.get("to_model").and_then(Json::as_str)) {
                f.set_model(m);
            }
            Outcome::default()
        }
        Some("PostToolUse") if ev.get("tool_name").and_then(Json::as_str) == Some("Bash") => {
            post_bash(&ev, cfg, files.as_ref())
        }
        _ => Outcome::default(),
    }
}

fn post_bash(
    ev: &serde_json::Map<String, Json>,
    cfg: &Config,
    files: Option<&SessionFiles>,
) -> Outcome {
    let Some(resp) = ev.get("tool_response").and_then(Json::as_object) else {
        return kept(0, "bad_input");
    };
    let Some(stdout) = resp.get("stdout").and_then(Json::as_str) else {
        return kept(0, "bad_input");
    };
    let n = stdout.len();
    let set = |k: &str| {
        resp.get(k)
            .is_some_and(|v| !v.is_null() && v != &json!(false))
    };
    if set("persistedOutputPath") {
        return kept(n, "persisted");
    }
    if set("interrupted") {
        return kept(n, "interrupted");
    }
    if set("isImage") {
        return kept(n, "image");
    }
    if set("backgroundTaskId") {
        return kept(n, "background");
    }
    // A non-zero exit Claude Code read as success (grep finding nothing).
    if set("returnCodeInterpretation") {
        return kept(n, "exit_code");
    }
    let model = cfg
        .model
        .clone()
        .or_else(|| files.and_then(SessionFiles::model))
        .or_else(|| {
            // `claude -p` sends SessionStart without a model; the
            // transcript names it on every assistant line.
            let m = ev
                .get("transcript_path")
                .and_then(Json::as_str)
                .and_then(|p| transcript_model(Path::new(p)))?;
            if let Some(f) = files {
                f.set_model(&m);
            }
            Some(m)
        });
    let sel = model
        .as_deref()
        .map(|m| Selector::new(m, cfg.evidence.clone()));
    let ldf = match rewrite::convert(stdout, cfg.min_bytes, sel.as_ref()) {
        Ok(ldf) => ldf,
        Err(why) => {
            return Outcome {
                output: None,
                event: Some((n, n, Some(why))),
                model,
            };
        }
    };
    let mut note = format!(
        "ldf-mcp: this command printed {n} bytes of JSON; it is shown re-encoded as LDF \
         ({} bytes, lossless). The command itself still prints JSON, so parse it as JSON in scripts.",
        ldf.len()
    );
    let primer = files.is_none_or(|f| !f.primed.exists());
    if primer {
        note.push_str("\n\n");
        note.push_str(PRIMER);
    }
    // The note is context the model reads too: it has to pay for itself.
    let out_bytes = ldf.len() + note.len();
    if out_bytes >= n {
        return Outcome {
            output: None,
            event: Some((n, n, Some(Kept::NotShorter))),
            model,
        };
    }
    if primer && let Some(f) = files {
        f.mark_primed();
    }
    let mut replaced = resp.clone();
    replaced.insert("stdout".into(), Json::String(ldf));
    let out = json!({
        "hookSpecificOutput": {
            "hookEventName": "PostToolUse",
            "updatedToolOutput": replaced,
            "additionalContext": note,
        }
    });
    Outcome {
        output: Some(out.to_string()),
        event: Some((n, out_bytes, None)),
        model,
    }
}

fn kept(n: usize, code: &'static str) -> Outcome {
    Outcome {
        output: None,
        event: Some((n, n, Some(Kept::Hook(code)))),
        model: None,
    }
}

/// Claude Code session ids are UUIDs; anything else is not used in a path.
fn valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// How much of the end of a transcript [`transcript_model`] reads.
const TRANSCRIPT_TAIL: u64 = 256 * 1024;

/// The model of the last assistant message in the last
/// [`TRANSCRIPT_TAIL`] bytes of a Claude Code transcript. Only a string
/// that looks like a model id is taken; nothing else is kept.
fn transcript_model(path: &Path) -> Option<String> {
    use std::io::{Seek, SeekFrom};
    if !path.is_absolute() || path.extension().is_none_or(|e| e != "jsonl") {
        return None;
    }
    let mut f = fs::File::open(path).ok()?;
    let len = f.metadata().ok()?.len();
    f.seek(SeekFrom::Start(len.saturating_sub(TRANSCRIPT_TAIL)))
        .ok()?;
    let mut buf = Vec::new();
    f.take(TRANSCRIPT_TAIL).read_to_end(&mut buf).ok()?;
    let text = String::from_utf8_lossy(&buf);
    const KEY: &str = "\"model\":\"";
    text.rmatch_indices(KEY).find_map(|(i, _)| {
        let rest = &text[i + KEY.len()..];
        let id = &rest[..rest.find('"')?];
        let ok = (3..=80).contains(&id.len())
            && id.starts_with(|c: char| c.is_ascii_alphanumeric())
            && id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.[]:/".contains(&b));
        ok.then(|| base_model(id).to_string())
    })
}

/// `claude-opus-5-5[1m]` and `claude-opus-5-5` share their evidence.
fn base_model(m: &str) -> &str {
    m.split('[').next().unwrap_or(m).trim()
}

struct SessionFiles {
    dir: PathBuf,
    model: PathBuf,
    primed: PathBuf,
}

impl SessionFiles {
    fn new(dir: &Path, id: &str) -> Self {
        SessionFiles {
            dir: dir.to_path_buf(),
            model: dir.join(format!("{id}.model")),
            primed: dir.join(format!("{id}.primed")),
        }
    }

    fn start(&self, model: Option<&str>, source: Option<&str>) {
        prune(&self.dir);
        if let Some(m) = model {
            self.set_model(m);
        }
        // The primer left the context with the old conversation.
        if matches!(source, Some("compact" | "clear")) {
            let _ = fs::remove_file(&self.primed);
        }
    }

    fn set_model(&self, m: &str) {
        let m = base_model(m);
        if !m.is_empty() && fs::create_dir_all(&self.dir).is_ok() {
            let _ = fs::write(&self.model, m);
        }
    }

    fn model(&self) -> Option<String> {
        fs::read_to_string(&self.model)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Records that this context has the primer. Two parallel calls may
    /// both send it; that costs tokens once, never correctness.
    fn mark_primed(&self) {
        if fs::create_dir_all(&self.dir).is_ok() {
            let _ = fs::write(&self.primed, b"");
        }
    }
}

fn prune(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let now = SystemTime::now();
    for e in entries.flatten() {
        let old = e
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| now.duration_since(t).ok())
            .is_some_and(|age| age > SESSION_TTL);
        if old {
            let _ = fs::remove_file(e.path());
        }
    }
}

/// The whole command: reads stdin, prints the reply, records one event for
/// `PostToolUse` in the statistics file `stats` (`None`: record nothing).
/// Never returns an error; the exit code is always 0.
pub fn run(cfg: &Config, stats: Option<PathBuf>) {
    let start = Instant::now();
    let mut input = Vec::new();
    let read = std::io::stdin()
        .lock()
        .take(MAX_INPUT + 1)
        .read_to_end(&mut input);
    let outcome = match read {
        Ok(_) if input.len() as u64 > MAX_INPUT => Outcome {
            output: None,
            event: Some((input.len(), input.len(), Some(Kept::Hook("too_large")))),
            model: None,
        },
        Ok(_) => handle(&input, cfg),
        Err(_) => return,
    };
    if let Some(out) = &outcome.output {
        use std::io::Write;
        let mut so = std::io::stdout().lock();
        // A reply that cannot be written whole is no reply: Claude Code
        // treats invalid JSON as a non-blocking error and keeps the original.
        let _ = so.write_all(out.as_bytes()).and_then(|()| so.flush());
    }
    if let Some((inb, outb, why)) = outcome.event {
        let micros = start.elapsed().as_micros() as u64;
        Recorder::new(stats, "hook", "claude-code", outcome.model.as_deref())
            .record_text("Bash", inb, outb, why, micros, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(dir: Option<&Path>) -> Config {
        Config {
            min_bytes: 0,
            model: None,
            evidence: None,
            sessions: dir.map(Path::to_path_buf),
        }
    }

    fn rows(n: usize) -> String {
        let r: Vec<String> = (0..n)
            .map(|i| format!(r#"{{"id":{i},"name":"user {i}","active":{}}}"#, i % 2 == 0))
            .collect();
        format!("[{}]", r.join(","))
    }

    fn bash(stdout: &str, extra: Json) -> Vec<u8> {
        let mut resp = json!({"stdout": stdout, "stderr": "warn: x", "interrupted": false, "isImage": false, "noOutputExpected": false});
        for (k, v) in extra.as_object().unwrap() {
            resp[k] = v.clone();
        }
        json!({"session_id": "s1", "hook_event_name": "PostToolUse", "tool_name": "Bash",
               "tool_input": {"command": "gh api x"}, "tool_response": resp, "tool_use_id": "toolu_1"})
        .to_string()
        .into_bytes()
    }

    #[test]
    fn json_stdout_is_replaced_and_everything_else_kept() {
        let text = rows(80);
        let o = handle(&bash(&text, json!({"gitOperation": {"k": 1}})), &cfg(None));
        let out: Json = serde_json::from_str(o.output.as_deref().unwrap()).unwrap();
        let h = &out["hookSpecificOutput"];
        assert_eq!(h["hookEventName"], "PostToolUse");
        let r = &h["updatedToolOutput"];
        assert_eq!(r["stderr"], "warn: x");
        assert_eq!(r["interrupted"], false);
        assert_eq!(r["noOutputExpected"], false);
        assert_eq!(r["gitOperation"], json!({"k": 1}));
        let ldf = r["stdout"].as_str().unwrap();
        assert!(ldf.starts_with("@records id:i,name:s,active:b\n"), "{ldf}");
        assert_eq!(ldf::decode(ldf).unwrap().len(), 80);
        let note = h["additionalContext"].as_str().unwrap();
        assert!(note.contains("still prints JSON"));
        // The note is charged to the conversion.
        let (i, out_b, why) = o.event.unwrap();
        assert_eq!((i, out_b, why), (text.len(), ldf.len() + note.len(), None));
        assert!(out_b < i);
    }

    #[test]
    fn a_conversion_that_does_not_pay_for_its_note_is_not_made() {
        // 20 rows: LDF alone is shorter, LDF plus the primer is not.
        let text = rows(20);
        assert!(rewrite::convert_text(&text, 0).unwrap().len() < text.len());
        let o = handle(&bash(&text, json!({})), &cfg(None));
        assert!(o.output.is_none());
        assert_eq!(o.event.unwrap().2, Some(Kept::NotShorter));
    }

    #[test]
    fn leaves_everything_that_is_not_a_complete_dataset() {
        let c = cfg(None);
        let mixed = format!("fetching...\n{}", rows(5));
        let trailing = format!("{}\ndone", rows(5));
        for s in [
            "",
            "plain text\nline 2",
            "[1,2,3]",
            "{\"ok\":true}",
            "[{\"a\":1},",
            &mixed,
            &trailing,
        ] {
            let o = handle(&bash(s, json!({})), &c);
            assert!(o.output.is_none(), "{s:?}");
            assert!(o.event.unwrap().2.is_some());
        }
    }

    #[test]
    fn flags_that_change_meaning_pass_through() {
        let c = cfg(None);
        let text = rows(80);
        for (extra, code) in [
            (
                json!({"persistedOutputPath": "/x", "persistedOutputSize": 99_999}),
                "persisted",
            ),
            (json!({"interrupted": true}), "interrupted"),
            (json!({"isImage": true}), "image"),
            (json!({"backgroundTaskId": "b1"}), "background"),
            (
                json!({"returnCodeInterpretation": "No matches found"}),
                "exit_code",
            ),
        ] {
            let o = handle(&bash(&text, extra), &c);
            assert!(o.output.is_none(), "{code}");
            assert_eq!(o.event.unwrap().2.unwrap().code(), code);
        }
    }

    #[test]
    fn other_tools_events_and_garbage_print_nothing() {
        let c = cfg(None);
        let read = json!({"hook_event_name": "PostToolUse", "tool_name": "Read", "tool_response": {"stdout": rows(9)}});
        let o = handle(read.to_string().as_bytes(), &c);
        assert!(o.output.is_none() && o.event.is_none());
        let o = handle(b"not json", &c);
        assert!(o.output.is_none());
        assert_eq!(o.event.unwrap().2.unwrap().code(), "bad_input");
        let o = handle(
            br#"{"hook_event_name":"PostToolUse","tool_name":"Bash","tool_response":"x"}"#,
            &c,
        );
        assert!(o.output.is_none());
    }

    #[test]
    fn session_model_drives_the_selector_and_primer_goes_once() {
        let dir = std::env::temp_dir().join(format!("ldf-hook-unit-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let c = cfg(Some(&dir));
        let start = |model: &str, source: &str| {
            let ev = json!({"session_id": "s1", "hook_event_name": "SessionStart", "model": model, "source": source});
            assert!(handle(ev.to_string().as_bytes(), &c).output.is_none());
        };
        start("gpt-6.1-sol", "startup");
        // No evidence for this model: JSON.
        let o = handle(&bash(&rows(80), json!({})), &c);
        assert_eq!(o.event.unwrap().2.unwrap().code(), "selector");
        let switch = json!({"session_id": "s1", "hook_event_name": "PostModelSwitch", "to_model": "claude-opus-5-5[1m]"});
        handle(switch.to_string().as_bytes(), &c);
        assert_eq!(
            fs::read_to_string(dir.join("s1.model")).unwrap(),
            "claude-opus-5-5"
        );
        let note = |o: Outcome| {
            let out: Json = serde_json::from_str(&o.output.unwrap()).unwrap();
            out["hookSpecificOutput"]["additionalContext"]
                .as_str()
                .unwrap()
                .to_string()
        };
        assert!(note(handle(&bash(&rows(80), json!({})), &c)).contains("@records order_id"));
        assert!(!note(handle(&bash(&rows(80), json!({})), &c)).contains("@records order_id"));
        start("claude-opus-5-5", "compact");
        assert!(note(handle(&bash(&rows(80), json!({})), &c)).contains("@records order_id"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn model_from_the_transcript_tail() {
        let dir = std::env::temp_dir().join(format!("ldf-hook-tr-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let t = dir.join("t.jsonl");
        let pad = "x".repeat(300 * 1024);
        fs::write(
            &t,
            format!(
                "{{\"message\":{{\"model\":\"claude-opus-5-5\"}}}}\n{{\"pad\":\"{pad}\"}}\n\
                 {{\"message\":{{\"model\":\"claude-haiku-5-5[1m]\"}}}}\n\
                 {{\"message\":{{\"model\":\"<synthetic>\"}}}}\n"
            ),
        )
        .unwrap();
        assert_eq!(transcript_model(&t).as_deref(), Some("claude-haiku-5-5"));
        assert_eq!(transcript_model(Path::new("t.jsonl")), None, "relative");
        assert_eq!(transcript_model(&dir.join("missing.jsonl")), None);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn session_ids_never_become_paths() {
        assert!(valid_id("9f1c2d3e-aaaa-bbbb-cccc-0123456789ab"));
        assert!(!valid_id("../../etc/passwd"));
        assert!(!valid_id("a/b"));
        assert!(!valid_id(""));
    }
}
