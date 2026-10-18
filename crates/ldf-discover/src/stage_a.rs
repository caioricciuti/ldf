//! Stage A: every configuration on every dataset, exact o200k tokens,
//! lossy candidates dropped, survivors ranked against compact JSON and
//! LDF v0.1. Local, no API calls.
//!
//! Most knobs do nothing on most datasets (nesting knobs on flat data), so
//! each configuration is encoded once to hash its output, and only the
//! distinct outputs are round-trip checked and tokenized.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use ldf_bench::roundtrip::{self, Level};
use ldf_bench::tokens::Tokenizers;
use ldf_data::Value;
use serde::Serialize;

use crate::candidate::Candidate;
use crate::encode;
use crate::knobs::{Config, KNOBS};

/// Formats from `ldf-formats` reported next to the space.
pub const REFERENCES: [&str; 9] = [
    "json", "jsonl", "csv", "tsv", "toon", "toon-tab", "yaml", "ldf", "ldf-v0",
];

pub fn lossless(level: &Level) -> bool {
    matches!(
        level,
        Level::Byte | Level::Semantic | Level::SemanticUnordered
    )
}

pub struct Counter(Tokenizers);

impl Counter {
    pub fn new() -> Result<Counter, String> {
        Tokenizers::new(&["o200k_base"]).map(Counter)
    }
    pub fn count(&self, text: &str) -> u64 {
        self.0.count(text).first().map(|c| c.tokens).unwrap_or(0)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Reference {
    pub format: String,
    pub level: String,
    pub detail: Option<String>,
    pub tokens: Option<u64>,
    pub ratio_json: Option<f64>,
}

/// One distinct output: the configurations that produce it, its round-trip
/// level and its tokens.
#[derive(Debug, Clone, Serialize)]
pub struct Distinct {
    pub id: String,
    pub configs: usize,
    pub level: String,
    pub detail: Option<String>,
    pub bytes: u64,
    pub tokens: u64,
    pub ratio_json: f64,
    pub ratio_ldf: f64,
    #[serde(skip)]
    pub lossless: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct KnobEffect {
    pub knob: String,
    /// Does any change of this knob alone change the output?
    pub matters: bool,
    /// Lowest lossless tokens per knob value.
    pub best_by_value: Vec<(String, Option<u64>)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatasetResult {
    pub name: String,
    pub records: usize,
    pub json_tokens: u64,
    pub ldf_tokens: u64,
    /// The engine's v0.1 point is byte-identical to `crates/ldf`.
    pub v01_identical: bool,
    pub references: Vec<Reference>,
    pub distinct: usize,
    pub lossy: usize,
    pub decode_errors: usize,
    pub lossless: usize,
    /// Lossless and at most `max_ratio` of compact JSON.
    pub passing: usize,
    pub top: Vec<Distinct>,
    pub knobs: Vec<KnobEffect>,
    /// First lossy and decode-error details, for debugging the codec.
    pub failures: Vec<Distinct>,
    /// Per configuration (space order): tokens when lossless.
    #[serde(skip)]
    pub per_config: Vec<Option<u64>>,
}

pub struct Settings {
    pub max_ratio: f64,
    pub top: usize,
    pub threads: usize,
}

fn hash_text(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

/// Splits `0..n` into one range per thread and runs `f` on each index.
fn parallel<T: Send>(n: usize, threads: usize, f: impl Fn(usize) -> T + Sync) -> Vec<T> {
    let chunk = n.div_ceil(threads.max(1)).max(1);
    let f = &f;
    std::thread::scope(|s| {
        let handles: Vec<_> = (0..n)
            .step_by(chunk)
            .map(|start| {
                s.spawn(move || (start..(start + chunk).min(n)).map(f).collect::<Vec<T>>())
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().expect("worker panicked"))
            .collect()
    })
}

pub fn reference(name: &str, records: &[Value], counter: &Counter, json: u64) -> Reference {
    let fmt = ldf_formats::by_name(name).expect("known reference format");
    let rt = roundtrip::check(fmt.as_ref(), records);
    let tokens = fmt
        .encode_to_string(records)
        .ok()
        .map(|t| counter.count(&t));
    Reference {
        format: name.to_string(),
        level: rt.level.label().to_string(),
        detail: rt.detail,
        tokens,
        ratio_json: tokens.map(|t| t as f64 / json as f64),
    }
}

pub fn run_dataset(
    name: &str,
    records: &[Value],
    space: &[Config],
    groups: &[Vec<usize>],
    counter: &Counter,
    s: &Settings,
) -> DatasetResult {
    let json = ldf_formats::by_name("json")
        .expect("json")
        .encode_to_string(records)
        .map(|t| counter.count(&t))
        .unwrap_or(1);
    let ldf_text = ldf::encode(records);
    let ldf_tokens = counter.count(&ldf_text);
    let v01_identical = encode(records, &Config::LDF_V01) == ldf_text;
    let references = REFERENCES
        .iter()
        .map(|r| reference(r, records, counter, json))
        .collect();

    // Pass 1: hash every configuration's output.
    let hashes = parallel(space.len(), s.threads, |i| {
        hash_text(&encode(records, &space[i]))
    });
    let mut by_hash: HashMap<u64, usize> = HashMap::new();
    let mut reps: Vec<usize> = Vec::new();
    let mut counts: Vec<usize> = Vec::new();
    let slot: Vec<usize> = hashes
        .iter()
        .enumerate()
        .map(|(i, h)| {
            *by_hash.entry(*h).or_insert_with(|| {
                reps.push(i);
                counts.push(0);
                reps.len() - 1
            })
        })
        .collect();
    for d in &slot {
        counts[*d] += 1;
    }

    // Pass 2: round trip and tokens per distinct output.
    let distinct: Vec<Distinct> = parallel(reps.len(), s.threads, |d| {
        let cfg = space[reps[d]];
        let rt = roundtrip::check(&Candidate::new(cfg), records);
        let text = encode(records, &cfg);
        let tokens = counter.count(&text);
        Distinct {
            id: cfg.id(),
            configs: counts[d],
            lossless: lossless(&rt.level),
            level: rt.level.label().to_string(),
            detail: rt.detail,
            bytes: text.len() as u64,
            tokens,
            ratio_json: tokens as f64 / json as f64,
            ratio_ldf: tokens as f64 / ldf_tokens as f64,
        }
    });

    let per_config: Vec<Option<u64>> = slot
        .iter()
        .map(|&d| distinct[d].lossless.then_some(distinct[d].tokens))
        .collect();

    let knobs = (0..KNOBS.len())
        .map(|k| knob_effect(k, space, &slot, &per_config, &groups[k]))
        .collect();

    let mut ranked: Vec<&Distinct> = distinct.iter().filter(|d| d.lossless).collect();
    ranked.sort_by_key(|d| (d.tokens, d.id.clone()));
    let passing = ranked
        .iter()
        .filter(|d| d.ratio_json <= s.max_ratio)
        .count();
    let failures: Vec<Distinct> = distinct
        .iter()
        .filter(|d| !d.lossless)
        .take(20)
        .cloned()
        .collect();
    DatasetResult {
        name: name.to_string(),
        records: records.len(),
        json_tokens: json,
        ldf_tokens,
        v01_identical,
        references,
        distinct: distinct.len(),
        lossy: distinct.iter().filter(|d| d.level == "lossy").count(),
        decode_errors: distinct
            .iter()
            .filter(|d| d.level == "decode-error")
            .count(),
        lossless: ranked.len(),
        passing,
        top: ranked
            .iter()
            .filter(|d| d.ratio_json <= s.max_ratio)
            .take(s.top)
            .map(|d| (*d).clone())
            .collect(),
        knobs,
        failures,
        per_config,
    }
}

/// `group[i]` numbers the configurations that agree with configuration `i`
/// on every knob except one; the knob matters when a group holds two
/// different outputs.
fn knob_effect(
    k: usize,
    space: &[Config],
    slot: &[usize],
    per_config: &[Option<u64>],
    group: &[usize],
) -> KnobEffect {
    let mut first: HashMap<usize, usize> = HashMap::new();
    let mut matters = false;
    for (i, g) in group.iter().enumerate() {
        match first.get(g) {
            Some(&d) if d != slot[i] => matters = true,
            Some(_) => {}
            None => {
                first.insert(*g, slot[i]);
            }
        }
    }
    let mut best: Vec<(String, Option<u64>)> = Vec::new();
    for (i, c) in space.iter().enumerate() {
        let v = c.knob_value(k);
        let pos = match best.iter().position(|(bv, _)| *bv == v) {
            Some(p) => p,
            None => {
                best.push((v, None));
                best.len() - 1
            }
        };
        if let Some(t) = per_config[i] {
            let e = &mut best[pos].1;
            *e = Some(e.map_or(t, |b| b.min(t)));
        }
    }
    KnobEffect {
        knob: KNOBS[k].to_string(),
        matters,
        best_by_value: best,
    }
}

/// For each knob, a group number per configuration: equal when the
/// configurations agree on every other knob.
pub fn knob_groups(space: &[Config]) -> Vec<Vec<usize>> {
    let parts: Vec<Vec<String>> = space
        .iter()
        .map(|c| c.id().split(' ').map(str::to_string).collect())
        .collect();
    (0..KNOBS.len())
        .map(|k| {
            let mut ids: HashMap<String, usize> = HashMap::new();
            parts
                .iter()
                .map(|p| {
                    let key: Vec<&str> = p
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != k)
                        .map(|(_, s)| s.as_str())
                        .collect();
                    let n = ids.len();
                    *ids.entry(key.join(" ")).or_insert(n)
                })
                .collect()
        })
        .collect()
}
