//! The stdio pump: client <-> this process <-> downstream server.
//!
//! Lines are forwarded as bytes. A line is only rewritten when it parses as
//! a JSON-RPC response to a request this proxy saw the client send; anything
//! else (server requests and notifications, invalid JSON, invalid UTF-8)
//! passes through untouched.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value as Json;

use crate::rewrite::{Kept, Selector, rewrite_initialize, rewrite_tool_result, rewrite_tools_list};
use crate::stats::Recorder;

/// How long the child gets to exit after the client closed our stdin (and
/// we closed the child's) before it is killed. The MCP stdio transport
/// expects a server to exit on end of input.
pub const SHUTDOWN_GRACE: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct Config {
    /// Downstream server command and arguments.
    pub command: Vec<String>,
    /// Forward `tools/call` results unchanged (A/B against the rewrite).
    pub passthrough: bool,
    /// Do not touch `initialize` instructions or tool descriptions.
    pub plain: bool,
    /// Text blocks shorter than this many bytes are never converted.
    pub min_bytes: usize,
    /// One stderr line per converted result.
    pub log: bool,
    /// Pick JSON or LDF per result for one model; `None` converts whenever
    /// LDF is shorter.
    pub selector: Option<Selector>,
    /// Local statistics, one line per `tools/call` response.
    pub stats: Arc<Recorder>,
}

impl Config {
    /// Whether the client is told about LDF: not with `--plain`, and not
    /// when the selector has no evidence for the model (every result stays
    /// JSON then).
    fn primed(&self) -> bool {
        !self.plain && self.selector.as_ref().is_none_or(Selector::knows_model)
    }
}

/// A request the client sent: its method, the tool name for `tools/call`,
/// and when it passed through.
struct Request {
    method: String,
    tool: Option<String>,
    at: Instant,
}

/// Requests remembered until the downstream reply arrives or the client
/// cancels them. Ids are keyed by their JSON text so numeric and string ids
/// never collide.
type Pending = Arc<Mutex<HashMap<String, Request>>>;

/// Runs the proxy until the client closes stdin or the child exits. Returns
/// the child's exit code.
pub fn run(cfg: Config) -> Result<i32, String> {
    let (prog, args) = cfg
        .command
        .split_first()
        .ok_or("proxy needs a downstream command after --")?;
    let mut child = Command::new(prog)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("cannot start {prog}: {e}"))?;
    let child_stdin = child.stdin.take().ok_or("no child stdin")?;
    let child_stdout = child.stdout.take().ok_or("no child stdout")?;
    let child = Arc::new(Mutex::new(child));
    let pending: Pending = Arc::new(Mutex::new(HashMap::new()));

    // Client -> child. Only records which method an id belongs to.
    let up_pending = Arc::clone(&pending);
    let up_child = Arc::clone(&child);
    let upstream = thread::spawn(move || {
        let stdin = std::io::stdin();
        let _ = pump_upstream(stdin.lock(), child_stdin, &up_pending);
        // The child's stdin is closed now. Give it the grace period to exit
        // on its own, then kill it so the proxy cannot hang.
        stop_after_grace(&up_child, SHUTDOWN_GRACE);
    });

    // Child -> client, with the rewrites.
    let stdout = std::io::stdout();
    let status = pump_downstream(child_stdout, stdout.lock(), &pending, &cfg);
    if status.is_err() {
        // The client side is gone or unreadable output arrived: nothing can
        // be delivered any more, so do not leave the child running.
        if let Ok(mut c) = child.lock() {
            let _ = c.kill();
        }
    }
    let exit = wait_child(&child);
    // The upstream thread ends when the client closes stdin; after the child
    // is gone there is nothing to forward to, so do not wait for it.
    drop(upstream);
    status?;
    exit
}

/// Waits for the child without holding its lock, so the upstream thread can
/// still kill it (a child may close stdout and keep running).
fn wait_child(child: &Mutex<Child>) -> Result<i32, String> {
    loop {
        {
            let mut c = child.lock().map_err(|_| "child handle poisoned")?;
            if let Some(s) = c.try_wait().map_err(|e| e.to_string())? {
                return Ok(s.code().unwrap_or(1));
            }
        }
        thread::sleep(Duration::from_millis(20));
    }
}

