//! Runs the official TOON v4.4.0 conformance fixtures against our encoder
//! and decoder. Fixture provenance: tests/toon-fixtures/README.md.
//!
//! Each file is one test; a failing case is reported by file path and array
//! index as the fixture README asks. The whole suite must pass; there is no
//! allow-list of known failures.

use std::path::Path;

use ldf_data::Value;
use ldf_formats::toon::{DecodeOptions, Delimiter, EncodeOptions, decode_value, encode_value};

#[derive(serde::Deserialize)]
struct FixtureFile {
    version: String,
    category: String,
    tests: Vec<Case>,
}

#[derive(serde::Deserialize)]
struct Case {
    name: String,
    input: Value,
    expected: Value,
    #[serde(default)]
    options: Options,
    #[serde(default, rename = "shouldError")]
    should_error: bool,
    #[serde(default, rename = "minSpecVersion")]
    min_spec_version: Option<String>,
}

#[derive(serde::Deserialize, Default)]
struct Options {
    #[serde(default)]
    delimiter: Option<String>,
    #[serde(default, rename = "indentSize")]
    indent_size: Option<usize>,
    #[serde(default)]
    strict: Option<bool>,
}

const TARGET: &str = "4.4";

fn version_le(a: &str, b: &str) -> bool {
    let p = |s: &str| -> Vec<u64> { s.split('.').map(|x| x.parse().unwrap_or(0)).collect() };
    p(a) <= p(b)
}

fn run_file(path: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap();
    let file: FixtureFile = serde_json::from_str(&text).unwrap();
    let mut failures = Vec::new();
    for (i, case) in file.tests.iter().enumerate() {
        if let Some(min) = &case.min_spec_version
            && !version_le(min, TARGET)
        {
            continue;
        }
        let tag = format!("{}[{i}] ({})", path.display(), case.name);
        match file.category.as_str() {
            "encode" => {
                let opts = EncodeOptions {
                    indent: case.options.indent_size.unwrap_or(2),
                    delimiter: case
                        .options
                        .delimiter
                        .as_deref()
                        .and_then(Delimiter::parse)
                        .unwrap_or_default(),
                };
                let got = encode_value(&case.input, &opts);
                let want = case.expected.as_str().unwrap_or_default();
                if got != want {
                    failures.push(format!("{tag}\n  want: {want:?}\n  got:  {got:?}"));
                }
            }
            "decode" => {
                let opts = DecodeOptions {
                    indent: case.options.indent_size.unwrap_or(2),
                    strict: case.options.strict.unwrap_or(true),
                };
                let input = case.input.as_str().unwrap_or_default();
                match (decode_value(input, &opts), case.should_error) {
                    (Err(_), true) => {}
                    (Ok(v), true) => failures.push(format!("{tag}\n  expected an error, got {v}")),
                    (Err(e), false) => failures.push(format!(
                        "{tag}\n  unexpected error: {e}\n  input: {input:?}"
                    )),
                    (Ok(got), false) => {
                        let want = &case.expected;
                        if !got.semantic_eq(want) {
                            failures.push(format!(
                                "{tag}\n  want: {want}\n  got:  {got}\n  input: {input:?}"
                            ));
                        }
                    }
                }
            }
            other => panic!("unknown category {other}"),
        }
    }
    assert!(
        file.version.starts_with("4."),
        "fixture {} targets spec {}",
        path.display(),
        file.version
    );
    failures
}

fn run_dir(dir: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/toon-fixtures")
        .join(dir);
    let mut files: Vec<_> = std::fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    assert!(!files.is_empty());
    let mut all = Vec::new();
    let mut total = 0;
    for f in &files {
        let text = std::fs::read_to_string(f).unwrap();
        let file: FixtureFile = serde_json::from_str(&text).unwrap();
        total += file.tests.len();
        all.extend(run_file(f));
    }
    eprintln!("{dir}: {} cases, {} failures", total, all.len());
    if !all.is_empty() {
        panic!(
            "{} of {total} {dir} fixtures failed:\n{}",
            all.len(),
            all.join("\n")
        );
    }
}

#[test]
fn encode_fixtures() {
    run_dir("encode");
}

#[test]
fn decode_fixtures() {
    run_dir("decode");
}
