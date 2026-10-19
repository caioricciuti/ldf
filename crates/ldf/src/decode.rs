//! Decoder (spec sections 3, 4, 7).

use std::collections::HashMap;
use std::fmt;

use ldf_data::{Number, Object, Value};

use crate::ROOT;
use crate::plan::ColType;
use crate::text::json_string_end;

#[derive(Debug, Clone)]
pub struct DecodeError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}
impl std::error::Error for DecodeError {}

type Result<T> = std::result::Result<T, DecodeError>;

fn err<T>(line: usize, message: impl Into<String>) -> Result<T> {
    Err(DecodeError {
        line,
        message: message.into(),
    })
}

#[derive(Debug, Clone)]
struct HeaderCol {
    path: Vec<String>,
    ty: ColType,
    /// The bare `~` column: a JSON object of extra keys merged into the row.
    rest: bool,
    /// Element keys of a hoisted `[k1,k2]` column.
    keys: Vec<String>,
}

#[derive(Debug)]
struct Table {
    name: String,
    cols: Vec<HeaderCol>,
    /// Each row: one slot per column. `None` = absent.
    rows: Vec<Vec<Option<Value>>>,
    first_line: usize,
}

/// Decodes a document into its records.
pub fn decode(text: &str) -> Result<Vec<Value>> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut tables: Vec<Table> = Vec::new();
    for (i, raw) in text.split('\n').enumerate() {
        let line_no = i + 1;
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix('@') {
            let (name, cols) = parse_header(rest, line_no)?;
            tables.push(Table {
                name,
                cols,
                rows: Vec::new(),
                first_line: line_no,
            });
            continue;
        }
        let Some(table) = tables.last_mut() else {
            return err(line_no, "row before any header");
        };
        let row = parse_row(line, &table.cols, line_no)?;
        table.rows.push(row);
    }
    assemble(tables)
}

fn parse_header(rest: &str, line_no: usize) -> Result<(String, Vec<HeaderCol>)> {
    let (name, cols_text) = match rest.split_once(' ') {
        Some((n, c)) => (n, c),
        None => (rest, ""),
    };
    if name.is_empty() {
        return err(line_no, "header without a table name");
    }
    let mut cols = Vec::new();
    for entry in split_header_entries(cols_text) {
        let entry = entry.trim_end();
        if entry.is_empty() {
            return err(line_no, "empty column in header");
        }
        // The type code is after the last ':' outside quotes.
        let colon = find_last_unquoted(entry, ':').ok_or_else(|| DecodeError {
            line: line_no,
            message: format!("column {entry:?} has no type"),
        })?;
        let (path_text, ty_text) = (&entry[..colon], &entry[colon + 1..]);
        let ty_text = ty_text.strip_suffix('?').unwrap_or(ty_text);
        let mut keys = Vec::new();
        let ty = if let Some(list) = ty_text.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
            for k in split_header_entries(list) {
                match parse_path(k, line_no)?.as_slice() {
                    [seg] => keys.push(seg.clone()),
                    _ => return err(line_no, format!("bad key {k:?} in column {entry:?}")),
                }
            }
            if keys.is_empty() {
                return err(line_no, format!("empty key list in column {entry:?}"));
            }
            ColType::Hoisted
        } else {
            let mut chars = ty_text.chars();
            match (chars.next().and_then(ColType::from_code), chars.next()) {
                (Some(t), None) => t,
                _ => {
                    return err(
                        line_no,
                        format!("unknown type {ty_text:?} in column {entry:?}"),
                    );
                }
            }
        };
        let rest = path_text == "~";
        if rest && ty != ColType::Json {
            return err(line_no, "the ~ column must have type j");
        }
        cols.push(HeaderCol {
            path: parse_path(path_text, line_no)?,
            ty,
            rest,
            keys,
        });
    }
    if cols.is_empty() {
        return err(line_no, "header without columns");
    }
    Ok((name.to_string(), cols))
}

/// Splits header entries on commas outside quotes and outside the key
/// list of a hoisted column.
fn split_header_entries(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;
    let mut depth = 0usize;
    let b = s.as_bytes();
    while i < b.len() {
        match b[i] {
            b'"' => {
                i += json_string_end(&s[i..]).unwrap_or(1);
            }
            b'[' => {
                depth += 1;
                i += 1;
            }
            b']' => {
                depth = depth.saturating_sub(1);
                i += 1;
            }
            b',' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
                i += 1;
            }
            _ => i += 1,
        }
    }
    if start < s.len() || !s.is_empty() {
        out.push(&s[start..]);
    }
    out
}

