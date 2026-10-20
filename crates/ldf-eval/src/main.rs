//! `ldf-eval`: build evaluation bundles and score responses.
//!
//! ```text
//! ldf-eval build --input fixtures/small/flat_users.json [--records 50] [--per-kind 3] [--kinds a,b]
//!                [--formats all] [--seed 1] [--extra name=file]... [--candidate name=ID]...
//!                --out eval/out/flat_users
//! ldf-eval score --bundle eval/out/flat_users --responses eval/out/flat_users/responses.jsonl
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ldf_data::input::{ReadOptions, read_all};
use ldf_eval::bundle::{BundleInput, Manifest, Tokenizer, write_bundle};
use ldf_eval::score::{Response, grade, render_markdown};
use ldf_eval::tasks::{Generator, Task, TaskKind};

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}

fn usage() -> ExitCode {
    eprintln!(
        "usage:\n  ldf-eval build --input FILE --out DIR [--records N] [--per-kind N] [--kinds a,b] [--formats all|a,b] [--seed N]\n\
         \x20     [--extra name=file]... [--candidate name=ID]...\n\
         \x20 ldf-eval score --bundle DIR --responses FILE [--out FILE.md]"
    );
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("build") => build(&args),
        Some("score") => score(&args),
        _ => usage(),
    }
}

fn build(args: &[String]) -> ExitCode {
    let (Some(input), Some(out)) = (flag(args, "--input"), flag(args, "--out")) else {
        return usage();
    };
    let records_n: usize = flag(args, "--records")
        .and_then(|v| v.parse().ok())
        .unwrap_or(50);
    let per_kind: usize = flag(args, "--per-kind")
        .and_then(|v| v.parse().ok())
        .unwrap_or(3);
    let seed: u64 = flag(args, "--seed")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let mut formats =
        match ldf_formats::select(&flag(args, "--formats").unwrap_or_else(|| "all".into())) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::from(2);
            }
        };
    // --candidate name=ID: a configuration from `ldf-discover`, shown to the
    // model as `ldf` like every other LDF variant.
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a != "--candidate" {
            continue;
        }
        let Some((name, id)) = it.next().and_then(|s| s.split_once('=')) else {
            return usage();
        };
        let Some(config) = ldf_discover::Config::from_id(id) else {
            eprintln!("unknown candidate id {id:?}; see `ldf-discover space`");
            return ExitCode::from(2);
        };
        // Format names are &'static str; a CLI run builds a handful.
        let name: &'static str = Box::leak(name.to_string().into_boxed_str());
        formats.push(Box::new(ldf_discover::Candidate { config, name }));
    }
    let path = PathBuf::from(&input);
    let mut records = match read_all(&path, &ReadOptions::default()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    records.truncate(records_n);
    let mut tasks = Generator::new(&records, seed, per_kind).generate();
    // --kinds a,b: keep only these kinds. Applied after generation, so the
    // kept questions are the ones a full build asks.
    if let Some(list) = flag(args, "--kinds") {
        let wanted: Vec<&str> = list.split(',').map(str::trim).collect();
        if let Some(bad) = wanted
            .iter()
            .find(|w| !TaskKind::all().iter().any(|k| k.name() == **w))
        {
            eprintln!("unknown task kind {bad:?}");
            return ExitCode::from(2);
        }
        tasks.retain(|t| wanted.contains(&t.kind.name()));
    }
    if tasks.is_empty() {
        eprintln!("no tasks could be generated (no usable key field?)");
        return ExitCode::FAILURE;
    }
    let tokenizer = Tokenizer::o200k();
    let dataset = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("dataset")
        .to_string();
    // --extra name=path pairs: pre-encoded renderings of the same records.
    let mut extra: Vec<(String, String)> = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == "--extra" {
            let Some(spec) = it.next() else {
                return usage();
            };
            let Some((name, path)) = spec.split_once('=') else {
                return usage();
            };
            match fs::read_to_string(path) {
                Ok(text) => extra.push((name.to_string(), text)),
                Err(e) => {
                    eprintln!("{path}: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
    }
    let bundle = BundleInput {
        dataset: &dataset,
        source_path: &input,
        seed,
        records: &records,
        tasks: &tasks,
        formats: &formats,
        extra: &extra,
        tokenizer: tokenizer.as_ref(),
    };
    match write_bundle(Path::new(&out), &bundle) {
        Ok(m) => {
            println!(
                "bundle {}: {} records, {} tasks, {} prompts across {} formats{}",
                out,
                m.record_count,
                m.tasks,
                m.prompts,
                m.formats.len(),
                m.max_prompt_tokens_o200k
                    .map(|t| format!(", largest prompt {t} o200k tokens"))
                    .unwrap_or_default()
            );
            let kinds: Vec<String> = m.per_kind.iter().map(|(k, n)| format!("{k} {n}")).collect();
            println!("  tasks per kind (asked {per_kind}): {}", kinds.join(", "));
            for (f, t) in &m.data_tokens_o200k {
                println!("  {f:<12} data tokens (o200k_base): {t}");
            }
            for (f, r) in &m.unsupported {
                println!("  {f:<12} unsupported: {r}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn score(args: &[String]) -> ExitCode {
    let (Some(bundle), Some(responses)) = (flag(args, "--bundle"), flag(args, "--responses"))
    else {
        return usage();
    };
    let bundle = PathBuf::from(bundle);
    let read_jsonl = |p: &Path| -> Result<Vec<String>, String> {
        Ok(fs::read_to_string(p)
            .map_err(|e| format!("{}: {e}", p.display()))?
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(String::from)
            .collect())
    };
    let tasks: Vec<Task> = match read_jsonl(&bundle.join("tasks.jsonl")).and_then(|ls| {
        ls.iter()
            .map(|l| serde_json::from_str(l).map_err(|e| e.to_string()))
            .collect()
    }) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let resps: Vec<Response> = match read_jsonl(Path::new(&responses)).and_then(|ls| {
        ls.iter()
            .map(|l| serde_json::from_str(l).map_err(|e| e.to_string()))
            .collect()
    }) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let manifest: Option<Manifest> = fs::read_to_string(bundle.join("manifest.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());
    let graded = grade(&tasks, &resps);
    let md = render_markdown(
        &graded,
        manifest
            .as_ref()
            .map(|m| m.unsupported.as_slice())
            .unwrap_or(&[]),
    );
    let graded_path = bundle.join("graded.jsonl");
    let mut lines = String::new();
    for g in &graded {
        lines.push_str(&serde_json::to_string(g).unwrap());
        lines.push('\n');
    }
    if let Err(e) = fs::write(&graded_path, lines) {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }
    if let Some(out) = flag(args, "--out") {
        if let Err(e) = fs::write(&out, &md) {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
        println!("wrote {out} and {}", graded_path.display());
    } else {
        print!("{md}");
    }
    ExitCode::SUCCESS
}
