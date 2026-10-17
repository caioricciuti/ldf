//! Numeric domain of the value model.
//!
//! The documented numeric domain is `i64`, `u64` (for integers above
//! `i64::MAX`) and `f64`. Integers outside the `u64` range and decimals that
//! do not fit `f64` exactly are out of domain; JSON readers map them to the
//! nearest `f64` and this loss is counted as a *semantic round-trip failure*
//! by the benchmark, never hidden.

use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Number {
    I64(i64),
    U64(u64),
    F64(f64),
}

impl Number {
    pub fn as_f64(self) -> f64 {
        match self {
            Number::I64(v) => v as f64,
            Number::U64(v) => v as f64,
            Number::F64(v) => v,
        }
    }

    /// Integer view when the number is mathematically an integer that fits
    /// in `i128`. Used by semantic equality and aggregation tasks.
    pub fn as_i128(self) -> Option<i128> {
        match self {
            Number::I64(v) => Some(v as i128),
            Number::U64(v) => Some(v as i128),
            Number::F64(v) => {
                if v.is_finite() && v.fract() == 0.0 && v.abs() < 1e30 {
                    Some(v as i128)
                } else {
                    None
                }
            }
        }
    }

    pub fn is_integer_repr(self) -> bool {
        !matches!(self, Number::F64(_))
    }

    pub fn is_finite(self) -> bool {
        match self {
            Number::F64(v) => v.is_finite(),
            _ => true,
        }
    }

    /// Mathematical equality: `1 == 1.0`, `-0.0 == 0`, NaN never equal.
    pub fn semantic_eq(self, other: Number) -> bool {
        match (self.as_i128(), other.as_i128()) {
            (Some(a), Some(b)) => a == b,
            (None, None) => self.as_f64() == other.as_f64(),
            _ => false,
        }
    }

    pub fn semantic_cmp(self, other: Number) -> Option<Ordering> {
        match (self.as_i128(), other.as_i128()) {
            (Some(a), Some(b)) => Some(a.cmp(&b)),
            _ => self.as_f64().partial_cmp(&other.as_f64()),
        }
    }

    /// Canonical decimal text shared by TOON and the CSV family.
    ///
    /// Integers print as integers. Floats print in the shortest form that
    /// round-trips. Values with `1e-6 <= |x| < 1e21` (or zero) never use an
    /// exponent, matching the TOON v4.4 canonical range (which itself matches
    /// JavaScript `Number#toString`). Outside that range the exponent form
    /// `d.ddde[+-]x` is used. `-0.0` prints as `0`. Non-finite values are the
    /// caller's problem: this returns `None` for them.
    pub fn canonical_text(self) -> Option<String> {
        let mut s = String::new();
        self.write_canonical(&mut s).then_some(s)
    }

