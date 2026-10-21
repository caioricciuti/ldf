//! Local statistics: one JSON line per tool response, appended to a file on
//! this machine, and the summary `ldf-mcp stats` prints from it.
//!
//! The Claude Code hook (`ldf-mcp hook claude-code`) writes the same lines
//! with mode `hook`, server `claude-code` and tool `Bash`; its `convert_us`
//! is the whole invocation after process start, reading the input included.
//!
//! A line holds the time, the mode and server label, the tool name, the
//! outcome and its reason code, byte counts and timings. It never holds a
//! response body, tool arguments, environment values or anything else the
//! data could leak through. Nothing is sent anywhere.
//!
//! Location: `LDF_STATS_FILE` when set, else `events.jsonl` in
//! [`ldf_state_dir`] (`ldf` under `XDG_STATE_HOME`, `~/.local/state` or, on
//! Windows, `LOCALAPPDATA`).
//! `LDF_STATS=0` or `--no-stats` turns recording off. At 16 MiB the file is
//! renamed to `events.1.jsonl` (replacing the previous one) and a new one
//! starts, so at most twice that is kept.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value as Json, json};

use crate::rewrite::{Block, Kept, Rewritten};

/// Size at which the events file is rotated.
pub const ROTATE_BYTES: u64 = 16 * 1024 * 1024;

/// The events file this process would use, or `None` when recording is off
/// or no home directory is known.
pub fn default_path() -> Option<PathBuf> {
    if std::env::var("LDF_STATS").is_ok_and(|v| v == "0") {
        return None;
    }
    if let Some(p) = std::env::var_os("LDF_STATS_FILE").filter(|p| !p.is_empty()) {
        return Some(PathBuf::from(p));
    }
    ldf_state_dir().map(|d| d.join("events.jsonl"))
}

/// This tool's state directory: `LDF_STATE_DIR` when set, else `ldf` under
/// `XDG_STATE_HOME`, `~/.local/state` or, on Windows, `LOCALAPPDATA`. It
/// holds the statistics and the record of what `install` changed.
pub fn ldf_state_dir() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("LDF_STATE_DIR").filter(|p| !p.is_empty()) {
        return Some(PathBuf::from(p));
    }
    state_dir().map(|d| d.join("ldf"))
}

fn state_dir() -> Option<PathBuf> {
    let var = |k: &str| {
        std::env::var_os(k)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    if cfg!(windows) {
        return var("LOCALAPPDATA");
    }
    var("XDG_STATE_HOME").or_else(|| var("HOME").map(|h| h.join(".local").join("state")))
}

/// The file after rotation.
pub fn rotated(path: &Path) -> PathBuf {
    path.with_extension("1.jsonl")
}

/// Appends events for one server process. Failures to write are reported
/// once on stderr and otherwise ignored: statistics never affect a reply.
#[derive(Debug)]
pub struct Recorder {
    path: Option<PathBuf>,
    mode: &'static str,
    server: String,
    model: Option<String>,
    warned: AtomicBool,
}

impl Recorder {
    pub fn new(
        path: Option<PathBuf>,
        mode: &'static str,
        server: &str,
        model: Option<&str>,
    ) -> Self {
        Recorder {
            path,
            mode,
            server: server.to_string(),
            model: model.map(str::to_string),
            warned: AtomicBool::new(false),
        }
    }

    /// A recorder that records nothing.
    pub fn off() -> Self {
        Recorder::new(None, "", "", None)
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// One response of this crate's own servers: a single text block.
    pub fn record_text(
        &self,
        tool: &str,
        in_bytes: usize,
        out_bytes: usize,
        kept: Option<Kept>,
        micros: u64,
        passthrough: bool,
    ) {
        let done = Rewritten {
            blocks: vec![Block {
                in_bytes,
                out_bytes,
                kept,
                rows: None,
            }],
            micros,
        };
        self.record(tool, &done, passthrough, None);
    }

    /// One `tools/call` response. `call_ms` is the time from request to
    /// reply where the caller knows it.
    pub fn record(&self, tool: &str, done: &Rewritten, passthrough: bool, call_ms: Option<u64>) {
        let Some(path) = &self.path else { return };
        let in_bytes: usize = done.blocks.iter().map(|b| b.in_bytes).sum();
        // Passthrough sends the original whatever the blocks say.
        let out_bytes: usize = if passthrough {
            in_bytes
        } else {
            done.blocks.iter().map(|b| b.out_bytes).sum()
        };
        let (outcome, reason) = if passthrough {
            ("json", Some("passthrough"))
        } else if done.converted() {
            ("ldf", None)
        } else {
            (
                "json",
                Some(
                    done.blocks
                        .iter()
                        .find_map(|b| b.kept.as_ref().map(|k| k.code()))
                        .unwrap_or("no_text"),
                ),
            )
        };
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let mut ev = json!({
            "ts": ts,
            "mode": self.mode,
            "server": self.server,
            "tool": tool,
            "outcome": outcome,
            "in_bytes": in_bytes,
            "out_bytes": out_bytes,
            "blocks": done.blocks.len(),
            "convert_us": done.micros,
        });
        if let Some(r) = reason {
            ev["reason"] = json!(r);
        }
        if !passthrough && done.minified() {
            ev["minified"] = json!(true);
        }
        if !passthrough && let Some((shown, total)) = done.blocks.iter().find_map(|b| b.rows) {
            ev["rows_shown"] = json!(shown);
            ev["rows_total"] = json!(total);
        }
        if let Some(m) = &self.model {
            ev["model"] = json!(m);
        }
        if let Some(ms) = call_ms {
            ev["call_ms"] = json!(ms);
        }
        if let Err(e) = append(path, &ev)
            && !self.warned.swap(true, Ordering::Relaxed)
        {
            eprintln!("ldf-mcp: statistics not written to {}: {e}", path.display());
        }
    }
}

fn append(path: &Path, ev: &Json) -> std::io::Result<()> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        fs::create_dir_all(dir)?;
    }
    if fs::metadata(path).is_ok_and(|m| m.len() >= ROTATE_BYTES) {
        fs::rename(path, rotated(path))?;
    }
    let mut line = ev.to_string();
    line.push('\n');
    // One write per line with O_APPEND, so lines from several processes
    // sharing the file do not interleave.
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?
        .write_all(line.as_bytes())
}

