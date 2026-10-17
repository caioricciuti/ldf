//! Cell and key text rules (spec sections 3, 5, 6).

use std::fmt::Write as _;

use ldf_data::{Number, Value};

/// A bare path segment: `[A-Za-z_][A-Za-z0-9_]*`.
pub fn is_bare_segment(s: &str) -> bool {
    let mut it = s.chars();
    matches!(it.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && it.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub fn push_segment(seg: &str, out: &mut String) {
    if is_bare_segment(seg) {
        out.push_str(seg);
    } else {
        push_json_string(seg, out);
    }
}

pub fn push_path(path: &[String], out: &mut String) {
    for (i, seg) in path.iter().enumerate() {
        if i > 0 {
            out.push('.');
        }
        push_segment(seg, out);
    }
}

/// Spec 5: does a string cell need JSON quoting?
pub fn string_needs_quotes(s: &str) -> bool {
    if s.is_empty() || s == "null" || s == "true" || s == "false" {
        return true;
    }
    let first = s.chars().next().unwrap();
    let last = s.chars().next_back().unwrap();
    if matches!(first, ' ' | '\t') || matches!(last, ' ' | '\t') {
        return true;
    }
    s.chars()
        .any(|c| matches!(c, ',' | '"' | '\\' | '\r' | '\n') || c < '\u{20}')
}

pub fn push_json_string(s: &str, out: &mut String) {
    // serde_json escapes exactly the JSON set and leaves UTF-8 literal.
    out.push_str(&serde_json::to_string(s).expect("string serialises"));
}

pub fn push_string_cell(s: &str, out: &mut String) {
    if string_needs_quotes(s) {
        push_json_string(s, out);
    } else {
        out.push_str(s);
    }
}

/// Spec 6: float text that keeps its kind.
pub fn push_float(v: f64, out: &mut String) {
    if v.is_finite() {
        let _ = write!(out, "{v:?}");
    } else {
        out.push_str("null");
    }
}

pub fn push_int(n: Number, out: &mut String) {
    match n {
        Number::I64(v) => {
            let _ = write!(out, "{v}");
        }
        Number::U64(v) => {
            let _ = write!(out, "{v}");
        }
        Number::F64(v) => push_float(v, out),
    }
}

/// Compact JSON literal for `j` cells.
pub fn push_json_value(v: &Value, out: &mut String) {
    out.push_str(&serde_json::to_string(v).expect("value serialises"));
}

/// Index just past the closing quote of a JSON string literal that starts
/// at `s[0] == '"'`, or `None` when unterminated.
pub fn json_string_end(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let mut i = 1;
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            b'"' => return Some(i + 1),
            _ => i += 1,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoting() {
        for s in [
            "", "null", "true", " x", "x ", "a,b", "q\"q", "back\\", "a\nb",
        ] {
            assert!(string_needs_quotes(s), "{s:?}");
        }
        for s in [
            "42",
            "007",
            "1e5",
            "-",
            "-x",
            "#x",
            "[x]",
            "a:b",
            "hello world",
            "é🙂",
            "NaN",
        ] {
            assert!(!string_needs_quotes(s), "{s:?}");
        }
    }

    #[test]
    fn floats_keep_kind() {
        let f = |v: f64| {
            let mut s = String::new();
            push_float(v, &mut s);
            s
        };
        assert_eq!(f(1.0), "1.0");
        assert_eq!(f(0.1 + 0.2), "0.30000000000000004");
        assert_eq!(f(1e21), "1e21");
        assert_eq!(f(1e-7), "1e-7");
        assert_eq!(f(-0.0), "-0.0");
        assert_eq!(f(f64::NAN), "null");
    }

    #[test]
    fn segments() {
        assert!(is_bare_segment("customer_id"));
        assert!(!is_bare_segment("with space"));
        assert!(!is_bare_segment("dotted.key"));
        assert!(!is_bare_segment("1abc"));
        assert!(!is_bare_segment(""));
    }
}