    /// Writes the canonical form into `out`; returns `false` for NaN/inf
    /// without writing anything.
    pub fn write_canonical(self, out: &mut String) -> bool {
        use std::fmt::Write as _;
        match self {
            Number::I64(v) => {
                let _ = write!(out, "{v}");
                true
            }
            Number::U64(v) => {
                let _ = write!(out, "{v}");
                true
            }
            Number::F64(v) => {
                if !v.is_finite() {
                    return false;
                }
                if v == 0.0 {
                    out.push('0');
                    return true;
                }
                let a = v.abs();
                if (1e-6..1e21).contains(&a) {
                    // Rust's Display for f64 is shortest-round-trip and never
                    // uses an exponent; it prints 1.0 as "1".
                    let _ = write!(out, "{v}");
                } else {
                    // Rust's LowerExp prints e.g. "1e21", "1.5e-7", "-2e300".
                    // The spec wants an explicit exponent sign.
                    let e = format!("{v:e}");
                    match e.split_once('e') {
                        Some((m, x)) if !x.starts_with('-') => {
                            out.push_str(m);
                            out.push_str("e+");
                            out.push_str(x);
                        }
                        _ => out.push_str(&e),
                    }
                }
                true
            }
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.canonical_text() {
            Some(s) => f.write_str(&s),
            None => f.write_str("null"),
        }
    }
}

impl From<i64> for Number {
    fn from(v: i64) -> Self {
        Number::I64(v)
    }
}
impl From<u64> for Number {
    fn from(v: u64) -> Self {
        if v <= i64::MAX as u64 {
            Number::I64(v as i64)
        } else {
            Number::U64(v)
        }
    }
}
impl From<f64> for Number {
    fn from(v: f64) -> Self {
        Number::F64(v)
    }
}
impl From<i32> for Number {
    fn from(v: i32) -> Self {
        Number::I64(v as i64)
    }
}
impl From<u32> for Number {
    fn from(v: u32) -> Self {
        Number::I64(v as i64)
    }
}

/// Parses an unquoted token with exactly the JSON number grammar
/// (RFC 8259 section 6). Anything else returns `None`. This is the grammar
/// both TOON and our CSV type inference rely on, so it lives here.
pub fn parse_json_number(tok: &str) -> Option<Number> {
    let b = tok.as_bytes();
    if b.is_empty() {
        return None;
    }
    let mut i = 0;
    if b[i] == b'-' {
        i += 1;
    }
    if i >= b.len() {
        return None;
    }
    if b[i] == b'0' {
        i += 1;
    } else if b[i].is_ascii_digit() {
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
    } else {
        return None;
    }
    let mut is_float = false;
    if i < b.len() && b[i] == b'.' {
        is_float = true;
        i += 1;
        let start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == start {
            return None;
        }
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        is_float = true;
        i += 1;
        if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
            i += 1;
        }
        let start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == start {
            return None;
        }
    }
    if i != b.len() {
        return None;
    }
    if !is_float {
        if let Ok(v) = tok.parse::<i64>() {
            return Some(Number::I64(v));
        }
        if let Ok(v) = tok.parse::<u64>() {
            return Some(Number::U64(v));
        }
    }
    tok.parse::<f64>().ok().map(Number::F64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_forms() {
        let c = |n: Number| n.canonical_text().unwrap();
        assert_eq!(c(Number::I64(-12)), "-12");
        assert_eq!(c(Number::F64(1.0)), "1");
        assert_eq!(c(Number::F64(-0.0)), "0");
        assert_eq!(c(Number::F64(1.5)), "1.5");
        assert_eq!(c(Number::F64(0.000001)), "0.000001");
        assert_eq!(c(Number::F64(1e-7)), "1e-7");
        assert_eq!(c(Number::F64(1e21)), "1e+21");
        assert_eq!(c(Number::F64(1e20)), "100000000000000000000");
        assert_eq!(c(Number::F64(0.1 + 0.2)), "0.30000000000000004");
        assert!(Number::F64(f64::NAN).canonical_text().is_none());
    }

    #[test]
    fn json_number_grammar() {
        assert!(matches!(parse_json_number("0"), Some(Number::I64(0))));
        assert!(matches!(parse_json_number("-0"), Some(Number::I64(0))));
        assert!(matches!(parse_json_number("1e3"), Some(Number::F64(v)) if v == 1000.0));
        assert!(matches!(
            parse_json_number("18446744073709551615"),
            Some(Number::U64(_))
        ));
        for bad in [
            "05", "-05", ".5", "1.", "+5", "Infinity", "NaN", "0x10", "1_000", "", "-",
        ] {
            assert!(parse_json_number(bad).is_none(), "{bad:?} should not parse");
        }
    }

    #[test]
    fn semantic_equality() {
        assert!(Number::I64(1).semantic_eq(Number::F64(1.0)));
        assert!(Number::F64(-0.0).semantic_eq(Number::I64(0)));
        assert!(!Number::F64(1.5).semantic_eq(Number::I64(1)));
        assert!(!Number::F64(f64::NAN).semantic_eq(Number::F64(f64::NAN)));
    }
}
