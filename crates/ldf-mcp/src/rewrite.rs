//! Pure rewrite rules for the three message kinds the proxy touches.

use std::time::Instant;

use ldf_data::Value;
use ldf_data::input::{JsonEngine, parse_json_value};
use ldf_select::{Choice, Evidence, Policy};
use serde_json::Value as Json;

/// What the model is told once, at `initialize`, about the format it will
/// receive. Kept to the rules a reader needs; the full spec is
/// `docs/ldf-spec-v0.md`.
pub const PRIMER: &str = "Tool results from this server may be returned in LDF (Language Data Format), \
a lossless table encoding of JSON records. A document is one or more tables. A table is a header line \
`@name col:type,col:type,...` followed by one row per record, cells separated by commas, in header order. \
Types: i integer, f float, s string, b boolean, j any JSON value, t child table (the cell holds that \
record's number of child rows). A `?` after a type marks a column that may be null or absent. An empty \
cell means the key is absent from the record; `null` means the key is present with value null. Strings \
are bare unless they contain a comma, quote, backslash or newline, are empty, or read as null/true/false; \
those are JSON string literals. Column names are dotted paths into nested objects: `customer.name` is \
field `name` of object `customer`. An array of objects becomes a child table named `parent.field`; each \
child row starts with a join column holding the parent's first column value (or `_`, the parent's 1-based \
row number), followed by the element's fields. A column typed with a key list, `items:[sku,qty]`, holds \
the whole array in its cell as a JSON array with one array of values per element, in key order: \
`[[\"E5\",3],[\"C3\",4]]` is two elements, `{\"sku\":\"E5\",\"qty\":3}` and `{\"sku\":\"C3\",\"qty\":4}`. Example, two \
orders with a customer object and an items array:\n\
@records order_id:s,customer.name:s,items:t,total:f,note:s?\n\
ORD-1,Val Thompson,2,715.94,context compare\n\
ORD-2,Sol Ritchie,1,2757.03,null\n\
@records.items order_id:s,sku:s,qty:i,price:f\n\
ORD-1,E5,3,28.78\n\
ORD-1,C3,4,157.4\n\
ORD-2,A1,1,2757.03";

/// Appended to every tool description in `tools/list`.
pub const TOOL_NOTE: &str =
    " Large JSON results come back as LDF tables (format described in the server instructions).";

/// Why a text block stayed as it was. [`Kept::code`] is what the local
/// stats record; only the selector's reason carries text, and that text is
/// about the evidence, never about the data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kept {
    /// Shorter than `min_bytes`.
    Small,
    /// Does not start like JSON or does not parse.
    NotJson,
    /// JSON, but not an array of objects or an object holding one.
    NotDataset,
    /// LDF would not be shorter.
    NotShorter,
    /// The selector kept JSON for this model, with its reason.
    Selector(String),
    /// The result is flagged `isError`: never touched.
    IsError,
}

impl Kept {
    pub fn code(&self) -> &'static str {
        match self {
            Kept::Small => "small",
            Kept::NotJson => "not_json",
            Kept::NotDataset => "not_dataset",
            Kept::NotShorter => "not_shorter",
            Kept::Selector(_) => "selector",
            Kept::IsError => "is_error",
        }
    }
}

/// Picks JSON or LDF for each result from measured evidence for one model
/// (`ldf-select`). Without a selector, LDF is used whenever it is shorter.
#[derive(Debug, Clone)]
pub struct Selector {
    pub model: String,
    pub evidence: Vec<Evidence>,
    pub policy: Policy,
}

impl Selector {
    /// `evidence` defaults to the evidence built into `ldf-select`.
    pub fn new(model: &str, evidence: Option<Vec<Evidence>>) -> Selector {
        Selector {
            model: model.to_string(),
            evidence: evidence.unwrap_or_else(ldf_select::builtin_evidence),
            policy: Policy::default(),
        }
    }

    /// False when the evidence holds nothing for this model, so every
    /// result stays JSON.
    pub fn knows_model(&self) -> bool {
        ldf_select::knows_model(&self.model, &self.evidence, &self.policy)
    }
}

