use std::io::{Read, Write};
use std::path::Path;
use std::process::ExitCode;

use ldf_mcp::proxy::{self, Config};
use ldf_mcp::rewrite::Selector;
use ldf_mcp::stats::{self, Recorder};
use ldf_mcp::{clickhouse, install};
use std::sync::Arc;

fn usage() -> ExitCode {
    eprintln!(
        "usage:\n  ldf-mcp proxy [--passthrough] [--plain] [--min-bytes N] [--columnar] [--max-result-bytes N] [--quiet] [--model ID [--evidence FILE]] -- COMMAND [ARGS...]\n\
         \x20     run COMMAND as the MCP server and rewrite its JSON tool results into LDF\n\
         \x20     --passthrough  forward tool results unchanged (for A/B comparison)\n\
         \x20     --columnar     also convert results shaped {{\"columns\": [...], \"rows\": [[...], ...]}} (MotherDuck's\n\
         \x20                    DuckDB server): the result minus its rows as one JSON line, then the rows as LDF;\n\
         \x20                    sent only when it decodes back to the exact result\n\
         \x20     --max-result-bytes N  cut rows from a result still longer than N bytes after rewriting, keeping\n\
         \x20                    the first rows that fit, and add a note saying how many were shown (about 2.2 to 3\n\
         \x20                    bytes per o200k token for LDF and JSON); errors and non-tabular results are never cut\n\
         \x20     --plain        leave initialize instructions and tool descriptions alone\n\
         \x20     --min-bytes N  never convert a text block shorter than N bytes (default 1024)\n\
         \x20     --quiet        no per-result stderr line\n\
         \x20     --model ID     pick JSON or LDF per result for this model from measured accuracy (ldf-select);\n\
         \x20                    a model without evidence gets JSON throughout and no LDF primer; without\n\
         \x20                    --model, LDF whenever it is shorter. --model works in all three server modes\n\
         \x20     --evidence FILE  evidence rows to use instead of the built-in copy (eval/analysis/evidence.py)\n\
         \x20     --name NAME    label for this server in the statistics (default: the command's file name)\n\
         \x20     --no-stats     record nothing (also LDF_STATS=0); --name and --no-stats work in all three modes\n\
         \x20 ldf-mcp clickhouse [--url http://127.0.0.1:8123] [--database default] [--user default] [--max-rows N]\n\
         \x20                    [--passthrough] [--min-bytes N] [--quiet] [--model ID [--evidence FILE]]\n\
         \x20     an MCP server with list_tables, describe_table and run_select (read-only) over ClickHouse HTTP;\n\
         \x20     rows come back as LDF; the password is read from CLICKHOUSE_PASSWORD, never from arguments\n\
         \x20 ldf-mcp files --root DIR [--max-rows N] [--passthrough] [--min-bytes N] [--quiet] [--model ID [--evidence FILE]]\n\
         \x20     an MCP server over a directory of json, jsonl, csv, tsv and parquet files: list_files,\n\
         \x20     describe_file and read_rows (paged, streamed), rows as LDF, paths confined to DIR\n\
         \x20 ldf-mcp convert [FILE] [--sparse T] [--hoist] [--block N]\n\
         \x20 ldf-mcp convert --columnar [--decode] [FILE]\n\
         \x20     a columnar JSON result to the frame --columnar sends (exit 3 and the reason when it would stay\n\
         \x20     JSON), or with --decode a frame back to the result as compact JSON\n\
         \x20     print FILE (json, jsonl, csv, tsv, parquet) or JSON from stdin as LDF; --sparse T moves keys\n\
         \x20     filled in fewer than T (0..1) of the rows into a per-row ~ object instead of a column;\n\
         \x20     --hoist writes an array of flat objects with one key list as a [k1,k2] column (LDF v0.2);\n\
         \x20     --block N streams FILE through the block encoder N records at a time (bounded memory)\n\
         \x20 ldf-mcp hook claude-code [--min-bytes N] [--model ID] [--evidence FILE] [--no-stats]\n\
         \x20     a Claude Code command hook (PostToolUse on Bash, SessionStart, PostModelSwitch): JSON datasets\n\
         \x20     in Bash stdout reach the model as LDF when shorter and the selector agrees for the session's\n\
         \x20     model; anything else prints nothing, which keeps the original output. Always exits 0\n\
         \x20 ldf-mcp stats [--days N] [--json] [--file PATH]\n\
         \x20     summarize the local statistics: responses, LDF or JSON and why, bytes, conversion time;\n\
         \x20     the file is LDF_STATS_FILE or ldf/events.jsonl under XDG_STATE_HOME, ~/.local/state or LOCALAPPDATA\n\
         \x20 ldf-mcp install claude-code [--hooks] [--wrap NAME]... [--wrap-all] [--add-files DIR [--files-name NAME]\n\
         \x20                             [--scope local|user]] [--model ID] [--apply]\n\
         \x20     --hooks adds the Bash output hook (ldf-mcp hook claude-code) to Claude Code's user settings.json;\n\
         \x20     --wrap and --add-files put stdio MCP servers behind ldf-mcp proxy, or add a files server, through\n\
         \x20     `claude mcp`; without --apply only prints the plan. Every change is recorded for uninstall\n\
         \x20 ldf-mcp uninstall claude-code [--apply]\n\
         \x20     restore what install changed (servers and hooks), where it is still what install wrote\n\
         \x20 ldf-mcp doctor\n\
         \x20     this binary, the statistics file, agents on PATH, Claude Code servers and their state\n\
         \x20 ldf-mcp --version\n\
         \x20 ldf-mcp fake-server [--ignore-eof]\n\
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
        Some("files") => run_files(&args[1..]),
        Some("stats") => run_stats(&args[1..]),
        Some("hook") => run_hook(&args[1..]),
        Some("install") => run_install(&args[1..]),
        Some("uninstall") => run_uninstall(&args[1..]),
        Some("doctor") => match install::Env::detect() {
            Ok(env) => {
                print!("{}", install::doctor(&env));
                ExitCode::SUCCESS
            }
            Err(e) => fail("ldf-mcp doctor", e),
        },
        Some("--version" | "-V") => {
            println!("ldf-mcp {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        // Tests stand in for the claude CLI with this binary.
        Some("mcp") if std::env::var("LDF_FAKE_CLAUDE").is_ok_and(|v| v == "1") => {
            match install::fake_claude(&args) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => fail("fake-claude", e),
            }
        }
        Some("fake-server") => {
            match ldf_mcp::fake::run(args[1..].iter().any(|a| a == "--ignore-eof")) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("ldf-mcp fake-server: {e}");
                    ExitCode::from(1)
                }
            }
        }
        _ => usage(),
    }
}

