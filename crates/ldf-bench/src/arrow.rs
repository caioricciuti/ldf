//! Parquet to LDF: the `Value` path (`ldf_data` reader, then the batch or
//! block encoder) against the direct Arrow path (`ldf::arrow`). Each
//! (file, path) runs in its own child process so peak RSS belongs to it.
//! Every iteration goes from the file on disk to LDF text; the block forms
//! hand each block to a sink and drop it, so their memory is bounded by the
//! block, as in a streaming writer.

use std::fs::File;
use std::path::Path;
use std::time::Instant;

use arrow::array::RecordBatch;
use ldf::arrow::{ArrowBlockEncoder, encode_batches};
use ldf::{BlockEncoder, Options, encode_with};
use ldf_data::Value;
use ldf_data::input::{InputKind, ReadOptions, open_kind};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use serde::{Deserialize, Serialize};

use crate::sys::{Usage, usage};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArrowPath {
    /// `ldf_data` Parquet stream to `Vec<Value>`, then `ldf::encode_with`.
    ValueBatch,
    /// Record batches, then `ldf::arrow::encode_batches`.
    DirectBatch,
    /// `ldf_data` Parquet stream into `ldf::BlockEncoder`.
    ValueBlock,
    /// Record batches into `ldf::arrow::ArrowBlockEncoder`.
    DirectBlock,
}

impl ArrowPath {
    pub const ALL: [ArrowPath; 4] = [
        ArrowPath::ValueBatch,
        ArrowPath::DirectBatch,
        ArrowPath::ValueBlock,
        ArrowPath::DirectBlock,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ArrowPath::ValueBatch => "value-batch",
            ArrowPath::DirectBatch => "direct-batch",
            ArrowPath::ValueBlock => "value-block",
            ArrowPath::DirectBlock => "direct-block",
        }
    }

    pub fn parse(s: &str) -> Option<ArrowPath> {
        ArrowPath::ALL.into_iter().find(|p| p.name() == s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrowMeasurement {
    pub path: ArrowPath,
    pub rows: u64,
    /// LDF bytes produced per iteration.
    pub bytes_out: u64,
    /// FNV-1a of the output, from one extra untimed run.
    pub output_hash: String,
    /// Wall-clock seconds per iteration; index 0 is the cold run.
    pub secs: Vec<f64>,
    pub usage_before: Usage,
    pub usage_after: Usage,
    pub error: Option<String>,
}

/// Counts and hashes what an encoder writes, then forgets it.
#[derive(Default)]
struct Sink {
    bytes: u64,
    hash: Option<u64>,
}

impl Sink {
    fn hashing() -> Sink {
        Sink {
            bytes: 0,
            hash: Some(0xcbf29ce484222325),
        }
    }
    fn take(&mut self, out: &mut String) {
        self.bytes += out.len() as u64;
        if let Some(h) = self.hash.as_mut() {
            for b in out.bytes() {
                *h ^= b as u64;
                *h = h.wrapping_mul(0x100000001b3);
            }
        }
        std::hint::black_box(&*out);
        out.clear();
    }
}

fn batches(
    path: &Path,
    batch: usize,
) -> Result<impl Iterator<Item = Result<RecordBatch, String>>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| e.to_string())?
        .with_batch_size(batch)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(reader.map(|b| b.map_err(|e| e.to_string())))
}

fn values(
    path: &Path,
    batch: usize,
) -> Result<impl Iterator<Item = Result<Value, String>>, String> {
    let opts = ReadOptions {
        parquet_batch_size: Some(batch),
        ..Default::default()
    };
    Ok(open_kind(path, InputKind::Parquet, &opts)
        .map_err(|e| e.to_string())?
        .map(|r| r.map_err(|e| e.to_string())))
}

/// One file to LDF along `which`. Returns the number of rows.
fn once(
    input: &Path,
    which: ArrowPath,
    batch: usize,
    block: usize,
    sink: &mut Sink,
) -> Result<u64, String> {
    let opts = Options::default();
    let mut out = String::new();
    let rows;
    match which {
        ArrowPath::ValueBatch => {
            let recs: Vec<Value> = values(input, batch)?.collect::<Result<_, _>>()?;
            rows = recs.len() as u64;
            out = encode_with(&recs, &opts);
            sink.take(&mut out);
        }
        ArrowPath::DirectBatch => {
            let bs: Vec<RecordBatch> = batches(input, batch)?.collect::<Result<_, _>>()?;
            rows = bs.iter().map(|b| b.num_rows() as u64).sum();
            out = encode_batches(&bs, &opts).map_err(|e| e.to_string())?;
            sink.take(&mut out);
        }
        ArrowPath::ValueBlock => {
            let mut enc = BlockEncoder::with_options(block, opts);
            let mut n = 0u64;
            for v in values(input, batch)? {
                enc.record(v?, &mut out);
                n += 1;
                if out.len() >= 1 << 16 {
                    sink.take(&mut out);
                }
            }
            enc.finish(&mut out);
            sink.take(&mut out);
            rows = n;
        }
        ArrowPath::DirectBlock => {
            let mut enc = ArrowBlockEncoder::new(block, opts);
            let mut n = 0u64;
            for b in batches(input, batch)? {
                let b = b?;
                n += b.num_rows() as u64;
                enc.batch(&b, &mut out).map_err(|e| e.to_string())?;
                if out.len() >= 1 << 16 {
                    sink.take(&mut out);
                }
            }
            enc.finish(&mut out).map_err(|e| e.to_string())?;
            sink.take(&mut out);
            rows = n;
        }
    }
    Ok(rows)
}

pub fn run(
    input: &Path,
    which: ArrowPath,
    iters: usize,
    batch: usize,
    block: usize,
) -> ArrowMeasurement {
    let before = usage();
    let mut secs = Vec::with_capacity(iters);
    let mut rows = 0;
    let mut bytes_out = 0;
    let mut error = None;
    for _ in 0..iters.max(1) {
        let mut sink = Sink::default();
        let t = Instant::now();
        match once(input, which, batch, block, &mut sink) {
            Ok(n) => {
                secs.push(t.elapsed().as_secs_f64());
                rows = n;
                bytes_out = sink.bytes;
            }
            Err(e) => {
                error = Some(e);
                break;
            }
        }
    }
    let after = usage();
    let mut output_hash = String::new();
    if error.is_none() {
        let mut sink = Sink::hashing();
        match once(input, which, batch, block, &mut sink) {
            Ok(_) => output_hash = format!("{:016x}", sink.hash.unwrap_or(0)),
            Err(e) => error = Some(e),
        }
    }
    ArrowMeasurement {
        path: which,
        rows,
        bytes_out,
        output_hash,
        secs,
        usage_before: before,
        usage_after: after,
        error,
    }
}