fn find_last_unquoted(s: &str, needle: char) -> Option<usize> {
    let mut last = None;
    let mut i = 0;
    let b = s.as_bytes();
    while i < b.len() {
        if b[i] == b'"' {
            i += json_string_end(&s[i..]).unwrap_or(1);
        } else {
            if b[i] == needle as u8 {
                last = Some(i);
            }
            i += 1;
        }
    }
    last
}

fn parse_path(s: &str, line_no: usize) -> Result<Vec<String>> {
    let mut segs = Vec::new();
    let mut i = 0;
    let b = s.as_bytes();
    while i <= b.len() {
        if i < b.len() && b[i] == b'"' {
            let end = json_string_end(&s[i..]).ok_or_else(|| DecodeError {
                line: line_no,
                message: format!("unterminated quoted segment in {s:?}"),
            })?;
            let seg: String = serde_json::from_str(&s[i..i + end]).map_err(|e| DecodeError {
                line: line_no,
                message: format!("bad quoted segment: {e}"),
            })?;
            segs.push(seg);
            i += end;
            if i < b.len() && b[i] != b'.' {
                return err(
                    line_no,
                    format!("expected '.' after quoted segment in {s:?}"),
                );
            }
            i += 1;
        } else {
            let start = i;
            while i < b.len() && b[i] != b'.' {
                i += 1;
            }
            let seg = &s[start..i];
            if seg.is_empty() {
                return err(line_no, format!("empty path segment in {s:?}"));
            }
            segs.push(seg.to_string());
            i += 1;
        }
    }
    Ok(segs)
}

fn parse_row(line: &str, cols: &[HeaderCol], line_no: usize) -> Result<Vec<Option<Value>>> {
    let mut out = Vec::with_capacity(cols.len());
    let mut cur = 0usize;
    for (ci, col) in cols.iter().enumerate() {
        let rest = &line[cur..];
        let (cell, used) = if rest.is_empty() || rest.starts_with(',') {
            (None, 0)
        } else if matches!(col.ty, ColType::Json | ColType::Hoisted)
            && !rest.starts_with("null,")
            && rest != "null"
        {
            let mut it = serde_json::Deserializer::from_str(rest).into_iter::<Value>();
            let v = it
                .next()
                .ok_or_else(|| DecodeError {
                    line: line_no,
                    message: "expected a JSON cell".into(),
                })?
                .map_err(|e| DecodeError {
                    line: line_no,
                    message: format!("bad JSON cell: {e}"),
                })?;
            let v = if col.ty == ColType::Hoisted {
                unhoist(v, &col.keys, line_no)?
            } else {
                v
            };
            (Some(v), it.byte_offset())
        } else if rest.starts_with('"') {
            let end = json_string_end(rest).ok_or_else(|| DecodeError {
                line: line_no,
                message: "unterminated quoted cell".into(),
            })?;
            let s: String = serde_json::from_str(&rest[..end]).map_err(|e| DecodeError {
                line: line_no,
                message: format!("bad quoted cell: {e}"),
            })?;
            (Some(Value::String(s)), end)
        } else {
            let end = rest.find(',').unwrap_or(rest.len());
            let tok = &rest[..end];
            (Some(typed(tok, col.ty, line_no)?), end)
        };
        cur += used;
        if ci + 1 < cols.len() {
            if line[cur..].starts_with(',') {
                cur += 1;
            } else {
                return err(line_no, format!("expected ',' after column {}", ci + 1));
            }
        }
        out.push(cell);
    }
    if cur != line.len() {
        return err(
            line_no,
            format!(
                "{} cells expected, extra content {:?}",
                cols.len(),
                &line[cur..]
            ),
        );
    }
    Ok(out)
}