fn run_proxy(args: &[String]) -> ExitCode {
    let mut cfg = Config {
        command: Vec::new(),
        passthrough: false,
        plain: false,
        min_bytes: 1024,
        columnar: false,
        max_bytes: None,
        log: true,
        selector: None,
        stats: Arc::new(Recorder::off()),
    };
    let mut sel = SelectFlags::default();
    let mut st = StatsFlags::default();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if st.take(a, &mut it) {
            continue;
        }
        match a.as_str() {
            "--passthrough" => cfg.passthrough = true,
            "--plain" => cfg.plain = true,
            "--columnar" => cfg.columnar = true,
            "--max-result-bytes" => match it.next().and_then(|v| v.parse().ok()) {
                Some(n) if n > 0 => cfg.max_bytes = Some(n),
                _ => return usage(),
            },
            "--model" | "--evidence" => match it.next() {
                Some(v) => sel.set(a, v),
                None => return usage(),
            },
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
    if cfg.command.is_empty() || st.bad {
        return usage();
    }
    let label = std::path::Path::new(&cfg.command[0])
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    cfg.stats = st.recorder("proxy", &label, sel.model.as_deref());
    cfg.selector = match sel.build() {
        Ok(s) => s,
        Err(e) => return fail("ldf-mcp", e),
    };
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
        password: std::env::var("CLICKHOUSE_PASSWORD")
            .ok()
            .filter(|p| !p.is_empty()),
        passthrough: false,
        min_bytes: 1024,
        max_rows: 10_000,
        log: true,
        selector: None,
        stats: Arc::new(Recorder::off()),
    };
    let mut sel = SelectFlags::default();
    let mut st = StatsFlags::default();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if st.take(a, &mut it) {
            continue;
        }
        match a.as_str() {
            "--passthrough" => cfg.passthrough = true,
            "--quiet" => cfg.log = false,
            "--url" | "--database" | "--user" | "--max-rows" | "--min-bytes" | "--model"
            | "--evidence" => {
                let Some(v) = it.next() else { return usage() };
                match a.as_str() {
                    "--model" | "--evidence" => sel.set(a, v),
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
    if st.bad {
        return usage();
    }
    cfg.stats = st.recorder("clickhouse", "clickhouse", sel.model.as_deref());
    cfg.selector = match sel.build() {
        Ok(s) => s,
        Err(e) => return fail("ldf-mcp clickhouse", e),
    };
    match clickhouse::run(cfg) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ldf-mcp clickhouse: {e}");
            ExitCode::from(1)
        }
    }
}

fn run_files(args: &[String]) -> ExitCode {
    let mut root: Option<String> = None;
    let mut cfg = ldf_mcp::files::Config {
        root: std::path::PathBuf::new(),
        max_rows: 1000,
        passthrough: false,
        min_bytes: 1024,
        log: true,
        selector: None,
        stats: Arc::new(Recorder::off()),
    };
    let mut sel = SelectFlags::default();
    let mut st = StatsFlags::default();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if st.take(a, &mut it) {
            continue;
        }
        match a.as_str() {
            "--passthrough" => cfg.passthrough = true,
            "--quiet" => cfg.log = false,
            "--model" | "--evidence" => match it.next() {
                Some(v) => sel.set(a, v),
                None => return usage(),
            },
            "--root" => match it.next() {
                Some(v) => root = Some(v.clone()),
                None => return usage(),
            },
            "--max-rows" => match it.next().and_then(|v| v.parse().ok()) {
                Some(n) if n > 0 => cfg.max_rows = n,
                _ => return usage(),
            },
            "--min-bytes" => match it.next().and_then(|v| v.parse().ok()) {
                Some(n) => cfg.min_bytes = n,
                None => return usage(),
            },
            _ => return usage(),
        }
    }
    let Some(root) = root else { return usage() };
    if st.bad {
        return usage();
    }
    cfg.stats = st.recorder("files", "files", sel.model.as_deref());
    cfg.root = std::path::PathBuf::from(root);
    cfg.selector = match sel.build() {
        Ok(s) => s,
        Err(e) => return fail("ldf-mcp files", e),
    };
    match ldf_mcp::files::run(cfg) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ldf-mcp files: {e}");
            ExitCode::from(1)
        }
    }
}

