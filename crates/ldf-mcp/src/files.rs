//! `ldf-mcp files`: an MCP server over a directory of data files. JSON,
//! JSONL, CSV, TSV and Parquet are read with `ldf_data`'s streaming
//! readers, so a file larger than memory can still be paged through, and
//! rows come back as LDF. Paths are confined to the root directory.

use std::path::{Path, PathBuf};

use ldf_data::Value;
use ldf_data::input::{InputKind, ReadOptions, open};
use serde_json::{Value as Json, json};

use crate::rewrite::{PRIMER, convert_text};
use crate::rpc;

#[derive(Debug, Clone)]
pub struct Config {
    /// Canonical root; every path argument resolves inside it.
    pub root: PathBuf,
    /// Rows per `read_rows` call at most.
    pub max_rows: usize,
    pub passthrough: bool,
    pub min_bytes: usize,
    pub log: bool,
}

/// Rows the schema is inferred from in `describe_file`.
const SAMPLE: usize = 200;

pub fn run(cfg: Config) -> Result<(), String> {
    let root = cfg
        .root
        .canonicalize()
        .map_err(|e| format!("root {}: {e}", cfg.root.display()))?;
    let cfg = Config { root, ..cfg };
    rpc::serve(|msg| handle(&cfg, msg))
}

pub fn handle(cfg: &Config, msg: &Json) -> Option<Json> {
    let id = msg.get("id")?;
    let method = msg.get("method").and_then(Json::as_str).unwrap_or("");
    let reply = match method {
        "initialize" => rpc::result(
            id,
            json!({
                "protocolVersion": msg.pointer("/params/protocolVersion").cloned().unwrap_or(json!("2025-06-18")),
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "ldf-mcp", "version": env!("CARGO_PKG_VERSION")},
                "instructions": format!(
                    "Read-only access to the data files under `{}` (json, jsonl, csv, tsv, parquet). Use list_files to see \
                     them, describe_file for a file's columns, then read_rows with limit and offset to page through it, \
                     at most {} rows per call.\n\n{PRIMER}",
                    cfg.root.display(), cfg.max_rows
                ),
            }),
        ),
        "ping" => rpc::result(id, json!({})),
        "tools/list" => rpc::result(id, json!({"tools": tools(cfg)})),
        "tools/call" => {
            let name = msg
                .pointer("/params/name")
                .and_then(Json::as_str)
                .unwrap_or("");
            let args = msg
                .pointer("/params/arguments")
                .cloned()
                .unwrap_or(json!({}));
            match call(cfg, name, &args) {
                Ok((text, note)) => rpc::result(id, rpc::tool_text(text, note, false)),
                Err(CallError::Protocol(m)) => rpc::error(id, -32602, m),
                Err(CallError::Tool(m)) => rpc::result(id, rpc::tool_text(m, None, true)),
            }
        }
        _ => rpc::error(id, -32601, format!("Method not found: {method}")),
    };
    Some(reply)
}

fn tools(cfg: &Config) -> Json {
    json!([
        {
            "name": "list_files",
            "description": "Lists the data files under the root, recursively: path, kind, bytes.",
            "inputSchema": {"type": "object", "properties": {}}
        },
        {
            "name": "describe_file",
            "description": format!("The columns of a file, inferred from its first {SAMPLE} records, as an LDF header."),
            "inputSchema": {"type": "object", "properties": {
                "path": {"type": "string", "description": "Path relative to the root, as list_files prints it"}
            }, "required": ["path"]}
        },
        {
            "name": "read_rows",
            "description": format!(
                "Reads records from a file in order. At most {} rows per call; a capped result says so in a second \
                 text block, page on with offset.",
                cfg.max_rows
            ),
            "inputSchema": {"type": "object", "properties": {
                "path": {"type": "string", "description": "Path relative to the root"},
                "limit": {"type": "integer", "description": "Rows to return (default and maximum: the server's cap)"},
                "offset": {"type": "integer", "description": "Rows to skip first (default 0)"}
            }, "required": ["path"]}
        }
    ])
}