/// Totals over a set of events.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Summary {
    pub responses: u64,
    pub converted: u64,
    /// Responses left as JSON, by reason code.
    pub kept: BTreeMap<String, u64>,
    pub in_bytes: u64,
    pub out_bytes: u64,
    /// Bytes in and out over converted responses only.
    pub converted_in: u64,
    pub converted_out: u64,
    /// Conversion time per response, microseconds, sorted.
    pub convert_us: Vec<u64>,
    /// (server, tool) -> (responses, converted, in bytes, out bytes)
    pub by_tool: BTreeMap<(String, String), (u64, u64, u64, u64)>,
    pub first_ts: Option<u64>,
    pub last_ts: Option<u64>,
    /// Lines that did not parse as events.
    pub bad_lines: u64,
}

/// Reads the rotated file and then the current one; events older than
/// `since` (Unix seconds) are skipped. A missing file is an empty summary.
pub fn summarize(path: &Path, since: Option<u64>) -> std::io::Result<Summary> {
    let mut s = Summary::default();
    for p in [rotated(path), path.to_path_buf()] {
        let f = match fs::File::open(&p) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e),
        };
        for line in BufReader::new(f).split(b'\n') {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            match serde_json::from_slice::<Json>(&line) {
                Ok(ev) => add(&mut s, &ev, since),
                Err(_) => s.bad_lines += 1,
            }
        }
    }
    s.convert_us.sort_unstable();
    Ok(s)
}

fn add(s: &mut Summary, ev: &Json, since: Option<u64>) {
    let n = |k: &str| ev.get(k).and_then(Json::as_u64).unwrap_or(0);
    let ts = n("ts");
    if since.is_some_and(|t| ts < t) {
        return;
    }
    let converted = ev.get("outcome").and_then(Json::as_str) == Some("ldf");
    let (inb, outb) = (n("in_bytes"), n("out_bytes"));
    s.responses += 1;
    s.in_bytes += inb;
    s.out_bytes += outb;
    if converted {
        s.converted += 1;
        s.converted_in += inb;
        s.converted_out += outb;
    } else {
        let why = ev.get("reason").and_then(Json::as_str).unwrap_or("unknown");
        *s.kept.entry(why.to_string()).or_default() += 1;
    }
    s.convert_us.push(n("convert_us"));
    let key = (
        ev.get("server")
            .and_then(Json::as_str)
            .unwrap_or("?")
            .to_string(),
        ev.get("tool")
            .and_then(Json::as_str)
            .unwrap_or("?")
            .to_string(),
    );
    let t = s.by_tool.entry(key).or_default();
    t.0 += 1;
    t.1 += u64::from(converted);
    t.2 += inb;
    t.3 += outb;
    s.first_ts = Some(s.first_ts.map_or(ts, |f| f.min(ts)));
    s.last_ts = Some(s.last_ts.map_or(ts, |l| l.max(ts)));
}

/// Rough tokens for a byte count: four bytes per token. Not a tokenizer.
pub fn estimate_tokens(bytes: u64) -> u64 {
    bytes.div_ceil(4)
}

