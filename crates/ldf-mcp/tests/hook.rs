//! `ldf-mcp hook claude-code` as Claude Code runs it: one process per
//! event, the event JSON on stdin, the reply (or nothing) on stdout, exit
//! status 0 every time. Payloads follow the shape Claude Code 2.1.304 sends
//! for the Bash tool (`tool_response` carries more than the four fields the
//! documentation shows, and all of them must survive).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

use serde_json::{Value as Json, json};

fn exe() -> &'static str {
    env!("CARGO_BIN_EXE_ldf-mcp")
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ldf-mcp-hook-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

struct Run {
    status: i32,
    stdout: String,
    stderr: String,
}

fn hook(state: &Path, args: &[&str], input: &[u8]) -> Run {
    let mut child = Command::new(exe())
        .args(["hook", "claude-code"])
        .args(args)
        .env("LDF_STATE_DIR", state)
        .env("LDF_STATS_FILE", state.join("events.jsonl"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // A writer that stops early (the hook refusing a huge input) is fine.
    let _ = child.stdin.take().unwrap().write_all(input);
    let out = child.wait_with_output().unwrap();
    Run {
        status: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8(out.stdout).unwrap(),
        stderr: String::from_utf8(out.stderr).unwrap(),
    }
}

const SESSION: &str = "3f0e7a52-1c2b-4d5e-8f90-a1b2c3d4e5f6";

fn post_bash(command: &str, stdout: &str, extra: Json) -> Vec<u8> {
    let mut resp = json!({
        "stdout": stdout,
        "stderr": "",
        "interrupted": false,
        "isImage": false,
        "noOutputExpected": false,
        "dangerouslyDisableSandbox": false
    });
    for (k, v) in extra.as_object().unwrap() {
        resp[k] = v.clone();
    }
    json!({
        "session_id": SESSION,
        "prompt_id": "550e8400-e29b-41d4-a716-446655440000",
        "transcript_path": "/tmp/x/transcript.jsonl",
        "cwd": "/tmp/x",
        "permission_mode": "auto",
        "effort": {"level": "high"},
        "hook_event_name": "PostToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": command, "description": "fetch rows", "timeout": 120000},
        "tool_response": resp,
        "tool_use_id": "toolu_01ABC",
        "duration_ms": 41
    })
    .to_string()
    .into_bytes()
}

fn session_start(model: &str, source: &str) -> Vec<u8> {
    json!({"session_id": SESSION, "hook_event_name": "SessionStart", "source": source,
           "model": model, "cwd": "/tmp/x", "transcript_path": "/tmp/x/t.jsonl"})
    .to_string()
    .into_bytes()
}

fn users(n: usize) -> String {
    let rows: Vec<Json> = (0..n)
        .map(|i| {
            json!({"id": i, "login": format!("user-{i}"), "site_admin": i % 7 == 0,
                   "score": (i as f64) * 1.25, "team": {"id": i % 4, "name": format!("team {}", i % 4)}})
        })
        .collect();
    serde_json::to_string_pretty(&rows).unwrap()
}

fn events(state: &Path) -> Vec<Json> {
    std::fs::read_to_string(state.join("events.jsonl"))
        .unwrap_or_default()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

/// The reply's replacement output, or `None` when the hook printed nothing.
fn replaced(r: &Run) -> Option<Json> {
    assert_eq!(r.status, 0, "stderr: {}", r.stderr);
    if r.stdout.is_empty() {
        return None;
    }
    let v: Json = serde_json::from_str(&r.stdout).expect("one JSON object");
    assert_eq!(v["hookSpecificOutput"]["hookEventName"], "PostToolUse");
    Some(v["hookSpecificOutput"].clone())
}

#[test]
fn pretty_json_array_becomes_ldf_and_round_trips() {
    let state = temp_dir("array");
    let text = users(120);
    let input = post_bash(
        "gh api /orgs/x/members",
        &text,
        json!({"stderr": "note: paginated"}),
    );
    let h = replaced(&hook(&state, &[], &input)).expect("converted");
    let r = &h["updatedToolOutput"];
    assert_eq!(r["stderr"], "note: paginated");
    assert_eq!(r["noOutputExpected"], false);
    assert_eq!(r["dangerouslyDisableSandbox"], false);
    let ldf = r["stdout"].as_str().unwrap();
    assert!(
        ldf.len() * 3 < text.len(),
        "{} vs {}",
        ldf.len(),
        text.len()
    );
    // Lossless at the semantic level: same records as the JSON.
    let back = ldf::decode(ldf).unwrap();
    let orig: Vec<ldf_data::Value> = serde_json::from_str(&text).unwrap();
    assert_eq!(back, orig);
    let ctx = h["additionalContext"].as_str().unwrap();
    assert!(ctx.contains("still prints JSON"));
    assert!(ctx.contains("Language Data Format"), "primer on first use");

    let ev = events(&state);
    assert_eq!(ev.len(), 1);
    assert_eq!(ev[0]["mode"], "hook");
    assert_eq!(ev[0]["tool"], "Bash");
    assert_eq!(ev[0]["outcome"], "ldf");
    assert_eq!(ev[0]["in_bytes"], text.len());
    assert_eq!(ev[0]["out_bytes"], ldf.len() + ctx.len(), "the note counts");
    let line = std::fs::read_to_string(state.join("events.jsonl")).unwrap();
    for leak in ["gh api", "/orgs/x", "user-1", "paginated", SESSION] {
        assert!(!line.contains(leak), "{leak} in stats");
    }
}

#[test]
fn nested_and_sparse_datasets_convert() {
    let state = temp_dir("shapes");
    let orders: Vec<Json> = (0..120)
        .map(|i| json!({"order": format!("O-{i}"), "items": [{"sku": "A", "qty": i}, {"sku": "B", "qty": 1}]}))
        .collect();
    let wrapped = json!({"total_count": 120, "items": orders}).to_string();
    let sparse: Vec<Json> = (0..150)
        .map(|i| {
            let mut o = json!({"id": i, "kind": "evt"});
            if i % 9 == 0 {
                o["error"] = json!({"code": 500 + i});
            }
            if i % 13 == 0 {
                o["retry"] = json!(true);
            }
            o
        })
        .collect();
    let sparse = serde_json::to_string(&sparse).unwrap();
    for text in [wrapped, sparse] {
        let h = replaced(&hook(
            &state,
            &[],
            &post_bash("curl -s x", &text, json!({})),
        ))
        .expect("converted");
        let ldf = h["updatedToolOutput"]["stdout"].as_str().unwrap();
        let back = ldf::decode(ldf).unwrap();
        let orig: ldf_data::Value = serde_json::from_str(&text).unwrap();
        let orig = match orig {
            ldf_data::Value::Array(a) => a,
            o => vec![o],
        };
        // Sparse keys move to `~`: same members, order may differ
        // (semantic-unordered, spec section 7).
        assert_eq!(back.len(), orig.len());
        for (a, b) in back.iter().zip(&orig) {
            let (a, b) = (a.as_object().unwrap(), b.as_object().unwrap());
            assert_eq!(a.len(), b.len());
            for (k, v) in b.iter() {
                assert_eq!(a.iter().find(|(k2, _)| k2 == k).map(|(_, v)| v), Some(v));
            }
        }
    }
}

#[test]
fn everything_else_prints_nothing_and_says_why() {
    let state = temp_dir("kept");
    let big = users(50);
    let cases: Vec<(Vec<u8>, &str)> = vec![
        (
            post_bash(
                "ls -la",
                "total 8\ndrwxr-xr-x  3 me  staff  96 Oct 20 a\n",
                json!({}),
            ),
            "small",
        ),
        (
            post_bash("cargo test", &"test x ... ok\n".repeat(200), json!({})),
            "not_json",
        ),
        (
            post_bash(
                "./fetch.sh",
                &format!("Fetching page 1...\n{big}"),
                json!({}),
            ),
            "not_json",
        ),
        (
            post_bash("./fetch.sh", &format!("{big}\nDone in 2s"), json!({})),
            "not_json",
        ),
        (
            post_bash("cat broken.json", &big[..big.len() - 3], json!({})),
            "not_json",
        ),
        (post_bash("true", "", json!({})), "small"),
        (
            post_bash(
                "jq .",
                &serde_json::to_string(&(0..2000).collect::<Vec<_>>()).unwrap(),
                json!({}),
            ),
            "not_dataset",
        ),
        (
            post_bash(
                "cat big.json",
                &big,
                json!({"persistedOutputPath": "/tmp/x/tool-results/b.txt", "persistedOutputSize": 812_345}),
            ),
            "persisted",
        ),
        (
            post_bash(
                "grep -c x f",
                &big,
                json!({"returnCodeInterpretation": "No matches found"}),
            ),
            "exit_code",
        ),
        (
            post_bash("sleep 9; cat f", &big, json!({"interrupted": true})),
            "interrupted",
        ),
        (
            post_bash("npm run dev", &big, json!({"backgroundTaskId": "bash_1"})),
            "background",
        ),
    ];
    for (input, code) in &cases {
        let r = hook(&state, &[], input);
        assert!(replaced(&r).is_none(), "{code}: {}", r.stdout);
    }
    let ev = events(&state);
    let got: Vec<&str> = ev.iter().map(|e| e["reason"].as_str().unwrap()).collect();
    let want: Vec<&str> = cases.iter().map(|c| c.1).collect();
    assert_eq!(got, want);
    assert!(ev.iter().all(|e| e["outcome"] == "json"));
}

#[test]
fn failures_other_tools_and_bad_input_never_fail_the_call() {
    let state = temp_dir("failure");
    let failure = json!({"session_id": SESSION, "hook_event_name": "PostToolUseFailure", "tool_name": "Bash",
        "tool_input": {"command": "false"}, "error": format!("Exit code 1\n{}", users(30)), "tool_use_id": "t"});
    let read = json!({"session_id": SESSION, "hook_event_name": "PostToolUse", "tool_name": "Read",
        "tool_input": {"file_path": "/x.json"}, "tool_response": {"type": "text", "file": {"content": users(30)}}});
    let cases: [Vec<u8>; 6] = [
        failure.to_string().into_bytes(),
        read.to_string().into_bytes(),
        b"".to_vec(),
        b"{not json".to_vec(),
        b"[1,2,3]".to_vec(),
        vec![0xff, 0xfe, 0x00, 0x7b],
    ];
    for input in &cases {
        let r = hook(&state, &[], input);
        assert_eq!((r.status, r.stdout.as_str()), (0, ""), "{}", r.stderr);
    }
    // A misconfigured hook says so with exit 1, which Claude Code shows to
    // the user only; exit 2 would put the message in front of the model.
    let r = hook(
        &state,
        &["--min-bytes", "x"],
        &post_bash("x", "[]", json!({})),
    );
    assert_eq!(r.status, 1);
    assert!(r.stdout.is_empty());
}

#[test]
fn oversized_input_is_not_read() {
    let state = temp_dir("huge");
    let row = r#"{"id":1,"name":"padding padding padding padding"},"#;
    let mut text = String::from("[");
    while text.len() < 17 * 1024 * 1024 {
        text.push_str(row);
    }
    text.push_str(r#"{"id":2,"name":"end"}]"#);
    let r = hook(&state, &[], &post_bash("cat huge.json", &text, json!({})));
    assert!(replaced(&r).is_none());
    assert_eq!(events(&state)[0]["reason"], "too_large");
}

#[test]
fn session_model_selects_and_primer_is_sent_once_per_context() {
    let state = temp_dir("session");
    let text = users(80);
    let input = post_bash("gh api x", &text, json!({}));
    // One passing row for Opus on flat data, so the test checks the hook's
    // wiring, not whatever evidence the release ships.
    let evidence = state.join("evidence.json");
    std::fs::write(
        &evidence,
        json!([{"model": "claude-opus-5-5", "run": "t/flat", "dataset": "flat", "records": 200,
            "traits": [], "family": "ldf", "thresholds": [0.0, 0.25, 0.5, 0.75, 1.0],
            "hoist_thresholds": [0.0, 0.25, 0.5, 0.75, 1.0], "format": "ldf", "n": 300, "k": 300,
            "json_k": 300, "wins": 0, "losses": 0, "p": 1.0, "tokens_ratio": 0.6}])
        .to_string(),
    )
    .unwrap();
    let ev = evidence.to_str().unwrap();
    let args = ["--evidence", ev];
    assert!(
        hook(&state, &args, &session_start("gpt-6.1-sol", "startup"))
            .stdout
            .is_empty()
    );
    assert!(
        replaced(&hook(&state, &args, &input)).is_none(),
        "no evidence for GPT"
    );
    assert_eq!(events(&state)[0]["reason"], "selector");

    let switch = json!({"session_id": SESSION, "hook_event_name": "PostModelSwitch",
                        "from_model": "gpt-6.1-sol", "to_model": "claude-opus-5-5"});
    assert!(
        hook(&state, &args, switch.to_string().as_bytes())
            .stdout
            .is_empty()
    );
    let primer = |r: &Run| {
        replaced(r).unwrap()["additionalContext"]
            .as_str()
            .unwrap()
            .contains("Language Data Format")
    };
    assert!(primer(&hook(&state, &args, &input)));
    assert_eq!(events(&state)[1]["model"], "claude-opus-5-5");
    assert!(!primer(&hook(&state, &args, &input)));
    assert!(!primer(&hook(&state, &args, &input)));
    // After compaction the primer is gone from the context: send it again.
    hook(&state, &args, &session_start("claude-opus-5-5", "compact"));
    assert!(primer(&hook(&state, &args, &input)));
    // --model overrides what the session reported.
    let r = hook(
        &state,
        &["--evidence", ev, "--model", "gpt-6.1-sol"],
        &input,
    );
    assert!(replaced(&r).is_none());
    assert!(
        !r.stderr.contains("user-"),
        "no data on stderr: {}",
        r.stderr
    );
}

#[test]
fn overhead_is_small() {
    let state = temp_dir("speed");
    let input = post_bash("gh api x", &users(200), json!({}));
    let t = Instant::now();
    for _ in 0..5 {
        assert!(replaced(&hook(&state, &[], &input)).is_some());
    }
    let per = t.elapsed() / 5;
    // Process start included; a debug build, so a loose bound.
    assert!(per.as_millis() < 2000, "{per:?}");
    let us: Vec<u64> = events(&state)
        .iter()
        .map(|e| e["convert_us"].as_u64().unwrap())
        .collect();
    assert!(us.iter().all(|&u| u > 0));
}
