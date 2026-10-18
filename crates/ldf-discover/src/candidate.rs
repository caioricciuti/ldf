//! A configuration behind the common [`Format`] trait, so the round-trip
//! check of `ldf-bench` applies to it unchanged.

use ldf_data::Value;
use ldf_formats::{Capabilities, DecodeError, EncodeError, Format};

use crate::knobs::Config;

pub struct Candidate(pub Config);

impl Format for Candidate {
    fn name(&self) -> &'static str {
        "candidate"
    }
    fn extension(&self) -> &'static str {
        "txt"
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            nested: true,
            typed: true,
            missing_vs_null: true,
            streaming_encode: false,
            requires_count: false,
        }
    }
    fn encode(&self, records: &[Value], out: &mut Vec<u8>) -> Result<(), EncodeError> {
        out.extend_from_slice(crate::encode(records, &self.0).as_bytes());
        Ok(())
    }
    fn decode(&self, bytes: &[u8]) -> Result<Vec<Value>, DecodeError> {
        let text = std::str::from_utf8(bytes).map_err(|e| DecodeError {
            format: "candidate",
            line: None,
            message: format!("invalid utf-8: {e}"),
        })?;
        crate::decode(text, &self.0).map_err(|e| DecodeError {
            format: "candidate",
            line: Some(e.line),
            message: e.message,
        })
    }
}
