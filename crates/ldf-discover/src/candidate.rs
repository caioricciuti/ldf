//! A configuration behind the common [`Format`] trait, so the round-trip
//! check of `ldf-bench` applies to it unchanged.

use ldf_data::Value;
use ldf_formats::{Capabilities, DecodeError, EncodeError, Format};

use crate::knobs::Config;

/// `name` labels the candidate in reports and bundle file names; prompts
/// call every candidate `ldf`, like the other LDF variants.
pub struct Candidate {
    pub config: Config,
    pub name: &'static str,
}

impl Candidate {
    pub fn new(config: Config) -> Self {
        Candidate {
            config,
            name: "candidate",
        }
    }
}

impl Format for Candidate {
    fn name(&self) -> &'static str {
        self.name
    }
    fn display_name(&self) -> &'static str {
        "ldf"
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
        out.extend_from_slice(crate::encode(records, &self.config).as_bytes());
        Ok(())
    }
    fn decode(&self, bytes: &[u8]) -> Result<Vec<Value>, DecodeError> {
        let text = std::str::from_utf8(bytes).map_err(|e| DecodeError {
            format: "candidate",
            line: None,
            message: format!("invalid utf-8: {e}"),
        })?;
        crate::decode(text, &self.config).map_err(|e| DecodeError {
            format: "candidate",
            line: Some(e.line),
            message: e.message,
        })
    }
}