/// [`convert`] without a selector.
pub fn convert_text(text: &str, min_bytes: usize) -> Option<String> {
    convert(text, min_bytes, None).ok()
}

/// Returns the LDF text for `text` when it is a JSON dataset worth
/// converting: a non-empty array of objects, or an object holding at least
/// one non-empty array of objects (which becomes a one-row root table with
/// child tables). Otherwise says why it stays as it is ([`Kept`]).
pub fn convert(text: &str, min_bytes: usize, sel: Option<&Selector>) -> Result<String, Kept> {
    if text.len() < min_bytes {
        return Err(Kept::Small);
    }
    let trimmed = text.trim_start();
    if !(trimmed.starts_with('[') || trimmed.starts_with('{')) {
        return Err(Kept::NotJson);
    }
    let mut bytes = text.as_bytes().to_vec();
    let root = parse_json_value(&mut bytes, JsonEngine::SerdeJson).map_err(|_| Kept::NotJson)?;
    let records: Vec<Value> = match root {
        Value::Array(items) if !items.is_empty() && items.iter().all(|v| v.as_object().is_some()) => items,
        Value::Object(ref o)
            if o.iter().any(|(_, v)| {
                matches!(v, Value::Array(a) if !a.is_empty() && a.iter().all(|v| v.as_object().is_some()))
            }) =>
        {
            vec![root]
        }
        _ => return Err(Kept::NotDataset),
    };
    let Some(sel) = sel else {
        let out = ldf::encode(&records);
        return if out.len() < text.len() {
            Ok(out)
        } else {
            Err(Kept::NotShorter)
        };
    };
    let (d, out) = ldf_select::select_ldf(
        &records,
        &sel.model,
        &sel.evidence,
        &sel.policy,
        text.len() as u64,
    );
    out.ok_or_else(|| {
        // The first reason the smallest LDF candidate was turned down.
        let why = d
            .verdicts
            .iter()
            .filter(|v| v.choice != Choice::Json && !v.accepted)
            .min_by_key(|v| v.tokens.unwrap_or(u64::MAX))
            .and_then(|v| v.notes.iter().rev().find(|n| !n.starts_with("ok: ")))
            .cloned()
            .unwrap_or_default();
        Kept::Selector(format!(
            "json for {} ({} rows): {why}",
            sel.model, d.profile.rows
        ))
    })
}

/// One text block of a `tools/call` result and what happened to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub in_bytes: usize,
    pub out_bytes: usize,
    /// `None` when the block was replaced by LDF.
    pub kept: Option<Kept>,
}

/// What [`rewrite_tool_result`] did to one result.
#[derive(Debug, Default)]
pub struct Rewritten {
    /// The text blocks, in order.
    pub blocks: Vec<Block>,
    /// Time spent deciding and encoding, in microseconds.
    pub micros: u64,
}

impl Rewritten {
    pub fn converted(&self) -> bool {
        self.blocks.iter().any(|b| b.kept.is_none())
    }
}

/// Rewrites the text content blocks of a `tools/call` result in place.
/// Results flagged `isError` and non-text blocks are left alone, as is
/// `structuredContent`.
pub fn rewrite_tool_result(
    result: &mut Json,
    min_bytes: usize,
    sel: Option<&Selector>,
) -> Rewritten {
    let start = Instant::now();
    let mut done = Rewritten::default();
    let is_error = result.get("isError").and_then(Json::as_bool) == Some(true);
    if let Some(content) = result.get_mut("content").and_then(Json::as_array_mut) {
        for block in content.iter_mut() {
            if block.get("type").and_then(Json::as_str) != Some("text") {
                continue;
            }
            let Some(text) = block.get("text").and_then(Json::as_str) else {
                continue;
            };
            let converted = if is_error {
                Err(Kept::IsError)
            } else {
                convert(text, min_bytes, sel)
            };
            match converted {
                Ok(ldf) => {
                    done.blocks.push(Block {
                        in_bytes: text.len(),
                        out_bytes: ldf.len(),
                        kept: None,
                    });
                    block["text"] = Json::String(ldf);
                }
                Err(why) => done.blocks.push(Block {
                    in_bytes: text.len(),
                    out_bytes: text.len(),
                    kept: Some(why),
                }),
            }
        }
    }
    done.micros = start.elapsed().as_micros() as u64;
    done
}