fn percentile(sorted: &[u64], p: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let i = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[i]
}

fn human(bytes: u64) -> String {
    match bytes {
        b if b >= 1 << 20 => format!("{:.1} MB", b as f64 / (1 << 20) as f64),
        b if b >= 1 << 10 => format!("{:.1} KB", b as f64 / (1 << 10) as f64),
        b => format!("{b} B"),
    }
}

fn reason_text(code: &str) -> &str {
    match code {
        "small" => "under --min-bytes",
        "not_json" => "not JSON",
        "not_dataset" => "JSON but not rows of objects",
        "not_shorter" => "LDF would not be shorter",
        "selector" => "selector kept JSON for the model",
        "is_error" => "error result, never touched",
        "out_of_domain" => "a number LDF's value model cannot hold exactly",
        "not_lossless" => "columnar frame would not decode to the same result",
        "passthrough" => "--passthrough",
        "no_text" => "no text content",
        "persisted" => "Bash output too large to show inline; Claude Code saved it to a file",
        "exit_code" => "non-zero exit Claude Code read as success",
        "interrupted" => "Bash command interrupted",
        "image" => "Bash output is an image",
        "background" => "Bash command moved to the background",
        "raw" => "LDF_RAW=1 in the command",
        "subagent" => "subagent whose model the hook cannot tell",
        "bad_input" => "hook input not understood",
        "too_large" => "hook input over 16 MiB",
        other => other,
    }
}

/// The report `ldf-mcp stats` prints.
pub fn render(s: &Summary, path: &Path) -> String {
    let mut o = String::new();
    let pct = |a: u64, b: u64| {
        if b == 0 {
            0.0
        } else {
            100.0 * a as f64 / b as f64
        }
    };
    o.push_str(&format!(
        "ldf-mcp stats, from {} (local only)\n\n",
        path.display()
    ));
    if s.responses == 0 {
        o.push_str("No tool responses recorded yet.\n");
        return o;
    }
    o.push_str(&format!("tool responses     {}\n", s.responses));
    o.push_str(&format!(
        "  sent as LDF      {} ({:.0}%)\n",
        s.converted,
        pct(s.converted, s.responses)
    ));
    o.push_str(&format!(
        "  left as JSON     {}\n",
        s.responses - s.converted
    ));
    let mut kept: Vec<(&String, &u64)> = s.kept.iter().collect();
    kept.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (code, n) in kept {
        o.push_str(&format!("    {n:>6}  {}\n", reason_text(code)));
    }
    let saved = s.in_bytes.saturating_sub(s.out_bytes);
    o.push_str(&format!(
        "\nbytes              {} in, {} out, {} saved ({:.0}%)\n",
        human(s.in_bytes),
        human(s.out_bytes),
        human(saved),
        pct(saved, s.in_bytes)
    ));
    if s.converted > 0 {
        o.push_str(&format!(
            "  converted only   {} -> {} ({:.0}% of the JSON)\n",
            human(s.converted_in),
            human(s.converted_out),
            pct(s.converted_out, s.converted_in)
        ));
    }
    o.push_str(&format!(
        "tokens saved       about {} (estimate: bytes / 4, not a tokenizer)\n",
        estimate_tokens(saved)
    ));
    o.push_str(&format!(
        "conversion time    p50 {:.1} ms, p95 {:.1} ms, max {:.1} ms per response\n",
        percentile(&s.convert_us, 0.5) as f64 / 1000.0,
        percentile(&s.convert_us, 0.95) as f64 / 1000.0,
        s.convert_us.last().copied().unwrap_or(0) as f64 / 1000.0
    ));
    let mut tools: Vec<_> = s.by_tool.iter().collect();
    tools.sort_by_key(|(_, t)| std::cmp::Reverse(t.2.saturating_sub(t.3)));
    o.push_str("\nby server and tool (most bytes saved first)\n");
    for ((server, tool), (n, c, i, out)) in tools.iter().take(15) {
        o.push_str(&format!(
            "  {server} / {tool}: {n} responses, {c} as LDF, {} -> {}\n",
            human(*i),
            human(*out)
        ));
    }
    o.push_str(
        "\nThese are serialization savings: bytes the agent did not have to put in the model's\n\
         context. What a model was billed depends on its tokenizer and on caching; compare with\n\
         the agent's own usage counters (site/src/content/docs/docs/mcp.md, \"Measuring\").\n",
    );
    if s.bad_lines > 0 {
        o.push_str(&format!("({} unreadable lines skipped)\n", s.bad_lines));
    }
    o
}

