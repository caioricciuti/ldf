//! Pure rewrite rules for the three message kinds the proxy touches.

use ldf_data::Value;
use ldf_data::input::{JsonEngine, parse_json_value};
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
row number), followed by the element's fields. Example, two orders with a customer object and an items \
array:\n\
@records order_id:s,customer.name:s,items:t,total:f,note:s?\n\
ORD-1,Val Thompson,2,715.94,context compare\n\
ORD-2,Sol Ritchie,1,2757.03,null\n\
@records.items order_id:s,sku:s,qty:i,price:f\n\
ORD-1,E5,3,28.78\n\
ORD-1,C3,4,157.4\n\
ORD-2,A1,1,2757.03";

/// Appended to every tool description in `tools/list`.
pub const TOOL_NOTE: &str = " Large JSON results come back as LDF tables (format described in the server instructions).";

/// One text block that was replaced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replaced {
    pub json_bytes: usize,
    pub ldf_bytes: usize,
}

/// Returns the LDF text for `text` when it is a JSON dataset worth
/// converting: a non-empty array of objects, or an object holding at least
/// one non-empty array of objects (which becomes a one-row root table with
/// child tables). `None` when the text is not JSON, not a dataset, shorter
/// than `min_bytes`, or when the LDF would not be shorter.
pub fn convert_text(text: &str, min_bytes: usize) -> Option<String> {
    if text.len() < min_bytes {
        return None;
    }
    let trimmed = text.trim_start();
    if !(trimmed.starts_with('[') || trimmed.starts_with('{')) {
        return None;
    }
    let mut bytes = text.as_bytes().to_vec();
    let root = parse_json_value(&mut bytes, JsonEngine::SerdeJson).ok()?;
    let records: Vec<Value> = match root {
        Value::Array(items) if !items.is_empty() && items.iter().all(|v| v.as_object().is_some()) => items,
        Value::Object(ref o)
            if o.iter().any(|(_, v)| {
                matches!(v, Value::Array(a) if !a.is_empty() && a.iter().all(|v| v.as_object().is_some()))
            }) =>
        {
            vec![root]
        }
        _ => return None,
    };
    let out = ldf::encode(&records);
    (out.len() < text.len()).then_some(out)
}

/// Rewrites the text content blocks of a `tools/call` result in place.
/// Results flagged `isError` and non-text blocks are left alone, as is
/// `structuredContent`.
pub fn rewrite_tool_result(result: &mut Json, min_bytes: usize) -> Vec<Replaced> {
    let mut done = Vec::new();
    if result.get("isError").and_then(Json::as_bool) == Some(true) {
        return done;
    }
    let Some(content) = result.get_mut("content").and_then(Json::as_array_mut) else {
        return done;
    };
    for block in content.iter_mut() {
        if block.get("type").and_then(Json::as_str) != Some("text") {
            continue;
        }
        let Some(text) = block.get("text").and_then(Json::as_str) else {
            continue;
        };
        if let Some(ldf) = convert_text(text, min_bytes) {
            done.push(Replaced {
                json_bytes: text.len(),
                ldf_bytes: ldf.len(),
            });
            block["text"] = Json::String(ldf);
        }
    }
    done
}

/// Appends [`TOOL_NOTE`] to every tool description of a `tools/list` result.
pub fn rewrite_tools_list(result: &mut Json) {
    let Some(tools) = result.get_mut("tools").and_then(Json::as_array_mut) else {
        return;
    };
    for tool in tools.iter_mut() {
        let desc = tool
            .get("description")
            .and_then(Json::as_str)
            .unwrap_or("")
            .trim_end()
            .to_string();
        tool["description"] = Json::String(if desc.is_empty() {
            TOOL_NOTE.trim_start().to_string()
        } else {
            desc + TOOL_NOTE
        });
    }
}

/// Appends [`PRIMER`] to the `instructions` of an `initialize` result.
pub fn rewrite_initialize(result: &mut Json) {
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
    result["instructions"] = Json::String(text);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const ROWS: &str = r#"[{"id":1,"name":"Ana","score":9.5},{"id":2,"name":"Bo","score":7.25},{"id":3,"name":"Cy","score":8.0}]"#;

    #[test]
    fn array_of_objects_becomes_a_table() {
        let out = convert_text(ROWS, 0).unwrap();
        assert_eq!(out, "@records id:i,name:s,score:f\n1,Ana,9.5\n2,Bo,7.25\n3,Cy,8.0\n");
        let back = ldf::decode(&out).unwrap();
        assert_eq!(back.len(), 3);
    }

    #[test]
    fn object_with_rows_becomes_root_plus_child_table() {
        let rows: Vec<String> = (1..=10).map(|i| format!(r#"{{"id":{i},"v":"value {i}"}}"#)).collect();
        let text = format!(r#"{{"meta":{{"elapsed":0.2}},"rows":[{}]}}"#, rows.join(","));
        let out = convert_text(&text, 0).unwrap();
        // With one parent row its first column is unique, so the child
        // table joins on it (spec section 7), here `meta.elapsed`.
        assert!(out.starts_with("@records meta.elapsed:f,rows:t\n0.2,10\n@records.rows meta.elapsed:f,id:i,v:s\n0.2,1,value 1\n"), "{out}");
        let back = ldf::decode(&out).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].get("rows").and_then(|r| r.as_array()).map(Vec::len), Some(10));
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
        let done = rewrite_tool_result(&mut ok, 0);
        assert_eq!(done.len(), 1);
        assert!(ok["content"][0]["text"].as_str().unwrap().starts_with("@records "));
        assert_eq!(ok["content"][1]["text"], "done");

        let mut err = json!({"isError": true, "content": [{"type": "text", "text": ROWS}]});
        assert!(rewrite_tool_result(&mut err, 0).is_empty());
        assert_eq!(err["content"][0]["text"], ROWS);
    }

    #[test]
    fn list_and_initialize_get_their_notes() {
        let mut list = json!({"tools": [{"name": "q", "description": "Runs SQL. "}, {"name": "n"}]});
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
}