/// `--no-stats` and `--name NAME`, shared by the three server modes.
#[derive(Default)]
struct StatsFlags {
    off: bool,
    name: Option<String>,
    bad: bool,
}

impl StatsFlags {
    /// Consumes `a` (and its value) when it is one of these flags.
    fn take<'a>(&mut self, a: &str, it: &mut impl Iterator<Item = &'a String>) -> bool {
        match a {
            "--no-stats" => self.off = true,
            "--name" => match it.next() {
                Some(v) => self.name = Some(v.clone()),
                None => self.bad = true,
            },
            _ => return false,
        }
        true
    }

    fn recorder(
        &self,
        mode: &'static str,
        default_name: &str,
        model: Option<&str>,
    ) -> Arc<Recorder> {
        let path = if self.off {
            None
        } else {
            stats::default_path()
        };
        let name = self.name.as_deref().unwrap_or(default_name);
        Arc::new(Recorder::new(path, mode, name, model))
    }
}

fn run_stats(args: &[String]) -> ExitCode {
    let mut as_json = false;
    let mut days: Option<u64> = None;
    let mut file: Option<std::path::PathBuf> = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--json" => as_json = true,
            "--days" => match it.next().and_then(|v| v.parse().ok()) {
                Some(n) => days = Some(n),
                None => return usage(),
            },
            "--file" => match it.next() {
                Some(v) => file = Some(v.into()),
                None => return usage(),
            },
            _ => return usage(),
        }
    }
    let Some(path) = file.or_else(stats::default_path) else {
        return fail(
            "ldf-mcp stats",
            "no statistics file: LDF_STATS=0, or no HOME / LOCALAPPDATA; pass --file".into(),
        );
    };
    let since = days.map(|d| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|n| n.as_secs())
            .unwrap_or(0)
            .saturating_sub(d * 86_400)
    });
    match stats::summarize(&path, since) {
        Ok(s) if as_json => {
            println!("{}", stats::to_json(&s));
            ExitCode::SUCCESS
        }
        Ok(s) => {
            print!("{}", stats::render(&s, &path));
            ExitCode::SUCCESS
        }
        Err(e) => fail("ldf-mcp stats", format!("{}: {e}", path.display())),
    }
}

