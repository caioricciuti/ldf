//! Streams a dataset to disk in the input containers the benchmark reads:
//! JSON array, JSONL, CSV (flat and uniform only) and Parquet.

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use ldf_data::Value;
use ldf_formats::{Format, StreamEncoder};

use crate::datasets::DatasetSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Container {
    Json,
    Jsonl,
    Csv,
    Parquet,
}

impl Container {
    pub fn parse(s: &str) -> Option<Container> {
        match s {
            "json" => Some(Container::Json),
            "jsonl" => Some(Container::Jsonl),
            "csv" => Some(Container::Csv),
            "parquet" => Some(Container::Parquet),
            _ => None,
        }
    }
    pub fn extension(self) -> &'static str {
        match self {
            Container::Json => "json",
            Container::Jsonl => "jsonl",
            Container::Csv => "csv",
            Container::Parquet => "parquet",
        }
    }
}

#[derive(Debug)]
pub struct Written {
    pub container: Container,
    pub path: PathBuf,
    pub bytes: u64,
    pub rows: u64,
}

/// Writes `rows` records of `spec` to `dir/<name>.<ext>` for each requested
/// container. Returns what was written and a note for each container that
/// was skipped because the dataset cannot be represented in it.
pub fn write_dataset(
    spec: &DatasetSpec,
    seed: u64,
    rows: u64,
    dir: &Path,
    containers: &[Container],
) -> std::io::Result<(Vec<Written>, Vec<String>)> {
    fs::create_dir_all(dir)?;
    let mut written = Vec::new();
    let mut skipped = Vec::new();
    let mut jsonl_path: Option<PathBuf> = None;

    type Sink = (Container, PathBuf, BufWriter<File>, Box<dyn StreamEncoder>);
    let mut sinks: Vec<Sink> = Vec::new();
    for &c in containers {
        let enc: Box<dyn StreamEncoder> = match c {
            Container::Json => ldf_formats::json::Json.stream_encoder().unwrap(),
            Container::Jsonl => ldf_formats::json::Jsonl.stream_encoder().unwrap(),
            Container::Csv => {
                if !(spec.flat && spec.uniform) {
                    skipped.push(format!(
                        "{}: csv skipped (dataset is {}{})",
                        spec.name,
                        if spec.flat { "" } else { "nested" },
                        if spec.uniform { "" } else { " non-uniform" }
                    ));
                    continue;
                }
                ldf_formats::tabular::Csv.stream_encoder().unwrap()
            }
            Container::Parquet => continue,
        };
        let path = dir.join(format!("{}.{}", spec.name, c.extension()));
        if c == Container::Jsonl {
            jsonl_path = Some(path.clone());
        }
        sinks.push((
            c,
            path.clone(),
            BufWriter::with_capacity(1 << 20, File::create(&path)?),
            enc,
        ));
    }

    let mut buf: Vec<u8> = Vec::with_capacity(1 << 16);
    for rec in spec.records(seed, rows) {
        for (_, _, file, enc) in sinks.iter_mut() {
            buf.clear();
            enc.record(&rec, &mut buf).map_err(std::io::Error::other)?;
            file.write_all(&buf)?;
        }
    }
    for (c, path, mut file, mut enc) in sinks {
        buf.clear();
        enc.finish(&mut buf).map_err(std::io::Error::other)?;
        file.write_all(&buf)?;
        file.flush()?;
        let bytes = fs::metadata(&path)?.len();
        written.push(Written {
            container: c,
            path,
            bytes,
            rows,
        });
    }

    if containers.contains(&Container::Parquet) {
        #[cfg(feature = "parquet")]
        {
            // Parquet is derived from the JSONL file so both hold identical
            // records. Write a temporary JSONL when it was not requested.
            let (src, temp) = match &jsonl_path {
                Some(p) => (p.clone(), false),
                None => {
                    let p = dir.join(format!("{}.tmp.jsonl", spec.name));
                    let (_, _) = write_dataset(spec, seed, rows, dir, &[Container::Jsonl])?;
                    fs::rename(dir.join(format!("{}.jsonl", spec.name)), &p)?;
                    (p, true)
                }
            };
            let path = dir.join(format!("{}.parquet", spec.name));
            match ldf_data::input::parquet::write_from_jsonl(&src, &path, 8192) {
                Ok(n) => written.push(Written {
                    container: Container::Parquet,
                    bytes: fs::metadata(&path)?.len(),
                    path,
                    rows: n as u64,
                }),
                Err(e) => {
                    let _ = fs::remove_file(&path);
                    skipped.push(format!("{}: parquet skipped ({e})", spec.name));
                }
            }
            if temp {
                fs::remove_file(src)?;
            }
        }
        #[cfg(not(feature = "parquet"))]
        {
            let _ = jsonl_path;
            skipped.push(format!("{}: parquet skipped (feature disabled)", spec.name));
        }
    }
    Ok((written, skipped))
}

/// Materialises records in memory (for tests and small fixtures).
pub fn collect(spec: &DatasetSpec, seed: u64, rows: u64) -> Vec<Value> {
    spec.records(seed, rows).collect()
}
