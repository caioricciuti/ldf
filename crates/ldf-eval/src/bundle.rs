//! Bundle writer: identical prompts across formats, with the data inline.

use std::fs;
use std::io::Write;
use std::path::Path;

use ldf_data::Value;
use ldf_formats::{EncodeError, Format};
use serde::{Deserialize, Serialize};

use crate::tasks::Task;

/// The prompt template. Kept deliberately plain and identical for every
/// format; only `{format_name}` and the data block change.
pub const SYSTEM_PROMPT: &str = "You answer questions about a dataset exactly and concisely. \
Reply with only the answer as a single JSON value (number, string, boolean, null or array) \
and nothing else: no explanation, no code fence, no units.";

pub const USER_TEMPLATE: &str = "Here is a dataset of {record_count} records in {format_name} format.\n\n\
<data>\n{data}\n</data>\n\n\
Question: {question}";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub prompt_id: String,
    pub task_id: String,
    pub format: String,
    pub system: String,
    pub user: String,
    pub data_bytes: u64,
    pub data_tokens_o200k: Option<u64>,
    pub prompt_tokens_o200k: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub dataset: String,
    pub source_path: String,
    pub record_count: usize,
    pub seed: u64,
    pub formats: Vec<String>,
    pub unsupported: Vec<(String, String)>,
    pub tasks: usize,
    pub prompts: usize,
    pub system_prompt: String,
    pub user_template: String,
    pub data_tokens_o200k: Vec<(String, u64)>,
    pub max_prompt_tokens_o200k: Option<u64>,
}

pub struct Tokenizer(tiktoken_rs::CoreBPE);

impl Tokenizer {
    pub fn o200k() -> Option<Tokenizer> {
        tiktoken_rs::o200k_base().ok().map(Tokenizer)
    }
    pub fn count(&self, s: &str) -> u64 {
        self.0.encode_ordinary(s).len() as u64
    }
}

pub struct BundleInput<'a> {
    pub dataset: &'a str,
    pub source_path: &'a str,
    pub seed: u64,
    pub records: &'a [Value],
    pub tasks: &'a [Task],
    pub formats: &'a [Box<dyn Format>],
    /// Externally encoded renderings of the same records, as (name, text).
    /// Lets a design candidate be evaluated before it has a Rust encoder.
    /// The caller is responsible for the text matching `records`.
    pub extra: &'a [(String, String)],
    pub tokenizer: Option<&'a Tokenizer>,
}

pub fn write_bundle(out: &Path, input: &BundleInput<'_>) -> Result<Manifest, String> {
    let BundleInput {
        dataset,
        source_path,
        seed,
        records,
        tasks,
        formats,
        extra,
        tokenizer,
    } = *input;
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let mut tasks_file = fs::File::create(out.join("tasks.jsonl")).map_err(|e| e.to_string())?;
    for t in tasks {
        writeln!(tasks_file, "{}", serde_json::to_string(t).unwrap()).map_err(|e| e.to_string())?;
    }
    let mut prompts_file =
        fs::File::create(out.join("prompts.jsonl")).map_err(|e| e.to_string())?;
    let data_dir = out.join("data");
    fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    let mut used = Vec::new();
    let mut unsupported = Vec::new();
    let mut data_tokens = Vec::new();
    let mut max_prompt_tokens: Option<u64> = None;
    let mut prompt_count = 0usize;
    let mut renderings: Vec<(String, String, String, String)> = Vec::new();
    for f in formats {
        match f.encode_to_string(records) {
            Ok(t) => renderings.push((
                f.name().to_string(),
                f.display_name().to_string(),
                f.extension().to_string(),
                t,
            )),
            Err(EncodeError::Unsupported(u)) => unsupported.push((f.name().to_string(), u.reason)),
            Err(e) => return Err(e.to_string()),
        }
    }
    for (name, text) in extra {
        renderings.push((
            name.clone(),
            name.clone(),
            format!("{name}.txt"),
            text.clone(),
        ));
    }
    for (name, display, ext, text) in &renderings {
        fs::write(data_dir.join(format!("{dataset}.{ext}")), text).map_err(|e| e.to_string())?;
        let dt = tokenizer.map(|t| t.count(text));
        if let Some(n) = dt {
            data_tokens.push((name.clone(), n));
        }
        used.push(name.clone());
        for t in tasks {
            let user = USER_TEMPLATE
                .replace("{record_count}", &records.len().to_string())
                .replace("{format_name}", display)
                .replace("{data}", text)
                .replace("{question}", &t.question);
            let pt = tokenizer.map(|tk| tk.count(SYSTEM_PROMPT) + tk.count(&user));
            if let Some(n) = pt {
                max_prompt_tokens = Some(max_prompt_tokens.map_or(n, |m| m.max(n)));
            }
            let p = Prompt {
                prompt_id: format!("{}__{}", t.id, name),
                task_id: t.id.clone(),
                format: name.clone(),
                system: SYSTEM_PROMPT.to_string(),
                user,
                data_bytes: text.len() as u64,
                data_tokens_o200k: dt,
                prompt_tokens_o200k: pt,
            };
            writeln!(prompts_file, "{}", serde_json::to_string(&p).unwrap())
                .map_err(|e| e.to_string())?;
            prompt_count += 1;
        }
    }
    let manifest = Manifest {
        dataset: dataset.to_string(),
        source_path: source_path.to_string(),
        record_count: records.len(),
        seed,
        formats: used,
        unsupported,
        tasks: tasks.len(),
        prompts: prompt_count,
        system_prompt: SYSTEM_PROMPT.to_string(),
        user_template: USER_TEMPLATE.to_string(),
        data_tokens_o200k: data_tokens,
        max_prompt_tokens_o200k: max_prompt_tokens,
    };
    fs::write(
        out.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    Ok(manifest)
}
