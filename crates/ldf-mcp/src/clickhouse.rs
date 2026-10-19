//! `ldf-mcp clickhouse`: a small MCP server over stdio whose tools run
//! read-only SQL against a ClickHouse HTTP endpoint and return the rows as
//! LDF. Networking is plain HTTP/1.1 over `std::net`, nothing else, so the
//! crate stays free of external dependencies. The password is read from
//! `CLICKHOUSE_PASSWORD` only, never from the command line.

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use serde_json::{Value as Json, json};

use crate::rewrite::{PRIMER, Selector, convert};

#[derive(Debug, Clone)]
pub struct Config {
    /// `http://host:port`, no path.
    pub url: String,
    pub database: String,
    pub user: String,
    pub password: Option<String>,
    /// Return JSON instead of LDF.
    pub passthrough: bool,
    /// Results whose JSON is shorter than this stay JSON.
    pub min_bytes: usize,
    /// Pick JSON or LDF per result for one model (`--model`).
    pub selector: Option<Selector>,
    /// `max_result_rows` with `result_overflow_mode=break`.
    pub max_rows: u64,
    pub log: bool,
}

pub fn run(cfg: Config) -> Result<(), String> {
    crate::rpc::serve(|msg| handle(&cfg, msg))
}

/// One JSON-RPC message in, at most one out (notifications get none).
pub fn handle(cfg: &Config, msg: &Json) -> Option<Json> {
    let id = msg.get("id")?.clone();
    let method = msg.get("method").and_then(Json::as_str).unwrap_or("");
    let result = match method {
        "initialize" => json!({
            "protocolVersion": msg.pointer("/params/protocolVersion").cloned().unwrap_or(json!("2025-06-18")),
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "ldf-mcp", "version": env!("CARGO_PKG_VERSION")},
            "instructions": format!(
                "Read-only access to the ClickHouse database `{}`. Use list_tables and describe_table to find \
                 columns, then run_select with one SELECT statement. Results are capped at {} rows; use WHERE, \
                 GROUP BY and LIMIT to keep them small.\n\n{PRIMER}",
                cfg.database, cfg.max_rows
            ),
        }),
        "ping" => json!({}),
        "tools/list" => json!({"tools": tools(cfg)}),
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
                Ok(Reply { text, note: None }) => {
                    json!({"content": [{"type": "text", "text": text}]})
                }
                Ok(Reply {
                    text,
                    note: Some(note),
                }) => json!({"content": [
                    {"type": "text", "text": text},
                    {"type": "text", "text": note}
                ]}),
                Err(CallError::Protocol(m)) => {
                    return Some(
                        json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32602, "message": m}}),
                    );
                }
                Err(CallError::Tool(m)) => {
                    json!({"content": [{"type": "text", "text": m}], "isError": true})
                }
            }
        }
        _ => {
            return Some(
                json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": format!("Method not found: {method}")}}),
            );
        }
    };
    Some(json!({"jsonrpc": "2.0", "id": id, "result": result}))
}

fn tools(cfg: &Config) -> Json {
    json!([
        {
            "name": "list_tables",
            "description": format!("Lists the tables of database `{}` with engine, row and byte counts.", cfg.database),
            "inputSchema": {"type": "object", "properties": {}}
        },
        {
            "name": "describe_table",
            "description": "Columns and types of one table.",
            "inputSchema": {"type": "object", "properties": {
                "table": {"type": "string", "description": "Table name, optionally database.table"}
            }, "required": ["table"]}
        },
        {
            "name": "run_select",
            "description": format!(
                "Runs one read-only SELECT statement and returns at most {} rows. A capped result says so in a \
                 second text block; narrow it with WHERE, aggregate with GROUP BY, or page with LIMIT n OFFSET m.",
                cfg.max_rows
            ),
            "inputSchema": {"type": "object", "properties": {
                "sql": {"type": "string", "description": "A single SELECT (or SHOW/DESCRIBE/EXPLAIN) statement without a FORMAT clause"}
            }, "required": ["sql"]}
        }
    ])
}

/// A tool result: the rows, and a note when the row cap was hit.
pub struct Reply {
    pub text: String,
    pub note: Option<String>,
}

enum CallError {
    /// Bad tool name or arguments: a JSON-RPC error.
    Protocol(String),
    /// The query failed: an `isError` result the model can read.
    Tool(String),
}

