//! `ldf-discover`: the format discovery engine.
//!
//! ```text
//! ldf-discover space                           knob list and space size
//! ldf-discover show --id ID --input F [--sample N]   one candidate's output
//! ldf-discover stage-a [--scale medium] [--sample 200] [--recheck 1000]
//!                      [--dataset a,b] [--max-ratio 0.9] [--top 10] [--out DIR]
//! ldf-discover profile --input F [--sample N]          structural traits, JSON
//! ldf-discover select --model M --input F [--sample N] [--evidence FILE]
//!                     [--min-n 50] [--max-drop 3] [--extrapolate-rows]
//!                     [--require-key-order] [--json]
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ldf_bench::roundtrip;
use ldf_data::Value;
use ldf_data::input::{ReadOptions, read_all};
use ldf_discover::knobs::{Config, KNOBS, space};
use ldf_discover::stage_a::{self, Counter, DatasetResult, Settings, knob_groups};
use ldf_discover::{Candidate, encode};
use serde::Serialize;

mod report;

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn load(path: &Path, sample: usize) -> Result<Vec<Value>, String> {
    let mut recs = read_all(path, &ReadOptions::default()).map_err(|e| e.to_string())?;
    recs.truncate(sample);
    Ok(recs)
}

fn config_by_id(id: &str) -> Result<Config, String> {
    Config::from_id(id)
        .ok_or_else(|| format!("no configuration with id {id:?}; see `ldf-discover space`"))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let res = match args.first().map(String::as_str) {
        Some("space") => cmd_space(),
        Some("show") => cmd_show(&args),
        Some("stage-a") => cmd_stage_a(&args),
        Some("profile") => cmd_profile(&args),
        Some("select") => cmd_select(&args),
        _ => Err(
            "usage: ldf-discover space | show --id ID --input F | stage-a [...] | profile --input F | select --model M --input F"
                .into(),
        ),
    };
    match res {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ldf-discover: {e}");
            ExitCode::FAILURE
        }
    }
}

fn sample_flag(args: &[String]) -> usize {
    flag(args, "--sample")
        .and_then(|s| s.parse().ok())
        .unwrap_or(usize::MAX)
}

fn cmd_profile(args: &[String]) -> Result<(), String> {
    let input = flag(args, "--input").ok_or("--input required")?;
    let recs = load(Path::new(input), sample_flag(args))?;
    let p = ldf_discover::select::profile(&recs);
    println!("{}", serde_json::to_string(&p).map_err(|e| e.to_string())?);
    Ok(())
}

