//! The stdio pump: client <-> this process <-> downstream server.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use serde_json::Value as Json;

use crate::rewrite::{Selector, rewrite_initialize, rewrite_tool_result, rewrite_tools_list};

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
}

impl Config {
    /// Whether the client is told about LDF: not with `--plain`, and not
    /// when the selector has no evidence for the model (every result stays
    /// JSON then).
    fn primed(&self) -> bool {
        !self.plain && self.selector.as_ref().is_none_or(Selector::knows_model)
    }
}

/// Request id and the method the client sent under it, remembered until the
/// downstream reply arrives. Ids are keyed by their JSON text so numeric
/// and string ids never collide.
type Pending = Arc<Mutex<HashMap<String, (String, Option<String>)>>>;

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
    let pending: Pending = Arc::new(Mutex::new(HashMap::new()));

    // Client -> child. Only records which method an id belongs to.
    let up_pending = Arc::clone(&pending);
    let upstream = thread::spawn(move || -> Result<(), String> {
        let stdin = std::io::stdin();
        let mut reader = BufReader::new(stdin.lock());
        let mut writer = BufWriter::new(child_stdin);
        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line).map_err(|e| e.to_string())?;
            if n == 0 {
                break; // client closed our stdin: closing the child's follows
            }
            if let Ok(msg) = serde_json::from_str::<Json>(line.trim_end()) {
                note_request(&up_pending, &msg);
            }
            writer
                .write_all(line.as_bytes())
                .map_err(|e| e.to_string())?;
            if !line.ends_with('\n') {
                writer.write_all(b"\n").map_err(|e| e.to_string())?;
            }
            writer.flush().map_err(|e| e.to_string())?;
        }
        Ok(())
    });

    // Child -> client, with the rewrites.
    let status = pump_downstream(child_stdout, &pending, &cfg);
    let exit = child
        .wait()
        .map(|s| s.code().unwrap_or(1))
        .map_err(|e| e.to_string());
    // The upstream thread ends when the client closes stdin; after the child
    // is gone there is nothing to forward to, so do not wait for it.
    drop(upstream);
    status?;
    exit
}

fn note_request(pending: &Pending, msg: &Json) {
    let (Some(id), Some(method)) = (msg.get("id"), msg.get("method").and_then(Json::as_str)) else {
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
        p.insert(id.to_string(), (method.to_string(), tool));
    }
}

fn pump_downstream<R: Read>(from_child: R, pending: &Pending, cfg: &Config) -> Result<(), String> {
    let mut reader = BufReader::new(from_child);
    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        let trimmed = line.trim_end_matches(['\n', '\r']);
        if trimmed.is_empty() {
            continue;
        }
        match rewrite_line(trimmed, pending, cfg) {
            Some(replaced) => {
                out.write_all(replaced.as_bytes())
                    .map_err(|e| e.to_string())?;
                out.write_all(b"\n").map_err(|e| e.to_string())?;
            }
            None => {
                out.write_all(trimmed.as_bytes())
                    .map_err(|e| e.to_string())?;
                out.write_all(b"\n").map_err(|e| e.to_string())?;
            }
        }
        out.flush().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Returns the rewritten message, or `None` to forward the line verbatim.
fn rewrite_line(line: &str, pending: &Pending, cfg: &Config) -> Option<String> {
    let mut msg: Json = serde_json::from_str(line).ok()?;
    let id = msg.get("id")?.to_string();
    let (method, tool) = pending.lock().ok()?.remove(&id)?;
    let result = msg.get_mut("result")?;
    match method.as_str() {
        "initialize" if cfg.primed() => rewrite_initialize(result),
        "tools/list" if cfg.primed() => rewrite_tools_list(result),
        "tools/call" if !cfg.passthrough => {
            let done = rewrite_tool_result(result, cfg.min_bytes, cfg.selector.as_ref());
            if cfg.log {
                for why in &done.kept {
                    eprintln!("ldf-mcp: {} {why}", tool.as_deref().unwrap_or("?"));
                }
            }
            if done.replaced.is_empty() {
                return None;
            }
            if cfg.log {
                for r in &done.replaced {
                    eprintln!(
                        "ldf-mcp: {} json {} B -> ldf {} B ({:.0}%)",
                        tool.as_deref().unwrap_or("?"),
                        r.json_bytes,
                        r.ldf_bytes,
                        100.0 * r.ldf_bytes as f64 / r.json_bytes as f64
                    );
                }
            }
        }
        _ => return None,
    }
    serde_json::to_string(&msg).ok()
}
