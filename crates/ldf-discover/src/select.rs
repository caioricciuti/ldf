//! The selector over the full candidate set, JSON, TOON and LDF v0.1 at
//! sparse threshold 0.5 and 0.75, measured in the caller's token count and
//! round-trip checked through `ldf-formats`. The decision rule lives in
//! `ldf-select` (shared with `ldf-mcp`, which offers JSON and LDF only).

use ldf_bench::roundtrip::{self, Level};
use ldf_data::Value;
use ldf_formats::Format;
use ldf_formats::ldf::LdfFormat;
pub use ldf_select::{
    CANDIDATES, Choice, Decision, Evidence, Measured, Policy, Profile, SPARSE_FILL, Traits,
    Verdict, profile, sign_test,
};

fn format_of(choice: Choice) -> Box<dyn Format> {
    match choice {
        Choice::Json => Box::new(ldf_formats::json::Json),
        Choice::Toon => Box::new(ldf_formats::toon::Toon::default()),
        Choice::Ldf { sparse_threshold } => {
            Box::new(LdfFormat::sparse("ldf", "ldf", sparse_threshold))
        }
    }
}

/// Decides for `records` and `model`. `count` returns the token count of a
/// text (o200k in the CLI: exact for GPT, a proxy for Claude).
pub fn select(
    records: &[Value],
    model: &str,
    evidence: &[Evidence],
    policy: &Policy,
    count: &dyn Fn(&str) -> u64,
) -> Decision {
    ldf_select::decide(records, model, evidence, policy, &CANDIDATES, &mut |c| {
        let fmt = format_of(c);
        let rt = roundtrip::check(fmt.as_ref(), records);
        let lossless = match rt.level {
            Level::Byte | Level::Semantic => true,
            Level::SemanticUnordered => !policy.require_key_order,
            _ => false,
        };
        let size = lossless
            .then(|| {
                let mut out = Vec::new();
                fmt.encode(records, &mut out)
                    .ok()
                    .map(|_| count(&String::from_utf8_lossy(&out)))
            })
            .flatten();
        let note = (!lossless).then(|| {
            format!(
                "not accepted at round-trip level {} {}",
                rt.level.label(),
                rt.detail.clone().unwrap_or_default()
            )
        });
        Measured {
            size,
            round_trip: rt.level.label().into(),
            note,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldf_data::obj;

    fn row(model: &str, family: &str) -> Evidence {
        Evidence {
            model: model.into(),
            run: "test".into(),
            dataset: "d".into(),
            records: 200,
            traits: vec![],
            family: family.into(),
            thresholds: vec![0.0, 0.25, 0.5, 0.75],
            format: family.into(),
            n: 100,
            k: 90,
            json_k: 90,
            wins: 2,
            losses: 2,
            p: sign_test(2, 2),
            tokens_ratio: 0.6,
        }
    }

    fn flat(n: usize) -> Vec<Value> {
        (0..n)
            .map(|i| obj! {"id" => i as i64, "name" => format!("user {i}"), "age" => (20 + i % 50) as i64, "ok" => i % 2 == 0})
            .collect()
    }

    fn bytes(s: &str) -> u64 {
        s.len() as u64
    }

    #[test]
    fn toon_is_a_candidate_here() {
        let ev = vec![row("m", "toon")];
        let d = select(&flat(50), "m", &ev, &Policy::default(), &bytes);
        assert_eq!(d.choice, Choice::Toon, "{d:?}");
        assert_eq!(
            select(&flat(50), "other", &ev, &Policy::default(), &bytes).choice,
            Choice::Json
        );
    }

    #[test]
    fn smallest_passing_format_wins() {
        let ev = vec![row("m", "ldf"), row("m", "toon")];
        let d = select(&flat(50), "m", &ev, &Policy::default(), &bytes);
        assert!(matches!(d.choice, Choice::Ldf { .. }), "{d:?}");
    }
}
