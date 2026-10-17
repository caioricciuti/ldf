//! Result schema and Markdown rendering.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::measure::{Measurement, Op};
use crate::roundtrip::{Level, RoundTrip};
use crate::stats::{Summary, fmt_bytes, fmt_secs, mbps, summarize};
use crate::sys::SysInfo;
use crate::tokens::TokenCount;

#[derive(Debug, Clone, Serialize)]
pub struct Run {
    pub tool: &'static str,
    pub started_utc: String,
    pub command: Vec<String>,
    pub iters: usize,
    pub sysinfo: SysInfo,
    pub datasets: Vec<DatasetResult>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct DatasetResult {
    pub name: String,
    pub path: String,
    pub input_kind: String,
    pub records: u64,
    pub input_bytes: u64,
    pub roundtrips: Vec<RoundTrip>,
    pub measurements: Vec<Measurement>,
    pub tokens: Vec<FormatTokens>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatTokens {
    pub format: String,
    pub bytes: u64,
    pub records: u64,
    pub counts: Vec<TokenCountOwned>,
    pub unsupported: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCountOwned {
    pub tokenizer: String,
    pub exact: bool,
    pub tokens: u64,
}

impl From<TokenCount> for TokenCountOwned {
    fn from(t: TokenCount) -> Self {
        TokenCountOwned {
            tokenizer: t.tokenizer,
            exact: t.exact,
            tokens: t.tokens,
        }
    }
}

fn warm(m: &Measurement) -> Summary {
    // Index 0 is the cold iteration; the summary covers the rest when there
    // is more than one sample.
    if m.secs.len() > 1 {
        summarize(&m.secs[1..])
    } else {
        summarize(&m.secs)
    }
}

fn cold(m: &Measurement) -> Option<f64> {
    m.secs.first().copied()
}

pub fn render_markdown(run: &Run) -> String {
    let mut md = String::new();
    let si = &run.sysinfo;
    let _ = writeln!(md, "# ldf-bench results\n");
    let _ = writeln!(
        md,
        "Started {} UTC. Command: `{}`\n",
        run.started_utc,
        run.command.join(" ")
    );
    let _ = writeln!(
        md,
        "Machine: {} ({}), {} logical CPUs, {} RAM, {}. {} profile, commit {}.\n",
        si.cpu,
        si.arch,
        si.logical_cpus,
        fmt_bytes(si.memory_bytes),
        si.os,
        si.cargo_profile,
        si.git_commit
    );
    let _ = writeln!(
        md,
        "Toolchain: {}. Crates: {}.\n",
        si.rustc,
        si.crate_versions
            .iter()
            .map(|(n, v)| format!("{n} {v}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let _ = writeln!(
        md,
        "Each operation runs in its own child process. Iteration 0 is the cold run; \
         medians and p95 are over the remaining {} warm iterations. MB = 1e6 bytes. \
         Peak RSS is the child process high-water mark after the operation, as reported by getrusage, \
         minus nothing: it includes the loaded input. See docs/benchmark-methodology.md.\n",
        run.iters.saturating_sub(1)
    );

    for ds in &run.datasets {
        let _ = writeln!(
            md,
            "## {} ({} records, {} as {})\n",
            ds.name,
            ds.records,
            fmt_bytes(ds.input_bytes),
            ds.input_kind
        );

        // Parse table.
        let parses: Vec<&Measurement> = ds
            .measurements
            .iter()
            .filter(|m| m.op == Op::Parse)
            .collect();
        let ios: Vec<&Measurement> = ds.measurements.iter().filter(|m| m.op == Op::Io).collect();
        if !parses.is_empty() || !ios.is_empty() {
            let _ = writeln!(md, "### Input: I/O and parse\n");
            let _ = writeln!(
                md,
                "| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |"
            );
            let _ = writeln!(md, "|---|---|---:|---:|---:|---:|---:|---:|");
            for m in ios.iter().chain(parses.iter()) {
                if let Some(e) = &m.error {
                    let _ = writeln!(
                        md,
                        "| {:?} | {} | error: {} | | | | | |",
                        m.op,
                        m.engine.clone().unwrap_or_default(),
                        e
                    );
                    continue;
                }
                let w = warm(m);
                let _ = writeln!(
                    md,
                    "| {} | {} | {} | {} | {} | {:.1} | {} | {} |",
                    match m.op {
                        Op::Io => "read file",
                        _ => "parse",
                    },
                    m.engine.clone().unwrap_or_else(|| "-".into()),
                    cold(m).map(fmt_secs).unwrap_or_default(),
                    fmt_secs(w.median),
                    fmt_secs(w.p95),
                    mbps(m.bytes, w.median),
                    if m.records > 0 && w.median > 0.0 {
                        format!("{:.0}", m.records as f64 / w.median)
                    } else {
                        "-".into()
                    },
                    fmt_bytes(m.usage_after.max_rss_bytes)
                );
            }
            md.push('\n');
        }

        // Format table.
        let json_bytes = ds
            .roundtrips
            .iter()
            .find(|r| r.format == "json")
            .map(|r| r.output_bytes)
            .unwrap_or(0);
        let mut by_format: BTreeMap<&str, (Option<&Measurement>, Option<&Measurement>)> =
            BTreeMap::new();
        for m in &ds.measurements {
            if let Some(f) = &m.format {
                let e = by_format.entry(f.as_str()).or_default();
                match m.op {
                    Op::Encode => e.0 = Some(m),
                    Op::Decode => e.1 = Some(m),
                    _ => {}
                }
            }
        }
        if !ds.roundtrips.is_empty() {
            let _ = writeln!(md, "### Formats\n");
            let _ = writeln!(
                md,
                "| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |"
            );
            let _ = writeln!(
                md,
                "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
            );
            for rt in &ds.roundtrips {
                let (enc, dec) = by_format
                    .get(rt.format.as_str())
                    .copied()
                    .unwrap_or((None, None));
                if rt.level == Level::Unsupported {
                    let _ = writeln!(
                        md,
                        "| {} | unsupported: {} | | | | | | | | | | |",
                        rt.format,
                        rt.detail.clone().unwrap_or_default()
                    );
                    continue;
                }
                let ratio = if json_bytes > 0 {
                    format!("{:.2}", rt.output_bytes as f64 / json_bytes as f64)
                } else {
                    "-".into()
                };
                let cell = |m: Option<&Measurement>| -> (String, String, String, String) {
                    match m {
                        Some(m) if m.error.is_none() && !m.secs.is_empty() => {
                            let w = warm(m);
                            (
                                cold(m).map(fmt_secs).unwrap_or_default(),
                                fmt_secs(w.median),
                                format!("{:.1}", mbps(m.bytes, w.median)),
                                fmt_bytes(m.usage_after.max_rss_bytes),
                            )
                        }
                        Some(m) => (
                            format!("error: {}", m.error.clone().unwrap_or_default()),
                            String::new(),
                            String::new(),
                            String::new(),
                        ),
                        None => ("-".into(), "-".into(), "-".into(), "-".into()),
                    }
                };
                let e = cell(enc);
                let d = cell(dec);
                let level = match &rt.detail {
                    Some(det) if rt.level != Level::Byte && rt.level != Level::Semantic => {
                        format!("{} ({})", rt.level.label(), det)
                    }
                    _ => rt.level.label().to_string(),
                };
                let _ = writeln!(
                    md,
                    "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                    rt.format,
                    level,
                    fmt_bytes(rt.output_bytes),
                    ratio,
                    e.0,
                    e.1,
                    e.2,
                    d.0,
                    d.1,
                    d.2,
                    e.3,
                    d.3
                );
            }
            md.push('\n');
        }

        // Token table.
        if !ds.tokens.is_empty() {
            let tokenizers: Vec<String> = {
                let mut t: Vec<String> = ds
                    .tokens
                    .iter()
                    .flat_map(|f| f.counts.iter().map(|c| c.tokenizer.clone()))
                    .collect();
                t.sort();
                t.dedup();
                t
            };
            let json_tokens: BTreeMap<&str, u64> = ds
                .tokens
                .iter()
                .find(|f| f.format == "json")
                .map(|f| {
                    f.counts
                        .iter()
                        .map(|c| (c.tokenizer.as_str(), c.tokens))
                        .collect()
                })
                .unwrap_or_default();
            let _ = writeln!(
                md,
                "### Tokens ({} records)\n",
                ds.tokens.first().map(|t| t.records).unwrap_or(0)
            );
            let mut header = String::from("| format | bytes |");
            let mut sep = String::from("|---|---:|");
            for t in &tokenizers {
                let _ = write!(header, " {t} | vs json | bytes/token |");
                sep.push_str("---:|---:|---:|");
            }
            let _ = writeln!(md, "{header}\n{sep}");
            for f in &ds.tokens {
                if let Some(u) = &f.unsupported {
                    let _ = writeln!(md, "| {} | unsupported: {u} |", f.format);
                    continue;
                }
                let mut row = format!("| {} | {} |", f.format, fmt_bytes(f.bytes));
                for t in &tokenizers {
                    match f.counts.iter().find(|c| &c.tokenizer == t) {
                        Some(c) => {
                            let vs = json_tokens
                                .get(t.as_str())
                                .map(|j| format!("{:.2}", c.tokens as f64 / *j as f64))
                                .unwrap_or_else(|| "-".into());
                            let bpt = if c.tokens > 0 {
                                format!("{:.2}", f.bytes as f64 / c.tokens as f64)
                            } else {
                                "-".into()
                            };
                            let _ = write!(
                                row,
                                " {}{} | {vs} | {bpt} |",
                                c.tokens,
                                if c.exact { "" } else { " (est.)" }
                            );
                        }
                        None => row.push_str(" - | - | - |"),
                    }
                }
                let _ = writeln!(md, "{row}");
            }
            let _ = writeln!(
                md,
                "\nClaude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.\n"
            );
        }
    }
    md
}