/// A hoisted cell back to an array of objects: each positional array
/// zipped with the column's keys.
fn unhoist(v: Value, keys: &[String], line_no: usize) -> Result<Value> {
    let Value::Array(rows) = v else {
        return err(line_no, "hoisted cell is not an array");
    };
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let Value::Array(vals) = r else {
            return err(line_no, "hoisted element is not an array");
        };
        if vals.len() != keys.len() {
            return err(
                line_no,
                format!(
                    "hoisted element has {} values for {} keys",
                    vals.len(),
                    keys.len()
                ),
            );
        }
        out.push(Value::Object(keys.iter().cloned().zip(vals).collect()));
    }
    Ok(Value::Array(out))
}

fn typed(tok: &str, ty: ColType, line_no: usize) -> Result<Value> {
    if tok == "null" {
        return Ok(Value::Null);
    }
    Ok(match ty {
        ColType::Int => match tok.parse::<i64>() {
            Ok(v) => Value::from(v),
            Err(_) => match tok.parse::<u64>() {
                Ok(v) => Value::from(v),
                Err(_) => return err(line_no, format!("bad integer {tok:?}")),
            },
        },
        ColType::Float => match tok.parse::<f64>() {
            Ok(v) if tok.chars().any(|c| c.is_ascii_digit()) => Value::Number(Number::F64(v)),
            _ => return err(line_no, format!("bad float {tok:?}")),
        },
        ColType::Bool => match tok {
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            _ => return err(line_no, format!("bad boolean {tok:?}")),
        },
        ColType::Str => Value::String(tok.to_string()),
        ColType::Table => match tok.parse::<u64>() {
            Ok(n) => Value::from(n),
            Err(_) => return err(line_no, format!("bad child count {tok:?}")),
        },
        ColType::Json | ColType::Hoisted => {
            return err(line_no, "unreachable: json handled by caller");
        }
    })
}

// ---- assembly -------------------------------------------------------------

/// Insert `value` at a dotted path, creating objects along the way.
///
/// `new_top` says the caller knows the first segment is not in `obj` yet
/// (no earlier column of the row shares it), so the key is pushed without a
/// scan; that keeps a wide flat row O(columns) instead of O(columns^2).
/// Otherwise the last entry is tried first, since flattened sub-fields of
/// one object are adjacent in the header, before a full scan.
fn insert_path(obj: &mut Object, path: &[String], value: Value, new_top: bool) {
    let (first, rest) = path.split_first().expect("non-empty path");
    let found = if new_top {
        None
    } else if obj.last().is_some_and(|(k, _)| k == first) {
        Some(obj.len() - 1)
    } else {
        obj.iter().position(|(k, _)| k == first)
    };
    if rest.is_empty() {
        match found {
            Some(i) => obj[i].1 = value,
            None => obj.push((first.clone(), value)),
        }
        return;
    }
    let i = match found {
        Some(i) => i,
        None => {
            obj.push((first.clone(), Value::Object(Vec::new())));
            obj.len() - 1
        }
    };
    if !matches!(obj[i].1, Value::Object(_)) {
        obj[i].1 = Value::Object(Vec::new());
    }
    if let Value::Object(child) = &mut obj[i].1 {
        insert_path(child, rest, value, false);
    }
}

