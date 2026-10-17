//! Baseline serialization formats.
//!
//! Every format implements [`Format`]: encode a slice of records, decode
//! bytes back into records, and declare its capabilities. A format that
//! cannot represent a dataset returns [`EncodeError::Unsupported`] with a
//! reason; it never flattens, drops or retypes data to make it fit.
//!
//! Formats:
//! * `json`         compact JSON array (serde_json)
//! * `json-pretty`  two-space indented JSON array (serde_json)
//! * `jsonl`        one compact JSON object per line
//! * `csv`, `tsv`   RFC 4180 style, flat records only
//! * `markdown`     GitHub-flavoured pipe table, flat records only
//! * `toon`         Token-Oriented Object Notation, spec v4.4, written from
//!   the spec and checked against its conformance fixtures
//! * `yaml`         block-style YAML 1.2 (own emitter, yaml-rust2 decoder)

use std::fmt;

use ldf_data::Value;

pub mod json;
pub mod tabular;
pub mod toon;
pub mod yaml;

/// What a format can do. Reported in benchmark output so that "unsupported"
/// cells are explained rather than blank.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capabilities {
    /// Can represent nested arrays and objects.
    pub nested: bool,
    /// Distinguishes strings from numbers, booleans and null on decode.
    pub typed: bool,
    /// Distinguishes a missing field from an explicit null.
    pub missing_vs_null: bool,
    /// Can encode record by record without seeing the whole dataset.
    pub streaming_encode: bool,
    /// Needs the record count before emitting the first byte.
    pub requires_count: bool,
}

#[derive(Debug, Clone)]
pub struct Unsupported {
    pub format: &'static str,
    pub reason: String,
}

#[derive(Debug)]
pub enum EncodeError {
    Unsupported(Unsupported),
    Io(std::io::Error),
    Other(String),
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::Unsupported(u) => write!(f, "{} cannot encode: {}", u.format, u.reason),
            EncodeError::Io(e) => write!(f, "io: {e}"),
            EncodeError::Other(e) => f.write_str(e),
        }
    }
}
impl std::error::Error for EncodeError {}
impl From<std::io::Error> for EncodeError {
    fn from(e: std::io::Error) -> Self {
        EncodeError::Io(e)
    }
}

#[derive(Debug, Clone)]
pub struct DecodeError {
    pub format: &'static str,
    pub line: Option<usize>,
    pub message: String,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(l) => write!(
                f,
                "{} decode error at line {l}: {}",
                self.format, self.message
            ),
            None => write!(f, "{} decode error: {}", self.format, self.message),
        }
    }
}
impl std::error::Error for DecodeError {}

/// Record-at-a-time encoder for formats that support it.
pub trait StreamEncoder {
    fn record(&mut self, rec: &Value, out: &mut Vec<u8>) -> Result<(), EncodeError>;
    fn finish(&mut self, out: &mut Vec<u8>) -> Result<(), EncodeError>;
}

pub trait Format: Send + Sync {
    fn name(&self) -> &'static str;
    fn extension(&self) -> &'static str;
    fn capabilities(&self) -> Capabilities;

    /// Encodes the whole dataset. Output is appended to `out`.
    fn encode(&self, records: &[Value], out: &mut Vec<u8>) -> Result<(), EncodeError>;

    /// Decodes a document produced by [`Format::encode`] back into records.
    fn decode(&self, bytes: &[u8]) -> Result<Vec<Value>, DecodeError>;

    /// A streaming encoder, when the format can emit records one at a time.
    fn stream_encoder(&self) -> Option<Box<dyn StreamEncoder>> {
        None
    }

    fn encode_to_string(&self, records: &[Value]) -> Result<String, EncodeError> {
        let mut out = Vec::new();
        self.encode(records, &mut out)?;
        String::from_utf8(out).map_err(|e| EncodeError::Other(e.to_string()))
    }
}

/// All baseline formats, in report order.
pub fn all_formats() -> Vec<Box<dyn Format>> {
    vec![
        Box::new(json::Json),
        Box::new(json::JsonPretty),
        Box::new(json::Jsonl),
        Box::new(tabular::Csv),
        Box::new(tabular::Tsv),
        Box::new(tabular::Markdown),
        Box::new(toon::Toon::default()),
        Box::new(toon::Toon::tab()),
        Box::new(yaml::YamlFormat),
    ]
}

pub fn format_names() -> Vec<&'static str> {
    all_formats().iter().map(|f| f.name()).collect()
}

pub fn by_name(name: &str) -> Option<Box<dyn Format>> {
    all_formats().into_iter().find(|f| f.name() == name)
}

/// Resolves a comma-separated list or `all`.
pub fn select(spec: &str) -> Result<Vec<Box<dyn Format>>, String> {
    if spec == "all" {
        return Ok(all_formats());
    }
    spec.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|n| {
            by_name(n).ok_or_else(|| {
                format!("unknown format {n:?}; known: {}", format_names().join(", "))
            })
        })
        .collect()
}

pub(crate) fn unsupported(format: &'static str, reason: impl Into<String>) -> EncodeError {
    EncodeError::Unsupported(Unsupported {
        format,
        reason: reason.into(),
    })
}
