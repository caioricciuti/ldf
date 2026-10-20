//! Adaptive format selector: picks JSON, TOON or LDF (sparse threshold 0.5
//! or 0.75, with or without hoisted arrays; without hoisting the document
//! is valid v0.1) for one dataset and one model, from measured evidence
//! only.
//!
//! A candidate other than JSON is accepted when all of these hold:
//! 1. it round-trips this dataset: byte or semantic level, or
//!    semantic-unordered (every value kept, object member order changed,
//!    which spec section 7 documents for keys in the `~` column; JSON
//!    gives member order no meaning) unless the policy requires key order;
//! 2. there is evidence for this exact model and this candidate on a
//!    measured dataset with the same structural traits ([`Traits`]), and
//!    every such evidence row in the dataset's size band passes the
//!    [`Policy`]: enough questions, no significant loss against JSON on the
//!    same questions (exact sign test), and an accuracy drop no larger than
//!    the tolerance. The size band is the smallest measured size at or
//!    above the dataset's row count: 150 rows are judged by evidence at 200
//!    records, 600 rows by evidence at 1 000, and a loss measured at 1 000
//!    records does not withdraw LDF from 150;
//! 3. the dataset is no larger than the largest measured one, unless the
//!    policy allows extrapolation (then the largest band judges it);
//! 4. it saves at least `min_saving` of JSON's size on this dataset.
//!
//! Among accepted candidates the smallest wins. JSON is the fallback for
//! unknown models, uncovered shapes and anything that fails a check, so the
//! selector is never less safe than not using it.
//!
//! The caller measures each candidate ([`Measured`]): `ldf-discover`
//! counts o200k tokens and checks TOON through `ldf-formats`; `ldf-mcp`
//! offers JSON and LDF only and measures bytes with [`measure_ldf`].
//! Evidence rows come from `eval/analysis/evidence.py`; a copy of
//! `reports/evidence/evidence.json` ships in this crate
//! ([`builtin_evidence`]).

use std::collections::BTreeSet;

use ldf_data::Value;
use serde::{Deserialize, Serialize};

/// Structural traits that changed model behaviour or layout in the runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Traits {
    /// Arrays of objects: child tables in LDF, list form in TOON.
    pub child_tables: bool,
    /// Values of varying kind or irregular objects: JSON cells in LDF.
    pub json_cells: bool,
    /// A top-level key with a value in under 75 percent of rows, where the
    /// sparse threshold changes the LDF layout.
    pub sparse: bool,
}

