//! Adapter exposing LDF v0 (`crates/ldf`) through the [`Format`] trait.

use ldf_data::Value;

use crate::{Capabilities, DecodeError, EncodeError, Format, StreamEncoder};

/// `ldf` is the current layout (v0.1: sparse-key rule at the default
/// threshold). `ldf-v0` keeps every key as a column, for comparisons.
/// Variants share the display name `ldf` so evaluation prompts do not
/// change between them.
pub struct LdfFormat {
    name: &'static str,
    extension: &'static str,
    options: ::ldf::Options,
}

impl LdfFormat {
    pub fn current() -> Self {
        LdfFormat {
            name: "ldf",
            extension: "ldf",
            options: ::ldf::Options::default(),
        }
    }
    pub fn v0() -> Self {
        LdfFormat {
            name: "ldf-v0",
            extension: "v0.ldf",
            options: ::ldf::Options::V0,
        }
    }
    pub fn sparse(name: &'static str, extension: &'static str, threshold: f64) -> Self {
        LdfFormat {
            name,
            extension,
            options: ::ldf::Options {
                sparse_threshold: threshold,
            },
        }
    }
}

impl Format for LdfFormat {
    fn name(&self) -> &'static str {
        self.name
    }
    fn display_name(&self) -> &'static str {
        "ldf"
    }
    fn extension(&self) -> &'static str {
        self.extension
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            nested: true,
            typed: true,
            missing_vs_null: true,
            streaming_encode: true,
            requires_count: false,
        }
    }
    fn encode(&self, records: &[Value], out: &mut Vec<u8>) -> Result<(), EncodeError> {
        out.extend_from_slice(::ldf::encode_with(records, &self.options).as_bytes());
        Ok(())
    }
    fn decode(&self, bytes: &[u8]) -> Result<Vec<Value>, DecodeError> {
        let text = std::str::from_utf8(bytes).map_err(|e| DecodeError {
            format: "ldf",
            line: None,
            message: format!("invalid utf-8: {e}"),
        })?;
        ::ldf::decode(text).map_err(|e| DecodeError {
            format: "ldf",
            line: Some(e.line),
            message: e.message,
        })
    }
    fn stream_encoder(&self) -> Option<Box<dyn StreamEncoder>> {
        Some(Box::new(LdfStream {
            inner: ::ldf::StreamEncoder::default(),
            buf: String::new(),
        }))
    }
}

struct LdfStream {
    inner: ::ldf::StreamEncoder,
    buf: String,
}

impl StreamEncoder for LdfStream {
    fn record(&mut self, rec: &Value, out: &mut Vec<u8>) -> Result<(), EncodeError> {
        self.buf.clear();
        self.inner.record(rec, &mut self.buf);
        out.extend_from_slice(self.buf.as_bytes());
        Ok(())
    }
    fn finish(&mut self, out: &mut Vec<u8>) -> Result<(), EncodeError> {
        self.buf.clear();
        self.inner.finish(&mut self.buf);
        out.extend_from_slice(self.buf.as_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldf_data::obj;

    #[test]
    fn streamed_output_decodes_to_the_same_records() {
        let recs = vec![
            obj! {"a" => 1, "n" => obj! {"x" => 1}, "items" => vec![obj! {"v" => 1}]},
            obj! {"a" => 2, "n" => obj! {"x" => 2}, "items" => Vec::<Value>::new()},
        ];
        let mut streamed = Vec::new();
        let mut enc = LdfFormat::v0().stream_encoder().unwrap();
        for r in &recs {
            enc.record(r, &mut streamed).unwrap();
        }
        enc.finish(&mut streamed).unwrap();
        assert_eq!(
            LdfFormat::v0().decode(&streamed).unwrap(),
            recs,
            "{}",
            String::from_utf8_lossy(&streamed)
        );
    }
}
