//! Scoring of model responses against the bundle's tasks.
//!
//! Outcomes are kept apart so that a bad number can be traced to its cause:
//! * `correct`, `wrong`: the model answered and we could compare;
//! * `malformed`: the response could not be parsed into an answer;
//! * `truncated`: the provider reported the output hit `max_tokens`;
//! * `context_exceeded`: the request was rejected for length;
//! * `api_error`: any other transport or provider error;
//! * `format_unsupported`: the format could not encode the dataset, so no
//!   prompt existed (counted from the manifest, not from responses).

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::answer::{parse_response, parse_wrapped};
use crate::tasks::{Task, TaskKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub prompt_id: String,
    pub task_id: String,
    pub format: String,
    pub provider: String,
    pub model: String,
    pub text: Option<String>,
    pub finish_reason: Option<String>,
    pub error: Option<String>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub latency_ms: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Correct,
    Wrong,
    /// The right value inside a one-key object (`{"id": 94}` for `94`).
    /// Not counted as correct; reported so the answer-shape slips are
    /// visible apart from misreads.
    Wrapped,
    Malformed,
    Truncated,
    ContextExceeded,
    ApiError,
}

#[derive(Debug, Clone, Serialize)]
pub struct Graded {
    pub prompt_id: String,
    pub task_id: String,
    pub kind: TaskKind,
    pub format: String,
    pub model: String,
    pub outcome: Outcome,
    pub expected: String,
    pub got: Option<String>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub latency_ms: Option<f64>,
}

pub fn grade(tasks: &[Task], responses: &[Response]) -> Vec<Graded> {
    let by_id: BTreeMap<&str, &Task> = tasks.iter().map(|t| (t.id.as_str(), t)).collect();
    responses
        .iter()
        .filter_map(|r| {
            let task = by_id.get(r.task_id.as_str())?;
            let outcome = if let Some(err) = &r.error {
                let e = err.to_lowercase();
                if e.contains("context")
                    || e.contains("too long")
                    || e.contains("maximum")
                    || e.contains("prompt is too")
                {
                    Outcome::ContextExceeded
                } else {
                    Outcome::ApiError
                }
            } else if matches!(r.finish_reason.as_deref(), Some("length" | "max_tokens")) {
                Outcome::Truncated
            } else {
                let text = r.text.as_deref().unwrap_or("");
                let wrapped = || {
                    parse_wrapped(text)
                        .is_some_and(|inner| task.answer.accepts(&inner, &task.tolerance))
                };
                match parse_response(text) {
                    None if wrapped() => Outcome::Wrapped,
                    None => Outcome::Malformed,
                    Some(got) => {
                        if task.answer.accepts(&got, &task.tolerance) {
                            Outcome::Correct
                        } else if wrapped() {
                            Outcome::Wrapped
                        } else {
                            Outcome::Wrong
                        }
                    }
                }
            };
            Some(Graded {
                prompt_id: r.prompt_id.clone(),
                task_id: r.task_id.clone(),
                kind: task.kind,
                format: r.format.clone(),
                model: r.model.clone(),
                outcome,
                expected: serde_json::to_string(&task.answer).unwrap(),
                got: r.text.clone(),
                input_tokens: r.input_tokens,
                output_tokens: r.output_tokens,
                latency_ms: r.latency_ms,
            })
        })
        .collect()
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Cell {
    pub n: usize,
    pub correct: usize,
    pub wrong: usize,
    pub wrapped: usize,
    pub malformed: usize,
    pub truncated: usize,
    pub context_exceeded: usize,
    pub api_error: usize,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub latency_ms_sum: f64,
    pub latency_n: usize,
}

impl Cell {
    fn add(&mut self, g: &Graded) {
        self.n += 1;
        match g.outcome {
            Outcome::Correct => self.correct += 1,
            Outcome::Wrong => self.wrong += 1,
            Outcome::Wrapped => self.wrapped += 1,
            Outcome::Malformed => self.malformed += 1,
            Outcome::Truncated => self.truncated += 1,
            Outcome::ContextExceeded => self.context_exceeded += 1,
            Outcome::ApiError => self.api_error += 1,
        }
        self.input_tokens += g.input_tokens.unwrap_or(0);
        self.output_tokens += g.output_tokens.unwrap_or(0);
        if let Some(l) = g.latency_ms {
            self.latency_ms_sum += l;
            self.latency_n += 1;
        }
    }
    pub fn accuracy(&self) -> f64 {
        if self.n == 0 {
            0.0
        } else {
            self.correct as f64 / self.n as f64
        }
    }
}

pub type ByFormat = BTreeMap<(String, String), Cell>;
pub type ByKind = BTreeMap<(String, String, TaskKind), Cell>;

/// Per (model, format) and per (model, format, kind) summaries.
pub fn summarize(graded: &[Graded]) -> (ByFormat, ByKind) {
    let mut by_format: ByFormat = BTreeMap::new();
    let mut by_kind: ByKind = BTreeMap::new();
    for g in graded {
        by_format
            .entry((g.model.clone(), g.format.clone()))
            .or_default()
            .add(g);
        by_kind
            .entry((g.model.clone(), g.format.clone(), g.kind))
            .or_default()
            .add(g);
    }
    (by_format, by_kind)
}

pub fn render_markdown(graded: &[Graded], unsupported: &[(String, String)]) -> String {
    let (by_format, by_kind) = summarize(graded);
    let mut md = String::new();
    let _ = writeln!(md, "# LLM comprehension results\n");
    let _ = writeln!(
        md,
        "| model | format | n | accuracy | correct | wrong | wrapped | malformed | truncated | ctx exceeded | api error | mean input tokens | mean latency |"
    );
    let _ = writeln!(
        md,
        "|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"
    );
    for ((model, format), c) in &by_format {
        let _ = writeln!(
            md,
            "| {model} | {format} | {} | {:.1}% | {} | {} | {} | {} | {} | {} | {} | {:.0} | {} |",
            c.n,
            c.accuracy() * 100.0,
            c.correct,
            c.wrong,
            c.wrapped,
            c.malformed,
            c.truncated,
            c.context_exceeded,
            c.api_error,
            if c.n > 0 {
                c.input_tokens as f64 / c.n as f64
            } else {
                0.0
            },
            if c.latency_n > 0 {
                format!("{:.0} ms", c.latency_ms_sum / c.latency_n as f64)
            } else {
                "-".into()
            }
        );
    }
    for (f, reason) in unsupported {
        let _ = writeln!(
            md,
            "| - | {f} | 0 | format unsupported: {reason} | | | | | | | | |"
        );
    }
    let _ = writeln!(md, "\n## By task kind\n");
    let _ = writeln!(md, "| model | format | kind | n | accuracy |");
    let _ = writeln!(md, "|---|---|---|---:|---:|");
    for ((model, format, kind), c) in &by_kind {
        let _ = writeln!(
            md,
            "| {model} | {format} | {} | {} | {:.1}% |",
            serde_json::to_string(kind).unwrap().trim_matches('"'),
            c.n,
            c.accuracy() * 100.0
        );
    }
    md
}
