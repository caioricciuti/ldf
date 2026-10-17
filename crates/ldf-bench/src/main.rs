//! `ldf-bench`: baseline benchmark CLI.
//!
//! ```text
//! ldf-bench roundtrip  --input F... | --scale S [--dataset a,b] [--formats all]
//! ldf-bench perf       --input F... | --scale S [--dataset a,b] [--formats all]
//!                      [--iters 7] [--engines serde_json,simd-json] [--out results/<ts>]
//!                      [--in-process]
//! ldf-bench tokens     --input F... | --scale S [--dataset a,b] [--formats all]
//!                      [--tokenizers o200k_base,cl100k_base] [--sample N] [--out results/<ts>]
//! ldf-bench count      [--tokenizers o200k_base,cl100k_base] FILE...   (tokens of any text files)
//! ldf-bench sysinfo
//! ldf-bench child ...  (internal: one measurement in a fresh process)
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use ldf_bench::measure::{Measurement, Op, Plan, run_decode, run_encode, run_io, run_parse};
use ldf_bench::report::{DatasetResult, FormatTokens, Run, render_markdown};
use ldf_bench::roundtrip;
use ldf_bench::tokens::Tokenizers;
use ldf_bench::{Input, kind_from_name, kind_name, timestamp_compact};
use ldf_data::input::JsonEngine;
use ldf_formats::{EncodeError, Format};

struct Args {
    cmd: String,
    flags: Vec<(String, String)>,
}

impl Args {
    fn parse() -> Args {
        let mut it = std::env::args().skip(1);
        let cmd = it.next().unwrap_or_default();
        let mut flags = Vec::new();
        let rest: Vec<String> = it.collect();
        let mut i = 0;
        while i < rest.len() {
            let a = &rest[i];
            if let Some(name) = a.strip_prefix("--") {
                let next = rest.get(i + 1);
                if let Some(v) = next.filter(|v| !v.starts_with("--")) {
                    flags.push((name.to_string(), v.clone()));
                    i += 2;
                } else {
                    flags.push((name.to_string(), "true".into()));
                    i += 1;
                }
            } else {
                flags.push(("input".into(), a.clone()));
                i += 1;
            }
        }
        Args { cmd, flags }
    }
    fn get(&self, name: &str) -> Option<&str> {
        self.flags
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }
    fn all(&self, name: &str) -> Vec<&str> {
        self.flags
            .iter()
            .filter(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
            .collect()
    }
    fn has(&self, name: &str) -> bool {
        self.get(name).is_some()
    }
    fn num<T: std::str::FromStr>(&self, name: &str, default: T) -> T {
        self.get(name)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }
}

fn usage() -> ExitCode {
    eprintln!(
        "usage:\n  ldf-bench roundtrip|perf|tokens (--input FILE)... | --scale small|medium|large [--dataset a,b]\n\
         \x20   [--formats all|a,b] [--iters N] [--engines serde_json,simd-json] [--out DIR] [--in-process]\n\
         \x20   [--tokenizers o200k_base,cl100k_base] [--sample N]\n  ldf-bench sysinfo"
    );
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args = Args::parse();
    match args.cmd.as_str() {
        "sysinfo" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&ldf_bench::sys::sysinfo()).unwrap()
            );
            ExitCode::SUCCESS
        }
        "child" => child(&args),
        "count" => match count(&args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
        "roundtrip" | "perf" | "tokens" => match orchestrate(&args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
        _ => usage(),
    }
}

// ---- token counts of arbitrary files --------------------------------------------

