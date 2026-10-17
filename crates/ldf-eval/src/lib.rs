//! LLM comprehension evaluation: task generation with programmatic ground
//! truth, prompt bundling per format, and scoring of model responses.
//!
//! The Rust side never talks to an API. It writes a *bundle*:
//!
//! * `tasks.jsonl`: one task per line with its canonical answer;
//! * `prompts.jsonl`: one prompt per (task, format) with the serialized data
//!   inline, identical wording across formats;
//! * `manifest.json`: dataset, record count, formats, prompt template hash,
//!   and per-format token counts (exact for OpenAI tokenizers).
//!
//! `eval/run_eval.py` sends prompts (only after an explicit spend
//! confirmation) and writes `responses.jsonl`; `ldf-eval score` grades them.

pub mod answer;
pub mod bundle;
pub mod score;
pub mod tasks;

pub use answer::Answer;
pub use tasks::{Task, TaskKind};
