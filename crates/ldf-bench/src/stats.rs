//! Summary statistics over repeated measurements. All raw samples are kept
//! in the JSON output; these summaries exist for the tables.

use serde::Serialize;

#[derive(Debug, Clone, Serialize, Default)]
pub struct Summary {
    pub n: usize,
    pub min: f64,
    pub median: f64,
    pub mean: f64,
    pub p95: f64,
    pub max: f64,
    /// Median absolute deviation, a robust spread estimate.
    pub mad: f64,
}

pub fn summarize(samples: &[f64]) -> Summary {
    if samples.is_empty() {
        return Summary::default();
    }
    let mut s = samples.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = s.len();
    let median = percentile(&s, 50.0);
    let mut dev: Vec<f64> = s.iter().map(|x| (x - median).abs()).collect();
    dev.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Summary {
        n,
        min: s[0],
        median,
        mean: s.iter().sum::<f64>() / n as f64,
        p95: percentile(&s, 95.0),
        max: s[n - 1],
        mad: percentile(&dev, 50.0),
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.len() == 1 {
        return sorted[0];
    }
    let rank = p / 100.0 * (sorted.len() - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    sorted[lo] + (sorted[hi] - sorted[lo]) * (rank - lo as f64)
}

pub fn fmt_bytes(b: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut v = b as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{b} B")
    } else {
        format!("{v:.1} {}", UNITS[i])
    }
}

pub fn fmt_secs(s: f64) -> String {
    if s < 1e-3 {
        format!("{:.1} µs", s * 1e6)
    } else if s < 1.0 {
        format!("{:.2} ms", s * 1e3)
    } else {
        format!("{s:.2} s")
    }
}

/// MB/s with MB = 1e6 bytes.
pub fn mbps(bytes: u64, secs: f64) -> f64 {
    if secs <= 0.0 {
        0.0
    } else {
        bytes as f64 / 1e6 / secs
    }
}
