//! JSON family: compact array, pretty array, JSON Lines.

use ldf_data::Value;
use ldf_data::input::{JsonEngine, jsonl_stream, parse_json_document};

use crate::{Capabilities, DecodeError, EncodeError, Format, StreamEncoder};

const CAPS: Capabilities = Capabilities {
    nested: true,
    typed: true,
    missing_vs_null: true,
    streaming_encode: true,
    requires_count: false,
};

fn decode_err(format: &'static str, e: impl std::fmt::Display) -> DecodeError {
    DecodeError {
        format,
        line: None,
        message: e.to_string(),
    }
}

pub struct Json;

impl Format for Json {
    fn name(&self) -> &'static str {
        "json"
    }
    fn extension(&self) -> &'static str {
        "json"
    }
    fn capabilities(&self) -> Capabilities {
        CAPS
    }
    fn encode(&self, records: &[Value], out: &mut Vec<u8>) -> Result<(), EncodeError> {
        serde_json::to_writer(out, records).map_err(|e| EncodeError::Other(e.to_string()))
    }
    fn decode(&self, bytes: &[u8]) -> Result<Vec<Value>, DecodeError> {
        parse_json_document(bytes.to_vec(), JsonEngine::SerdeJson)
            .map_err(|e| decode_err("json", e))
    }
    fn stream_encoder(&self) -> Option<Box<dyn StreamEncoder>> {
        Some(Box::new(JsonArrayStream { first: true }))
    }
}

struct JsonArrayStream {
    first: bool,
}

impl StreamEncoder for JsonArrayStream {
    fn record(&mut self, rec: &Value, out: &mut Vec<u8>) -> Result<(), EncodeError> {
        out.push(if self.first { b'[' } else { b',' });
        self.first = false;
        serde_json::to_writer(out, rec).map_err(|e| EncodeError::Other(e.to_string()))
    }
    fn finish(&mut self, out: &mut Vec<u8>) -> Result<(), EncodeError> {
        if self.first {
            out.push(b'[');
        }
        out.push(b']');
        Ok(())
    }
}

pub struct JsonPretty;

impl Format for JsonPretty {
    fn name(&self) -> &'static str {
        "json-pretty"
    }
    fn extension(&self) -> &'static str {
        "pretty.json"
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            streaming_encode: false,
            ..CAPS
        }
    }
    fn encode(&self, records: &[Value], out: &mut Vec<u8>) -> Result<(), EncodeError> {
        serde_json::to_writer_pretty(out, records).map_err(|e| EncodeError::Other(e.to_string()))
    }
    fn decode(&self, bytes: &[u8]) -> Result<Vec<Value>, DecodeError> {
        parse_json_document(bytes.to_vec(), JsonEngine::SerdeJson)
            .map_err(|e| decode_err("json-pretty", e))
    }
}

pub struct Jsonl;

impl Format for Jsonl {
    fn name(&self) -> &'static str {
        "jsonl"
    }
    fn extension(&self) -> &'static str {
        "jsonl"
    }
    fn capabilities(&self) -> Capabilities {
        CAPS
    }
    fn encode(&self, records: &[Value], out: &mut Vec<u8>) -> Result<(), EncodeError> {
        for rec in records {
            serde_json::to_writer(&mut *out, rec).map_err(|e| EncodeError::Other(e.to_string()))?;
            out.push(b'\n');
        }
        Ok(())
    }
    fn decode(&self, bytes: &[u8]) -> Result<Vec<Value>, DecodeError> {
        jsonl_stream(std::io::Cursor::new(bytes.to_vec()), JsonEngine::SerdeJson)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| decode_err("jsonl", e))
    }
    fn stream_encoder(&self) -> Option<Box<dyn StreamEncoder>> {
        Some(Box::new(JsonlStream))
    }
}

struct JsonlStream;

impl StreamEncoder for JsonlStream {
    fn record(&mut self, rec: &Value, out: &mut Vec<u8>) -> Result<(), EncodeError> {
        serde_json::to_writer(&mut *out, rec).map_err(|e| EncodeError::Other(e.to_string()))?;
        out.push(b'\n');
        Ok(())
    }
    fn finish(&mut self, _out: &mut Vec<u8>) -> Result<(), EncodeError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldf_data::obj;

    #[test]
    fn json_family_round_trips_byte_exact() {
        let recs = vec![
            obj! {"a" => 1, "b" => "x"},
            obj! {"a" => 2.5, "b" => Value::Null},
        ];
        for f in [&Json as &dyn Format, &JsonPretty, &Jsonl] {
            let bytes = {
                let mut b = Vec::new();
                f.encode(&recs, &mut b).unwrap();
                b
            };
            let back = f.decode(&bytes).unwrap();
            assert_eq!(back, recs, "{}", f.name());
            let mut again = Vec::new();
            f.encode(&back, &mut again).unwrap();
            assert_eq!(again, bytes, "{} byte round trip", f.name());
        }
    }

    #[test]
    fn stream_matches_batch() {
        let recs = vec![obj! {"a" => 1}, obj! {"a" => 2}];
        for f in [&Json as &dyn Format, &Jsonl] {
            let mut batch = Vec::new();
            f.encode(&recs, &mut batch).unwrap();
            let mut streamed = Vec::new();
            let mut enc = f.stream_encoder().unwrap();
            for r in &recs {
                enc.record(r, &mut streamed).unwrap();
            }
            enc.finish(&mut streamed).unwrap();
            assert_eq!(batch, streamed);
        }
    }
}
