//! Benchmark library behind the `ldf-bench` binary and the Criterion
//! microbenchmarks. See `site/src/content/docs/docs/methodology.md`.

pub mod arrow;
pub mod measure;
pub mod report;
pub mod roundtrip;
pub mod stats;
pub mod sys;
pub mod tokens;

use std::path::{Path, PathBuf};

use ldf_data::input::InputKind;

/// A dataset file selected for a run.
#[derive(Debug, Clone)]
pub struct Input {
    pub name: String,
    pub path: PathBuf,
    pub kind: InputKind,
}

impl Input {
    pub fn from_path(path: &Path) -> Result<Input, String> {
        let kind = InputKind::from_path(path).map_err(|e| e.to_string())?;
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("input")
            .to_string();
        Ok(Input {
            name,
            path: path.to_path_buf(),
            kind,
        })
    }
}

pub fn kind_name(kind: InputKind) -> &'static str {
    match kind {
        InputKind::Json => "json",
        InputKind::Jsonl => "jsonl",
        InputKind::Csv => "csv",
        InputKind::Tsv => "tsv",
        InputKind::Parquet => "parquet",
    }
}

pub fn kind_from_name(s: &str) -> Option<InputKind> {
    match s {
        "json" => Some(InputKind::Json),
        "jsonl" => Some(InputKind::Jsonl),
        "csv" => Some(InputKind::Csv),
        "tsv" => Some(InputKind::Tsv),
        "parquet" => Some(InputKind::Parquet),
        _ => None,
    }
}

/// Seconds since the Unix epoch rendered as `YYYYMMDD-HHMMSS` (UTC), for
/// result directory names without a date crate.
pub fn timestamp_compact() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (d, s) = (secs / 86_400, secs % 86_400);
    let z = d as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };
    format!(
        "{year:04}{month:02}{day:02}-{:02}{:02}{:02}",
        s / 3600,
        (s % 3600) / 60,
        s % 60
    )
}