/// Appends [`TOOL_NOTE`] to every tool description of a `tools/list` result.
pub fn rewrite_tools_list(result: &mut Json) {
    let Some(tools) = result.get_mut("tools").and_then(Json::as_array_mut) else {
        return;
    };
    for tool in tools.iter_mut() {
        // A malformed entry is the client's to reject, not ours to crash on.
        let Some(tool) = tool.as_object_mut() else {
            continue;
        };
        let desc = tool
            .get("description")
            .and_then(Json::as_str)
            .unwrap_or("")
            .trim_end()
            .to_string();
        let note = if desc.is_empty() {
            TOOL_NOTE.trim_start().to_string()
        } else {
            desc + TOOL_NOTE
        };
        tool.insert("description".into(), Json::String(note));
    }
}

/// Appends [`PRIMER`] to the `instructions` of an `initialize` result.
pub fn rewrite_initialize(result: &mut Json) {
    let Some(result) = result.as_object_mut() else {
        return;
    };
    let existing = result
        .get("instructions")
        .and_then(Json::as_str)
        .unwrap_or("")
        .trim_end()
        .to_string();
    let text = if existing.is_empty() {
        PRIMER.to_string()
    } else {
        existing + "\n\n" + PRIMER
    };
    result.insert("instructions".into(), Json::String(text));
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const ROWS: &str = r#"[{"id":1,"name":"Ana","score":9.5},{"id":2,"name":"Bo","score":7.25},{"id":3,"name":"Cy","score":8.0}]"#;

    #[test]
    fn array_of_objects_becomes_a_table() {
        let out = convert_text(ROWS, 0).unwrap();
        assert_eq!(
            out,
            "@records id:i,name:s,score:f\n1,Ana,9.5\n2,Bo,7.25\n3,Cy,8.0\n"
        );
        let back = ldf::decode(&out).unwrap();
        assert_eq!(back.len(), 3);
    }

    #[test]
    fn object_with_rows_becomes_root_plus_child_table() {
        let rows: Vec<String> = (1..=10)
            .map(|i| format!(r#"{{"id":{i},"v":"value {i}"}}"#))
            .collect();
        let text = format!(
            r#"{{"meta":{{"elapsed":0.2}},"rows":[{}]}}"#,
            rows.join(",")
        );
        let out = convert_text(&text, 0).unwrap();
        // With one parent row its first column is unique, so the child
        // table joins on it (spec section 7), here `meta.elapsed`.
        assert!(out.starts_with("@records meta.elapsed:f,rows:t\n0.2,10\n@records.rows meta.elapsed:f,id:i,v:s\n0.2,1,value 1\n"), "{out}");
        let back = ldf::decode(&out).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(
            back[0].get("rows").and_then(|r| r.as_array()).map(Vec::len),
            Some(10)
        );
    }

    #[test]
    fn leaves_what_is_not_a_dataset() {
        assert_eq!(convert_text("plain text", 0), None);
        assert_eq!(convert_text("[1,2,3]", 0), None);
        assert_eq!(convert_text("[]", 0), None);
        assert_eq!(convert_text(r#"{"ok":true}"#, 0), None);
        assert_eq!(convert_text("[{\"a\":1}", 0), None, "truncated JSON");
    }

    #[test]
    fn respects_min_bytes_and_only_shrinks() {
        assert_eq!(convert_text(ROWS, 10_000), None);
        // One tiny record: the header costs more than the braces it saves.
        assert_eq!(convert_text(r#"[{"a":1}]"#, 0), None);
    }

    #[test]
    fn tool_result_text_blocks_are_replaced_and_errors_are_not() {
        let mut ok = json!({"content": [
            {"type": "text", "text": ROWS},
            {"type": "text", "text": "done"},
            {"type": "image", "data": "AAAA", "mimeType": "image/png"}
        ]});
        let done = rewrite_tool_result(&mut ok, 0, None);
        assert!(done.converted());
        assert_eq!(done.blocks.len(), 2, "text blocks only");
        assert_eq!(done.blocks[1].kept, Some(Kept::NotJson));
        assert!(
            ok["content"][0]["text"]
                .as_str()
                .unwrap()
                .starts_with("@records ")
        );
        assert_eq!(ok["content"][1]["text"], "done");

        let mut err = json!({"isError": true, "content": [{"type": "text", "text": ROWS}]});
        let done = rewrite_tool_result(&mut err, 0, None);
        assert!(!done.converted());
        assert_eq!(done.blocks[0].kept, Some(Kept::IsError));
        assert_eq!(err["content"][0]["text"], ROWS);
        assert_eq!(err["content"][0]["text"], ROWS);
    }

    #[test]
    fn list_and_initialize_get_their_notes() {
        let mut list =
            json!({"tools": [{"name": "q", "description": "Runs SQL. "}, {"name": "n"}]});
        rewrite_tools_list(&mut list);
        assert_eq!(
            list["tools"][0]["description"],
            format!("Runs SQL.{TOOL_NOTE}")
        );
        assert_eq!(list["tools"][1]["description"], TOOL_NOTE.trim_start());

        let mut init = json!({"protocolVersion": "2025-06-18", "instructions": "Be nice."});
        rewrite_initialize(&mut init);
        let s = init["instructions"].as_str().unwrap();
        assert!(s.starts_with("Be nice.\n\nTool results"));
        let mut bare = json!({"protocolVersion": "2025-06-18"});
        rewrite_initialize(&mut bare);
        assert_eq!(bare["instructions"], PRIMER);
    }

    #[test]
    fn primer_example_is_valid_ldf() {
        let example = PRIMER.split_once("array:\n").unwrap().1;
        let recs = ldf::decode(example).unwrap();
        assert_eq!(recs.len(), 2);
    }

    fn rows(n: usize) -> String {
        let r: Vec<String> = (0..n)
            .map(|i| format!(r#"{{"id":{i},"name":"user {i}","score":{}.5}}"#, i % 9))
            .collect();
        format!("[{}]", r.join(","))
    }

    fn evidence(model: &str) -> Vec<Evidence> {
        vec![Evidence {
            model: model.into(),
            run: "r".into(),
            dataset: "d".into(),
            records: 200,
            traits: vec![],
            family: "ldf".into(),
            thresholds: vec![0.5, 0.75],
            hoist_thresholds: vec![0.5, 0.75],
            format: "ldf".into(),
            n: 80,
            k: 70,
            json_k: 70,
            wins: 1,
            losses: 1,
            p: 1.0,
            tokens_ratio: 0.6,
        }]
    }

    #[test]
    fn selector_converts_for_a_measured_model_only() {
        let text = rows(50);
        let sel = Selector::new("m", Some(evidence("m")));
        assert!(sel.knows_model());
        let out = convert(&text, 0, Some(&sel)).unwrap();
        assert_eq!(ldf::decode(&out).unwrap().len(), 50);

        let other = Selector::new("other", Some(evidence("m")));
        assert!(!other.knows_model());
        let Err(Kept::Selector(why)) = convert(&text, 0, Some(&other)) else {
            panic!("selector reason expected");
        };
        assert!(why.contains("no evidence for model other"), "{why}");

        // More rows than measured: JSON, with the reason.
        let Err(Kept::Selector(why)) = convert(&rows(300), 0, Some(&sel)) else {
            panic!("selector reason expected");
        };
        assert!(why.contains("extrapolation is off"), "{why}");

        // Not a dataset: the selector never ran.
        assert_eq!(convert("[1,2]", 0, Some(&sel)), Err(Kept::NotDataset));
    }

    #[test]
    fn builtin_evidence_keeps_gpt_on_json() {
        // GPT runs have under 50 questions per cell: not enough to count.
        let sel = Selector::new("gpt-6.1-sol", None);
        assert!(!sel.knows_model());
        assert!(Selector::new("claude-haiku-5-5", None).knows_model());
        assert!(Selector::new("claude-opus-5-5", None).knows_model());
        assert!(convert(&rows(50), 0, Some(&sel)).is_err());
    }
}
