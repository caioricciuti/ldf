//! `ldf-gen`: deterministic fixture generator.
//!
//! ```text
//! ldf-gen [--out fixtures] [--scale small|medium|large] [--rows N]
//!         [--dataset name,name|all] [--seed N] [--containers json,jsonl,csv,parquet]
//! ldf-gen list
//! ```

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use ldf_fixtures::writer::{Container, write_dataset};
use ldf_fixtures::{DEFAULT_SEED, Scale, catalog};

fn usage() -> ExitCode {
    eprintln!(
        "usage: ldf-gen [list] [--out DIR] [--scale small|medium|large] [--rows N] \
         [--dataset NAME[,NAME]|all] [--seed N] [--containers json,jsonl,csv,parquet]"
    );
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("list") {
        for d in catalog() {
            println!(
                "{:<18} flat={:<5} uniform={:<5} rows={:?}\n    {}",
                d.name, d.flat, d.uniform, d.rows, d.description
            );
        }
        return ExitCode::SUCCESS;
    }
    let mut out = PathBuf::from("fixtures");
    let mut scale = Scale::Small;
    let mut rows: Option<u64> = None;
    let mut datasets = "all".to_string();
    let mut seed = DEFAULT_SEED;
    let mut containers = vec![
        Container::Json,
        Container::Jsonl,
        Container::Csv,
        Container::Parquet,
    ];
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = || it.next().cloned();
        match a.as_str() {
            "--out" => out = PathBuf::from(val().unwrap_or_default()),
            "--scale" => match val().as_deref().and_then(Scale::parse) {
                Some(s) => scale = s,
                None => return usage(),
            },
            "--rows" => match val().and_then(|v| v.parse().ok()) {
                Some(n) => rows = Some(n),
                None => return usage(),
            },
            "--dataset" => datasets = val().unwrap_or_default(),
            "--seed" => match val().and_then(|v| v.parse().ok()) {
                Some(n) => seed = n,
                None => return usage(),
            },
            "--containers" => {
                let Some(list) = val() else { return usage() };
                let parsed: Option<Vec<Container>> =
                    list.split(',').map(Container::parse).collect();
                match parsed {
                    Some(c) => containers = c,
                    None => return usage(),
                }
            }
            _ => return usage(),
        }
    }
    let selected: Vec<_> = if datasets == "all" {
        catalog()
    } else {
        let names: Vec<&str> = datasets.split(',').collect();
        let all = catalog();
        let picked: Vec<_> = all
            .into_iter()
            .filter(|d| names.contains(&d.name))
            .collect();
        if picked.len() != names.len() {
            eprintln!("unknown dataset in {datasets:?}; run `ldf-gen list`");
            return ExitCode::from(2);
        }
        picked
    };
    let dir = out.join(scale.name());
    println!("seed={seed} scale={} out={}", scale.name(), dir.display());
    for spec in selected {
        let n = rows.unwrap_or_else(|| spec.rows_for(scale));
        let t = Instant::now();
        match write_dataset(&spec, seed, n, &dir, &containers) {
            Ok((written, skipped)) => {
                let secs = t.elapsed().as_secs_f64();
                for w in &written {
                    println!(
                        "  {:<18} {:<8} {:>10} rows {:>14} bytes  {}",
                        spec.name,
                        w.container.extension(),
                        w.rows,
                        w.bytes,
                        w.path.display()
                    );
                }
                for s in skipped {
                    println!("  note: {s}");
                }
                println!("  {:<18} generated in {secs:.2}s", spec.name);
            }
            Err(e) => {
                eprintln!("{}: {e}", spec.name);
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}
