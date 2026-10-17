use ldf_data::{Number, Value};

#[test]
fn big_integer_literal_parses_to_nearest_double() {
    let v: Value = serde_json::from_str("123456789012345680000").unwrap();
    let Value::Number(Number::F64(f)) = v else {
        panic!("expected f64, got {v:?}")
    };
    assert_eq!(
        format!("{}", 1.2345678901234568e20_f64),
        "123456789012345680000"
    );
    assert_eq!(
        f.to_bits(),
        1.2345678901234568e20_f64.to_bits(),
        "serde_json gave {f:e}"
    );
    let v2: Value = serde_json::from_str("0.30000000000000004").unwrap();
    assert_eq!(v2.to_string(), "0.30000000000000004");
}