enum CallError {
    Protocol(String),
    Tool(String),
}

fn call(cfg: &Config, name: &str, args: &Json) -> Result<(String, Option<String>), CallError> {
    match name {
        "list_files" => {
            let mut rows: Vec<Value> = Vec::new();
            walk(&cfg.root, &cfg.root, &mut rows).map_err(CallError::Tool)?;
            rows.sort_by(|a, b| {
                let ka = a.get("path").and_then(Value::as_str).unwrap_or("");
                let kb = b.get("path").and_then(Value::as_str).unwrap_or("");
                ka.cmp(kb)
            });
            if rows.is_empty() {
                return Ok(("(no data files)".into(), None));
            }
            Ok((render(cfg, "list_files", &rows), None))
        }
        "describe_file" => {
            let path = resolve(cfg, arg_str(args, "path")?)?;
            let sample: Vec<Value> = open_file(&path)?
                .take(SAMPLE)
                .collect::<Result<_, _>>()
                .map_err(|e| CallError::Tool(e.to_string()))?;
            if sample.is_empty() {
                return Ok(("(empty file)".into(), None));
            }
            let plan = ldf::plan::plan(&sample);
            let mut text = format!(
                "{} records sampled. LDF header:\n{}",
                sample.len(),
                plan.header()
            );
            for c in &plan.columns {
                if let Some(child) = &c.child {
                    text.push('\n');
                    text.push_str(&child.header());
                }
            }
            Ok((text, None))
        }
        "read_rows" => {
            let path = resolve(cfg, arg_str(args, "path")?)?;
            let limit = args
                .get("limit")
                .and_then(Json::as_u64)
                .map(|n| n as usize)
                .unwrap_or(cfg.max_rows)
                .clamp(1, cfg.max_rows);
            let offset = args.get("offset").and_then(Json::as_u64).unwrap_or(0) as usize;
            let mut it = open_file(&path)?.skip(offset);
            let mut rows: Vec<Value> = Vec::with_capacity(limit);
            for rec in it.by_ref().take(limit) {
                rows.push(rec.map_err(|e| CallError::Tool(e.to_string()))?);
            }
            if rows.is_empty() {
                return Ok(("(no rows)".into(), None));
            }
            // One more record decides whether the file goes on.
            let more = it.next().is_some();
            let note = more.then(|| {
                format!(
                    "Showing rows {} to {}; more follow. Call again with offset {}.",
                    offset + 1,
                    offset + rows.len(),
                    offset + rows.len()
                )
            });
            Ok((render(cfg, "read_rows", &rows), note))
        }
        other => Err(CallError::Protocol(format!("Unknown tool: {other}"))),
    }
}

fn arg_str<'a>(args: &'a Json, key: &str) -> Result<&'a str, CallError> {
    args.get(key)
        .and_then(Json::as_str)
        .ok_or_else(|| CallError::Protocol(format!("`{key}` is required")))
}

/// Resolves a relative path inside the root and refuses anything else.
fn resolve(cfg: &Config, rel: &str) -> Result<PathBuf, CallError> {
    let candidate = cfg.root.join(rel);
    let full = candidate
        .canonicalize()
        .map_err(|e| CallError::Tool(format!("{rel}: {e}")))?;
    if !full.starts_with(&cfg.root) {
        return Err(CallError::Protocol(format!("{rel}: outside the root")));
    }
    if !full.is_file() {
        return Err(CallError::Tool(format!("{rel}: not a file")));
    }
    Ok(full)
}

fn open_file(
    path: &Path,
) -> Result<Box<dyn Iterator<Item = Result<Value, ldf_data::input::InputError>>>, CallError> {
    open(path, &ReadOptions::default())
        .map(|s| Box::new(s) as Box<dyn Iterator<Item = _>>)
        .map_err(|e| CallError::Tool(format!("{}: {e}", path.display())))
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<Value>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }
        let meta = entry.metadata().map_err(|e| e.to_string())?;
        if meta.is_dir() {
            walk(root, &path, out)?;
            continue;
        }
        let Ok(kind) = InputKind::from_path(&path) else {
            continue;
        };
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        out.push(ldf_data::obj! {
            "path" => rel,
            "kind" => format!("{kind:?}").to_ascii_lowercase(),
            "bytes" => meta.len()
        });
    }
    Ok(())
}

