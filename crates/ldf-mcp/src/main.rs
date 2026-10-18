use std::io::{Read, Write};
use std::path::Path;
use std::process::ExitCode;

use ldf_mcp::clickhouse;
use ldf_mcp::proxy::{self, Config};

fn usage() -> ExitCode {
    eprintln!(
        "usage:\n  ldf-mcp proxy [--passthrough] [--plain] [--min-bytes N] [--quiet] -- COMMAND [ARGS...]\n\
         \x20     run COMMAND as the MCP server and rewrite its JSON tool results into LDF\n\
         \x20     --passthrough  forward tool results unchanged (for A/B comparison)\n\
         \x20     --plain        leave initialize instructions and tool descriptions alone\n\
         \x20     --min-bytes N  never convert a text block shorter than N bytes (default 1024)\n\
         \x20     --quiet        no per-result stderr line\n\
         \x20 ldf-mcp clickhouse [--url http://127.0.0.1:8123] [--database default] [--user default] [--max-rows N]\n\
         \x20                    [--passthrough] [--min-bytes N] [--quiet]\n\
         \x20     an MCP server with list_tables, describe_table and run_select (read-only) over ClickHouse HTTP;\n\
         \x20     rows come back as LDF; the password is read from CLICKHOUSE_PASSWORD, never from arguments\n\
         \x20 ldf-mcp convert [FILE] [--sparse T]\n\
         \x20     print FILE (json, jsonl, csv, tsv, parquet) or JSON from stdin as LDF; --sparse T moves keys\n\
         \x20     filled in fewer than T (0..1) of the rows into a per-row ~ object instead of a column\n\
         \x20 ldf-mcp fake-server\n\
         \x20     a stand-in downstream server with one tool, `query`, that returns generated rows"
    );
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("proxy") => run_proxy(&args[1..]),
        Some("convert") => convert(&args[1..]),
        Some("clickhouse") => run_clickhouse(&args[1..]),
        Some("fake-server") => match ldf_mcp::fake::run() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("ldf-mcp fake-server: {e}");
                ExitCode::from(1)
            }
        },
        _ => usage(),
    }
}

fn run_proxy(args: &[String]) -> ExitCode {
    let mut cfg = Config {
        command: Vec::new(),
        passthrough: false,
        plain: false,
        min_bytes: 1024,
        log: true,
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--passthrough" => cfg.passthrough = true,
            "--plain" => cfg.plain = true,
            "--quiet" => cfg.log = false,
            "--min-bytes" => match it.next().and_then(|v| v.parse().ok()) {
                Some(n) => cfg.min_bytes = n,
                None => return usage(),
            },
            "--" => {
                cfg.command = it.cloned().collect();
                break;
            }
            _ => return usage(),
        }
    }
    if cfg.command.is_empty() {
        return usage();
    }
    match proxy::run(cfg) {
        Ok(code) => ExitCode::from(code.clamp(0, 255) as u8),
        Err(e) => {
            eprintln!("ldf-mcp: {e}");
            ExitCode::from(1)
        }
    }
}

fn run_clickhouse(args: &[String]) -> ExitCode {
    let mut cfg = clickhouse::Config {
        url: "http://127.0.0.1:8123".into(),
        database: "default".into(),
        user: "default".into(),
        password: std::env::var("CLICKHOUSE_PASSWORD").ok().filter(|p| !p.is_empty()),
        passthrough: false,
        min_bytes: 1024,
        max_rows: 10_000,
        log: true,
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--passthrough" => cfg.passthrough = true,
            "--quiet" => cfg.log = false,
            "--url" | "--database" | "--user" | "--max-rows" | "--min-bytes" => {
                let Some(v) = it.next() else { return usage() };
                match a.as_str() {
                    "--url" => cfg.url = v.clone(),
                    "--database" => cfg.database = v.clone(),
                    "--user" => cfg.user = v.clone(),
                    "--max-rows" => match v.parse() {
                        Ok(n) => cfg.max_rows = n,
                        Err(_) => return usage(),
                    },
                    _ => match v.parse() {
                        Ok(n) => cfg.min_bytes = n,
                        Err(_) => return usage(),
                    },
                }
            }
            _ => return usage(),
        }
    }
    match clickhouse::run(cfg) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ldf-mcp clickhouse: {e}");
            ExitCode::from(1)
        }
    }
}

fn convert(args: &[String]) -> ExitCode {
    let mut path: Option<&str> = None;
    let mut opts = ldf::plan::Options::default();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--sparse" => match it.next().and_then(|v| v.parse::<f64>().ok()) {
                Some(t) if (0.0..=1.0).contains(&t) => opts.sparse_threshold = t,
                _ => return usage(),
            },
            _ if path.is_none() => path = Some(a.as_str()),
            _ => return usage(),
        }
    }
    let records = match path {
        Some(p) => ldf_data::input::read_all(Path::new(p), &ldf_data::input::ReadOptions::default())
            .map_err(|e| e.to_string()),
        None => {
            let mut bytes = Vec::new();
            std::io::stdin()
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())
                .and_then(|_| {
                    ldf_data::input::parse_json_document(bytes, ldf_data::input::JsonEngine::SerdeJson)
                        .map_err(|e| e.to_string())
                })
        }
    };
    let records = match records {
        Ok(r) => r,
        Err(e) => {
            eprintln!("ldf-mcp convert: {e}");
            return ExitCode::from(1);
        }
    };
    let out = ldf::encode_with(&records, &opts);
    let mut stdout = std::io::stdout().lock();
    if stdout.write_all(out.as_bytes()).and_then(|_| stdout.flush()).is_err() {
        return ExitCode::from(1);
    }
    eprintln!("ldf-mcp convert: {} records, {} bytes of LDF", records.len(), out.len());
    ExitCode::SUCCESS
}