fn call(cfg: &Config, name: &str, args: &Json) -> Result<Reply, CallError> {
    let sql = match name {
        "list_tables" => "SELECT name, engine, total_rows, total_bytes FROM system.tables \
                          WHERE database = currentDatabase() ORDER BY name"
            .to_string(),
        "describe_table" => {
            let table = args
                .get("table")
                .and_then(Json::as_str)
                .ok_or_else(|| CallError::Protocol("describe_table needs `table`".into()))?;
            if !is_identifier(table) {
                return Err(CallError::Protocol(format!("not a table name: {table}")));
            }
            format!("DESCRIBE TABLE {table}")
        }
        "run_select" => args
            .get("sql")
            .and_then(Json::as_str)
            .map(|s| s.trim().trim_end_matches(';').to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| CallError::Protocol("run_select needs `sql`".into()))?,
        other => return Err(CallError::Protocol(format!("Unknown tool: {other}"))),
    };
    let body = query(cfg, &sql).map_err(CallError::Tool)?;
    let rows: Vec<&str> = body.lines().filter(|l| !l.trim().is_empty()).collect();
    if rows.is_empty() {
        return Ok(Reply {
            text: "(no rows)".to_string(),
            note: None,
        });
    }
    // ClickHouse stops at max_result_rows silently (result_overflow_mode
    // break); the model needs to know the result is a prefix.
    let note = (rows.len() as u64 >= cfg.max_rows).then(|| {
        format!(
            "Result capped at {} rows; more may exist. Narrow it with WHERE, aggregate with GROUP BY, \
             or page with LIMIT {} OFFSET {}.",
            cfg.max_rows, cfg.max_rows, cfg.max_rows
        )
    });
    let json_text = format!("[{}]", rows.join(","));
    if cfg.passthrough {
        return Ok(Reply {
            text: json_text,
            note,
        });
    }
    match convert(&json_text, cfg.min_bytes, cfg.selector.as_ref()) {
        Ok(ldf) => {
            if cfg.log {
                eprintln!(
                    "ldf-mcp: {name} {} rows, json {} B -> ldf {} B ({:.0}%)",
                    rows.len(),
                    json_text.len(),
                    ldf.len(),
                    100.0 * ldf.len() as f64 / json_text.len() as f64
                );
            }
            Ok(Reply { text: ldf, note })
        }
        Err(why) => {
            if let (true, Some(why)) = (cfg.log, why) {
                eprintln!("ldf-mcp: {name} {why}");
            }
            Ok(Reply {
                text: json_text,
                note,
            })
        }
    }
}

fn is_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.split('.').count() <= 2
        && s.split('.').all(|seg| {
            let mut chars = seg.chars();
            matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
                && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}

/// Runs `sql` with the HTTP interface and returns the JSONEachRow body.
/// `readonly=1` makes ClickHouse refuse anything that writes.
pub fn query(cfg: &Config, sql: &str) -> Result<String, String> {
    if !is_identifier(&cfg.database) {
        return Err(format!("not a database name: {}", cfg.database));
    }
    let path = format!(
        "/?database={}&default_format=JSONEachRow&readonly=1&max_result_rows={}&result_overflow_mode=break\
         &output_format_json_quote_64bit_integers=0&output_format_json_named_tuples_as_objects=1",
        cfg.database, cfg.max_rows
    );
    let mut headers = vec![("X-ClickHouse-User", cfg.user.clone())];
    if let Some(p) = &cfg.password {
        headers.push(("X-ClickHouse-Key", p.clone()));
    }
    let (status, body) = http_post(&cfg.url, &path, &headers, sql.as_bytes())?;
    if status == 200 {
        Ok(body)
    } else {
        Err(format!("ClickHouse HTTP {status}: {}", body.trim()))
    }
}

/// Minimal HTTP/1.1 POST: one request, `Connection: close`, chunked or
/// length-delimited response. `base` is `http://host[:port]`.
pub fn http_post(
    base: &str,
    path: &str,
    headers: &[(&str, String)],
    body: &[u8],
) -> Result<(u16, String), String> {
    let (host, port) = parse_base(base)?;
    let addr = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|e| format!("resolve {host}: {e}"))?
        .next()
        .ok_or_else(|| format!("resolve {host}: no address"))?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(10))
        .map_err(|e| format!("connect {host}:{port}: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(300)))
        .map_err(|e| e.to_string())?;
    let mut req = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n",
        body.len()
    );
    for (k, v) in headers {
        req.push_str(k);
        req.push_str(": ");
        req.push_str(v);
        req.push_str("\r\n");
    }
    req.push_str("\r\n");
    stream
        .write_all(req.as_bytes())
        .map_err(|e| e.to_string())?;
    stream.write_all(body).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).map_err(|e| e.to_string())?;
    parse_response(&raw)
}

