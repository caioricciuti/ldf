//! YAML baseline: block style, two-space indent, one sequence item per record.
//!
//! The emitter is ours (about a hundred lines, YAML 1.2 core schema). It
//! quotes conservatively so that any 1.1 or 1.2 parser reads strings back as
//! strings: anything that looks like null, a boolean (including the 1.1
//! `yes/no/on/off` family), a number, or starts with an indicator character
//! is double-quoted with JSON-style escapes. Floats keep a decimal point or
//! exponent so integer-valued floats come back as floats.
//!
//! Decoding uses `yaml-rust2` (pure Rust, YAML 1.2) and maps its node types
//! onto [`Value`]. Anchors and aliases are rejected: the encoder never emits
//! them, and silently expanding them would hide a size difference.

use std::fmt::Write as _;

use ldf_data::{Number, Value};
use yaml_rust2::{Yaml, YamlLoader};

use crate::{Capabilities, DecodeError, EncodeError, Format, StreamEncoder};

pub struct YamlFormat;

const CAPS: Capabilities = Capabilities {
    nested: true,
    typed: true,
    missing_vs_null: true,
    streaming_encode: true,
    requires_count: false,
};

// ---- scalar text ----------------------------------------------------------

fn looks_like_special(s: &str) -> bool {
    // YAML 1.2 core schema null, bool, int, float, plus the YAML 1.1 bool
    // words that older parsers still coerce.
    let lower = s.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "" | "~" | "null" | "true" | "false" | "yes" | "no" | "on" | "off" | "y" | "n" | ".inf" | "-.inf" | "+.inf" | ".nan"
    ) {
        return true;
    }
    let b = lower.as_bytes();
    let mut i = 0;
    if b[i] == b'+' || b[i] == b'-' {
        i += 1;
    }
    let rest = &lower[i..];
    if rest.starts_with("0x") && rest.len() > 2 && rest[2..].bytes().all(|c| c.is_ascii_hexdigit()) {
        return true;
    }
    if rest.starts_with("0o") && rest.len() > 2 && rest[2..].bytes().all(|c| (b'0'..=b'7').contains(&c)) {
        return true;
    }
    // [0-9]+ | (\.[0-9]+ | [0-9]+(\.[0-9]*)?)([eE][-+]?[0-9]+)?
    let rb = rest.as_bytes();
    let mut j = 0;
    let mut digits = 0;
    while j < rb.len() && rb[j].is_ascii_digit() {
        j += 1;
        digits += 1;
    }
    if j < rb.len() && rb[j] == b'.' {
        j += 1;
        while j < rb.len() && rb[j].is_ascii_digit() {
            j += 1;
            digits += 1;
        }
    }
    if digits == 0 {
        return false;
    }
    if j < rb.len() && (rb[j] == b'e') {
        j += 1;
        if j < rb.len() && (rb[j] == b'+' || rb[j] == b'-') {
            j += 1;
        }
        let s2 = j;
        while j < rb.len() && rb[j].is_ascii_digit() {
            j += 1;
        }
        if j == s2 {
            return false;
        }
    }
    j == rb.len()
}

fn needs_quotes(s: &str) -> bool {
    if looks_like_special(s) {
        return true;
    }
    let first = s.chars().next().unwrap();
    let last = s.chars().next_back().unwrap();
    if first.is_whitespace() || last.is_whitespace() || last == ':' {
        return true;
    }
    if matches!(first, '-' | '?' | ':' | ',' | '[' | ']' | '{' | '}' | '#' | '&' | '*' | '!' | '|' | '>' | '\'' | '"' | '%' | '@' | '`') {
        return true;
    }
    if s.contains(": ") || s.contains(" #") || s.contains('\t') {
        return true;
    }
    s.chars().any(|c| c < ' ' || c == '\u{7f}' || c == '\u{feff}' || c == '\u{2028}' || c == '\u{2029}')
}

fn push_quoted(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' || c == '\u{7f}' || c == '\u{feff}' || c == '\u{2028}' || c == '\u{2029}' => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn push_string(s: &str, out: &mut String) {
    if needs_quotes(s) {
        push_quoted(s, out);
    } else {
        out.push_str(s);
    }
}

fn push_number(n: Number, out: &mut String) {
    match n {
        Number::I64(v) => {
            let _ = write!(out, "{v}");
        }
        Number::U64(v) => {
            let _ = write!(out, "{v}");
        }
        Number::F64(v) => {
            if v.is_nan() {
                out.push_str(".nan");
            } else if v.is_infinite() {
                out.push_str(if v > 0.0 { ".inf" } else { "-.inf" });
            } else {
                // Debug keeps a ".0" on integral values and uses the shortest
                // round-trip digits; both are valid YAML core floats.
                let _ = write!(out, "{v:?}");
            }
        }
    }
}