fn cmd_select(args: &[String]) -> Result<(), String> {
    use ldf_discover::select::{Evidence, Policy, select};
    let model = flag(args, "--model").ok_or("--model required")?;
    let input = flag(args, "--input").ok_or("--input required")?;
    let ev_path = flag(args, "--evidence").unwrap_or("docs/reports/evidence/evidence.json");
    let evidence: Vec<Evidence> =
        serde_json::from_slice(&fs::read(ev_path).map_err(|e| format!("{ev_path}: {e}"))?)
            .map_err(|e| format!("{ev_path}: {e}"))?;
    let mut policy = Policy::default();
    if let Some(v) = flag(args, "--min-n").and_then(|s| s.parse().ok()) {
        policy.min_n = v;
    }
    if let Some(v) = flag(args, "--max-drop").and_then(|s| s.parse().ok()) {
        policy.max_drop_pp = v;
    }
    policy.extrapolate_rows = args.iter().any(|a| a == "--extrapolate-rows");
    policy.require_key_order = args.iter().any(|a| a == "--require-key-order");
    let recs = load(Path::new(input), sample_flag(args))?;
    let tk = ldf_bench::tokens::Tokenizers::new(&["o200k_base"])?;
    let count = |s: &str| tk.count(s)[0].tokens;
    let d = select(&recs, model, &evidence, &policy, &count);
    if args.iter().any(|a| a == "--json") {
        println!(
            "{}",
            serde_json::to_string_pretty(&d).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    println!(
        "model {model}, {} rows, traits {:?} (min fill {:.2})",
        d.profile.rows,
        d.profile.traits.names(),
        d.profile.min_fill
    );
    for v in &d.verdicts {
        println!(
            "  {:<32} {:>8} o200k  {:<9} round trip {}",
            v.choice.name(),
            v.tokens
                .map(|t| t.to_string())
                .unwrap_or_else(|| "-".into()),
            if v.accepted { "accepted" } else { "rejected" },
            v.round_trip
        );
        for n in &v.notes {
            println!("      {n}");
        }
    }
    println!("choice: {}", d.choice.name());
    Ok(())
}

fn cmd_space() -> Result<(), String> {
    let s = space();
    println!("{} configurations", s.len());
    for (k, name) in KNOBS.iter().enumerate() {
        let mut vals: Vec<String> = Vec::new();
        for c in &s {
            let v = c.knob_value(k);
            if !vals.contains(&v) {
                vals.push(v);
            }
        }
        println!("  {name:<22} {}", vals.join(", "));
    }
    println!("ldf v0.1: {}", Config::LDF_V01.id());
    Ok(())
}

fn cmd_show(args: &[String]) -> Result<(), String> {
    let id = flag(args, "--id").ok_or("--id required")?;
    let input = flag(args, "--input").ok_or("--input required")?;
    let sample = flag(args, "--sample")
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);
    let cfg = config_by_id(id)?;
    let recs = load(Path::new(input), sample)?;
    print!("{}", encode(&recs, &cfg));
    let rt = roundtrip::check(&Candidate::new(cfg), &recs);
    eprintln!(
        "{} records, round trip {}{}",
        recs.len(),
        rt.level.label(),
        rt.detail.map(|d| format!(" ({d})")).unwrap_or_default()
    );
    Ok(())
}

#[derive(Serialize)]
struct Recheck {
    dataset: String,
    records: usize,
    json_tokens: u64,
    ldf_tokens: u64,
    rows: Vec<RecheckRow>,
}

#[derive(Serialize)]
struct RecheckRow {
    id: String,
    rank_at_sample: usize,
    level: String,
    tokens: u64,
    ratio_json: f64,
    ratio_ldf: f64,
}

#[derive(Serialize)]
struct Global {
    id: String,
    mean_ratio_json: f64,
    mean_ratio_ldf: f64,
    max_ratio_json: f64,
}

#[derive(Serialize)]
struct StageA<'a> {
    scale: String,
    sample: usize,
    recheck: usize,
    tokenizer: &'static str,
    max_ratio: f64,
    space: usize,
    machine_threads: usize,
    seconds: f64,
    datasets: &'a [DatasetResult],
    global: Vec<Global>,
    v01_global: Option<Global>,
    rechecks: Vec<Recheck>,
}

