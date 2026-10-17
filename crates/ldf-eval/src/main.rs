//! `ldf-eval`: build evaluation bundles and score responses.
//!
//! ```text
//! ldf-eval build --input fixtures/small/flat_users.json [--records 50] [--per-kind 3]
//!                [--formats all] [--seed 1] --out eval/out/flat_users
//! ldf-eval score --bundle eval/out/flat_users --responses eval/out/flat_users/responses.jsonl
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ldf_data::input::{ReadOptions, read_all};
use ldf_eval::bundle::{BundleInput, Manifest, Tokenizer, write_bundle};
use ldf_eval::score::{Response, grade, render_markdown};
use ldf_eval::tasks::{Generator, Task};

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}

fn usage() -> ExitCode {
    eprintln!(
        "usage:\n  ldf-eval build --input FILE --out DIR [--records N] [--per-kind N] [--formats all|a,b] [--seed N]\n\
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
    let formats =
        match ldf_formats::select(&flag(args, "--formats").unwrap_or_else(|| "all".into())) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::from(2);
            }
        };
    let path = PathBuf::from(&input);
    let mut records = match read_all(&path, &ReadOptions::default()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    records.truncate(records_n);
    let tasks = Generator::new(&records, seed, per_kind).generate();
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
    let bundle = BundleInput {
        dataset: &dataset,
        source_path: &input,
        seed,
        records: &records,
        tasks: &tasks,
        formats: &formats,
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