fn count(args: &Args) -> Result<(), String> {
    let spec = args.get("tokenizers").unwrap_or("o200k_base,cl100k_base");
    let names: Vec<&str> = spec.split(',').map(str::trim).collect();
    let tk = Tokenizers::new(&names)?;
    let files = args.all("input");
    if files.is_empty() {
        return Err("pass one or more files".into());
    }
    println!(
        "| file | bytes | {} |",
        names
            .iter()
            .map(|n| format!("{n} | bytes/token"))
            .collect::<Vec<_>>()
            .join(" | ")
    );
    println!(
        "|---|---:|{}",
        names.iter().map(|_| "---:|---:|").collect::<String>()
    );
    for f in files {
        let text = fs::read_to_string(f).map_err(|e| format!("{f}: {e}"))?;
        let counts = tk.count(&text);
        let cells: Vec<String> = counts
            .iter()
            .map(|c| {
                format!(
                    "{} | {:.2}",
                    c.tokens,
                    if c.tokens > 0 {
                        text.len() as f64 / c.tokens as f64
                    } else {
                        0.0
                    }
                )
            })
            .collect();
        println!(
            "| {} | {} | {} |",
            Path::new(f)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(f),
            text.len(),
            cells.join(" | ")
        );
    }
    Ok(())
}

// ---- input selection ----------------------------------------------------------

fn select_inputs(args: &Args) -> Result<Vec<Input>, String> {
    let mut inputs = Vec::new();
    for p in args.all("input") {
        inputs.push(Input::from_path(Path::new(p))?);
    }
    if let Some(scale) = args.get("scale") {
        let dir = PathBuf::from(args.get("fixtures").unwrap_or("fixtures")).join(scale);
        let wanted: Option<Vec<&str>> = args.get("dataset").map(|d| d.split(',').collect());
        let kind = args.get("kind").unwrap_or("json");
        for spec in ldf_fixtures::catalog() {
            if wanted.as_ref().is_some_and(|w| !w.contains(&spec.name)) {
                continue;
            }
            let path = dir.join(format!("{}.{kind}", spec.name));
            if !path.exists() {
                return Err(format!(
                    "{} not found; run `ldf-gen --scale {scale}` first",
                    path.display()
                ));
            }
            inputs.push(Input::from_path(&path)?);
        }
    }
    if inputs.is_empty() {
        return Err("no inputs: pass --input FILE or --scale small|medium|large".into());
    }
    Ok(inputs)
}

fn engines(args: &Args) -> Result<Vec<JsonEngine>, String> {
    let spec = args.get("engines").unwrap_or("serde_json,simd-json");
    spec.split(',')
        .map(|e| match e.trim() {
            "serde_json" | "serde" => Ok(JsonEngine::SerdeJson),
            #[cfg(feature = "simd")]
            "simd-json" | "simd" => Ok(JsonEngine::SimdJson),
            other => Err(format!("unknown engine {other:?}")),
        })
        .collect()
}

fn engine_flag(e: JsonEngine) -> &'static str {
    match e {
        JsonEngine::SerdeJson => "serde_json",
        #[cfg(feature = "simd")]
        JsonEngine::SimdJson => "simd-json",
    }
}

// ---- child process: one measurement ----------------------------------------------

fn child(args: &Args) -> ExitCode {
    let Some(input) = args.get("input") else {
        return usage();
    };
    let Some(kind) = args.get("kind").and_then(kind_from_name) else {
        return usage();
    };
    let Some(op) = args.get("op").and_then(Op::parse) else {
        return usage();
    };
    let engine = match engines(&Args {
        cmd: String::new(),
        flags: vec![(
            "engines".into(),
            args.get("engine").unwrap_or("serde_json").into(),
        )],
    }) {
        Ok(e) => e[0],
        Err(_) => return usage(),
    };
    let plan = Plan {
        input: Path::new(input),
        kind,
        engine,
        iters: args.num("iters", 1usize).max(1),
    };
    let m = match op {
        Op::Io => run_io(&plan),
        Op::Parse => run_parse(&plan),
        Op::Encode | Op::Decode => {
            let Some(f) = args.get("format").and_then(ldf_formats::by_name) else {
                return usage();
            };
            if op == Op::Encode {
                run_encode(&plan, f.as_ref())
            } else {
                run_decode(&plan, f.as_ref())
            }
        }
    };
    println!("{}", serde_json::to_string(&m).unwrap());
    ExitCode::SUCCESS
}

