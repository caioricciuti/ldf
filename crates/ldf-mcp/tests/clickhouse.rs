//! End-to-end against a real ClickHouse. Runs only when `LDF_CLICKHOUSE_URL`
//! is set (for example `http://127.0.0.1:8124` with the tables that
//! `docs/ldf-mcp.md` describes, `LDF_CLICKHOUSE_USER` optional); otherwise
//! it is skipped with a note.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use serde_json::{Value as Json, json};

#[test]
fn clickhouse_mode_end_to_end() {
    let Ok(url) = std::env::var("LDF_CLICKHOUSE_URL") else {
        eprintln!("LDF_CLICKHOUSE_URL not set, skipping");
        return;
    };
    let exe = env!("CARGO_BIN_EXE_ldf-mcp");
    let mut child = Command::new(exe)
        .args(["clickhouse", "--url", &url, "--quiet", "--max-rows", "500"])
        .args([
            "--user",
            &std::env::var("LDF_CLICKHOUSE_USER").unwrap_or_else(|_| "default".into()),
        ])
        .env("LDF_STATS", "0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn");
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut call = |id: u64, method: &str, params: Json| -> Json {
        let mut s = serde_json::to_string(
            &json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}),
        )
        .unwrap();
        s.push('\n');
        stdin.write_all(s.as_bytes()).unwrap();
        stdin.flush().unwrap();
        let mut line = String::new();
        stdout.read_line(&mut line).unwrap();
        let v: Json = serde_json::from_str(line.trim_end()).unwrap();
        assert_eq!(v["id"], id);
        v
    };

    let init = call(
        1,
        "initialize",
        json!({"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}}),
    );
    assert!(
        init["result"]["instructions"]
            .as_str()
            .unwrap()
            .contains("run_select")
    );

    let tables = call(
        2,
        "tools/call",
        json!({"name": "list_tables", "arguments": {}}),
    );
    let text = tables["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("users"), "{text}");

    let rows = call(
        3,
        "tools/call",
        json!({"name": "run_select", "arguments": {"sql": "SELECT * FROM users ORDER BY id LIMIT 300"}}),
    );
    let text = rows["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.starts_with("@records "), "{text}");
    let records = ldf::decode(text).unwrap();
    assert_eq!(records.len(), 300);

    let nested = call(
        4,
        "tools/call",
        json!({"name": "run_select", "arguments": {"sql": "SELECT * FROM orders ORDER BY order_id LIMIT 100"}}),
    );
    let text = nested["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("\n@records.items "),
        "child table expected: {text}"
    );
    assert_eq!(ldf::decode(text).unwrap().len(), 100);

    let write = call(
        5,
        "tools/call",
        json!({"name": "run_select", "arguments": {"sql": "DROP TABLE users"}}),
    );
    assert_eq!(
        write["result"]["isError"], true,
        "readonly must refuse writes"
    );
    let text = write["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("readonly") || text.contains("READONLY"),
        "{text}"
    );

    drop(stdin);
    assert!(child.wait().unwrap().success());
}
