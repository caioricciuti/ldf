//! Pure rewrite rules for the three message kinds the proxy touches.

use std::time::Instant;

use ldf_data::Value;
use ldf_data::input::{JsonEngine, parse_json_value};
use ldf_select::{Choice, Evidence, Policy};
use serde_json::{Value as Json, json};

/// What the model is told once, at `initialize`, about the format it will
/// receive. Kept to the rules a reader needs; the full spec is
/// `site/src/content/docs/docs/spec.md`.
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

/// Appended to [`PRIMER`] when the columnar adapter is on (`--columnar`).
pub const COLUMNAR_PRIMER: &str = "A columnar result (`columns` and `rows`) comes back as one line of JSON \
with every field of the original result except that `rows` is the string \"@records\", followed by the \
rows as the LDF table `@records`: one row per original row array, one column per entry of `columns`, \
`null` for SQL NULL.";

/// The value `rows` takes in the first line of a columnar frame.
pub const ROWS_MARK: &str = "@records";

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
    /// A number outside the value model's domain (an integer beyond u64,
    /// a decimal f64 cannot hold exactly): LDF would change it.
    OutOfDomain,
    /// Columnar, but the frame would not decode to the same result
    /// (duplicate column names, ragged rows, a value LDF cannot keep).
    NotLossless,
    /// A reason only the Claude Code hook has (`crate::hook`), as its code.
    Hook(&'static str),
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
            Kept::OutOfDomain => "out_of_domain",
            Kept::NotLossless => "not_lossless",
            Kept::Hook(code) => code,
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

/// [`convert_with`] without the columnar adapter.
pub fn convert(text: &str, min_bytes: usize, sel: Option<&Selector>) -> Result<String, Kept> {
    convert_with(text, min_bytes, sel, false)
}

/// Returns the LDF text for `text` when it is a JSON dataset worth
/// converting: a non-empty array of objects, or an object holding at least
/// one non-empty array of objects (which becomes a one-row root table with
/// child tables). With `columnar`, an object with `columns` and `rows`
/// becomes a columnar frame ([`columnar_frame`]) first. LDF must be shorter
/// than the JSON minified, so whitespace never counts as a saving.
/// Otherwise says why it stays as it is ([`Kept`]).
pub fn convert_with(
    text: &str,
    min_bytes: usize,
    sel: Option<&Selector>,
    columnar: bool,
) -> Result<String, Kept> {
    if text.len() < min_bytes {
        return Err(Kept::Small);
    }
    let trimmed = text.trim_start();
    if !(trimmed.starts_with('[') || trimmed.starts_with('{')) {
        return Err(Kept::NotJson);
    }
    let mut bytes = text.as_bytes().to_vec();
    let root = parse_json_value(&mut bytes, JsonEngine::SerdeJson).map_err(|_| Kept::NotJson)?;
    if !numbers_in_domain(text) {
        return Err(Kept::OutOfDomain);
    }
    let json_len = minify_json(text).map_or(text.len(), |m| m.len());
    if columnar && columnar_shape(&root).is_some() {
        return columnar_frame(&root, json_len, sel);
    }
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
    encode_records(&records, json_len, sel)
}

/// LDF for `records` when it is shorter than `json_len` bytes and, with a
/// selector, when the selector accepts it.
fn encode_records(
    records: &[Value],
    json_len: usize,
    sel: Option<&Selector>,
) -> Result<String, Kept> {
    let Some(sel) = sel else {
        let out = ldf::encode(records);
        return if out.len() < json_len {
            Ok(out)
        } else {
            Err(Kept::NotShorter)
        };
    };
    let (d, out) = ldf_select::select_ldf(
        records,
        &sel.model,
        &sel.evidence,
        &sel.policy,
        json_len as u64,
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

/// The column names and row arrays of a columnar result: an object with
/// exactly one `columns` (non-empty, unique strings) and one `rows`
/// (non-empty, every row an array as long as `columns`), as MotherDuck's
/// DuckDB server answers a query. Any other member is kept as it is.
pub fn columnar_shape(root: &Value) -> Option<(Vec<&str>, &[Value])> {
    let o = root.as_object()?;
    let one = |key: &str| {
        let mut it = o.iter().filter(|(k, _)| k == key);
        let first = it.next().map(|(_, v)| v);
        if it.next().is_some() { None } else { first }
    };
    let columns: Vec<&str> = one("columns")?
        .as_array()?
        .iter()
        .map(Value::as_str)
        .collect::<Option<_>>()?;
    let rows = one("rows")?.as_array()?;
    let unique = columns
        .iter()
        .enumerate()
        .all(|(i, c)| !columns[..i].contains(c));
    let ragged = rows
        .iter()
        .any(|r| r.as_array().is_none_or(|a| a.len() != columns.len()));
    (!columns.is_empty() && unique && !rows.is_empty() && !ragged).then_some((columns, rows))
}

/// A columnar result as a frame: line one is the result as compact JSON
/// with `rows` replaced by [`ROWS_MARK`], the rest is the LDF table of the
/// rows as records keyed by `columns`. Every value is present in every
/// record, so SQL NULL is `null` and an empty cell never appears. Returned
/// only when [`decode_columnar_frame`] gives back exactly `root` (same
/// members, order and number kinds) and the frame is shorter than
/// `json_len`. With a selector the result stays JSON: no model has been
/// graded on a frame, and the selector's sparse evidence has absent keys
/// where a frame has SQL NULLs.
pub fn columnar_frame(
    root: &Value,
    json_len: usize,
    sel: Option<&Selector>,
) -> Result<String, Kept> {
    let (columns, rows) = columnar_shape(root).ok_or(Kept::NotDataset)?;
    if let Some(sel) = sel {
        return Err(Kept::Selector(format!(
            "json for {} ({} rows): no evidence for columnar frames",
            sel.model,
            rows.len()
        )));
    }
    let records: Vec<Value> = rows
        .iter()
        .filter_map(Value::as_array)
        .map(|r| {
            Value::Object(
                columns
                    .iter()
                    .zip(r)
                    .map(|(c, v)| (c.to_string(), v.clone()))
                    .collect(),
            )
        })
        .collect();
    let Some(o) = root.as_object() else {
        return Err(Kept::NotDataset);
    };
    let head = Value::Object(
        o.iter()
            .map(|(k, v)| {
                let v = if k == "rows" {
                    Value::String(ROWS_MARK.into())
                } else {
                    v.clone()
                };
                (k.clone(), v)
            })
            .collect(),
    );
    let mut frame = String::new();
    ldf::text::push_json_value(&head, &mut frame);
    frame.push('\n');
    let room = json_len.saturating_sub(frame.len());
    let table = encode_records(&records, room, None)?;
    frame.push_str(&table);
    if decode_columnar_frame(&frame).ok().as_ref() != Some(root) {
        return Err(Kept::NotLossless);
    }
    Ok(frame)
}

/// The original result of a frame [`columnar_frame`] wrote.
pub fn decode_columnar_frame(frame: &str) -> Result<Value, String> {
    let (head, table) = frame
        .split_once('\n')
        .ok_or("no table after the first line")?;
    let mut bytes = head.as_bytes().to_vec();
    let head = parse_json_value(&mut bytes, JsonEngine::SerdeJson).map_err(|e| e.to_string())?;
    let Value::Object(members) = head else {
        return Err("first line is not a JSON object".into());
    };
    let columns: Vec<&str> = members
        .iter()
        .find(|(k, _)| k == "columns")
        .and_then(|(_, v)| v.as_array())
        .and_then(|a| a.iter().map(Value::as_str).collect())
        .ok_or("no columns")?;
    let records = ldf::decode(table).map_err(|e| e.to_string())?;
    let mut rows = Vec::with_capacity(records.len());
    for (i, r) in records.iter().enumerate() {
        let o = r.as_object().ok_or(format!("row {i} is not a record"))?;
        if o.len() != columns.len() {
            return Err(format!(
                "row {i} has {} values for {} columns",
                o.len(),
                columns.len()
            ));
        }
        let row = columns
            .iter()
            .map(|c| r.get(c).cloned().ok_or(format!("row {i} has no {c}")))
            .collect::<Result<Vec<_>, _>>()?;
        rows.push(Value::Array(row));
    }
    let mut rows = Some(Value::Array(rows));
    let mut out = Vec::with_capacity(members.len());
    for (k, v) in &members {
        let v = if k == "rows" && v.as_str() == Some(ROWS_MARK) {
            rows.take().ok_or("two rows members")?
        } else {
            v.clone()
        };
        out.push((k.clone(), v));
    }
    if rows.is_some() {
        return Err("no rows member".into());
    }
    Ok(Value::Object(out))
}

/// False when some number in the JSON `text` would not survive the value
/// model: an integer literal outside i64 and u64, or a decimal whose digits
/// the nearest f64 does not reproduce. Strings are skipped.
pub fn numbers_in_domain(text: &str) -> bool {
    let b = text.as_bytes();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'"' => match ldf::text::json_string_end(&text[i..]) {
                Some(n) => i += n,
                None => return true,
            },
            b'-' | b'0'..=b'9' => {
                let start = i;
                while i < b.len() && matches!(b[i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
                {
                    i += 1;
                }
                if !number_in_domain(&text[start..i]) {
                    return false;
                }
            }
            _ => i += 1,
        }
    }
    true
}

fn number_in_domain(lit: &str) -> bool {
    if !lit.contains(['.', 'e', 'E']) {
        return lit.parse::<i64>().is_ok() || lit.parse::<u64>().is_ok();
    }
    match lit.parse::<f64>() {
        Ok(f) if f.is_finite() => decimal_digits(lit) == decimal_digits(&format!("{f:e}")),
        _ => false,
    }
}

/// Sign, significant digits and decimal exponent of a decimal literal, so
/// `1.50`, `15e-1` and `1.5e0` compare equal. Zero of either sign is
/// `(false, "", 0)`.
fn decimal_digits(lit: &str) -> (bool, String, i64) {
    let (neg, lit) = match lit.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, lit),
    };
    let (mant, exp) = match lit.find(['e', 'E']) {
        Some(p) => (&lit[..p], lit[p + 1..].parse::<i64>().unwrap_or(0)),
        None => (lit, 0),
    };
    let (int, frac) = mant.split_once('.').unwrap_or((mant, ""));
    let all = format!("{int}{frac}");
    let lead = all.len() - all.trim_start_matches('0').len();
    let digits = all
        .trim_start_matches('0')
        .trim_end_matches('0')
        .to_string();
    if digits.is_empty() {
        return (false, digits, 0);
    }
    (neg, digits, exp + int.len() as i64 - lead as i64)
}

/// One text block of a `tools/call` result and what happened to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub in_bytes: usize,
    pub out_bytes: usize,
    /// `None` when the block was replaced by LDF.
    pub kept: Option<Kept>,
    /// Rows shown and rows in the result, when [`cap_rows`] cut it.
    pub rows: Option<(usize, usize)>,
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

    /// True when some block kept as JSON went out minified.
    pub fn minified(&self) -> bool {
        self.blocks
            .iter()
            .any(|b| b.kept.is_some() && b.out_bytes < b.in_bytes)
    }
}

