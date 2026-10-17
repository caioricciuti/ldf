//! Token counting.
//!
//! Exact counts are available for OpenAI tokenizers through `tiktoken-rs`,
//! which embeds the `o200k_base` (GPT-4o, GPT-4.1, GPT-5 family) and
//! `cl100k_base` (GPT-4, GPT-3.5) vocabularies. Anthropic does not publish
//! the Claude 3+ tokenizer; exact Claude counts come only from the
//! `count_tokens` API or from `usage` fields of real responses, which the
//! Python runner in `eval/` records. Nothing here estimates Claude tokens.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TokenCount {
    pub tokenizer: String,
    /// Exact means a real tokenizer implementation produced it.
    pub exact: bool,
    pub tokens: u64,
}

pub struct Tokenizers {
    o200k: Option<tiktoken_rs::CoreBPE>,
    cl100k: Option<tiktoken_rs::CoreBPE>,
}

impl Tokenizers {
    pub fn new(names: &[&str]) -> Result<Tokenizers, String> {
        let mut t = Tokenizers {
            o200k: None,
            cl100k: None,
        };
        for n in names {
            match *n {
                "o200k_base" => {
                    t.o200k = Some(tiktoken_rs::o200k_base().map_err(|e| e.to_string())?)
                }
                "cl100k_base" => {
                    t.cl100k = Some(tiktoken_rs::cl100k_base().map_err(|e| e.to_string())?)
                }
                other => {
                    return Err(format!(
                        "unknown tokenizer {other:?}; known: o200k_base, cl100k_base"
                    ));
                }
            }
        }
        Ok(t)
    }

    pub fn count(&self, text: &str) -> Vec<TokenCount> {
        let mut out = Vec::new();
        if let Some(bpe) = &self.o200k {
            out.push(TokenCount {
                tokenizer: "o200k_base".into(),
                exact: true,
                tokens: bpe.encode_ordinary(text).len() as u64,
            });
        }
        if let Some(bpe) = &self.cl100k {
            out.push(TokenCount {
                tokenizer: "cl100k_base".into(),
                exact: true,
                tokens: bpe.encode_ordinary(text).len() as u64,
            });
        }
        out
    }
}