impl Traits {
    pub fn names(&self) -> BTreeSet<&'static str> {
        let mut s = BTreeSet::new();
        if self.child_tables {
            s.insert("child_tables");
        }
        if self.json_cells {
            s.insert("json_cells");
        }
        if self.sparse {
            s.insert("sparse");
        }
        s
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Profile {
    pub rows: usize,
    pub top_keys: usize,
    /// Lowest share of rows with a non-null value, over top-level keys.
    pub min_fill: f64,
    pub traits: Traits,
}

/// Share of rows under which a key counts as sparse.
pub const SPARSE_FILL: f64 = 0.75;

pub fn profile(records: &[Value]) -> Profile {
    let plan = ldf::plan::plan_with(records, &ldf::Options::V0);
    let child_tables = plan.columns.iter().any(|c| c.child.is_some());
    let json_cells = plan.scalar_rows
        || plan
            .columns
            .iter()
            .any(|c| c.ty == ldf::plan::ColType::Json);
    let mut keys: Vec<(&str, usize)> = Vec::new();
    for r in records {
        let Some(o) = r.as_object() else { continue };
        for (k, v) in o {
            let i = match keys.iter().position(|(kk, _)| *kk == k.as_str()) {
                Some(i) => i,
                None => {
                    keys.push((k.as_str(), 0));
                    keys.len() - 1
                }
            };
            if !v.is_null() {
                keys[i].1 += 1;
            }
        }
    }
    let n = records.len().max(1) as f64;
    let min_fill = keys.iter().map(|(_, f)| *f as f64 / n).fold(1.0, f64::min);
    Profile {
        rows: records.len(),
        top_keys: keys.len(),
        min_fill,
        traits: Traits {
            child_tables,
            json_cells,
            sparse: min_fill < SPARSE_FILL,
        },
    }
}

/// One paired comparison against JSON: one model, one dataset, one format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub model: String,
    pub run: String,
    pub dataset: String,
    pub records: usize,
    pub traits: Vec<String>,
    /// `json`, `toon`, `ldf` (a v0.1 document) or `other`.
    pub family: String,
    /// For `ldf`: every sparse threshold that gives this exact document.
    #[serde(default)]
    pub thresholds: Vec<f64>,
    /// For `ldf`: every sparse threshold that gives this exact document
    /// with hoisted arrays (`ldf::Options::hoist`). Equal to `thresholds`
    /// when the dataset has nothing to hoist.
    #[serde(default)]
    pub hoist_thresholds: Vec<f64>,
    pub format: String,
    pub n: usize,
    pub k: usize,
    pub json_k: usize,
    /// Questions this format got right and JSON wrong, and the reverse.
    pub wins: usize,
    pub losses: usize,
    pub p: f64,
    pub tokens_ratio: f64,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Policy {
    /// Fewest paired questions an evidence row needs to count.
    pub min_n: usize,
    /// Largest accepted accuracy drop against JSON, in percentage points.
    pub max_drop_pp: f64,
    /// Significance level of the sign test for a loss against JSON.
    pub alpha: f64,
    /// Accept datasets larger than the largest measured one.
    pub extrapolate_rows: bool,
    /// Smallest token saving against JSON worth switching for.
    pub min_saving: f64,
    /// Reject candidates whose decode changes object member order.
    pub require_key_order: bool,
}

impl Default for Policy {
    fn default() -> Self {
        Policy {
            min_n: 50,
            max_drop_pp: 3.0,
            alpha: 0.05,
            extrapolate_rows: false,
            min_saving: 0.05,
            require_key_order: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(tag = "format", rename_all = "kebab-case")]
pub enum Choice {
    Json,
    Toon,
    Ldf { sparse_threshold: f64, hoist: bool },
}

impl Choice {
    pub fn name(&self) -> String {
        match self {
            Choice::Json => "json".into(),
            Choice::Toon => "toon".into(),
            Choice::Ldf {
                sparse_threshold,
                hoist: false,
            } => format!("ldf (sparse threshold {sparse_threshold})"),
            Choice::Ldf {
                sparse_threshold,
                hoist: true,
            } => format!("ldf (sparse threshold {sparse_threshold}, hoisted)"),
        }
    }

    /// Whether an evidence row measured this candidate.
    pub fn matches(&self, e: &Evidence) -> bool {
        match self {
            Choice::Json => e.family == "json",
            Choice::Toon => e.family == "toon",
            Choice::Ldf {
                sparse_threshold,
                hoist,
            } => {
                let ts = if *hoist {
                    &e.hoist_thresholds
                } else {
                    &e.thresholds
                };
                e.family == "ldf" && ts.iter().any(|t| (t - sparse_threshold).abs() < 1e-9)
            }
        }
    }
}

const fn ldf_at(sparse_threshold: f64, hoist: bool) -> Choice {
    Choice::Ldf {
        sparse_threshold,
        hoist,
    }
}

/// The candidates, JSON first. Unhoisted before hoisted: on data with
/// nothing to hoist both give the same text, and the v0.1 point wins the tie.
pub const CANDIDATES: [Choice; 6] = [
    Choice::Json,
    Choice::Toon,
    ldf_at(0.5, false),
    ldf_at(0.75, false),
    ldf_at(0.5, true),
    ldf_at(0.75, true),
];

#[derive(Debug, Clone, Serialize)]
pub struct Verdict {
    pub choice: Choice,
    /// Size in the caller's unit: o200k tokens in `ldf-discover`, bytes in
    /// [`select_ldf`].
    pub tokens: Option<u64>,
    /// Round-trip level on this dataset (`ldf-bench` levels).
    pub round_trip: String,
    pub accepted: bool,
    /// Why it was rejected, or the evidence it was accepted on.
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Decision {
    pub model: String,
    pub profile: Profile,
    pub policy: Policy,
    pub choice: Choice,
    pub verdicts: Vec<Verdict>,
}

/// Exact sign test, two-sided, on the questions where one side was right.
pub fn sign_test(wins: usize, losses: usize) -> f64 {
    let n = wins + losses;
    if n == 0 {
        return 1.0;
    }
    let k = wins.min(losses);
    // Sum of C(n, i) / 2^n for i <= k, in floating point via logs.
    let ln_half_n = n as f64 * 0.5f64.ln();
    let mut ln_c = 0.0f64;
    let mut sum = 0.0f64;
    for i in 0..=k {
        if i > 0 {
            ln_c += ((n - i + 1) as f64).ln() - (i as f64).ln();
        }
        sum += (ln_c + ln_half_n).exp();
    }
    (2.0 * sum).min(1.0)
}

/// Checks one evidence row against the policy; `Err` says why it fails.
fn row_passes(e: &Evidence, policy: &Policy) -> Result<(), String> {
    let tag = format!("{} on {} ({} records)", e.format, e.dataset, e.records);
    let drop = 100.0 * (e.json_k as f64 - e.k as f64) / e.n as f64;
    if drop > policy.max_drop_pp {
        return Err(format!(
            "{tag}: {:.1} points below JSON ({}/{} vs {}/{}), tolerance {}",
            drop, e.k, e.n, e.json_k, e.n, policy.max_drop_pp
        ));
    }
    if e.losses > e.wins && e.p < policy.alpha {
        return Err(format!(
            "{tag}: significantly worse than JSON (+{} / -{}, p={:.3})",
            e.wins, e.losses, e.p
        ));
    }
    Ok(())
}

/// The evidence this crate was built with, `reports/evidence/evidence.json`.
pub const EVIDENCE_JSON: &str = include_str!("../evidence.json");

pub fn builtin_evidence() -> Vec<Evidence> {
    // The file is checked in and tested to parse.
    serde_json::from_str(EVIDENCE_JSON).unwrap_or_default()
}

/// One candidate measured on the dataset: its size (tokens or bytes, the
/// caller's unit) when it round-trips at an accepted level, `None` with a
/// note when it does not.
#[derive(Debug, Clone)]
pub struct Measured {
    pub size: Option<u64>,
    pub round_trip: String,
    pub note: Option<String>,
}

/// Encodes `records` as LDF at `sparse_threshold`, hoisted or not, decodes
/// it back and measures it in bytes. Returns the text with the measurement;
/// the text is `None` when the round trip is not accepted under `policy`.
pub fn measure_ldf(
    records: &[Value],
    sparse_threshold: f64,
    hoist: bool,
    policy: &Policy,
) -> (Option<String>, Measured) {
    let opts = ldf::Options {
        sparse_threshold,
        hoist,
        ..ldf::Options::default()
    };
    let text = ldf::encode_with(records, &opts);
    let (level, detail) = match ldf::decode(&text) {
        Err(e) => ("decode-error", Some(e.to_string())),
        Ok(back) => {
            let input = Value::Array(records.to_vec());
            let back = Value::Array(back);
            if back == input {
                ("byte", None)
            } else if back.semantic_eq(&input) {
                ("semantic", None)
            } else if back.semantic_eq_unordered(&input) {
                ("semantic-unordered", None)
            } else {
                ("lossy", back.first_difference(&input))
            }
        }
    };
    let accepted = match level {
        "byte" | "semantic" => true,
        "semantic-unordered" => !policy.require_key_order,
        _ => false,
    };
    let note = (!accepted).then(|| {
        format!(
            "not accepted at round-trip level {level} {}",
            detail.unwrap_or_default()
        )
    });
    let size = accepted.then_some(text.len() as u64);
    (
        accepted.then_some(text),
        Measured {
            size,
            round_trip: level.into(),
            note,
        },
    )
}

/// Whether the evidence holds any row for `model` that the policy counts.
pub fn knows_model(model: &str, evidence: &[Evidence], policy: &Policy) -> bool {
    evidence
        .iter()
        .any(|e| e.model == model && e.n >= policy.min_n)
}

/// The candidates a reader primed for LDF only can take, JSON first.
pub const LDF_CANDIDATES: [Choice; 5] = [
    Choice::Json,
    ldf_at(0.5, false),
    ldf_at(0.75, false),
    ldf_at(0.5, true),
    ldf_at(0.75, true),
];

/// JSON or LDF ([`LDF_CANDIDATES`]), measured in bytes. `json_bytes` is the
/// size of the JSON the reader gets otherwise. Returns the decision and,
/// when LDF won, its text.
pub fn select_ldf(
    records: &[Value],
    model: &str,
    evidence: &[Evidence],
    policy: &Policy,
    json_bytes: u64,
) -> (Decision, Option<String>) {
    let mut texts: Vec<(Choice, String)> = Vec::new();
    let d = decide(
        records,
        model,
        evidence,
        policy,
        &LDF_CANDIDATES,
        &mut |c| match c {
            Choice::Ldf {
                sparse_threshold,
                hoist,
            } => {
                let (text, m) = measure_ldf(records, sparse_threshold, hoist, policy);
                if let Some(t) = text {
                    texts.push((c, t));
                }
                m
            }
            _ => Measured {
                size: Some(json_bytes),
                round_trip: "byte".into(),
                note: None,
            },
        },
    );
    let text = texts
        .into_iter()
        .find(|(c, _)| *c == d.choice)
        .map(|(_, t)| t);
    (d, text)
}

/// Decides for `records` and `model` among `candidates`, which must start
/// with [`Choice::Json`]. `measure` is called once per candidate.
pub fn decide(
    records: &[Value],
    model: &str,
    evidence: &[Evidence],
    policy: &Policy,
    candidates: &[Choice],
    measure: &mut dyn FnMut(Choice) -> Measured,
) -> Decision {
    let prof = profile(records);
    let target: BTreeSet<&str> = prof.traits.names();
    let mut verdicts = Vec::new();
    let mut json_size = None;
    for &choice in candidates {
        let m = measure(choice);
        let mut notes: Vec<String> = m.note.into_iter().collect();
        let tokens = m.size;
        if choice == Choice::Json {
            json_size = tokens;
            notes.push("baseline and fallback".into());
            verdicts.push(Verdict {
                choice,
                tokens,
                round_trip: m.round_trip,
                accepted: tokens.is_some(),
                notes,
            });
            continue;
        }
        let mut ok = tokens.is_some();
        if ok {
            ok = check_evidence(choice, model, &target, &prof, evidence, policy, &mut notes);
        }
        if ok && let (Some(t), Some(j)) = (tokens, json_size) {
            let ratio = t as f64 / j as f64;
            if ratio > 1.0 - policy.min_saving {
                notes.push(format!(
                    "{:.2} of JSON's size here, saving under {:.0} percent",
                    ratio,
                    policy.min_saving * 100.0
                ));
                ok = false;
            }
        }
        verdicts.push(Verdict {
            choice,
            tokens,
            round_trip: m.round_trip,
            accepted: ok,
            notes,
        });
    }
    let choice = verdicts
        .iter()
        .filter(|v| v.accepted)
        .min_by_key(|v| v.tokens.unwrap_or(u64::MAX))
        .map(|v| v.choice)
        .unwrap_or(Choice::Json);
    Decision {
        model: model.to_string(),
        profile: prof,
        policy: *policy,
        choice,
        verdicts,
    }
}

fn check_evidence(
    choice: Choice,
    model: &str,
    target: &BTreeSet<&str>,
    prof: &Profile,
    evidence: &[Evidence],
    policy: &Policy,
    notes: &mut Vec<String>,
) -> bool {
    let small = evidence
        .iter()
        .filter(|e| e.model == model && choice.matches(e) && e.n < policy.min_n)
        .count();
    if small > 0 {
        notes.push(format!(
            "{small} evidence rows with under {} questions ignored",
            policy.min_n
        ));
    }
    let rows: Vec<&Evidence> = evidence
        .iter()
        .filter(|e| e.model == model && choice.matches(e) && e.n >= policy.min_n)
        .filter(|e| {
            let set: BTreeSet<&str> = e.traits.iter().map(String::as_str).collect();
            set == *target
        })
        .collect();
    if rows.is_empty() {
        notes.push(
            if evidence
                .iter()
                .any(|e| e.model == model && e.n >= policy.min_n)
            {
                format!(
                    "no evidence for {} on a measured dataset with traits {:?}",
                    choice.name(),
                    target
                )
            } else {
                format!(
                    "no evidence for model {model} with at least {} questions",
                    policy.min_n
                )
            },
        );
        return false;
    }
    let measured = rows.iter().map(|e| e.records).max().unwrap_or(0);
    let mut ok = true;
    if prof.rows > measured && !policy.extrapolate_rows {
        notes.push(format!(
            "{} rows, largest measured dataset has {measured}; extrapolation is off",
            prof.rows
        ));
        ok = false;
    }
    let band = rows
        .iter()
        .map(|e| e.records)
        .filter(|&r| r >= prof.rows)
        .min()
        .unwrap_or(measured);
    let other = rows.iter().filter(|e| e.records != band).count();
    if other > 0 {
        notes.push(format!(
            "{other} evidence rows at other sizes not used: {} rows are judged at {band} records",
            prof.rows
        ));
    }
    for e in rows.iter().filter(|e| e.records == band) {
        match row_passes(e, policy) {
            Ok(()) => notes.push(format!(
                "ok: {} on {} ({} records, {}): {}/{} vs JSON {}/{}, +{} / -{}, p={:.2}",
                e.format,
                e.dataset,
                e.records,
                e.run,
                e.k,
                e.n,
                e.json_k,
                e.n,
                e.wins,
                e.losses,
                e.p
            )),
            Err(why) => {
                notes.push(why);
                ok = false;
            }
        }
    }
    ok
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldf_data::obj;

    #[test]
    fn sign_test_matches_known_values() {
        assert!((sign_test(0, 0) - 1.0).abs() < 1e-12);
        assert!((sign_test(5, 15) - 0.04138946533203125).abs() < 1e-9);
        assert!((sign_test(15, 5) - sign_test(5, 15)).abs() < 1e-12);
        assert!((sign_test(2, 2) - 1.0).abs() < 1e-12);
    }

    fn row(
        model: &str,
        family: &str,
        traits: &[&str],
        k: usize,
        json_k: usize,
        wins: usize,
        losses: usize,
    ) -> Evidence {
        Evidence {
            model: model.into(),
            run: "test".into(),
            dataset: "d".into(),
            records: 200,
            traits: traits.iter().map(|s| s.to_string()).collect(),
            family: family.into(),
            thresholds: vec![0.0, 0.25, 0.5, 0.75],
            hoist_thresholds: vec![0.0, 0.25, 0.5, 0.75],
            format: family.into(),
            n: 100,
            k,
            json_k,
            wins,
            losses,
            p: sign_test(wins, losses),
            tokens_ratio: 0.6,
        }
    }

    fn flat(n: usize) -> Vec<Value> {
        (0..n)
            .map(|i| obj! {"id" => i as i64, "name" => format!("user {i}"), "age" => (20 + i % 50) as i64, "ok" => i % 2 == 0})
            .collect()
    }

    fn select(records: &[Value], model: &str, ev: &[Evidence], p: &Policy) -> Decision {
        let mut json = String::new();
        ldf::text::push_json_value(&Value::Array(records.to_vec()), &mut json);
        select_ldf(records, model, ev, p, json.len() as u64).0
    }

    #[test]
    fn unknown_model_gets_json() {
        let ev = vec![row("m", "ldf", &[], 90, 90, 0, 0)];
        let d = select(&flat(50), "other", &ev, &Policy::default());
        assert_eq!(d.choice, Choice::Json);
    }

    #[test]
    fn measured_model_on_flat_data_gets_the_smallest_passing_format() {
        let ev = vec![
            row("m", "ldf", &[], 90, 90, 2, 2),
            row("m", "toon", &[], 90, 90, 2, 2),
        ];
        let d = select(&flat(50), "m", &ev, &Policy::default());
        assert!(matches!(d.choice, Choice::Ldf { .. }), "{d:?}");
    }

    #[test]
    fn losses_rows_and_traits_fall_back_to_json() {
        let p = Policy::default();
        // Four points below JSON: rejected.
        let ev = vec![row("m", "ldf", &[], 86, 90, 1, 5)];
        assert_eq!(select(&flat(50), "m", &ev, &p).choice, Choice::Json);
        // More rows than measured: rejected unless extrapolation is on.
        let ev = vec![row("m", "ldf", &[], 90, 90, 0, 0)];
        assert_eq!(select(&flat(300), "m", &ev, &p).choice, Choice::Json);
        let loose = Policy {
            extrapolate_rows: true,
            ..p
        };
        assert!(matches!(
            select(&flat(300), "m", &ev, &loose).choice,
            Choice::Ldf { .. }
        ));
        // Sparse data, evidence only on flat data: not covered.
        let mut sparse = flat(50);
        for (i, r) in sparse.iter_mut().enumerate() {
            if let Value::Object(o) = r
                && i % 3 == 0
            {
                o.push(("rare".into(), Value::from(1)));
            }
        }
        let d = select(&sparse, "m", &ev, &p);
        assert!(d.profile.traits.sparse);
        assert_eq!(d.choice, Choice::Json);
    }

    #[test]
    fn each_size_is_judged_by_its_own_band() {
        let p = Policy::default();
        let at = |records: usize, k: usize, losses: usize| Evidence {
            records,
            ..row("m", "ldf", &[], k, 90, 0, losses)
        };
        // Passes at 200, fails at 1 000.
        let ev = vec![at(200, 90, 0), at(1000, 80, 10)];
        let ldf = |n: usize, ev: &[Evidence], p: &Policy| {
            matches!(select(&flat(n), "m", ev, p).choice, Choice::Ldf { .. })
        };
        assert!(ldf(150, &ev, &p), "150 rows: the 200 band");
        assert!(ldf(200, &ev, &p), "200 rows: the 200 band");
        assert!(!ldf(201, &ev, &p), "201 rows: the 1 000 band");
        assert!(!ldf(1000, &ev, &p));
        assert!(!ldf(1001, &ev, &p), "beyond the largest band");
        // Fails at 200, passes at 1 000.
        let ev = vec![at(200, 80, 10), at(1000, 90, 0)];
        assert!(!ldf(150, &ev, &p));
        assert!(ldf(600, &ev, &p));
        let d = select(&flat(600), "m", &ev, &p);
        let ldf_notes = &d.verdicts[1].notes;
        assert!(
            ldf_notes
                .iter()
                .any(|n| n.contains("judged at 1000 records")),
            "{ldf_notes:?}"
        );
        // With extrapolation the largest band judges larger inputs.
        let loose = Policy {
            extrapolate_rows: true,
            ..p
        };
        assert!(ldf(1500, &ev, &loose));
    }
}

#[cfg(test)]
mod builtin_tests {
    use super::*;

    #[test]
    fn builtin_evidence_parses_and_matches_the_report() {
        let ev: Vec<Evidence> = serde_json::from_str(EVIDENCE_JSON).unwrap();
        assert!(!ev.is_empty());
        let report = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../reports/evidence/evidence.json");
        // Absent in the published crate; in the repository the copy must match.
        if let Ok(text) = std::fs::read_to_string(report) {
            assert!(
                text == EVIDENCE_JSON,
                "crates/ldf-select/evidence.json differs from reports/evidence/evidence.json; copy it over"
            );
        }
    }

    #[test]
    fn selected_ldf_text_decodes_to_the_records() {
        let recs: Vec<Value> = (0..60)
            .map(|i| ldf_data::obj! {"id" => i as i64, "name" => format!("n{i}")})
            .collect();
        let ev = vec![Evidence {
            model: "m".into(),
            run: "r".into(),
            dataset: "d".into(),
            records: 100,
            traits: vec![],
            family: "ldf".into(),
            thresholds: vec![0.5, 0.75],
            hoist_thresholds: vec![0.5, 0.75],
            format: "ldf".into(),
            n: 60,
            k: 50,
            json_k: 50,
            wins: 0,
            losses: 0,
            p: 1.0,
            tokens_ratio: 0.6,
        }];
        let mut json = String::new();
        ldf::text::push_json_value(&Value::Array(recs.clone()), &mut json);
        let (d, text) = select_ldf(&recs, "m", &ev, &Policy::default(), json.len() as u64);
        assert!(matches!(d.choice, Choice::Ldf { .. }), "{d:?}");
        assert_eq!(ldf::decode(&text.unwrap()).unwrap(), recs);
        let (d, text) = select_ldf(&recs, "other", &ev, &Policy::default(), json.len() as u64);
        assert_eq!(d.choice, Choice::Json);
        assert!(text.is_none());
    }
}