/// `text` without the whitespace between JSON tokens, when that is
/// shorter. Every token is kept byte for byte, so numbers and escapes read
/// exactly as before (semantic equality). Call it only on text that parsed
/// as JSON.
pub fn minify_json(text: &str) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let (mut in_string, mut escaped) = (false, false);
    for c in text.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
        } else if matches!(c, ' ' | '\t' | '\n' | '\r') {
            continue;
        } else if c == '"' {
            in_string = true;
        }
        out.push(c);
    }
    (out.len() < text.len()).then_some(out)
}

/// How [`rewrite_tool_result`] treats a result.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rules {
    /// Text blocks shorter than this are left alone.
    pub min_bytes: usize,
    /// Convert `columns` + `rows` results too ([`columnar_frame`]).
    pub columnar: bool,
    /// Cut rows from a result whose text would still be longer than this
    /// many bytes ([`cap_rows`]), and say so in a note.
    pub max_bytes: Option<usize>,
}

/// Rewrites the text content blocks of a `tools/call` result in place: LDF
/// where [`convert_with`] gives it, otherwise JSON that parsed goes out
/// minified; with `rules.max_bytes`, rows are cut to fit and a note block
/// follows the cut block. Results flagged `isError`, blocks under
/// `min_bytes` and non-text blocks are left alone. `structuredContent` is
/// left alone too, except when it is exactly `{"result": S}` with `S` the
/// text of a rewritten block, which is how FastMCP wraps a tool that
/// returns a string: then `S` is replaced by the same rewritten text (and
/// the note, after a blank line). Claude Code shows the model
/// `structuredContent` when it is present, not the text block.
pub fn rewrite_tool_result(result: &mut Json, rules: &Rules, sel: Option<&Selector>) -> Rewritten {
    let start = Instant::now();
    let mut done = Rewritten::default();
    let is_error = result.get("isError").and_then(Json::as_bool) == Some(true);
    let wrapped: Option<String> = result
        .get("structuredContent")
        .and_then(Json::as_object)
        .filter(|o| o.len() == 1)
        .and_then(|o| o.get("result"))
        .and_then(Json::as_str)
        .map(str::to_string);
    let mut rewrapped: Option<String> = None;
    let mut notes: Vec<(usize, String)> = Vec::new();
    if let Some(content) = result.get_mut("content").and_then(Json::as_array_mut) {
        for (i, block) in content.iter_mut().enumerate() {
            if block.get("type").and_then(Json::as_str) != Some("text") {
                continue;
            }
            let Some(text) = block.get("text").and_then(Json::as_str) else {
                continue;
            };
            let mirrored = rewrapped.is_none() && wrapped.as_deref() == Some(text);
            let converted = if is_error {
                Err(Kept::IsError)
            } else {
                convert_with(text, rules.min_bytes, sel, rules.columnar)
            };
            let (mut out, kept) = match converted {
                Ok(ldf) => (Some(ldf), None),
                Err(why) => {
                    // Parsed as JSON but stays JSON: at least drop the
                    // pretty-printing.
                    let parsed = matches!(
                        why,
                        Kept::NotDataset
                            | Kept::NotShorter
                            | Kept::Selector(_)
                            | Kept::OutOfDomain
                            | Kept::NotLossless
                    );
                    (if parsed { minify_json(text) } else { None }, Some(why))
                }
            };
            let mut rows = None;
            let len = out.as_ref().map_or(text.len(), String::len);
            if let Some(max) = rules.max_bytes
                && !is_error
                && len > max
                && let Some(cut) = cap_rows(text, max, rules, sel)
            {
                notes.push((i, cut.note()));
                rows = Some((cut.shown, cut.total));
                out = Some(cut.text);
            }
            let in_bytes = text.len();
            if let Some(out) = out {
                if mirrored {
                    let note = notes
                        .last()
                        .filter(|(j, _)| *j == i)
                        .map(|(_, n)| n.as_str());
                    rewrapped = Some(match note {
                        Some(n) => format!("{out}\n\n{n}"),
                        None => out.clone(),
                    });
                }
                done.blocks.push(Block {
                    in_bytes,
                    out_bytes: out.len(),
                    kept,
                    rows,
                });
                block["text"] = Json::String(out);
            } else {
                done.blocks.push(Block {
                    in_bytes,
                    out_bytes: in_bytes,
                    kept,
                    rows,
                });
            }
        }
        for (i, note) in notes.into_iter().rev() {
            content.insert(i + 1, json!({"type": "text", "text": note}));
        }
    }
    if let Some(text) = rewrapped {
        result["structuredContent"]["result"] = Json::String(text);
    }
    done.micros = start.elapsed().as_micros() as u64;
    done
}