fn push_scalar(v: &Value, out: &mut String) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(n) => push_number(*n, out),
        Value::String(s) => push_string(s, out),
        Value::Array(a) if a.is_empty() => out.push_str("[]"),
        Value::Object(o) if o.is_empty() => out.push_str("{}"),
        Value::Array(_) | Value::Object(_) => unreachable!("push_scalar on a non-empty container"),
    }
}

fn is_inline(v: &Value) -> bool {
    match v {
        Value::Array(a) => a.is_empty(),
        Value::Object(o) => o.is_empty(),
        _ => true,
    }
}

fn indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

/// Writes the fields of a non-empty object. The first field may continue a
/// `- ` line (`on_dash_line`), the rest start at `depth`.
fn push_object(o: &[(String, Value)], depth: usize, on_dash_line: bool, out: &mut String) {
    for (i, (k, v)) in o.iter().enumerate() {
        if !(i == 0 && on_dash_line) {
            indent(out, depth);
        }
        push_string(k, out);
        out.push(':');
        if is_inline(v) {
            out.push(' ');
            push_scalar(v, out);
            out.push('\n');
        } else {
            out.push('\n');
            push_block(v, depth + 1, out);
        }
    }
}

/// Writes a non-empty container as block content at `depth`.
fn push_block(v: &Value, depth: usize, out: &mut String) {
    match v {
        Value::Object(o) => push_object(o, depth, false, out),
        Value::Array(a) => {
            for item in a {
                push_item(item, depth, out);
            }
        }
        _ => unreachable!(),
    }
}

/// One sequence item whose dash sits at `depth`.
fn push_item(item: &Value, depth: usize, out: &mut String) {
    indent(out, depth);
    match item {
        Value::Object(o) if !o.is_empty() => {
            out.push_str("- ");
            push_object(o, depth + 1, true, out);
        }
        Value::Array(a) if !a.is_empty() => {
            out.push_str("-\n");
            for inner in a {
                push_item(inner, depth + 1, out);
            }
        }
        scalar => {
            out.push_str("- ");
            push_scalar(scalar, out);
            out.push('\n');
        }
    }
}

// ---- decoding -------------------------------------------------------------

fn from_yaml(y: &Yaml) -> Result<Value, String> {
    Ok(match y {
        Yaml::Null => Value::Null,
        Yaml::Boolean(b) => Value::Bool(*b),
        Yaml::Integer(i) => Value::from(*i),
        Yaml::Real(s) => {
            let t = s.trim_start_matches('+');
            let f = match t {
                ".inf" | ".Inf" | ".INF" => f64::INFINITY,
                "-.inf" | "-.Inf" | "-.INF" => f64::NEG_INFINITY,
                ".nan" | ".NaN" | ".NAN" => f64::NAN,
                _ => t.parse::<f64>().map_err(|e| format!("bad float {s:?}: {e}"))?,
            };
            Value::from(f)
        }
        Yaml::String(s) => Value::String(s.clone()),
        Yaml::Array(a) => Value::Array(a.iter().map(from_yaml).collect::<Result<_, _>>()?),
        Yaml::Hash(h) => {
            let mut fields = Vec::with_capacity(h.len());
            for (k, v) in h.iter() {
                let key = match k {
                    Yaml::String(s) => s.clone(),
                    Yaml::Integer(i) => i.to_string(),
                    Yaml::Real(r) => r.clone(),
                    Yaml::Boolean(b) => b.to_string(),
                    Yaml::Null => String::new(),
                    other => return Err(format!("unsupported key type {other:?}")),
                };
                fields.push((key, from_yaml(v)?));
            }
            Value::Object(fields)
        }
        Yaml::Alias(_) => return Err("aliases are not supported".into()),
        Yaml::BadValue => return Err("bad value".into()),
    })
}