/// Polls the child until it exits or `grace` passes, then kills it.
fn stop_after_grace(child: &Mutex<Child>, grace: Duration) {
    let start = Instant::now();
    loop {
        match child.lock() {
            Ok(mut c) => match c.try_wait() {
                Ok(Some(_)) | Err(_) => return,
                Ok(None) if start.elapsed() >= grace => {
                    let _ = c.kill();
                    return;
                }
                Ok(None) => {}
            },
            Err(_) => return,
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn pump_upstream<R: Read, W: Write>(
    from_client: R,
    to_child: W,
    pending: &Pending,
) -> Result<(), String> {
    let mut reader = BufReader::new(from_client);
    let mut writer = BufWriter::new(to_child);
    let mut line = Vec::new();
    loop {
        line.clear();
        let n = reader
            .read_until(b'\n', &mut line)
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Ok(()); // client closed our stdin: closing the child's follows
        }
        if let Ok(msg) = serde_json::from_slice::<Json>(trim_eol(&line)) {
            note_request(pending, &msg);
        }
        writer.write_all(&line).map_err(|e| e.to_string())?;
        if !line.ends_with(b"\n") {
            writer.write_all(b"\n").map_err(|e| e.to_string())?;
        }
        writer.flush().map_err(|e| e.to_string())?;
    }
}

fn trim_eol(line: &[u8]) -> &[u8] {
    let mut end = line.len();
    while end > 0 && matches!(line[end - 1], b'\n' | b'\r') {
        end -= 1;
    }
    &line[..end]
}

fn note_request(pending: &Pending, msg: &Json) {
    let Some(method) = msg.get("method").and_then(Json::as_str) else {
        return;
    };
    let Some(id) = msg.get("id") else {
        // A cancelled request may never be answered; forget it.
        if method == "notifications/cancelled"
            && let Some(rid) = msg.pointer("/params/requestId")
            && let Ok(mut p) = pending.lock()
        {
            p.remove(&rid.to_string());
        }
        return;
    };
    let tool = if method == "tools/call" {
        msg.pointer("/params/name")
            .and_then(Json::as_str)
            .map(str::to_string)
    } else {
        None
    };
    if let Ok(mut p) = pending.lock() {
        p.insert(
            id.to_string(),
            Request {
                method: method.to_string(),
                tool,
                at: Instant::now(),
            },
        );
    }
}

fn pump_downstream<R: Read, W: Write>(
    from_child: R,
    to_client: W,
    pending: &Pending,
    cfg: &Config,
) -> Result<(), String> {
    let mut reader = BufReader::new(from_child);
    let mut out = BufWriter::new(to_client);
    let mut line = Vec::new();
    loop {
        line.clear();
        let n = reader
            .read_until(b'\n', &mut line)
            .map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        let trimmed = trim_eol(&line);
        if trimmed.is_empty() {
            continue;
        }
        let replaced = std::str::from_utf8(trimmed)
            .ok()
            .and_then(|text| rewrite_line(text, pending, cfg));
        match replaced {
            Some(r) => out.write_all(r.as_bytes()),
            None => out.write_all(trimmed),
        }
        .and_then(|_| out.write_all(b"\n"))
        .and_then(|_| out.flush())
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Returns the rewritten message, or `None` to forward the line verbatim.
fn rewrite_line(line: &str, pending: &Pending, cfg: &Config) -> Option<String> {
    let mut msg: Json = serde_json::from_str(line).ok()?;
    // A server-to-client request (sampling, roots, elicitation) carries a
    // method and may reuse an id the client also used: it is not a reply.
    if msg.get("method").is_some() {
        return None;
    }
    let id = msg.get("id")?.to_string();
    let req = pending.lock().ok()?.remove(&id)?;
    let result = msg.get_mut("result")?;
    let tool = req.tool.as_deref().unwrap_or("?");
    match req.method.as_str() {
        "initialize" if cfg.primed() => rewrite_initialize(result),
        "tools/list" if cfg.primed() => rewrite_tools_list(result),
        "tools/call" => {
            let call_ms = Some(req.at.elapsed().as_millis() as u64);
            if cfg.passthrough {
                // Measure what passes, change nothing: with no size large
                // enough to convert, the rewrite only counts the blocks.
                let done = rewrite_tool_result(result, usize::MAX, None);
                cfg.stats.record(tool, &done, true, call_ms);
                return None;
            }
            let done = rewrite_tool_result(result, cfg.min_bytes, cfg.selector.as_ref());
            cfg.stats.record(tool, &done, false, call_ms);
            if cfg.log {
                for b in &done.blocks {
                    match &b.kept {
                        None => eprintln!(
                            "ldf-mcp: {tool} json {} B -> ldf {} B ({:.0}%)",
                            b.in_bytes,
                            b.out_bytes,
                            100.0 * b.out_bytes as f64 / b.in_bytes as f64
                        ),
                        Some(Kept::Selector(why)) => eprintln!("ldf-mcp: {tool} {why}"),
                        Some(_) => {}
                    }
                }
            }
            if !done.converted() {
                return None;
            }
        }
        _ => return None,
    }
    serde_json::to_string(&msg).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn cfg() -> Config {
        Config {
            command: vec![],
            passthrough: false,
            plain: false,
            min_bytes: 0,
            log: false,
            selector: None,
            stats: Arc::new(Recorder::off()),
        }
    }

    #[test]
    fn cancelled_requests_are_forgotten() {
        let p: Pending = Arc::new(Mutex::new(HashMap::new()));
        note_request(
            &p,
            &json!({"jsonrpc": "2.0", "id": 7, "method": "tools/call", "params": {"name": "q"}}),
        );
        note_request(&p, &json!({"jsonrpc": "2.0", "id": "7", "method": "ping"}));
        assert_eq!(p.lock().unwrap().len(), 2);
        note_request(
            &p,
            &json!({"jsonrpc": "2.0", "method": "notifications/cancelled", "params": {"requestId": 7}}),
        );
        let left: Vec<String> = p.lock().unwrap().keys().cloned().collect();
        assert_eq!(left, vec!["\"7\"".to_string()]);
    }

    #[test]
    fn a_server_request_does_not_consume_the_pending_client_id() {
        let p: Pending = Arc::new(Mutex::new(HashMap::new()));
        note_request(
            &p,
            &json!({"jsonrpc": "2.0", "id": 1, "method": "initialize"}),
        );
        let req = r#"{"jsonrpc":"2.0","id":1,"method":"roots/list"}"#;
        assert!(rewrite_line(req, &p, &cfg()).is_none());
        let reply = r#"{"jsonrpc":"2.0","id":1,"result":{"instructions":"x"}}"#;
        let out: Json = serde_json::from_str(&rewrite_line(reply, &p, &cfg()).unwrap()).unwrap();
        assert!(
            out["result"]["instructions"]
                .as_str()
                .unwrap()
                .contains("LDF")
        );
        assert!(p.lock().unwrap().is_empty());
    }

    #[test]
    fn non_object_results_do_not_panic() {
        let p: Pending = Arc::new(Mutex::new(HashMap::new()));
        note_request(
            &p,
            &json!({"jsonrpc": "2.0", "id": 1, "method": "initialize"}),
        );
        note_request(
            &p,
            &json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
        );
        note_request(
            &p,
            &json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call"}),
        );
        assert!(rewrite_line(r#"{"jsonrpc":"2.0","id":1,"result":[1]}"#, &p, &cfg()).is_some());
        let list = rewrite_line(
            r#"{"jsonrpc":"2.0","id":2,"result":{"tools":["x",{"name":"a"}]}}"#,
            &p,
            &cfg(),
        )
        .unwrap();
        let list: Json = serde_json::from_str(&list).unwrap();
        assert_eq!(list["result"]["tools"][0], "x");
        assert!(
            list["result"]["tools"][1]["description"]
                .as_str()
                .unwrap()
                .contains("LDF")
        );
        assert!(rewrite_line(r#"{"jsonrpc":"2.0","id":3,"result":"text"}"#, &p, &cfg()).is_none());
    }
}
