//! A minimal downstream MCP server with one tool, `query`, that returns a
//! generated JSON row set. Used by the integration tests and handy for
//! trying the proxy without a real database server behind it.

use std::io::{BufRead, Write};

use serde_json::{Value as Json, json};

pub fn run() -> Result<(), String> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
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
                json!({"jsonrpc": "2.0", "id": id, "result": {"content": [{"type": "text", "text": text}]}})
            }
            _ => {
                json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": format!("Method not found: {method}")}})
            }
        };
        let s = serde_json::to_string(&reply).map_err(|e| e.to_string())?;
        out.write_all(s.as_bytes()).map_err(|e| e.to_string())?;
        out.write_all(b"\n").map_err(|e| e.to_string())?;
        out.flush().map_err(|e| e.to_string())?;
    }
    Ok(())
}
