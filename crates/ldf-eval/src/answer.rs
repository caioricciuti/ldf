//! Canonical answers and lenient-but-principled comparison.
//!
//! Models are asked to answer with a bare JSON value. Comparison rules:
//! * numbers: equal within an absolute tolerance (default 1e-9) or a
//!   relative tolerance for aggregates (default 0.5 percent, stated per task);
//! * strings: exact after trimming and unquoting;
//! * sets (e.g. "list the ids"): order-insensitive, duplicates ignored;
//! * lists: order-sensitive;
//! * null: the literal `null`, or the words "null"/"missing"/"none" alone.
//!
//! A response that cannot be parsed into a comparable value is *malformed*,
//! which the scorer reports separately from *wrong*.

use ldf_data::{Number, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Answer {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
    /// Order-insensitive collection of scalars.
    Set(Vec<Scalar>),
    /// Order-sensitive list of scalars.
    List(Vec<Scalar>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Scalar {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
}

impl Scalar {
    pub fn from_value(v: &Value) -> Option<Scalar> {
        Some(match v {
            Value::Null => Scalar::Null,
            Value::Bool(b) => Scalar::Bool(*b),
            Value::Number(n) => Scalar::Number(n.as_f64()),
            Value::String(s) => Scalar::Text(s.clone()),
            _ => return None,
        })
    }

    fn matches(&self, other: &Scalar, tol: &Tolerance) -> bool {
        match (self, other) {
            (Scalar::Null, Scalar::Null) => true,
            (Scalar::Bool(a), Scalar::Bool(b)) => a == b,
            (Scalar::Number(a), Scalar::Number(b)) => tol.close(*a, *b),
            (Scalar::Text(a), Scalar::Text(b)) => norm(a) == norm(b),
            // A model may answer 42 where the data held "42"; accept when
            // the text parses to the same number.
            (Scalar::Number(a), Scalar::Text(t)) | (Scalar::Text(t), Scalar::Number(a)) => {
                t.trim().parse::<f64>().is_ok_and(|b| tol.close(*a, b))
            }
            (Scalar::Bool(a), Scalar::Text(t)) | (Scalar::Text(t), Scalar::Bool(a)) => {
                matches!(
                    (a, norm(t).as_str()),
                    (true, "true" | "yes") | (false, "false" | "no")
                )
            }
            (Scalar::Null, Scalar::Text(t)) | (Scalar::Text(t), Scalar::Null) => {
                matches!(norm(t).as_str(), "null" | "none" | "missing" | "n/a" | "")
            }
            _ => false,
        }
    }
}

fn norm(s: &str) -> String {
    s.trim()
        .trim_matches(|c| c == '"' || c == '\'' || c == '`')
        .trim()
        .to_lowercase()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Tolerance {
    pub abs: f64,
    pub rel: f64,
}

impl Default for Tolerance {
    fn default() -> Self {
        Tolerance {
            abs: 1e-9,
            rel: 0.0,
        }
    }
}

impl Tolerance {
    pub fn aggregate() -> Self {
        Tolerance {
            abs: 1e-6,
            rel: 0.005,
        }
    }
    pub fn close(&self, a: f64, b: f64) -> bool {
        let d = (a - b).abs();
        d <= self.abs || d <= self.rel * a.abs().max(b.abs())
    }
}

impl Answer {
    pub fn from_value(v: &Value) -> Option<Answer> {
        Some(match v {
            Value::Null => Answer::Null,
            Value::Bool(b) => Answer::Bool(*b),
            Value::Number(n) => Answer::Number(n.as_f64()),
            Value::String(s) => Answer::Text(s.clone()),
            Value::Array(items) => Answer::List(
                items
                    .iter()
                    .map(Scalar::from_value)
                    .collect::<Option<_>>()?,
            ),
            Value::Object(_) => return None,
        })
    }

    pub fn number(n: Number) -> Answer {
        Answer::Number(n.as_f64())
    }

    /// Does a model's parsed response match this expected answer?
    pub fn accepts(&self, got: &Answer, tol: &Tolerance) -> bool {
        match (self, got) {
            (Answer::Set(want), Answer::Set(g) | Answer::List(g)) => {
                want.len() == g.len()
                    && want.iter().all(|w| g.iter().any(|x| w.matches(x, tol)))
                    && g.iter().all(|x| want.iter().any(|w| w.matches(x, tol)))
            }
            (Answer::Set(want), single) if want.len() == 1 => {
                single_scalar(single).is_some_and(|s| want[0].matches(&s, tol))
            }
            (Answer::List(want), Answer::List(g) | Answer::Set(g)) => {
                want.len() == g.len() && want.iter().zip(g).all(|(w, x)| w.matches(x, tol))
            }
            (Answer::List(want), single) if want.len() == 1 => {
                single_scalar(single).is_some_and(|s| want[0].matches(&s, tol))
            }
            (want, Answer::List(g) | Answer::Set(g)) if g.len() == 1 => {
                single_scalar(want).is_some_and(|w| w.matches(&g[0], tol))
            }
            (want, got) => match (single_scalar(want), single_scalar(got)) {
                (Some(w), Some(g)) => w.matches(&g, tol),
                _ => false,
            },
        }
    }
}

fn single_scalar(a: &Answer) -> Option<Scalar> {
    Some(match a {
        Answer::Null => Scalar::Null,
        Answer::Bool(b) => Scalar::Bool(*b),
        Answer::Number(n) => Scalar::Number(*n),
        Answer::Text(t) => Scalar::Text(t.clone()),
        Answer::Set(_) | Answer::List(_) => return None,
    })
}

/// Parses a raw model response into an [`Answer`]. Accepts a bare JSON
/// value, a JSON value inside a Markdown code fence, or, failing that, the
/// trimmed text as a string (numbers and booleans recognised).
pub fn parse_response(raw: &str) -> Option<Answer> {
    let mut text = raw.trim();
    if let Some(start) = text.find("```") {
        let after = &text[start + 3..];
        let after = after.strip_prefix("json").unwrap_or(after);
        if let Some(end) = after.find("```") {
            text = after[..end].trim();
        }
    }
    if text.is_empty() {
        return None;
    }
    if let Ok(v) = serde_json::from_str::<Value>(text)
        && let Some(a) = Answer::from_value(&v)
    {
        return Some(a);
    }
    // Plain text fallbacks.
    let n = norm(text);
    if let Ok(f) = n.parse::<f64>() {
        return Some(Answer::Number(f));
    }
    match n.as_str() {
        "true" | "yes" => return Some(Answer::Bool(true)),
        "false" | "no" => return Some(Answer::Bool(false)),
        "null" | "none" | "missing" => return Some(Answer::Null),
        _ => {}
    }
    // Comma separated scalars become a list.
    if text.contains(',') && !text.contains(' ') {
        let items: Vec<Scalar> = text
            .split(',')
            .map(|p| {
                let p = norm(p);
                p.parse::<f64>()
                    .map(Scalar::Number)
                    .unwrap_or(Scalar::Text(p))
            })
            .collect();
        return Some(Answer::List(items));
    }
    Some(Answer::Text(text.trim_matches('"').to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_and_matching() {
        let t = Tolerance::default();
        assert!(Answer::Number(42.0).accepts(&parse_response("42").unwrap(), &t));
        assert!(Answer::Number(42.0).accepts(&parse_response("```json\n42\n```").unwrap(), &t));
        assert!(
            Answer::Text("Ada Lovelace".into())
                .accepts(&parse_response("\"Ada Lovelace\"").unwrap(), &t)
        );
        assert!(Answer::Null.accepts(&parse_response("null").unwrap(), &t));
        assert!(
            Answer::Set(vec![Scalar::Number(1.0), Scalar::Number(2.0)])
                .accepts(&parse_response("[2, 1]").unwrap(), &t)
        );
        assert!(
            !Answer::List(vec![Scalar::Number(1.0), Scalar::Number(2.0)])
                .accepts(&parse_response("[2, 1]").unwrap(), &t)
        );
        assert!(
            Answer::Number(1234.5)
                .accepts(&parse_response("1234.49").unwrap(), &Tolerance::aggregate())
        );
        assert!(
            !Answer::Number(1234.5)
                .accepts(&parse_response("1200").unwrap(), &Tolerance::aggregate())
        );
        assert!(parse_response("   ").is_none());
    }
}
