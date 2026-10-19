//! The stdio loop shared by the servers in this crate: one JSON-RPC message
//! per line in, at most one per line out, notifications answered with
//! nothing, a line that is not JSON (or not UTF-8) answered with a parse
//! error. Logging goes to stderr only, as the transport requires.

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
    let mut input = stdin.lock();
    let mut line = Vec::new();
    loop {
        line.clear();
        if input
            .read_until(b'\n', &mut line)
            .map_err(|e| e.to_string())?
            == 0
        {
            break;
        }
        if line.trim_ascii().is_empty() {
            continue;
        }
        // JSON-RPC: a line that is not JSON gets a parse error with a null id.
        let reply = match serde_json::from_slice::<Json>(line.trim_ascii()) {
            Ok(msg) => handle(&msg),
            Err(_) => Some(error(&Json::Null, -32700, "Parse error")),
        };
        let Some(reply) = reply else {
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