fn assemble(tables: Vec<Table>) -> Result<Vec<Value>> {
    // Build every table's rows into objects (child arrays as placeholders
    // holding the declared count), then attach children in reverse order.
    struct Built {
        name: String,
        objs: Vec<Value>,
        /// Per row, the Table columns: (path, declared count).
        pending: Vec<Vec<(Vec<String>, u64)>>,
        join_key: Option<HeaderCol>,
        join_values: Vec<Option<Value>>,
        first_line: usize,
    }
    let mut built: Vec<Built> = Vec::new();
    for t in tables {
        let is_root = t.name == ROOT;
        let scalar_rows =
            t.cols.len() == 1 && t.cols[0].path == ["_"] && t.cols[0].ty == ColType::Json;
        let (join_key, data_cols): (Option<HeaderCol>, &[HeaderCol]) = if is_root {
            (None, &t.cols[..])
        } else {
            (Some(t.cols[0].clone()), &t.cols[1..])
        };
        let mut objs = Vec::with_capacity(t.rows.len());
        let mut pending = Vec::with_capacity(t.rows.len());
        let mut join_values = Vec::with_capacity(t.rows.len());
        // A column opens a new top-level key when no earlier column shares
        // its first path segment; such keys are pushed without a lookup.
        let new_top: Vec<bool> = data_cols
            .iter()
            .enumerate()
            .map(|(i, c)| !data_cols[..i].iter().any(|d| d.path[0] == c.path[0]))
            .collect();
        for row in t.rows {
            let (jv, cells) = if is_root {
                (None, &row[..])
            } else {
                (row[0].clone(), &row[1..])
            };
            join_values.push(jv);
            if scalar_rows {
                objs.push(cells[0].clone().unwrap_or(Value::Null));
                pending.push(Vec::new());
                continue;
            }
            let mut obj = Object::with_capacity(data_cols.len());
            let mut pend = Vec::new();
            for ((col, cell), fresh) in data_cols.iter().zip(cells).zip(&new_top) {
                let Some(v) = cell else { continue };
                if col.rest {
                    match v {
                        Value::Object(pairs) => obj.extend(pairs.iter().cloned()),
                        Value::Null => {}
                        _ => return err(t.first_line, "~ cell is not a JSON object"),
                    }
                    continue;
                }
                if col.ty == ColType::Table && !v.is_null() {
                    let n = v.as_number().map(|n| n.as_f64() as u64).unwrap_or(0);
                    pend.push((col.path.clone(), n));
                    insert_path(&mut obj, &col.path, Value::Array(Vec::new()), *fresh);
                } else {
                    insert_path(&mut obj, &col.path, v.clone(), *fresh);
                }
            }
            objs.push(Value::Object(obj));
            pending.push(pend);
        }
        built.push(Built {
            name: t.name,
            objs,
            pending,
            join_key,
            join_values,
            first_line: t.first_line,
        });
    }

    // Attach children, deepest first. A table name may appear several times
    // (a block encoder writes one child table per block); all its rows are
    // one table, so each name is attached once, from all its occurrences.
    let names: Vec<String> = built.iter().map(|b| b.name.clone()).collect();
    let mut order: Vec<usize> = (0..built.len())
        .filter(|&i| built[i].name != ROOT && !names[..i].contains(&names[i]))
        .collect();
    order.sort_by_key(|&i| std::cmp::Reverse((names[i].matches('.').count(), i)));
    for ci in order {
        let child_name = built[ci].name.clone();
        let occurrences: Vec<usize> = (0..built.len())
            .filter(|&i| built[i].name == child_name)
            .collect();
        // Parent table: the longest proper prefix of the child's name that
        // names a table.
        let parent_name = {
            let mut found = None;
            let mut prefix = child_name.as_str();
            while let Some(pos) = prefix.rfind('.') {
                prefix = &prefix[..pos];
                if names.iter().any(|n| n == prefix) {
                    found = Some(prefix.to_string());
                    break;
                }
            }
            found.ok_or_else(|| DecodeError {
                line: built[ci].first_line,
                message: format!("no parent table for {child_name:?}"),
            })?
        };
        let field_path =
            crate_parse_field_path(&child_name[parent_name.len() + 1..], built[ci].first_line)?;
        let join_key = built[ci].join_key.clone().expect("child has join column");
        let ordinal = join_key.path == ["_"];
        // Group child rows by join value once (O(children)), keyed by the
        // JSON text of the join value so 1 and 1.0 land in the same bucket.
        let mut groups: HashMap<String, Vec<Value>> = HashMap::new();
        for &oi in &occurrences {
            let child = &mut built[oi];
            let objs = std::mem::take(&mut child.objs);
            let joins = std::mem::take(&mut child.join_values);
            for (jv, obj) in joins.into_iter().zip(objs) {
                let key = match jv {
                    Some(Value::Number(n)) => n.canonical_text().unwrap_or_default(),
                    Some(v) => v.to_string(),
                    None => String::new(),
                };
                groups.entry(key).or_default().push(obj);
            }
        }
        // Parent rows are the concatenation of all tables with that name.
        let mut parent_rows: Vec<(usize, usize)> = Vec::new(); // (table index, row index)
        for (ti, b) in built.iter().enumerate() {
            if b.name == parent_name {
                for ri in 0..b.objs.len() {
                    parent_rows.push((ti, ri));
                }
            }
        }
        for (pos, (ti, ri)) in parent_rows.iter().enumerate() {
            let want = built[*ti].pending[*ri]
                .iter()
                .find(|(p, _)| *p == field_path)
                .map(|(_, n)| *n);
            let Some(count) = want else { continue };
            let key = if ordinal {
                (pos + 1).to_string()
            } else {
                match &built[*ti].objs[*ri] {
                    Value::Object(o) => match crate::plan::lookup(o, &join_key.path) {
                        Some(Value::Number(n)) => n.canonical_text().unwrap_or_default(),
                        Some(v) => v.to_string(),
                        None => String::new(),
                    },
                    _ => String::new(),
                }
            };
            let items = groups.remove(&key).unwrap_or_default();
            if items.len() as u64 != count {
                return err(
                    built[ci].first_line,
                    format!(
                        "{child_name}: parent row {} declares {count} children, found {}",
                        pos + 1,
                        items.len()
                    ),
                );
            }
            if let Value::Object(o) = &mut built[*ti].objs[*ri] {
                insert_path(o, &field_path, Value::Array(items), false);
            }
        }
        if let Some((k, v)) = groups.iter().next() {
            return err(
                built[ci].first_line,
                format!(
                    "{child_name}: {} child rows with join value {k:?} have no parent",
                    v.len()
                ),
            );
        }
    }
    let mut out = Vec::new();
    for b in built {
        if b.name == ROOT {
            out.extend(b.objs);
        }
    }
    Ok(out)
}