fn cmd_stage_a(args: &[String]) -> Result<(), String> {
    let scale = flag(args, "--scale").unwrap_or("medium").to_string();
    let sample: usize = flag(args, "--sample")
        .and_then(|s| s.parse().ok())
        .unwrap_or(200);
    let recheck: usize = flag(args, "--recheck")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);
    let max_ratio: f64 = flag(args, "--max-ratio")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.9);
    let top: usize = flag(args, "--top")
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    let out = PathBuf::from(
        flag(args, "--out")
            .map(str::to_string)
            .unwrap_or_else(|| format!("results/discovery/{}", ldf_bench::timestamp_compact())),
    );
    let dir = PathBuf::from("fixtures").join(&scale);
    let mut names: Vec<String> = fs::read_dir(&dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let p = e.path();
            (p.extension()? == "json").then(|| p.file_stem()?.to_str().map(str::to_string))?
        })
        .collect();
    names.sort();
    if let Some(sel) = flag(args, "--dataset") {
        let want: Vec<&str> = sel.split(',').collect();
        names.retain(|n| want.contains(&n.as_str()));
    }
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let settings = Settings {
        max_ratio,
        top,
        threads,
    };
    let space = space();
    let groups = knob_groups(&space);
    let counter = Counter::new()?;
    let started = std::time::Instant::now();
    let mut results = Vec::new();
    for name in &names {
        let t = std::time::Instant::now();
        let recs = load(&dir.join(format!("{name}.json")), sample)?;
        let r = stage_a::run_dataset(name, &recs, &space, &groups, &counter, &settings);
        eprintln!(
            "{name}: {} records, {} distinct, {} lossless, {} passing, best {:.3} of json, {:.3} of ldf ({:.1}s)",
            r.records,
            r.distinct,
            r.lossless,
            r.passing,
            r.top.first().map_or(f64::NAN, |d| d.ratio_json),
            r.top.first().map_or(f64::NAN, |d| d.ratio_ldf),
            t.elapsed().as_secs_f64()
        );
        results.push(r);
    }

    // One configuration for every dataset: lossless everywhere, ranked by
    // mean ratio to JSON so large datasets do not dominate.
    let score = |i: usize| -> Option<Global> {
        let mut rj = Vec::new();
        let mut rl = Vec::new();
        for r in &results {
            let t = r.per_config[i]? as f64;
            rj.push(t / r.json_tokens as f64);
            rl.push(t / r.ldf_tokens as f64);
        }
        let n = rj.len().max(1) as f64;
        Some(Global {
            id: space[i].id(),
            mean_ratio_json: rj.iter().sum::<f64>() / n,
            mean_ratio_ldf: rl.iter().sum::<f64>() / n,
            max_ratio_json: rj.iter().copied().fold(0.0, f64::max),
        })
    };
    let mut global: Vec<Global> = (0..space.len()).filter_map(score).collect();
    global.sort_by(|a, b| a.mean_ratio_json.total_cmp(&b.mean_ratio_json));
    global.truncate(top);
    let v01_global = space
        .iter()
        .position(|c| *c == Config::LDF_V01)
        .and_then(score);

    let mut rechecks = Vec::new();
    if recheck > sample {
        for r in &results {
            let recs = load(&dir.join(format!("{}.json", r.name)), recheck)?;
            if recs.len() <= sample {
                continue;
            }
            let json = ldf_formats::by_name("json")
                .expect("json")
                .encode_to_string(&recs)
                .map(|t| counter.count(&t))
                .unwrap_or(1);
            let ldf_tokens = counter.count(&ldf::encode(&recs));
            let mut rows: Vec<RecheckRow> = r
                .top
                .iter()
                .enumerate()
                .map(|(rank, d)| {
                    let cfg = config_by_id(&d.id).expect("id from the space");
                    let rt = roundtrip::check(&Candidate::new(cfg), &recs);
                    let tokens = counter.count(&encode(&recs, &cfg));
                    RecheckRow {
                        id: d.id.clone(),
                        rank_at_sample: rank + 1,
                        level: rt.level.label().to_string(),
                        tokens,
                        ratio_json: tokens as f64 / json as f64,
                        ratio_ldf: tokens as f64 / ldf_tokens as f64,
                    }
                })
                .collect();
            rows.sort_by_key(|x| x.tokens);
            rechecks.push(Recheck {
                dataset: r.name.clone(),
                records: recs.len(),
                json_tokens: json,
                ldf_tokens,
                rows,
            });
        }
    }

    let doc = StageA {
        scale,
        sample,
        recheck,
        tokenizer: "o200k_base",
        max_ratio,
        space: space.len(),
        machine_threads: threads,
        seconds: started.elapsed().as_secs_f64(),
        datasets: &results,
        global,
        v01_global,
        rechecks,
    };
    fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())?;
    fs::write(out.join("stage-a.json"), json).map_err(|e| e.to_string())?;
    let md = report::render(&serde_json::to_value(&doc).map_err(|e| e.to_string())?);
    fs::write(out.join("stage-a.md"), md).map_err(|e| e.to_string())?;
    eprintln!(
        "wrote {}/stage-a.{{json,md}} in {:.1}s",
        out.display(),
        started.elapsed().as_secs_f64()
    );
    Ok(())
}
