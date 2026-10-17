//! TOON (Token-Oriented Object Notation), implemented from the specification
//! at <https://github.com/toon-format/spec>, version 4.4.0 (working draft,
//! 2026-10-08). This is an independent Rust implementation written for the
//! benchmark; it is validated against the spec's own conformance fixtures
//! in `tests/toon_fixtures.rs`.
//!
//! Numeric domain: `i64`, `u64`, `f64` (see [`ldf_data::Number`]). NaN and
//! infinities encode as `null` per spec section 3. Decoded numeric tokens
//! outside the domain fall back to `f64` (documented approximation).
//!
//! Dataset mapping: a dataset is encoded as a root array of its records.

pub mod decode;
pub mod encode;
pub mod text;

use ldf_data::Value;

use crate::{Capabilities, DecodeError, EncodeError, Format};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Delimiter {
    #[default]
    Comma,
    Tab,
    Pipe,
}

impl Delimiter {
    pub fn ch(self) -> char {
        match self {
            Delimiter::Comma => ',',
            Delimiter::Tab => '\t',
            Delimiter::Pipe => '|',
        }
    }
    /// Symbol written inside the bracket segment (empty for comma).
    pub fn header_symbol(self) -> &'static str {
        match self {
            Delimiter::Comma => "",
            Delimiter::Tab => "\t",
            Delimiter::Pipe => "|",
        }
    }
    pub fn parse(s: &str) -> Option<Delimiter> {
        match s {
            "comma" | "," => Some(Delimiter::Comma),
            "tab" | "\t" => Some(Delimiter::Tab),
            "pipe" | "|" => Some(Delimiter::Pipe),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodeOptions {
    pub indent: usize,
    pub delimiter: Delimiter,
}

impl Default for EncodeOptions {
    fn default() -> Self {
        EncodeOptions {
            indent: 2,
            delimiter: Delimiter::Comma,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodeOptions {
    pub indent: usize,
    pub strict: bool,
}

impl Default for DecodeOptions {
    fn default() -> Self {
        DecodeOptions {
            indent: 2,
            strict: true,
        }
    }
}

/// Encodes any value as a TOON document (no trailing newline).
pub fn encode_value(value: &Value, opts: &EncodeOptions) -> String {
    encode::Encoder::new(*opts).encode_document(value)
}

/// Decodes a TOON document into a value.
pub fn decode_value(text: &str, opts: &DecodeOptions) -> Result<Value, decode::Error> {
    decode::decode(text, opts)
}

/// The TOON baseline format for the benchmark: a dataset is a root array.
#[derive(Debug, Clone, Copy)]
pub struct Toon {
    pub options: EncodeOptions,
    name: &'static str,
}

impl Default for Toon {
    fn default() -> Self {
        Toon {
            options: EncodeOptions::default(),
            name: "toon",
        }
    }
}

impl Toon {
    pub fn tab() -> Self {
        Toon {
            options: EncodeOptions {
                indent: 2,
                delimiter: Delimiter::Tab,
            },
            name: "toon-tab",
        }
    }
    pub fn pipe() -> Self {
        Toon {
            options: EncodeOptions {
                indent: 2,
                delimiter: Delimiter::Pipe,
            },
            name: "toon-pipe",
        }
    }
}

impl Format for Toon {
    fn name(&self) -> &'static str {
        self.name
    }
    fn extension(&self) -> &'static str {
        "toon"
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            nested: true,
            typed: true,
            missing_vs_null: true,
            // The root header carries `[N]` and the tabular/list decision
            // needs every record, so a one-pass streaming encoder is not
            // possible without a count and a schema commitment up front.
            streaming_encode: false,
            requires_count: true,
        }
    }
    fn encode(&self, records: &[Value], out: &mut Vec<u8>) -> Result<(), EncodeError> {
        let mut enc = encode::Encoder::new(self.options);
        enc.encode_root_array(records);
        out.extend_from_slice(enc.finish().as_bytes());
        Ok(())
    }
    fn decode(&self, bytes: &[u8]) -> Result<Vec<Value>, DecodeError> {
        let text = std::str::from_utf8(bytes).map_err(|e| DecodeError {
            format: self.name,
            line: None,
            message: format!("invalid utf-8: {e}"),
        })?;
        let v = decode::decode(text, &DecodeOptions::default()).map_err(|e| DecodeError {
            format: self.name,
            line: Some(e.line),
            message: e.message,
        })?;
        match v {
            Value::Array(items) => Ok(items),
            other => Err(DecodeError {
                format: self.name,
                line: None,
                message: format!(
                    "expected a root array of records, found {}",
                    other.type_name()
                ),
            }),
        }
    }
}