fn crate_parse_field_path(s: &str, line_no: usize) -> Result<Vec<String>> {
    parse_path(s, line_no)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::encode;
    use ldf_data::obj;

    #[test]
    fn round_trip_byte_exact() {
        let recs = vec![
            obj! {"order_id" => "ORD-1", "customer" => obj! {"id" => 4654, "address" => obj! {"city" => "São Paulo"}},
            "items" => vec![obj! {"sku" => "E5", "qty" => 3, "price" => 28.78}, obj! {"sku" => "C3", "qty" => 4, "price" => 157.4}],
            "total" => 715.94, "shipped" => true, "note" => "a,b", "tags" => vec![Value::from("x")], "empty" => Vec::<Value>::new(), "eo" => obj! {}},
            obj! {"order_id" => "ORD-2", "customer" => obj! {"id" => 50, "address" => obj! {"city" => "Oslo"}},
            "items" => Vec::<Value>::new(), "total" => 1.0, "shipped" => false, "note" => Value::Null, "big" => u64::MAX, "neg0" => -0.0},
        ];
        let text = encode(&recs);
        let back = decode(&text).unwrap();
        assert_eq!(back, recs, "{text}");
        assert_eq!(encode(&back), text);
    }

    #[test]
    fn ordinal_join_and_grandchildren() {
        let recs = vec![
            obj! {"k" => 1, "items" => vec![obj! {"v" => 1, "subs" => vec![obj! {"z" => "a"}]}, obj! {"v" => 2, "subs" => Vec::<Value>::new()}]},
            obj! {"k" => 1, "items" => vec![obj! {"v" => 3, "subs" => vec![obj! {"z" => "b"}, obj! {"z" => "c"}]}]},
        ];
        let text = encode(&recs);
        assert!(text.contains("@records.items _:i,v:i,subs:t\n"), "{text}");
        assert!(text.contains("@records.items.subs v:i,z:s\n"), "{text}");
        let back = decode(&text).unwrap();
        assert_eq!(back, recs, "{text}");
    }

    #[test]
    fn scalars_and_tricky_keys() {
        let recs = vec![
            Value::from(1),
            Value::from("x"),
            Value::Null,
            obj! {"a" => 1},
        ];
        let text = encode(&recs);
        assert_eq!(decode(&text).unwrap(), recs, "{text}");
        let recs = vec![
            obj! {"with space" => 1, "dotted.key" => obj! {"x" => 2}, "" => "e", "ключ" => true, "a:b" => "c"},
        ];
        let text = encode(&recs);
        assert_eq!(decode(&text).unwrap(), recs, "{text}");
    }

    #[test]
    fn errors() {
        assert!(decode("1,2\n").is_err());
        assert!(decode("@records a:i,b:s\n1\n").is_err());
        assert!(decode("@records a:i\nx\n").is_err());
        assert!(decode("@records a:t\n2\n@records.a a:i,v:i\n2,1\n").is_err());
    }
}