/// The summary as JSON, for scripts.
pub fn to_json(s: &Summary) -> Json {
    let saved = s.in_bytes.saturating_sub(s.out_bytes);
    json!({
        "responses": s.responses,
        "converted": s.converted,
        "kept": s.kept,
        "in_bytes": s.in_bytes,
        "out_bytes": s.out_bytes,
        "saved_bytes": saved,
        "converted_in_bytes": s.converted_in,
        "converted_out_bytes": s.converted_out,
        "estimated_tokens_saved": estimate_tokens(saved),
        "token_estimate": "bytes / 4, not a tokenizer",
        "convert_ms": {
            "p50": percentile(&s.convert_us, 0.5) as f64 / 1000.0,
            "p95": percentile(&s.convert_us, 0.95) as f64 / 1000.0,
            "max": s.convert_us.last().copied().unwrap_or(0) as f64 / 1000.0,
        },
        "by_tool": s.by_tool.iter().map(|((server, tool), (n, c, i, o))| json!({
            "server": server, "tool": tool, "responses": n, "converted": c, "in_bytes": i, "out_bytes": o
        })).collect::<Vec<_>>(),
        "first_ts": s.first_ts,
        "last_ts": s.last_ts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn done(blocks: Vec<Block>) -> Rewritten {
        Rewritten {
            blocks,
            micros: 1500,
        }
    }

    #[test]
    fn records_and_summarizes_without_bodies() {
        let dir = std::env::temp_dir().join(format!("ldf-stats-test-{}", std::process::id()));
        let path = dir.join("events.jsonl");
        let _ = fs::remove_dir_all(&dir);
        let rec = Recorder::new(Some(path.clone()), "proxy", "srv", Some("m"));
        let ldf = done(vec![Block {
            in_bytes: 1000,
            out_bytes: 400,
            kept: None,
            rows: None,
        }]);
        let small = done(vec![Block {
            in_bytes: 50,
            out_bytes: 50,
            kept: Some(Kept::Small),
            rows: None,
        }]);
        let sel = done(vec![Block {
            in_bytes: 900,
            out_bytes: 900,
            kept: Some(Kept::Selector("secret-looking reason".into())),
            rows: None,
        }]);
        rec.record("query", &ldf, false, Some(12));
        rec.record("query", &small, false, None);
        rec.record("other", &sel, false, None);
        rec.record("query", &ldf, true, None);
        let text = fs::read_to_string(&path).unwrap();
        assert_eq!(text.lines().count(), 4);
        assert!(
            !text.contains("secret-looking"),
            "only reason codes are stored"
        );
        let first: Json = serde_json::from_str(text.lines().next().unwrap()).unwrap();
        assert_eq!(first["outcome"], "ldf");
        assert_eq!(first["call_ms"], 12);
        assert_eq!(first["model"], "m");

        let s = summarize(&path, None).unwrap();
        assert_eq!(s.responses, 4);
        assert_eq!(s.converted, 1);
        assert_eq!(s.kept.get("small"), Some(&1));
        assert_eq!(s.kept.get("selector"), Some(&1));
        assert_eq!(s.kept.get("passthrough"), Some(&1));
        assert_eq!((s.in_bytes, s.out_bytes), (2950, 2350));
        assert_eq!((s.converted_in, s.converted_out), (1000, 400));
        let report = render(&s, &path);
        assert!(report.contains("sent as LDF      1 (25%)"), "{report}");
        assert!(report.contains("not a tokenizer"));
        assert_eq!(summarize(&path, Some(u64::MAX)).unwrap().responses, 0);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rotation_keeps_one_old_file_and_summaries_read_both() {
        let dir = std::env::temp_dir().join(format!("ldf-stats-rot-{}", std::process::id()));
        let path = dir.join("events.jsonl");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        // A full file of one valid event padded with blank lines.
        let mut big = String::from(r#"{"ts":1,"outcome":"ldf","in_bytes":10,"out_bytes":5}"#);
        big.push('\n');
        big.push_str(&"\n".repeat(ROTATE_BYTES as usize));
        fs::write(&path, big).unwrap();
        let rec = Recorder::new(Some(path.clone()), "files", "f", None);
        rec.record("read_rows", &done(vec![]), false, None);
        assert!(rotated(&path).exists());
        assert!(fs::metadata(&path).unwrap().len() < 1000);
        let s = summarize(&path, None).unwrap();
        assert_eq!(s.responses, 2);
        assert_eq!(s.kept.get("no_text"), Some(&1));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn an_unwritable_path_does_not_fail_the_caller() {
        let rec = Recorder::new(
            Some(PathBuf::from("/dev/null/x/events.jsonl")),
            "proxy",
            "s",
            None,
        );
        rec.record("t", &done(vec![]), false, None);
        rec.record("t", &done(vec![]), false, None);
    }
}