fn parse_base(base: &str) -> Result<(String, u16), String> {
    let rest = base
        .strip_prefix("http://")
        .ok_or_else(|| format!("url must start with http:// (got {base}); TLS is not supported"))?;
    let rest = rest.trim_end_matches('/');
    let (host, port) = match rest.rsplit_once(':') {
        Some((h, p)) if !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()) => {
            (h, p.parse::<u16>().map_err(|e| e.to_string())?)
        }
        _ => (rest, 80),
    };
    if host.is_empty() || host.contains('/') {
        return Err(format!("bad url {base}"));
    }
    Ok((host.to_string(), port))
}

fn parse_response(raw: &[u8]) -> Result<(u16, String), String> {
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("malformed HTTP response: no header end")?;
    let head = std::str::from_utf8(&raw[..split]).map_err(|e| e.to_string())?;
    let mut lines = head.split("\r\n");
    let status_line = lines.next().unwrap_or("");
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("malformed status line: {status_line}"))?;
    let chunked = lines.any(|l| {
        let l = l.to_ascii_lowercase();
        l.starts_with("transfer-encoding:") && l.contains("chunked")
    });
    let body_raw = &raw[split + 4..];
    let body = if chunked {
        dechunk(body_raw)?
    } else {
        body_raw.to_vec()
    };
    Ok((status, String::from_utf8_lossy(&body).into_owned()))
}

fn dechunk(mut data: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(data.len());
    loop {
        let line_end = data
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or("malformed chunked body: no size line")?;
        let size_text = std::str::from_utf8(&data[..line_end]).map_err(|e| e.to_string())?;
        let size_hex = size_text.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size_hex, 16)
            .map_err(|_| format!("malformed chunk size: {size_text}"))?;
        data = &data[line_end + 2..];
        if size == 0 {
            return Ok(out);
        }
        if data.len() < size {
            return Err("truncated chunked body".into());
        }
        out.extend_from_slice(&data[..size]);
        data = &data[size..];
        data = data.strip_prefix(b"\r\n").unwrap_or(data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_urls() {
        assert_eq!(
            parse_base("http://127.0.0.1:8124").unwrap(),
            ("127.0.0.1".into(), 8124)
        );
        assert_eq!(
            parse_base("http://ch.local/").unwrap(),
            ("ch.local".into(), 80)
        );
        assert!(parse_base("https://x").is_err());
        assert!(parse_base("http://").is_err());
    }

    #[test]
    fn identifiers() {
        assert!(is_identifier("users"));
        assert!(is_identifier("db.t_1"));
        assert!(!is_identifier("users; DROP"));
        assert!(!is_identifier("1abc"));
        assert!(!is_identifier("a.b.c"));
    }

    #[test]
    fn chunked_and_plain_responses() {
        let raw = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\n{\"a\":\r\n3;ext\r\n1}\n\r\n0\r\n\r\n";
        assert_eq!(
            parse_response(raw).unwrap(),
            (200, "{\"a\":1}\n".to_string())
        );
        let raw = b"HTTP/1.1 400 Bad Request\r\nContent-Length: 4\r\n\r\noops";
        assert_eq!(parse_response(raw).unwrap(), (400, "oops".to_string()));
        assert!(dechunk(b"zz\r\n").is_err());
    }

    fn cfg() -> Config {
        Config {
            url: "http://127.0.0.1:1".into(),
            database: "default".into(),
            user: "default".into(),
            password: None,
            passthrough: false,
            min_bytes: 0,
            selector: None,
            max_rows: 10,
            log: false,
        }
    }

    #[test]
    fn protocol_without_a_server() {
        let c = cfg();
        let init = handle(&c, &json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18"}})).unwrap();
        assert_eq!(init["result"]["protocolVersion"], "2025-06-18");
        assert!(
            init["result"]["instructions"]
                .as_str()
                .unwrap()
                .contains("@records")
        );
        let list = handle(&c, &json!({"id": 2, "method": "tools/list"})).unwrap();
        assert_eq!(list["result"]["tools"].as_array().unwrap().len(), 3);
        assert!(handle(&c, &json!({"method": "notifications/initialized"})).is_none());
        let bad = handle(&c, &json!({"id": 3, "method": "tools/call", "params": {"name": "describe_table", "arguments": {"table": "x; drop"}}})).unwrap();
        assert_eq!(bad["error"]["code"], -32602);
        // Nothing listens on port 1: a tool error, not a protocol error.
        let down = handle(
            &c,
            &json!({"id": 4, "method": "tools/call", "params": {"name": "list_tables"}}),
        )
        .unwrap();
        assert_eq!(down["result"]["isError"], true);
    }
}
