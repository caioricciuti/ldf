//! The stdio loop shared by the servers in this crate: one JSON-RPC message
//! per line in, at most one per line out, notifications answered with
//! nothing. Logging goes to stderr only, as the transport requires.

use std::io::{BufRead, Write};

use serde_json::Value as Json;

/// Reads stdin to the end, handing each parseable message to `handle` and
/// writing whatever it returns.
pub fn serve<F>(mut handle: F) -> Result<(), String>
where
    F: FnMut(&Json) -> Option<Json>,
{
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
        let Some(reply) = handle(&msg) else {
            continue;
        };
        let s = serde_json::to_string(&reply).map_err(|e| e.to_string())?;
        out.write_all(s.as_bytes()).map_err(|e| e.to_string())?;
        out.write_all(b"\n").map_err(|e| e.to_string())?;
        out.flush().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// A JSON-RPC error reply.
pub fn error(id: &Json, code: i64, message: impl Into<String>) -> Json {
    serde_json::json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message.into()}})
}

/// A JSON-RPC result reply.
pub fn result(id: &Json, result: Json) -> Json {
    serde_json::json!({"jsonrpc": "2.0", "id": id, "result": result})
}

/// A tool result with one text block, or two when a note follows the data.
pub fn tool_text(text: String, note: Option<String>, is_error: bool) -> Json {
    let mut content = vec![serde_json::json!({"type": "text", "text": text})];
    if let Some(n) = note {
        content.push(serde_json::json!({"type": "text", "text": n}));
    }
    let mut r = serde_json::json!({"content": content});
    if is_error {
        r["isError"] = Json::Bool(true);
    }
    r
}
