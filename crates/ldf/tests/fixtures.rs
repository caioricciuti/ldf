//! LDF v0 conformance fixtures, in the same JSON shape as the TOON suite so
//! a second implementation can reuse them. Encode cases compare exact text;
//! decode cases compare records with strict structural equality (number
//! kinds included) or expect an error.

use std::path::Path;

use ldf_data::Value;

#[derive(serde::Deserialize)]
struct FixtureFile {
    category: String,
    tests: Vec<Case>,
}

#[derive(serde::Deserialize)]
struct Case {
    name: String,
    input: Value,
    expected: Value,
    #[serde(default, rename = "shouldError")]
    should_error: bool,
}

fn run(file: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(file);
    let text = std::fs::read_to_string(&path).unwrap();
    let f: FixtureFile = serde_json::from_str(&text).unwrap();
    let mut failures = Vec::new();
    for (i, case) in f.tests.iter().enumerate() {
        let tag = format!("{file}[{i}] {}", case.name);
        match f.category.as_str() {
            "encode" => {
                let input = case.input.as_array().expect("encode input is an array");
                let got = ldf::encode(input);
                let want = case.expected.as_str().unwrap();
                if got != want {
                    failures.push(format!("{tag}\n  want: {want:?}\n  got:  {got:?}"));
                } else if let Ok(back) = ldf::decode(&got)
                    && back != *input
                {
                    failures.push(format!(
                        "{tag}\n  encode output does not decode back to the input: {got:?}"
                    ));
                }
            }
            "decode" => {
                let input = case.input.as_str().unwrap();
                match (ldf::decode(input), case.should_error) {
                    (Err(_), true) => {}
                    (Ok(v), true) => failures.push(format!(
                        "{tag}\n  expected an error, got {}",
                        Value::Array(v)
                    )),
                    (Err(e), false) => failures.push(format!("{tag}\n  unexpected error: {e}")),
                    (Ok(got), false) => {
                        let want = case.expected.as_array().unwrap();
                        if got != *want {
                            failures.push(format!(
                                "{tag}\n  want: {}\n  got:  {}",
                                case.expected,
                                Value::Array(got)
                            ));
                        }
                    }
                }
            }
            other => panic!("unknown category {other}"),
        }
    }
    eprintln!(
        "{file}: {} cases, {} failures",
        f.tests.len(),
        failures.len()
    );
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn encode_fixtures() {
    run("encode.json");
}

#[test]
fn decode_fixtures() {
    run("decode.json");
}
