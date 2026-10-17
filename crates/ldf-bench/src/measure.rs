//! Timed operations. These run inside a child process (one per dataset,
//! format and operation) so that peak RSS from `getrusage` is attributable
//! and the first iteration is a genuine cold start.

use std::path::Path;
use std::time::Instant;

use ldf_data::Value;
use ldf_data::input::{InputKind, JsonEngine, ReadOptions, open_kind, parse_json_document};
use ldf_formats::Format;
use serde::{Deserialize, Serialize};

use crate::sys::{Usage, usage};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Op {
    /// Read the input file bytes (I/O only).
    Io,
    /// Parse the input container into records (after I/O).
    Parse,
    /// Encode in-memory records with a format.
    Encode,
    /// Decode a format's own output back into records.
    Decode,
}

impl Op {
    pub fn parse(s: &str) -> Option<Op> {
        match s {
            "io" => Some(Op::Io),
            "parse" => Some(Op::Parse),
            "encode" => Some(Op::Encode),
            "decode" => Some(Op::Decode),
            _ => None,
        }
    }
}

/// One child-process measurement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    pub op: Op,
    pub format: Option<String>,
    pub engine: Option<String>,
    pub records: u64,
    /// Bytes read for io/parse, bytes produced for encode, bytes consumed for decode.
    pub bytes: u64,
    /// Wall-clock seconds per iteration, in order. Index 0 is the cold run.
    pub secs: Vec<f64>,
    /// Process usage after setup (input loaded) and after the measured loop.
    pub usage_before: Usage,
    pub usage_after: Usage,
    pub error: Option<String>,
}

pub struct Plan<'a> {
    pub input: &'a Path,
    pub kind: InputKind,
    pub engine: JsonEngine,
    pub iters: usize,
}

fn engine_name(e: JsonEngine) -> String {
    match e {
        JsonEngine::SerdeJson => "serde_json".into(),
        #[cfg(feature = "simd")]
        JsonEngine::SimdJson => "simd-json".into(),
    }
}

/// Loads the whole input into records (what every encode/decode run needs).
pub fn load(plan: &Plan) -> Result<Vec<Value>, String> {
    let opts = ReadOptions {
        json_engine: plan.engine,
        ..Default::default()
    };
    open_kind(plan.input, plan.kind, &opts)
        .and_then(|s| s.collect::<Result<Vec<_>, _>>())
        .map_err(|e| e.to_string())
}

pub fn run_io(plan: &Plan) -> Measurement {
    let before = usage();
    let mut secs = Vec::with_capacity(plan.iters);
    let mut bytes = 0u64;
    let mut error = None;
    for _ in 0..plan.iters {
        let t = Instant::now();
        match std::fs::read(plan.input) {
            Ok(b) => {
                secs.push(t.elapsed().as_secs_f64());
                bytes = b.len() as u64;
                std::hint::black_box(&b);
            }
            Err(e) => {
                error = Some(e.to_string());
                break;
            }
        }
    }
    Measurement {
        op: Op::Io,
        format: None,
        engine: None,
        records: 0,
        bytes,
        secs,
        usage_before: before,
        usage_after: usage(),
        error,
    }
}

pub fn run_parse(plan: &Plan) -> Measurement {
    // Parse from memory for JSON (so I/O is excluded); for streaming
    // containers the reader owns the file handle and page cache keeps the
    // repeated reads warm, which the io measurement quantifies separately.
    let bytes = match std::fs::read(plan.input) {
        Ok(b) => b,
        Err(e) => {
            return Measurement {
                op: Op::Parse,
                format: None,
                engine: Some(engine_name(plan.engine)),
                records: 0,
                bytes: 0,
                secs: vec![],
                usage_before: usage(),
                usage_after: usage(),
                error: Some(e.to_string()),
            };
        }
    };
    let size = bytes.len() as u64;
    let before = usage();
    let mut secs = Vec::with_capacity(plan.iters);
    let mut records = 0u64;
    let mut error = None;
    for _ in 0..plan.iters {
        let t = Instant::now();
        let result: Result<Vec<Value>, String> = match plan.kind {
            InputKind::Json => {
                parse_json_document(bytes.clone(), plan.engine).map_err(|e| e.to_string())
            }
            _ => load(plan),
        };
        match result {
            Ok(recs) => {
                secs.push(t.elapsed().as_secs_f64());
                records = recs.len() as u64;
                std::hint::black_box(&recs);
            }
            Err(e) => {
                error = Some(e);
                break;
            }
        }
    }
    Measurement {
        op: Op::Parse,
        format: None,
        engine: Some(engine_name(plan.engine)),
        records,
        bytes: size,
        secs,
        usage_before: before,
        usage_after: usage(),
        error,
    }
}

pub fn run_encode(plan: &Plan, format: &dyn Format) -> Measurement {
    let records = match load(plan) {
        Ok(r) => r,
        Err(e) => return failed(Op::Encode, format, e),
    };
    let before = usage();
    let mut secs = Vec::with_capacity(plan.iters);
    let mut bytes = 0u64;
    let mut error = None;
    for _ in 0..plan.iters {
        let mut out = Vec::new();
        let t = Instant::now();
        match format.encode(&records, &mut out) {
            Ok(()) => {
                secs.push(t.elapsed().as_secs_f64());
                bytes = out.len() as u64;
                std::hint::black_box(&out);
            }
            Err(e) => {
                error = Some(e.to_string());
                break;
            }
        }
    }
    Measurement {
        op: Op::Encode,
        format: Some(format.name().into()),
        engine: None,
        records: records.len() as u64,
        bytes,
        secs,
        usage_before: before,
        usage_after: usage(),
        error,
    }
}

pub fn run_decode(plan: &Plan, format: &dyn Format) -> Measurement {
    let records = match load(plan) {
        Ok(r) => r,
        Err(e) => return failed(Op::Decode, format, e),
    };
    let mut encoded = Vec::new();
    if let Err(e) = format.encode(&records, &mut encoded) {
        return failed(Op::Decode, format, e.to_string());
    }
    let n = records.len() as u64;
    drop(records);
    let before = usage();
    let mut secs = Vec::with_capacity(plan.iters);
    let mut error = None;
    for _ in 0..plan.iters {
        let t = Instant::now();
        match format.decode(&encoded) {
            Ok(recs) => {
                secs.push(t.elapsed().as_secs_f64());
                std::hint::black_box(&recs);
            }
            Err(e) => {
                error = Some(e.to_string());
                break;
            }
        }
    }
    Measurement {
        op: Op::Decode,
        format: Some(format.name().into()),
        engine: None,
        records: n,
        bytes: encoded.len() as u64,
        secs,
        usage_before: before,
        usage_after: usage(),
        error,
    }
}

fn failed(op: Op, format: &dyn Format, error: String) -> Measurement {
    Measurement {
        op,
        format: Some(format.name().into()),
        engine: None,
        records: 0,
        bytes: 0,
        secs: vec![],
        usage_before: usage(),
        usage_after: usage(),
        error: Some(error),
    }
}