fn run_install(args: &[String]) -> ExitCode {
    if args.first().map(String::as_str) != Some(install::AGENT) {
        return usage();
    }
    let mut a = install::InstallArgs::default();
    let mut it = args[1..].iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--apply" => a.apply = true,
            "--hooks" => a.hooks = true,
            "--wrap-all" => a.wrap_all = true,
            "--wrap" | "--add-files" | "--files-name" | "--model" | "--scope" => {
                let Some(v) = it.next() else { return usage() };
                match arg.as_str() {
                    "--wrap" => a.wrap.push(v.clone()),
                    "--add-files" => a.add_files = Some(v.into()),
                    "--files-name" => a.files_name = Some(v.clone()),
                    "--model" => a.model = Some(v.clone()),
                    _ => match v.as_str() {
                        "local" => a.scope = Some(install::Scope::Local),
                        "user" => a.scope = Some(install::Scope::User),
                        _ => return usage(),
                    },
                }
            }
            _ => return usage(),
        }
    }
    let mcp = !a.wrap.is_empty() || a.wrap_all || a.add_files.is_some();
    let run = |env: install::Env| -> Result<String, String> {
        let mut text = String::new();
        if a.hooks {
            text.push_str(&install::install_hooks(&env, &a)?);
        }
        if mcp || !a.hooks {
            if a.hooks {
                text.push('\n');
            }
            text.push_str(&install::install(&env, &a)?);
        }
        Ok(text)
    };
    match install::Env::detect().and_then(run) {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => fail("ldf-mcp install", e),
    }
}

fn run_uninstall(args: &[String]) -> ExitCode {
    let apply = match args {
        [agent] if agent == install::AGENT => false,
        [agent, flag] if agent == install::AGENT && flag == "--apply" => true,
        _ => return usage(),
    };
    match install::Env::detect().and_then(|env| install::uninstall(&env, apply)) {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => fail("ldf-mcp uninstall", e),
    }
}

fn run_hook(args: &[String]) -> ExitCode {
    // Exit 2 would put stderr in front of the model on PostToolUse; a
    // usage error here exits 1, which Claude Code shows to the user only.
    let bad = || {
        eprintln!(
            "ldf-mcp hook: usage: ldf-mcp hook claude-code [--min-bytes N] [--model ID] [--evidence FILE] [--no-stats]"
        );
        ExitCode::from(1)
    };
    if args.first().map(String::as_str) != Some("claude-code") {
        return bad();
    }
    let mut cfg = ldf_mcp::hook::Config {
        min_bytes: 1024,
        model: None,
        evidence: None,
        sessions: stats::ldf_state_dir().map(|d| d.join("claude-code").join("sessions")),
    };
    let mut sel = SelectFlags::default();
    let mut st = StatsFlags::default();
    let mut it = args[1..].iter();
    while let Some(a) = it.next() {
        if st.take(a, &mut it) {
            continue;
        }
        match a.as_str() {
            "--model" | "--evidence" => match it.next() {
                Some(v) => sel.set(a, v),
                None => return bad(),
            },
            "--min-bytes" => match it.next().and_then(|v| v.parse().ok()) {
                Some(n) => cfg.min_bytes = n,
                None => return bad(),
            },
            _ => return bad(),
        }
    }
    if st.bad {
        return bad();
    }
    // The hook learns the model from the session, so unlike the server
    // modes it takes --evidence on its own. A file that cannot be read
    // leaves the built-in evidence: the hook never fails the tool call.
    if sel.model.is_none() {
        cfg.evidence = sel.evidence.as_deref().and_then(|p| load_evidence(p).ok());
    } else if let Some(s) = sel.build().unwrap_or(None) {
        cfg.model = Some(s.model);
        cfg.evidence = Some(s.evidence);
    }
    let stats_path = if st.off { None } else { stats::default_path() };
    // Whatever happens, Claude Code gets either a whole reply or nothing,
    // and nothing keeps the original tool output.
    std::panic::set_hook(Box::new(|_| {}));
    let _ = std::panic::catch_unwind(|| ldf_mcp::hook::run(&cfg, stats_path));
    ExitCode::SUCCESS
}

fn fail(who: &str, e: String) -> ExitCode {
    eprintln!("{who}: {e}");
    ExitCode::from(1)
}