impl Format for YamlFormat {
    fn name(&self) -> &'static str {
        "yaml"
    }
    fn extension(&self) -> &'static str {
        "yaml"
    }
    fn capabilities(&self) -> Capabilities {
        CAPS
    }
    fn encode(&self, records: &[Value], out: &mut Vec<u8>) -> Result<(), EncodeError> {
        let mut s = String::new();
        if records.is_empty() {
            s.push_str("[]\n");
        }
        for r in records {
            push_item(r, 0, &mut s);
        }
        out.extend_from_slice(s.as_bytes());
        Ok(())
    }
    fn decode(&self, bytes: &[u8]) -> Result<Vec<Value>, DecodeError> {
        let err = |m: String| DecodeError {
            format: "yaml",
            line: None,
            message: m,
        };
        let text = std::str::from_utf8(bytes).map_err(|e| err(e.to_string()))?;
        let docs = YamlLoader::load_from_str(text).map_err(|e| err(e.to_string()))?;
        let Some(root) = docs.first() else {
            return Ok(Vec::new());
        };
        match from_yaml(root).map_err(err)? {
            Value::Array(items) => Ok(items),
            other => Err(err(format!("expected a root sequence, found {}", other.type_name()))),
        }
    }
    fn stream_encoder(&self) -> Option<Box<dyn StreamEncoder>> {
        Some(Box::new(YamlStream { any: false }))
    }
}

struct YamlStream {
    any: bool,
}

impl StreamEncoder for YamlStream {
    fn record(&mut self, rec: &Value, out: &mut Vec<u8>) -> Result<(), EncodeError> {
        let mut s = String::new();
        push_item(rec, 0, &mut s);
        out.extend_from_slice(s.as_bytes());
        self.any = true;
        Ok(())
    }
    fn finish(&mut self, out: &mut Vec<u8>) -> Result<(), EncodeError> {
        if !self.any {
            out.extend_from_slice(b"[]\n");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldf_data::obj;

    #[test]
    fn shapes() {
        let recs = vec![
            obj! {"id" => 1, "name" => "Ada", "tags" => vec![Value::from("a"), Value::from("b,c")], "addr" => obj! {"city" => "Oslo"}, "e" => Vec::<Value>::new(), "o" => obj! {}},
            obj! {"id" => 2.0, "name" => "yes", "n" => Value::Null, "m" => Value::Array(vec![Value::Array(vec![Value::from(1), Value::from(2)])])},
        ];
        let s = YamlFormat.encode_to_string(&recs).unwrap();
        assert_eq!(
            s,
            "- id: 1\n  name: Ada\n  tags:\n    - a\n    - b,c\n  addr:\n    city: Oslo\n  e: []\n  o: {}\n- id: 2.0\n  name: \"yes\"\n  \"n\": null\n  m:\n    -\n      - 1\n      - 2\n"
        );
        let back = YamlFormat.decode(s.as_bytes()).unwrap();
        assert_eq!(back, recs);
    }

    #[test]
    fn tricky_strings_survive() {
        let tricky = [
            "", " lead", "trail ", "true", "No", "null", "~", "007", "1e5", "0x10", "0o17", ".5", "1.", "+1", "-", "- item", "key: value",
            "a #b", "#x", "[x]", "{x}", "x:", "a\nb", "tab\there", "quote\"s", "back\\slash", "\u{feff}bom", "zero\u{0}byte", "é🙂",
            "1_000", "0b1", ".inf", "-.inf", ".nan", "12:30", "2026-10-17", "@at", "`tick", "!tag", "&anchor", "*alias", "|lit", ">fold",
        ];
        let recs: Vec<Value> = tricky.iter().map(|t| obj! {"s" => *t, "k" => obj! {*t => 1}}).collect();
        let s = YamlFormat.encode_to_string(&recs).unwrap();
        let back = YamlFormat.decode(s.as_bytes()).unwrap();
        for (i, (a, b)) in recs.iter().zip(&back).enumerate() {
            assert_eq!(a, b, "case {i}: {:?}\n{s}", tricky[i]);
        }
    }

    #[test]
    fn numbers_keep_kind() {
        let recs = vec![obj! {"i" => 1, "f" => 1.0, "big" => 1e21, "tiny" => 1e-7, "neg0" => -0.0}];
        let s = YamlFormat.encode_to_string(&recs).unwrap();
        let back = YamlFormat.decode(s.as_bytes()).unwrap();
        assert!(back[0].semantic_eq(&recs[0]), "{s}");
        assert!(matches!(back[0].get("f"), Some(Value::Number(Number::F64(_)))));
        assert!(matches!(back[0].get("i"), Some(Value::Number(Number::I64(1)))));
        // yaml-rust2 has no u64: integers above i64::MAX come back as f64
        // (documented loss, reported as lossy by the round-trip check).
        let big = vec![obj! {"u" => u64::MAX}];
        let s = YamlFormat.encode_to_string(&big).unwrap();
        let back = YamlFormat.decode(s.as_bytes()).unwrap();
        assert!(matches!(back[0].get("u"), Some(Value::Number(Number::F64(_)))));
    }
}
