//! Round-trip validation. Runs before any timing so that a format is never
//! benchmarked on output that does not decode back to the input.
//!
//! Levels, from strongest to weakest:
//! * `byte`: encode(decode(encode(x))) == encode(x) and decoded == x with
//!   strict structural equality (number kinds preserved);
//! * `semantic`: decoded equals x by mathematical value, key order kept;
//! * `semantic-unordered`: as above but object key order may differ;
//! * `lossy`: decodes, but values differ (first difference recorded);
//! * `unsupported`: the format refused the dataset, with its reason;
//! * `decode-error`: the format produced output it cannot read back.

use ldf_data::Value;
use ldf_formats::{EncodeError, Format};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Level {
    Byte,
    Semantic,
    SemanticUnordered,
    Lossy,
    Unsupported,
    DecodeError,
}

impl Level {
    pub fn label(&self) -> &'static str {
        match self {
            Level::Byte => "byte",
            Level::Semantic => "semantic",
            Level::SemanticUnordered => "semantic-unordered",
            Level::Lossy => "lossy",
            Level::Unsupported => "unsupported",
            Level::DecodeError => "decode-error",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RoundTrip {
    pub format: String,
    pub level: Level,
    pub output_bytes: u64,
    pub detail: Option<String>,
}

pub fn check(format: &dyn Format, records: &[Value]) -> RoundTrip {
    let name = format.name().to_string();
    let mut out = Vec::new();
    match format.encode(records, &mut out) {
        Ok(()) => {}
        Err(EncodeError::Unsupported(u)) => {
            return RoundTrip {
                format: name,
                level: Level::Unsupported,
                output_bytes: 0,
                detail: Some(u.reason),
            };
        }
        Err(e) => {
            return RoundTrip {
                format: name,
                level: Level::DecodeError,
                output_bytes: 0,
                detail: Some(format!("encode failed: {e}")),
            };
        }
    }
    let output_bytes = out.len() as u64;
    let decoded = match format.decode(&out) {
        Ok(d) => d,
        Err(e) => {
            return RoundTrip {
                format: name,
                level: Level::DecodeError,
                output_bytes,
                detail: Some(e.to_string()),
            };
        }
    };
    let input = Value::Array(records.to_vec());
    let back = Value::Array(decoded);
    if back == input {
        let mut again = Vec::new();
        if format.encode(records, &mut again).is_ok() && again == out {
            return RoundTrip {
                format: name,
                level: Level::Byte,
                output_bytes,
                detail: None,
            };
        }
    }
    if back.semantic_eq(&input) {
        return RoundTrip {
            format: name,
            level: Level::Semantic,
            output_bytes,
            detail: None,
        };
    }
    if back.semantic_eq_unordered(&input) {
        return RoundTrip {
            format: name,
            level: Level::SemanticUnordered,
            output_bytes,
            detail: Some("object key order changed".into()),
        };
    }
    RoundTrip {
        format: name,
        level: Level::Lossy,
        output_bytes,
        detail: back.first_difference(&input),
    }
}
