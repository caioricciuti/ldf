//! Every configuration must decode back to its input; the v0.1 point must
//! be LDF v0.1 byte for byte.

use std::path::Path;

use ldf_bench::roundtrip::{self, Level};
use ldf_data::input::{ReadOptions, read_all};
use ldf_data::{Value, obj};
use ldf_discover::knobs::Config;
use ldf_discover::{Candidate, encode, space};

const DATASETS: [&str; 12] = [
    "flat_users",
    "nested_orders",
    "sparse_events",
    "high_card_strings",
    "repetitive_logs",
    "unicode_strings",
    "numeric_mix",
    "mixed_types",
    "wide_table",
    "deep_nesting",
    "empty_values",
    "long_text",
];

fn fixture(name: &str) -> Vec<Value> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/small")
        .join(format!("{name}.json"));
    read_all(&p, &ReadOptions::default()).expect("fixture")
}

fn assert_lossless(cfg: Config, recs: &[Value], what: &str) {
    let rt = roundtrip::check(&Candidate::new(cfg), recs);
    assert!(
        matches!(
            rt.level,
            Level::Byte | Level::Semantic | Level::SemanticUnordered
        ),
        "{what} with {}: {} {:?}\n{}",
        cfg.id(),
        rt.level.label(),
        rt.detail,
        encode(recs, &cfg)
    );
}

#[test]
fn v01_point_is_ldf_v01() {
    for name in DATASETS {
        let recs = fixture(name);
        assert_eq!(
            encode(&recs, &Config::LDF_V01),
            ldf::encode(&recs),
            "{name}"
        );
        assert_eq!(
            encode(&recs, &Config::LDF_V0),
            ldf::encode_with(&recs, &ldf::Options::V0),
            "{name}"
        );
    }
}

/// A deterministic slice of the space on every fixture. `ldf-discover
/// stage-a` runs all of it and reports any lossy output.
#[test]
fn sampled_space_is_lossless() {
    let s = space();
    let data: Vec<(&str, Vec<Value>)> = DATASETS.iter().map(|n| (*n, fixture(n))).collect();
    for cfg in s.iter().step_by(211) {
        for (name, recs) in &data {
            assert_lossless(*cfg, recs, name);
        }
    }
}

#[test]
fn edge_cases_are_lossless() {
    let cases: Vec<(&str, Vec<Value>)> = vec![
        (
            "null nested object",
            vec![
                obj! {"a" => obj! {"x" => 1}, "s" => "y"},
                obj! {"a" => Value::Null, "s" => "z"},
            ],
        ),
        (
            "string starting with @ in the first column",
            vec![obj! {"s" => "@x", "n" => 1}, obj! {"s" => "y", "n" => 2}],
        ),
        (
            "markers and keywords as strings",
            vec![
                obj! {"s" => "-", "t" => "?", "u" => ""},
                obj! {"s" => "null", "t" => "true", "u" => "1"},
                obj! {"s" => "[x]", "t" => "{y", "u" => "\\_a"},
                obj! {"s" => "a,b|c\td", "t" => "\"q\"", "u" => "line\nbreak\r"},
            ],
        ),
        (
            "every key sparse, some rows empty",
            vec![
                obj! {"a" => 1},
                obj! {},
                obj! {"b" => Value::Null},
                obj! {"c" => vec![obj! {"k" => 1}]},
            ],
        ),
        (
            "child key starting with @ and nested children",
            vec![
                obj! {"id" => "@1", "items" => vec![obj! {"k" => 1, "sub" => vec![obj! {"z" => true}]}]},
                obj! {"id" => "@2", "items" => Vec::<Value>::new()},
                obj! {"id" => "x", "items" => Value::Null},
            ],
        ),
        (
            "floats that look like ints",
            vec![
                obj! {"f" => 1.0, "g" => -0.0},
                obj! {"f" => 1e21, "g" => 2.5},
            ],
        ),
        (
            "scalar records",
            vec![Value::from(1), Value::from("x"), Value::Null],
        ),
    ];
    for (what, recs) in &cases {
        assert_eq!(encode(recs, &Config::LDF_V01), ldf::encode(recs), "{what}");
        for cfg in space() {
            assert_lossless(cfg, recs, what);
        }
    }
}

/// The `-` and empty null markers left the search after Stage B but the
/// codec still supports them.
#[test]
fn null_markers_outside_the_search_still_round_trip() {
    use ldf_discover::knobs::Nulls;
    let data: Vec<(&str, Vec<Value>)> = ["sparse_events", "empty_values", "mixed_types"]
        .iter()
        .map(|n| (*n, fixture(n)))
        .collect();
    for cfg in space().iter().step_by(97) {
        for nulls in [Nulls::DashEmpty, Nulls::EmptyQuestion] {
            let c = Config { nulls, ..*cfg };
            for (name, recs) in &data {
                assert_lossless(c, recs, name);
            }
        }
    }
}