/// A result with rows cut by [`cap_rows`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cut {
    /// The first `shown` rows, rewritten as the rules say.
    pub text: String,
    pub shown: usize,
    pub total: usize,
    pub max_bytes: usize,
}

impl Cut {
    /// The note the model gets next to the cut result.
    pub fn note(&self) -> String {
        format!(
            "Result cut by ldf-mcp: showing the first {} of {} rows, to stay under {} bytes; \
             the other rows were not sent. Narrow the query with WHERE, aggregate with GROUP BY, \
             or page with LIMIT and OFFSET.",
            self.shown, self.total, self.max_bytes
        )
    }
}

/// Keeps the most leading rows of a JSON result whose rewritten text fits
/// in `max_bytes`: a top-level array of objects, the `rows` of a columnar
/// result, or the one member of an object that is an array of objects.
/// Every other member is kept as it is. `None` when the result has no such
/// rows, did not parse, or holds a number outside the value model (cutting
/// re-serializes the JSON).
pub fn cap_rows(
    text: &str,
    max_bytes: usize,
    rules: &Rules,
    sel: Option<&Selector>,
) -> Option<Cut> {
    let mut bytes = text.as_bytes().to_vec();
    let root = parse_json_value(&mut bytes, JsonEngine::SerdeJson).ok()?;
    if !numbers_in_domain(text) {
        return None;
    }
    let records = |v: &Value| matches!(v, Value::Array(a) if !a.is_empty() && a.iter().all(|r| r.as_object().is_some()));
    // `None` is the root array, `Some(i)` the object member holding the rows.
    let place: Option<usize> = match &root {
        Value::Array(_) if records(&root) => None,
        Value::Object(o) => {
            let at = if columnar_shape(&root).is_some() {
                o.iter().position(|(k, _)| k == "rows")
            } else {
                let found: Vec<usize> = (0..o.len()).filter(|&i| records(&o[i].1)).collect();
                (found.len() == 1).then(|| found[0])
            };
            Some(at?)
        }
        _ => return None,
    };
    let rows: &Vec<Value> = match (place, &root) {
        (None, Value::Array(a)) => a,
        (Some(i), Value::Object(o)) => o[i].1.as_array()?,
        _ => return None,
    };
    let total = rows.len();
    let render = |k: usize| {
        let cut = Value::Array(rows[..k].to_vec());
        let v = match (place, &root) {
            (Some(i), Value::Object(o)) => {
                let mut o = o.clone();
                o[i].1 = cut;
                Value::Object(o)
            }
            _ => cut,
        };
        let mut s = String::new();
        ldf::text::push_json_value(&v, &mut s);
        convert_with(&s, rules.min_bytes, sel, rules.columnar).unwrap_or(s)
    };
    // The largest k whose text fits; 0 when not even one row does. Rows
    // are of similar size, so start from twice the share that fits and
    // widen only if even that fits: each step encodes k rows, not all.
    let guess = (total as u128 * max_bytes as u128 / text.len().max(1) as u128) as usize;
    let mut hi = (2 * guess + 16).min(total - 1);
    if render(hi).len() <= max_bytes {
        hi = total - 1;
    }
    let mut lo = 0usize;
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        if render(mid).len() <= max_bytes {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    Some(Cut {
        text: render(lo),
        shown: lo,
        total,
        max_bytes,
    })
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

/// Appends [`PRIMER`] to the `instructions` of an `initialize` result, and
/// [`COLUMNAR_PRIMER`] after it with `columnar`.
pub fn rewrite_initialize(result: &mut Json, columnar: bool) {
    let Some(result) = result.as_object_mut() else {
        return;
    };
    let existing = result
        .get("instructions")
        .and_then(Json::as_str)
        .unwrap_or("")
        .trim_end()
        .to_string();
    let mut primer = PRIMER.to_string();
    if columnar {
        primer = primer + "\n\n" + COLUMNAR_PRIMER;
    }
    let text = if existing.is_empty() {
        primer
    } else {
        existing + "\n\n" + &primer
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
    fn minify_keeps_strings_and_numbers() {
        let text = "{\n  \"a b\": \"x  \\\" y\\n\",\n  \"n\": [1.50, -0e0,\t2]\r\n}";
        let mini = minify_json(text).unwrap();
        assert_eq!(mini, r#"{"a b":"x  \" y\n","n":[1.50,-0e0,2]}"#);
        let a: Json = serde_json::from_str(text).unwrap();
        let b: Json = serde_json::from_str(&mini).unwrap();
        assert_eq!(a, b);
        assert_eq!(minify_json(r#"{"a":1}"#), None, "already minified");
    }

    #[test]
    fn json_that_stays_json_goes_out_minified() {
        // Columnar, as MotherDuck's DuckDB server answers: not records.
        let pretty =
            "{\n  \"columns\": [\n    \"id\"\n  ],\n  \"rows\": [\n    [\n      1\n    ]\n  ]\n}";
        let mut result = json!({"content": [{"type": "text", "text": pretty}]});
        let done = rewrite_tool_result(&mut result, &Rules::default(), None);
        assert!(!done.converted() && done.minified());
        assert_eq!(
            result["content"][0]["text"],
            r#"{"columns":["id"],"rows":[[1]]}"#
        );

        // Errors, small blocks and text that is not JSON stay byte for byte.
        let mut err = json!({"isError": true, "content": [{"type": "text", "text": pretty}]});
        assert!(!rewrite_tool_result(&mut err, &Rules::default(), None).minified());
        assert_eq!(err["content"][0]["text"], pretty);
        let mut small = json!({"content": [{"type": "text", "text": pretty}]});
        assert!(
            !rewrite_tool_result(
                &mut small,
                &Rules {
                    min_bytes: 10_000,
                    columnar: false,
                    ..Rules::default()
                },
                None
            )
            .minified()
        );
        let mut prose = json!({"content": [{"type": "text", "text": "{ not json }"}]});
        assert!(!rewrite_tool_result(&mut prose, &Rules::default(), None).minified());
        assert_eq!(prose["content"][0]["text"], "{ not json }");
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
        let done = rewrite_tool_result(&mut ok, &Rules::default(), None);
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
        let done = rewrite_tool_result(&mut err, &Rules::default(), None);
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
        rewrite_initialize(&mut init, false);
        let s = init["instructions"].as_str().unwrap();
        assert!(s.starts_with("Be nice.\n\nTool results"));
        let mut bare = json!({"protocolVersion": "2025-06-18"});
        rewrite_initialize(&mut bare, false);
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

    fn parse(text: &str) -> Value {
        let mut b = text.as_bytes().to_vec();
        parse_json_value(&mut b, JsonEngine::SerdeJson).unwrap()
    }

    /// MotherDuck's envelope, pretty-printed as its server sends it.
    fn duck(rows: &[&str], extra: &str) -> String {
        let rows: Vec<String> = rows.iter().map(|r| format!("[{r}]")).collect();
        // `Value` keeps member order; serde_json's map would sort it.
        let v = parse(&format!(
            r#"{{"success":true,"columns":["id","name","score","note","tags","pt","day"],"columnTypes":["BIGINT","VARCHAR","DOUBLE","VARCHAR","INTEGER[]","STRUCT(x INTEGER, y INTEGER)","DATE"],"rows":[{}],"rowCount":{}{extra}}}"#,
            rows.join(","),
            rows.len()
        ));
        serde_json::to_string_pretty(&v).unwrap()
    }

    fn duck_rows(n: usize) -> Vec<String> {
        (0..n)
            .map(|i| {
                let note = if i % 3 == 0 {
                    "null".to_string()
                } else {
                    format!("\"n, {i}\"")
                };
                format!(
                    r#"{i},"user {i}",{}.25,{note},[1,{i}],{{"x":{i},"y":-1}},"2020-01-0{}""#,
                    i % 7,
                    1 + i % 9
                )
            })
            .collect()
    }

    #[test]
    fn columnar_result_round_trips_exactly() {
        let rows = duck_rows(40);
        let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
        for extra in [
            "",
            r#","truncated":true,"warning":"Results limited to 40 rows.""#,
        ] {
            let text = duck(&rows, extra);
            let frame = convert_with(&text, 0, None, true).unwrap();
            let (head, table) = frame.split_once('\n').unwrap();
            assert!(
                head.starts_with(r#"{"success":true,"columns":["id","#),
                "{head}"
            );
            assert!(
                head.contains(r#""rows":"@records","rowCount":40"#),
                "{head}"
            );
            assert!(table.starts_with("@records id:i,name:s,score:f,note:s?,tags:j,pt.x:i,pt.y:i,day:s\n0,user 0,0.25,null,[1,0],0,-1,2020-01-01\n"), "{table}");
            assert_eq!(decode_columnar_frame(&frame).unwrap(), parse(&text));
            // Shorter than the JSON minified, not only than the pretty text.
            assert!(frame.len() < minify_json(&text).unwrap().len());
        }
        // Off by default: the same text is not a dataset, so it is minified.
        assert_eq!(convert(&duck(&rows, ""), 0, None), Err(Kept::NotDataset));
    }

    #[test]
    fn columnar_refuses_what_it_cannot_keep() {
        let ok = duck_rows(30);
        let ok: Vec<&str> = ok.iter().map(String::as_str).collect();
        let base = duck(&ok, "");
        // Duplicate column names: `SELECT 1 AS h, 2 AS h` is legal SQL.
        let dup = base.replacen("\"name\"", "\"id\"", 1);
        assert_eq!(convert_with(&dup, 0, None, true), Err(Kept::NotDataset));
        // A ragged row.
        let ragged = base.replacen("\"2020-01-01\"\n", "\"2020-01-01\", 9\n", 1);
        assert_ne!(ragged, base);
        assert_eq!(convert_with(&ragged, 0, None, true), Err(Kept::NotDataset));
        // HUGEINT beyond u64, as MotherDuck sends it: the value model would
        // turn it into a float, so nothing converts.
        let huge = base.replacen("\"rowCount\": 30", "\"rowCount\": 100000000000000000000", 1);
        assert_eq!(convert_with(&huge, 0, None, true), Err(Kept::OutOfDomain));
        assert_eq!(
            convert(r#"[{"a":100000000000000000000},{"a":1},{"a":2}]"#, 0, None),
            Err(Kept::OutOfDomain)
        );
        // Empty result and non-string column names.
        assert_eq!(
            columnar_shape(&parse(r#"{"columns":["a"],"rows":[]}"#)),
            None
        );
        assert_eq!(
            columnar_shape(&parse(r#"{"columns":[1],"rows":[[1]]}"#)),
            None
        );
        // An error envelope has no rows.
        let err = r#"{"success": false, "error": "Catalog Error: Table with name nope does not exist!", "errorType": "CatalogException"}"#;
        assert_eq!(convert_with(err, 0, None, true), Err(Kept::NotDataset));
    }

    #[test]
    fn columnar_nulls_that_make_ldf_larger_stay_json() {
        // Ten mostly-null columns, as a sparse table comes back from SQL:
        // the default layout moves them into `~` as "key":null pairs.
        let cols: Vec<String> = (0..10).map(|i| format!("\"c{i}\"")).collect();
        let rows: Vec<String> = (0..50)
            .map(|r| {
                let cells: Vec<String> = (0..10)
                    .map(|c| {
                        if (r + c) % 10 == 0 {
                            format!("{r}")
                        } else {
                            "null".into()
                        }
                    })
                    .collect();
                format!("[{r},{}]", cells.join(","))
            })
            .collect();
        let text = format!(
            r#"{{"columns":["id",{}],"rows":[{}]}}"#,
            cols.join(","),
            rows.join(",")
        );
        assert_eq!(convert_with(&text, 0, None, true), Err(Kept::NotShorter));
        let mut result = json!({"content": [{"type": "text", "text": text}]});
        let done = rewrite_tool_result(
            &mut result,
            &Rules {
                min_bytes: 0,
                columnar: true,
                ..Rules::default()
            },
            None,
        );
        assert_eq!(done.blocks[0].kept, Some(Kept::NotShorter));
    }

    #[test]
    fn columnar_in_a_tool_result_and_primer() {
        let rows = duck_rows(30);
        let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
        let text = duck(&rows, "");
        let structured = json!({"result": text});
        let mut result = json!({"content": [{"type": "text", "text": text}], "isError": false, "structuredContent": structured});
        assert!(
            rewrite_tool_result(
                &mut result,
                &Rules {
                    min_bytes: 0,
                    columnar: true,
                    ..Rules::default()
                },
                None
            )
            .converted()
        );
        let frame = result["content"][0]["text"].as_str().unwrap();
        assert_eq!(decode_columnar_frame(frame).unwrap(), parse(&text));
        // FastMCP's wrapped string follows the text, so both copies agree.
        assert_eq!(result["structuredContent"], json!({"result": frame}));

        // Any other structuredContent is left alone, as is a wrapped string
        // that is not the text.
        for other in [
            json!({"result": text, "n": 1}),
            json!({"result": "other"}),
            json!({"rows": [1]}),
        ] {
            let mut result =
                json!({"content": [{"type": "text", "text": text}], "structuredContent": other});
            rewrite_tool_result(
                &mut result,
                &Rules {
                    min_bytes: 0,
                    columnar: true,
                    ..Rules::default()
                },
                None,
            );
            assert_eq!(result["structuredContent"], other);
        }
        // Minified JSON follows too.
        let pretty = "{\n  \"a\": [\n    1\n  ]\n}";
        let mut result = json!({"content": [{"type": "text", "text": pretty}], "structuredContent": {"result": pretty}});
        rewrite_tool_result(&mut result, &Rules::default(), None);
        assert_eq!(result["structuredContent"]["result"], r#"{"a":[1]}"#);

        let sel = Selector::new("claude-opus-5-5", None);
        let Err(Kept::Selector(why)) = convert_with(&text, 0, Some(&sel), true) else {
            panic!("selector reason expected");
        };
        assert!(why.contains("no evidence for columnar frames"), "{why}");

        let mut init = json!({});
        rewrite_initialize(&mut init, true);
        assert!(
            init["instructions"]
                .as_str()
                .unwrap()
                .ends_with(COLUMNAR_PRIMER)
        );
    }

    #[test]
    fn domain_check_reads_literals_not_values() {
        for ok in [
            "1",
            "-0",
            "18446744073709551615",
            "-9223372036854775808",
            "1.50",
            "0.1",
            "1e21",
            "-2.5E-3",
            "123456789.125",
            "0.30000000000000004",
        ] {
            assert!(number_in_domain(ok), "{ok}");
        }
        for bad in [
            "18446744073709551616",
            "-9223372036854775809",
            "0.1000000000000000000001",
            "1e400",
            "9007199254740993.0",
        ] {
            assert!(!number_in_domain(bad), "{bad}");
        }
        assert!(numbers_in_domain(
            r#"{"s":"100000000000000000000","n":[1,2.5]}"#
        ));
        assert!(!numbers_in_domain(
            r#"{"s":"\"","n":100000000000000000000}"#
        ));
    }

    fn capped(max: usize, columnar: bool) -> Rules {
        Rules {
            columnar,
            max_bytes: Some(max),
            ..Rules::default()
        }
    }

    #[test]
    fn cap_keeps_leading_rows_and_says_so() {
        let text = rows(200);
        let mut result = json!({"content": [{"type": "text", "text": text}]});
        let done = rewrite_tool_result(&mut result, &capped(2000, false), None);
        let (shown, total) = done.blocks[0].rows.unwrap();
        assert_eq!(total, 200);
        assert!(shown > 10 && shown < 200, "{shown}");
        let content = result["content"].as_array().unwrap();
        assert_eq!(content.len(), 2, "note block after the cut block");
        let out = content[0]["text"].as_str().unwrap();
        assert!(out.len() <= 2000);
        let back = ldf::decode(out).unwrap();
        let mut b = text.as_bytes().to_vec();
        let Value::Array(all) = parse_json_value(&mut b, JsonEngine::SerdeJson).unwrap() else {
            panic!()
        };
        assert_eq!(back, all[..shown].to_vec(), "a prefix, values unchanged");
        let note = content[1]["text"].as_str().unwrap();
        assert!(
            note.contains(&format!("first {shown} of 200 rows")),
            "{note}"
        );
        // The largest prefix that fits: one more row does not.
        let mut one_more = String::new();
        ldf::text::push_json_value(&Value::Array(all[..=shown].to_vec()), &mut one_more);
        assert!(convert_with(&one_more, 0, None, false).unwrap().len() > 2000);

        // Not even one row fits: zero rows, still a note.
        let mut tiny = json!({"content": [{"type": "text", "text": text}]});
        let done = rewrite_tool_result(&mut tiny, &capped(10, false), None);
        assert_eq!(done.blocks[0].rows, Some((0, 200)));
    }

    #[test]
    fn cap_on_a_columnar_result_mirrors_the_wrapped_copy() {
        let rows = duck_rows(200);
        let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
        let text = duck(&rows, "");
        let mut result = json!({"content": [{"type": "text", "text": text}], "structuredContent": {"result": text}});
        let done = rewrite_tool_result(&mut result, &capped(3000, true), None);
        let (shown, _) = done.blocks[0].rows.unwrap();
        let out = result["content"][0]["text"].as_str().unwrap().to_string();
        let note = result["content"][1]["text"].as_str().unwrap().to_string();
        assert!(out.len() <= 3000);
        assert_eq!(
            result["structuredContent"]["result"],
            format!("{out}\n\n{note}")
        );
        let back = decode_columnar_frame(&out).unwrap();
        assert_eq!(
            back.get("rows").and_then(Value::as_array).map(Vec::len),
            Some(shown)
        );
        // Every other member as the server sent it, rowCount included.
        assert_eq!(
            back.get("rowCount")
                .and_then(Value::as_number)
                .and_then(|n| n.as_i128()),
            Some(200)
        );
        assert_eq!(back.get("columnTypes"), parse(&text).get("columnTypes"));
    }

    #[test]
    fn cap_leaves_errors_small_and_non_tabular_results_alone() {
        let text = rows(200);
        let mut err = json!({"isError": true, "content": [{"type": "text", "text": text}]});
        let done = rewrite_tool_result(&mut err, &capped(100, false), None);
        assert_eq!(done.blocks[0].rows, None);
        assert_eq!(err["content"].as_array().unwrap().len(), 1);
        assert_eq!(err["content"][0]["text"], text);

        let long = format!(r#"{{"report":"{}"}}"#, "x".repeat(5000));
        let mut one = json!({"content": [{"type": "text", "text": long}]});
        let done = rewrite_tool_result(&mut one, &capped(100, false), None);
        assert_eq!(done.blocks[0].rows, None, "nothing to cut");
        assert_eq!(one["content"].as_array().unwrap().len(), 1);

        let mut fits = json!({"content": [{"type": "text", "text": text}]});
        let done = rewrite_tool_result(&mut fits, &capped(1_000_000, false), None);
        assert_eq!(done.blocks[0].rows, None);
        assert_eq!(fits["content"].as_array().unwrap().len(), 1);

        // Two arrays of rows in one object: no way to tell which to cut.
        let two = format!(r#"{{"a":{},"b":{}}}"#, rows(50), rows(50));
        let mut both = json!({"content": [{"type": "text", "text": two}]});
        assert_eq!(
            rewrite_tool_result(&mut both, &capped(100, false), None).blocks[0].rows,
            None
        );
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