fn load_evidence(p: &str) -> Result<Vec<ldf_select::Evidence>, String> {
    std::fs::read(p)
        .map_err(|e| e.to_string())
        .and_then(|b| serde_json::from_slice(&b).map_err(|e| e.to_string()))
        .map_err(|e| format!("{p}: {e}"))
}

/// `--model` and `--evidence`, shared by the three server modes.
#[derive(Default)]
struct SelectFlags {
    model: Option<String>,
    evidence: Option<String>,
}

impl SelectFlags {
    fn set(&mut self, flag: &str, v: &str) {
        match flag {
            "--model" => self.model = Some(v.to_string()),
            _ => self.evidence = Some(v.to_string()),
        }
    }

    fn build(self) -> Result<Option<Selector>, String> {
        let Some(model) = self.model else {
            return match self.evidence {
                Some(_) => Err("--evidence needs --model".into()),
                None => Ok(None),
            };
        };
        let evidence = self.evidence.as_deref().map(load_evidence).transpose()?;
        let sel = Selector::new(&model, evidence);
        if !sel.knows_model() {
            eprintln!(
                "ldf-mcp: no evidence for model {model} with at least {} questions; every result stays JSON",
                sel.policy.min_n
            );
        }
        Ok(Some(sel))
    }
}

fn convert(args: &[String]) -> ExitCode {
    if args.iter().any(|a| a == "--columnar") {
        return convert_columnar(args);
    }
    let mut path: Option<&str> = None;
    let mut opts = ldf::plan::Options::default();
    let mut block: Option<usize> = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--sparse" => match it.next().and_then(|v| v.parse::<f64>().ok()) {
                Some(t) if (0.0..=1.0).contains(&t) => opts.sparse_threshold = t,
                _ => return usage(),
            },
            "--block" => match it.next().and_then(|v| v.parse::<usize>().ok()) {
                Some(n) if n > 0 => block = Some(n),
                _ => return usage(),
            },
            "--hoist" => opts.hoist = true,
            _ if path.is_none() => path = Some(a.as_str()),
            _ => return usage(),
        }
    }
    if let Some(p) = path.filter(|p| p.ends_with(".parquet"))
        && let Some(code) = convert_parquet_direct(p, block, opts)
    {
        return code;
    }
    if let Some(n) = block {
        return convert_streaming(path, n, opts);
    }
    let records = match path {
        Some(p) => {
            ldf_data::input::read_all(Path::new(p), &ldf_data::input::ReadOptions::default())
                .map_err(|e| e.to_string())
        }
        None => {
            let mut bytes = Vec::new();
            std::io::stdin()
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())
                .and_then(|_| {
                    ldf_data::input::parse_json_document(
                        bytes,
                        ldf_data::input::JsonEngine::SerdeJson,
                    )
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
    if stdout
        .write_all(out.as_bytes())
        .and_then(|_| stdout.flush())
        .is_err()
    {
        return ExitCode::from(1);
    }
    eprintln!(
        "ldf-mcp convert: {} records, {} bytes of LDF",
        records.len(),
        out.len()
    );
    ExitCode::SUCCESS
}

/// Parquet through the direct Arrow encoder (`ldf::arrow`), which writes
/// the same bytes as the record path two to four times faster. `None` when
/// the schema needs the record path (a list of structs is a child table).
fn convert_parquet_direct(
    path: &str,
    block: Option<usize>,
    opts: ldf::plan::Options,
) -> Option<ExitCode> {
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    let fail = |e: String| {
        eprintln!("ldf-mcp convert: {e}");
        Some(ExitCode::from(1))
    };
    let builder = match std::fs::File::open(path)
        .map_err(|e| e.to_string())
        .and_then(|f| ParquetRecordBatchReaderBuilder::try_new(f).map_err(|e| e.to_string()))
    {
        Ok(b) => b,
        Err(e) => return fail(e),
    };
    ldf::arrow::check_schema(builder.schema()).ok()?;
    let reader = match builder.build() {
        Ok(r) => r,
        Err(e) => return fail(e.to_string()),
    };
    let mut stdout = std::io::stdout().lock();
    let mut buf = String::new();
    let mut records = 0usize;
    let mut bytes = 0usize;
    let mut enc = block.map(|n| ldf::arrow::ArrowBlockEncoder::new(n, opts));
    let mut whole = Vec::new();
    for batch in reader {
        let batch = match batch {
            Ok(b) => b,
            Err(e) => return fail(e.to_string()),
        };
        records += batch.num_rows();
        match enc.as_mut() {
            Some(enc) => {
                if let Err(e) = enc.batch(&batch, &mut buf) {
                    return fail(e.to_string());
                }
                if !buf.is_empty() {
                    bytes += buf.len();
                    if stdout.write_all(buf.as_bytes()).is_err() {
                        return Some(ExitCode::from(1));
                    }
                    buf.clear();
                }
            }
            None => whole.push(batch),
        }
    }
    let tail = match enc.as_mut() {
        Some(enc) => enc.finish(&mut buf).map(|_| std::mem::take(&mut buf)),
        None => ldf::arrow::encode_batches(&whole, &opts),
    };
    let tail = match tail {
        Ok(t) => t,
        Err(e) => return fail(e.to_string()),
    };
    bytes += tail.len();
    if stdout
        .write_all(tail.as_bytes())
        .and_then(|_| stdout.flush())
        .is_err()
    {
        return Some(ExitCode::from(1));
    }
    match block {
        Some(n) => eprintln!(
            "ldf-mcp convert: {records} records, {bytes} bytes of LDF, blocks of {n} (Arrow path)"
        ),
        None => eprintln!("ldf-mcp convert: {records} records, {bytes} bytes of LDF (Arrow path)"),
    }
    Some(ExitCode::SUCCESS)
}

/// Streams a file through the block encoder: one block of records in
/// memory at a time, output written as each block completes.
fn convert_streaming(path: Option<&str>, block: usize, opts: ldf::plan::Options) -> ExitCode {
    let Some(p) = path else {
        eprintln!("ldf-mcp convert: --block needs a FILE (stdin is read whole)");
        return ExitCode::from(2);
    };
    let stream = match ldf_data::input::open(Path::new(p), &ldf_data::input::ReadOptions::default())
    {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ldf-mcp convert: {e}");
            return ExitCode::from(1);
        }
    };
    let mut enc = ldf::BlockEncoder::with_options(block, opts);
    let mut stdout = std::io::stdout().lock();
    let mut buf = String::new();
    let mut records = 0usize;
    let mut bytes = 0usize;
    for rec in stream {
        let rec = match rec {
            Ok(r) => r,
            Err(e) => {
                eprintln!("ldf-mcp convert: {e}");
                return ExitCode::from(1);
            }
        };
        records += 1;
        enc.record(rec, &mut buf);
        if !buf.is_empty() {
            bytes += buf.len();
            if stdout.write_all(buf.as_bytes()).is_err() {
                return ExitCode::from(1);
            }
            buf.clear();
        }
    }
    enc.finish(&mut buf);
    bytes += buf.len();
    if stdout
        .write_all(buf.as_bytes())
        .and_then(|_| stdout.flush())
        .is_err()
    {
        return ExitCode::from(1);
    }
    eprintln!("ldf-mcp convert: {records} records, {bytes} bytes of LDF, blocks of {block}");
    ExitCode::SUCCESS
}

/// `convert --columnar [--decode] [FILE]`: the proxy's columnar frame for a
/// result, or the result for a frame.
fn convert_columnar(args: &[String]) -> ExitCode {
    let decode = args.iter().any(|a| a == "--decode");
    let rest: Vec<&String> = args
        .iter()
        .filter(|a| *a != "--columnar" && *a != "--decode")
        .collect();
    let text = match rest.as_slice() {
        [] => {
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s).map(|_| s)
        }
        [p] if !p.starts_with("--") => std::fs::read_to_string(p),
        _ => return usage(),
    };
    let text = match text {
        Ok(t) => t,
        Err(e) => return fail("ldf-mcp convert", e.to_string()),
    };
    let out = if decode {
        match ldf_mcp::rewrite::decode_columnar_frame(&text) {
            Ok(v) => {
                let mut s = String::new();
                ldf::text::push_json_value(&v, &mut s);
                s.push('\n');
                s
            }
            Err(e) => return fail("ldf-mcp convert", e),
        }
    } else {
        match ldf_mcp::rewrite::convert_with(&text, 0, None, true) {
            Ok(frame) => frame,
            Err(why) => {
                eprintln!("ldf-mcp convert: stays JSON: {}", why.code());
                return ExitCode::from(3);
            }
        }
    };
    let mut stdout = std::io::stdout().lock();
    if stdout
        .write_all(out.as_bytes())
        .and_then(|_| stdout.flush())
        .is_err()
    {
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}
