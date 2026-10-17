//! Token-level rules shared by the TOON encoder and decoder: quoting,
//! escaping, key syntax, numeric-likeness (spec sections 4 and 7).

use ldf_data::{Number, Value};

/// Spec 7.2: numeric-like strings must be quoted. ASCII digits only,
/// case-insensitive exponent marker.
pub fn is_numeric_like(s: &str) -> bool {
    let b = s.as_bytes();
    let mut i = 0;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i == start {
        return false;
    }
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let s2 = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == s2 {
            return false;
        }
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        i += 1;
        if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
            i += 1;
        }
        let s3 = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == s3 {
            return false;
        }
    }
    i == b.len()
}

/// Spec 7.2: does this string value need quotes under `delim`?
pub fn needs_quotes(s: &str, delim: char) -> bool {
    if s.is_empty() {
        return true;
    }
    let first = s.chars().next().unwrap();
    let last = s.chars().next_back().unwrap();
    if matches!(first, ' ' | '\t') || matches!(last, ' ' | '\t') {
        return true;
    }
    if s == "true" || s == "false" || s == "null" {
        return true;
    }
    if first == '-' || first == '#' {
        return true;
    }
    if is_numeric_like(s) {
        return true;
    }
    s.chars().any(|c| {
        matches!(c, ':' | '"' | '\\' | '[' | ']' | '{' | '}') || c < '\u{20}' || c == delim
    })
}

/// Spec 7.3: keys that may stay unquoted.
pub fn is_bare_key(k: &str) -> bool {
    let mut chars = k.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
}

/// Spec 7.1 escaping, with surrounding quotes.
pub fn push_quoted(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < '\u{20}' => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

pub fn push_key(k: &str, out: &mut String) {
    if is_bare_key(k) {
        out.push_str(k);
    } else {
        push_quoted(k, out);
    }
}

pub fn push_string_value(s: &str, delim: char, out: &mut String) {
    if needs_quotes(s, delim) {
        push_quoted(s, out);
    } else {
        out.push_str(s);
    }
}

/// Encodes a primitive. Callers guarantee `v` is not an array or object.
pub fn push_primitive(v: &Value, delim: char, out: &mut String) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(n) => {
            if !n.write_canonical(out) {
                out.push_str("null");
            }
        }
        Value::String(s) => push_string_value(s, delim, out),
        Value::Array(_) | Value::Object(_) => unreachable!("push_primitive on a container"),
    }
}

/// Spec 7.1 unescaping of the *inside* of a quoted token.
pub fn unescape(inner: &str) -> Result<String, String> {
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('u') => {
                let mut code = 0u32;
                for _ in 0..4 {
                    let h = chars.next().ok_or("\\u escape needs four hex digits")?;
                    let d = h.to_digit(16).ok_or("\\u escape needs four hex digits")?;
                    code = code * 16 + d;
                }
                match char::from_u32(code) {
                    Some(ch) if !(0xD800..=0xDFFF).contains(&code) => out.push(ch),
                    _ => return Err(format!("surrogate code point \\u{code:04x} is not allowed")),
                }
            }
            Some(o) => return Err(format!("invalid escape sequence \\{o}")),
            None => return Err("dangling backslash".into()),
        }
    }
    Ok(out)
}

/// Spec 4: decodes an unquoted or quoted primitive token (already trimmed).
/// `[]` handling is the caller's job.
pub fn decode_primitive(tok: &str) -> Result<Value, String> {
    if let Some(rest) = tok.strip_prefix('"') {
        // Quoted-token boundary (7.4): the closing quote must be last.
        let inner = unquoted_span_end(rest)
            .filter(|&end| end + 1 == rest.len())
            .map(|end| &rest[..end])
            .ok_or_else(|| {
                if find_closing_quote(rest).is_none() {
                    "unterminated quoted string".to_string()
                } else {
                    "unexpected characters after closing quote".to_string()
                }
            })?;
        return unescape(inner).map(Value::String);
    }
    Ok(match tok {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        "null" => Value::Null,
        _ => match ldf_data::number::parse_json_number(tok) {
            Some(n) => Value::Number(n),
            None => Value::String(tok.to_owned()),
        },
    })
}

#[allow(dead_code)]
fn _unused(_: Number) {}

/// Index (in `rest`, which starts just after an opening quote) of the
/// closing quote, skipping escapes.
pub fn find_closing_quote(rest: &str) -> Option<usize> {
    let b = rest.as_bytes();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            b'"' => return Some(i),
            _ => i += 1,
        }
    }
    None
}

fn unquoted_span_end(rest: &str) -> Option<usize> {
    find_closing_quote(rest)
}

/// Scans `s` and calls `f(byte_index, ch)` for every character outside quoted
/// spans; stops at the first index for which `f` returns true and returns it.
pub fn find_unquoted(s: &str, mut f: impl FnMut(char) -> bool) -> Option<usize> {
    let mut in_quote = false;
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        if in_quote {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_quote = false;
            }
            continue;
        }
        if c == '"' {
            in_quote = true;
            continue;
        }
        if f(c) {
            return Some(i);
        }
    }
    None
}

/// Splits on unquoted `delim`, trimming spaces around every token and
/// preserving empty tokens (spec 11.2).
pub fn split_unquoted(s: &str, delim: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut in_quote = false;
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        if in_quote {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_quote = false;
            }
            continue;
        }
        if c == '"' {
            in_quote = true;
        } else if c == delim {
            parts.push(trim_spaces(&s[start..i]));
            start = i + c.len_utf8();
        }
    }
    parts.push(trim_spaces(&s[start..]));
    parts
}

/// Spec 12: token trimming removes U+0020 only.
pub fn trim_spaces(s: &str) -> &str {
    s.trim_matches(' ')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_like() {
        for s in ["42", "-3.14", "05", "+1", "1e-6", "1E5", "0"] {
            assert!(is_numeric_like(s), "{s}");
        }
        for s in ["1.", ".5", "1e", "abc", "1,000", "", "-", "0x10"] {
            assert!(!is_numeric_like(s), "{s}");
        }
    }

    #[test]
    fn quoting_rules() {
        assert!(needs_quotes("", ','));
        assert!(needs_quotes(" x", ','));
        assert!(needs_quotes("true", ','));
        assert!(needs_quotes("a,b", ','));
        assert!(!needs_quotes("a,b", '\t'));
        assert!(needs_quotes("a:b", ','));
        assert!(needs_quotes("-x", ','));
        assert!(needs_quotes("#x", ','));
        assert!(!needs_quotes("hello world", ','));
        assert!(!needs_quotes("héllo 🙂", ','));
    }

    #[test]
    fn escape_round_trip() {
        let s = "a\"b\\c\nd\te\u{1}";
        let mut q = String::new();
        push_quoted(s, &mut q);
        assert_eq!(q, "\"a\\\"b\\\\c\\nd\\te\\u0001\"");
        assert_eq!(unescape(&q[1..q.len() - 1]).unwrap(), s);
    }

    #[test]
    fn split_preserves_empties_and_quotes() {
        assert_eq!(
            split_unquoted("a, \"b,c\" ,,d", ','),
            vec!["a", "\"b,c\"", "", "d"]
        );
    }
}