/// Rows as LDF when that is shorter and allowed, JSON otherwise.
fn render(cfg: &Config, tool: &str, rows: &[Value]) -> String {
    let mut json_text = String::with_capacity(rows.len() * 64);
    json_text.push('[');
    for (i, r) in rows.iter().enumerate() {
        if i > 0 {
            json_text.push(',');
        }
        ldf::text::push_json_value(r, &mut json_text);
    }
    json_text.push(']');
    if cfg.passthrough {
        return json_text;
    }
    match convert_text(&json_text, cfg.min_bytes) {
        Some(ldf) => {
            if cfg.log {
                eprintln!(
                    "ldf-mcp: {tool} {} rows, json {} B -> ldf {} B ({:.0}%)",
                    rows.len(),
                    json_text.len(),
                    ldf.len(),
                    100.0 * ldf.len() as f64 / json_text.len() as f64
                );
            }
            ldf
        }
        None => json_text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> Config {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/small");
        Config {
            root: root.canonicalize().unwrap(),
            max_rows: 7,
            passthrough: false,
            min_bytes: 0,
            log: false,
        }
    }

    fn text(reply: &Json, i: usize) -> &str {
        reply["result"]["content"][i]["text"].as_str().unwrap()
    }

    #[test]
    fn lists_describes_and_pages() {
        let c = cfg();
        let list = handle(
            &c,
            &json!({"id": 1, "method": "tools/call", "params": {"name": "list_files"}}),
        )
        .unwrap();
        let t = text(&list, 0);
        assert!(t.starts_with("@records path:s,kind:s,bytes:i\n"), "{t}");
        assert!(t.contains("flat_users.json,json,"), "{t}");

        let d = handle(&c, &json!({"id": 2, "method": "tools/call", "params": {"name": "describe_file", "arguments": {"path": "nested_orders.json"}}})).unwrap();
        let t = text(&d, 0);
        assert!(t.contains("@records order_id:s,"), "{t}");
        assert!(t.contains("\n@records.items "), "{t}");

        let r = handle(&c, &json!({"id": 3, "method": "tools/call", "params": {"name": "read_rows", "arguments": {"path": "flat_users.jsonl", "limit": 5, "offset": 2}}})).unwrap();
        let rows = ldf::decode(text(&r, 0)).unwrap();
        assert_eq!(rows.len(), 5);
        assert_eq!(
            rows[0]
                .get("id")
                .and_then(Value::as_number)
                .map(|n| n.as_f64()),
            Some(3.0)
        );
        assert!(text(&r, 1).contains("offset 7"), "{}", text(&r, 1));

        // limit above the cap is clamped; the last page carries no note.
        let r = handle(&c, &json!({"id": 4, "method": "tools/call", "params": {"name": "read_rows", "arguments": {"path": "flat_users.jsonl", "limit": 100, "offset": 15}}})).unwrap();
        assert_eq!(ldf::decode(text(&r, 0)).unwrap().len(), 5);
        assert!(r["result"]["content"].as_array().unwrap().len() == 1);
    }

    #[test]
    fn stays_inside_the_root() {
        let c = cfg();
        let r = handle(&c, &json!({"id": 1, "method": "tools/call", "params": {"name": "read_rows", "arguments": {"path": "../../Cargo.toml"}}})).unwrap();
        assert_eq!(r["error"]["code"], -32602, "{r}");
        let r = handle(&c, &json!({"id": 2, "method": "tools/call", "params": {"name": "read_rows", "arguments": {"path": "nope.json"}}})).unwrap();
        assert_eq!(r["result"]["isError"], true);
    }
}
