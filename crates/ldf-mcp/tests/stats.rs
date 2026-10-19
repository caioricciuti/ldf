//! Local statistics end to end: the proxy and the files server append one
//! line per tool response to `LDF_STATS_FILE`, with byte counts and reason
//! codes and no response content, and `ldf-mcp stats --json` sums them.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{Value as Json, json};

fn exe() -> &'static str {
    env!("CARGO_BIN_EXE_ldf-mcp")
}

fn temp_file(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ldf-mcp-stats-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir.join("events.jsonl")
}

/// Runs `args` with `LDF_STATS_FILE`, sends `requests` one at a time and
/// returns the replies.
fn session(stats: &Path, args: &[&str], requests: &[Json]) -> Vec<Json> {
    let mut child = Command::new(exe())
        .args(args)
        .env("LDF_STATS_FILE", stats)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut replies = Vec::new();
    for r in requests {
        stdin.write_all(format!("{r}\n").as_bytes()).unwrap();
        stdin.flush().unwrap();
        if r.get("id").is_some() {
            let mut line = String::new();
            stdout.read_line(&mut line).unwrap();
            replies.push(serde_json::from_str(&line).unwrap());
        }
    }
    drop(stdin);
    assert!(child.wait().unwrap().success());
    replies
}

fn call(id: u64, tool: &str, args: Json) -> Json {
    json!({"jsonrpc": "2.0", "id": id, "method": "tools/call", "params": {"name": tool, "arguments": args}})
}

fn events(path: &Path) -> Vec<Json> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

#[test]
fn the_proxy_records_each_response_and_stats_sums_them() {
    let path = temp_file("proxy");
    let init = json!({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}}});
    session(
        &path,
        &[
            "proxy",
            "--quiet",
            "--name",
            "rows",
            "--",
            exe(),
            "fake-server",
        ],
        &[
            init.clone(),
            call(1, "query", json!({"rows": 100})),
            call(2, "query", json!({"rows": 2})),
        ],
    );
    session(
        &path,
        &[
            "proxy",
            "--quiet",
            "--passthrough",
            "--name",
            "rows",
            "--",
            exe(),
            "fake-server",
        ],
        &[init, call(1, "query", json!({"rows": 100}))],
    );
    let ev = events(&path);
    assert_eq!(ev.len(), 3, "initialize is not a tool response: {ev:?}");
    assert_eq!(ev[0]["outcome"], "ldf");
    assert_eq!(ev[0]["server"], "rows");
    assert_eq!(ev[0]["mode"], "proxy");
    assert_eq!(ev[0]["tool"], "query");
    assert!(ev[0]["out_bytes"].as_u64() < ev[0]["in_bytes"].as_u64());
    assert!(ev[0]["call_ms"].is_u64());
    assert_eq!(ev[1]["outcome"], "json");
    assert_eq!(ev[1]["reason"], "small");
    assert_eq!(ev[2]["reason"], "passthrough");
    assert_eq!(
        ev[2]["in_bytes"], ev[0]["in_bytes"],
        "same data, measured unchanged"
    );
    // No content of the rows ever lands in the file.
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(
        !raw.contains("example.com") && !raw.contains("user 1"),
        "{raw}"
    );

    let out = Command::new(exe())
        .args(["stats", "--json", "--file"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(out.status.success());
    let s: Json = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(s["responses"], 3);
    assert_eq!(s["converted"], 1);
    assert_eq!(s["kept"]["small"], 1);
    assert_eq!(s["kept"]["passthrough"], 1);
    let report = Command::new(exe())
        .args(["stats", "--file"])
        .arg(&path)
        .output()
        .unwrap();
    let text = String::from_utf8(report.stdout).unwrap();
    assert!(text.contains("tool responses     3"), "{text}");
    assert!(text.contains("rows / query"), "{text}");
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[test]
fn the_files_server_records_and_no_stats_records_nothing() {
    let path = temp_file("files");
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/small");
    let replies = session(
        &path,
        &["files", "--root", root, "--quiet"],
        &[call(
            1,
            "read_rows",
            json!({"path": "flat_users.json", "limit": 50}),
        )],
    );
    assert!(
        replies[0]["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .starts_with("@records ")
    );
    let ev = events(&path);
    assert_eq!(ev.len(), 1);
    assert_eq!(
        (ev[0]["mode"].as_str(), ev[0]["tool"].as_str()),
        (Some("files"), Some("read_rows"))
    );
    assert_eq!(ev[0]["outcome"], "ldf");

    let off = temp_file("off");
    session(
        &off,
        &["files", "--root", root, "--quiet", "--no-stats"],
        &[call(
            1,
            "read_rows",
            json!({"path": "flat_users.json", "limit": 50}),
        )],
    );
    assert!(!off.exists());
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}