fn spawn_child(
    input: &Input,
    op: Op,
    engine: JsonEngine,
    format: Option<&str>,
    iters: usize,
) -> Result<Measurement, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut cmd = Command::new(exe);
    cmd.arg("child")
        .arg("--input")
        .arg(&input.path)
        .arg("--kind")
        .arg(kind_name(input.kind))
        .arg("--op")
        .arg(match op {
            Op::Io => "io",
            Op::Parse => "parse",
            Op::Encode => "encode",
            Op::Decode => "decode",
        })
        .arg("--engine")
        .arg(engine_flag(engine))
        .arg("--iters")
        .arg(iters.to_string())
        .stdin(Stdio::null())
        .stderr(Stdio::inherit());
    if let Some(f) = format {
        cmd.arg("--format").arg(f);
    }
    let out = cmd.output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!("child exited with {}", out.status));
    }
    serde_json::from_slice(&out.stdout).map_err(|e| format!("bad child output: {e}"))
}

// ---- orchestrator ----------------------------------------------------------------

fn orchestrate(args: &Args) -> Result<(), String> {
    let inputs = select_inputs(args)?;
    let formats = ldf_formats::select(args.get("formats").unwrap_or("all"))?;
    let iters: usize = args.num("iters", 7usize).max(1);
    let in_process = args.has("in-process");
    let out_dir = PathBuf::from(
        args.get("out")
            .map(String::from)
            .unwrap_or_else(|| format!("results/{}", timestamp_compact())),
    );
    let tokenizers_spec = args.get("tokenizers").unwrap_or("o200k_base,cl100k_base");
    let sample: Option<usize> = args.get("sample").and_then(|s| s.parse().ok());
    let want_perf = args.cmd == "perf";
    let want_tokens = args.cmd == "tokens";

    let mut run = Run {
        tool: "ldf-bench",
        started_utc: timestamp_compact(),
        command: std::env::args().collect(),
        iters,
        sysinfo: ldf_bench::sys::sysinfo(),
        datasets: Vec::new(),
    };
    let tokenizers = if want_tokens {
        Some(Tokenizers::new(
            &tokenizers_spec
                .split(',')
                .map(str::trim)
                .collect::<Vec<_>>(),
        )?)
    } else {
        None
    };
    if want_tokens || want_perf {
        fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    }

    for input in &inputs {
        eprintln!("== {} ({})", input.name, input.path.display());
        let plan = Plan {
            input: &input.path,
            kind: input.kind,
            engine: JsonEngine::SerdeJson,
            iters: 1,
        };
        let mut records = ldf_bench::measure::load(&plan)?;
        if let Some(n) = sample
            && records.len() > n
        {
            records.truncate(n);
        }
        let mut ds = DatasetResult {
            name: input.name.clone(),
            path: input.path.display().to_string(),
            input_kind: kind_name(input.kind).into(),
            records: records.len() as u64,
            input_bytes: fs::metadata(&input.path).map(|m| m.len()).unwrap_or(0),
            ..Default::default()
        };

        // 1. Correctness first.
        for f in &formats {
            let rt = roundtrip::check(f.as_ref(), &records);
            eprintln!(
                "   {:<12} round trip: {}{}",
                rt.format,
                rt.level.label(),
                rt.detail
                    .as_deref()
                    .map(|d| format!(" ({d})"))
                    .unwrap_or_default()
            );
            ds.roundtrips.push(rt);
        }

        // 2. Tokens.
        if let Some(tk) = &tokenizers {
            let enc_dir = out_dir.join("encoded");
            fs::create_dir_all(&enc_dir).map_err(|e| e.to_string())?;
            for f in &formats {
                match f.encode_to_string(&records) {
                    Ok(text) => {
                        let path =
                            enc_dir.join(format!("{}.{}.{}", input.name, f.name(), f.extension()));
                        fs::write(&path, &text).map_err(|e| e.to_string())?;
                        let counts = tk.count(&text);
                        eprintln!(
                            "   {:<12} {:>10} bytes  {}",
                            f.name(),
                            text.len(),
                            counts
                                .iter()
                                .map(|c| format!("{}={}", c.tokenizer, c.tokens))
                                .collect::<Vec<_>>()
                                .join("  ")
                        );
                        ds.tokens.push(FormatTokens {
                            format: f.name().into(),
                            bytes: text.len() as u64,
                            records: records.len() as u64,
                            counts: counts.into_iter().map(Into::into).collect(),
                            unsupported: None,
                        });
                    }
                    Err(EncodeError::Unsupported(u)) => ds.tokens.push(FormatTokens {
                        format: f.name().into(),
                        bytes: 0,
                        records: records.len() as u64,
                        counts: vec![],
                        unsupported: Some(u.reason),
                    }),
                    Err(e) => return Err(e.to_string()),
                }
            }
        }

        // 3. Performance, one child process per measurement.
        if want_perf {
            let supported: Vec<&Box<dyn Format>> = formats
                .iter()
                .filter(|f| {
                    ds.roundtrips.iter().any(|r| {
                        r.format == f.name()
                            && r.level != roundtrip::Level::Unsupported
                            && r.level != roundtrip::Level::DecodeError
                    })
                })
                .collect();
            let mut jobs: Vec<(Op, JsonEngine, Option<&str>)> =
                vec![(Op::Io, JsonEngine::SerdeJson, None)];
            let parse_engines = if matches!(
                input.kind,
                ldf_data::input::InputKind::Json | ldf_data::input::InputKind::Jsonl
            ) {
                engines(args)?
            } else {
                vec![JsonEngine::SerdeJson]
            };
            for e in parse_engines {
                jobs.push((Op::Parse, e, None));
            }
            for f in &supported {
                jobs.push((Op::Encode, JsonEngine::SerdeJson, Some(f.name())));
                jobs.push((Op::Decode, JsonEngine::SerdeJson, Some(f.name())));
            }
            for (op, engine, format) in jobs {
                let m = if in_process {
                    let plan = Plan {
                        input: &input.path,
                        kind: input.kind,
                        engine,
                        iters,
                    };
                    match op {
                        Op::Io => run_io(&plan),
                        Op::Parse => run_parse(&plan),
                        Op::Encode => run_encode(
                            &plan,
                            ldf_formats::by_name(format.unwrap()).unwrap().as_ref(),
                        ),
                        Op::Decode => run_decode(
                            &plan,
                            ldf_formats::by_name(format.unwrap()).unwrap().as_ref(),
                        ),
                    }
                } else {
                    spawn_child(input, op, engine, format, iters)?
                };
                let label = format
                    .map(String::from)
                    .or(m.engine.clone())
                    .unwrap_or_default();
                match &m.error {
                    Some(e) => eprintln!("   {:?} {label}: error {e}", op),
                    None => {
                        let s = ldf_bench::stats::summarize(if m.secs.len() > 1 {
                            &m.secs[1..]
                        } else {
                            &m.secs
                        });
                        eprintln!(
                            "   {:<7} {:<12} cold {:>10}  median {:>10}  {:>8.1} MB/s  peak RSS {}",
                            format!("{op:?}").to_lowercase(),
                            label,
                            ldf_bench::stats::fmt_secs(m.secs[0]),
                            ldf_bench::stats::fmt_secs(s.median),
                            ldf_bench::stats::mbps(m.bytes, s.median),
                            ldf_bench::stats::fmt_bytes(m.usage_after.max_rss_bytes)
                        );
                    }
                }
                ds.measurements.push(m);
            }
        }
        run.datasets.push(ds);
    }

    if want_perf || want_tokens {
        let stem = if want_perf { "perf" } else { "tokens" };
        let json_path = out_dir.join(format!("{stem}.json"));
        let md_path = out_dir.join(format!("{stem}.md"));
        fs::write(&json_path, serde_json::to_string_pretty(&run).unwrap())
            .map_err(|e| e.to_string())?;
        fs::write(&md_path, render_markdown(&run)).map_err(|e| e.to_string())?;
        eprintln!("wrote {} and {}", json_path.display(), md_path.display());
    } else {
        print!("{}", render_markdown(&run));
    }
    Ok(())
}
