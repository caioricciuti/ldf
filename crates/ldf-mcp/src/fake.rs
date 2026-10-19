//! A minimal downstream MCP server with one tool, `query`, that returns a
//! generated JSON row set. Used by the integration tests and handy for
//! trying the proxy without a real database server behind it.
//!
//! Test behaviours, all off by default, as `query` arguments: `delay_ms`
//! answers from a thread after a pause (so replies can overtake each other),
//! `structured` adds `structuredContent`, `server_request` first sends a
//! server-to-client request under the same id, `garbage` first writes a
//! non-JSON line and a line of invalid UTF-8, `crash` exits with code 3
//! without answering. `ignore_eof` (the `fake-server --ignore-eof` flag)
//! keeps the process alive for a minute after stdin closes.

use std::io::{BufRead, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde_json::{Value as Json, json};

pub fn run(ignore_eof: bool) -> Result<(), String> {
    let stdin = std::io::stdin();
    let out = Arc::new(Mutex::new(std::io::stdout()));
    let mut workers = Vec::new();
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let msg: Json = match serde_json::from_str(&line) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let Some(id) = msg.get("id").cloned() else {
            continue; // notification
        };
        let method = msg.get("method").and_then(Json::as_str).unwrap_or("");
        let reply = match method {
            "initialize" => json!({"jsonrpc": "2.0", "id": id, "result": {
                "protocolVersion": msg.pointer("/params/protocolVersion").cloned().unwrap_or(json!("2025-06-18")),
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "ldf-mcp-fake", "version": env!("CARGO_PKG_VERSION")},
                "instructions": "Fake row server."
            }}),
            "ping" => json!({"jsonrpc": "2.0", "id": id, "result": {}}),
            "tools/list" => json!({"jsonrpc": "2.0", "id": id, "result": {"tools": [{
                "name": "query",
                "description": "Returns rows as a JSON array.",
                "inputSchema": {"type": "object", "properties": {
                    "rows": {"type": "integer", "description": "How many rows to return"},
                    "nested": {"type": "boolean", "description": "Wrap rows in an object with meta"}
                }}
            }]}}),
            "tools/call" => {
                let arg = |k: &str| msg.pointer(&format!("/params/arguments/{k}")).cloned();
                if arg("crash").and_then(|v| v.as_bool()) == Some(true) {
                    std::process::exit(3);
                }
                let n = msg
                    .pointer("/params/arguments/rows")
                    .and_then(Json::as_u64)
                    .unwrap_or(50) as usize;
                let nested = msg
                    .pointer("/params/arguments/nested")
                    .and_then(Json::as_bool)
                    .unwrap_or(false);
                let rows: Vec<Json> = (1..=n)
                    .map(|i| {
                        json!({
                            "id": i,
                            "name": format!("user {i}"),
                            "email": format!("user.{i}@example.com"),
                            "score": (i as f64) * 1.5,
                            "active": i % 3 != 0,
                            "note": if i % 7 == 0 { Json::Null } else { json!("row, with comma") }
                        })
                    })
                    .collect();
                let payload = if nested {
                    json!({"meta": {"rows": n, "elapsed_ms": 3}, "rows": rows})
                } else {
                    Json::Array(rows)
                };
                let text = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
                let mut result = json!({"content": [{"type": "text", "text": text}]});
                if arg("structured").and_then(|v| v.as_bool()) == Some(true) {
                    result["structuredContent"] = json!({"rows": payload});
                }
                let reply = json!({"jsonrpc": "2.0", "id": id, "result": result});
                let mut before = Vec::new();
                if arg("server_request").and_then(|v| v.as_bool()) == Some(true) {
                    before.push(
                        json!({"jsonrpc": "2.0", "id": id, "method": "roots/list"})
                            .to_string()
                            .into_bytes(),
                    );
                }
                if arg("garbage").and_then(|v| v.as_bool()) == Some(true) {
                    before.push(b"not json at all".to_vec());
                    before.push(vec![0xff, 0xfe, b'{', b'}']);
                }
                let delay = arg("delay_ms").and_then(|v| v.as_u64()).unwrap_or(0);
                let out = Arc::clone(&out);
                let send = move || {
                    thread::sleep(Duration::from_millis(delay));
                    let mut lines = before;
                    lines.push(reply.to_string().into_bytes());
                    write_lines(&out, &lines)
                };
                if delay > 0 {
                    workers.push(thread::spawn(send));
                } else {
                    send()?;
                }
                continue;
            }
            _ => {
                json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": format!("Method not found: {method}")}})
            }
        };
        write_lines(&out, &[reply.to_string().into_bytes()])?;
    }
    for w in workers {
        let _ = w.join();
    }
    if ignore_eof {
        thread::sleep(Duration::from_secs(60));
    }
    Ok(())
}

fn write_lines(out: &Mutex<std::io::Stdout>, lines: &[Vec<u8>]) -> Result<(), String> {
    let out = out.lock().map_err(|_| "stdout poisoned")?;
    let mut out = out.lock();
    for l in lines {
        out.write_all(l).map_err(|e| e.to_string())?;
        out.write_all(b"\n").map_err(|e| e.to_string())?;
    }
    out.flush().map_err(|e| e.to_string())
}
