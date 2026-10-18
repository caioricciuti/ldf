//! Decoder for any [`Config`]. The config is an input, not something the
//! document declares: a candidate is a (config, encoder, decoder) triple
//! and the round-trip check runs that triple.

use std::collections::HashMap;
use std::rc::Rc;

use ldf_data::number::parse_json_number;
use ldf_data::{Number, Object, Value};

use crate::knobs::{Config, Keys, Quoting, RestForm};
use crate::plan::ROOT;
use crate::text::json_string_end;

#[derive(Debug, Clone)]
pub struct DecodeError {
    pub line: usize,
    pub message: String,
}

type Result<T> = std::result::Result<T, DecodeError>;

#[derive(Debug, Clone, PartialEq)]
enum DKind {
    Int,
    Float,
    Str,
    Bool,
    Json,
    Table,
    Hoisted(Vec<String>),
    /// No type in the header: the cell says what it is.
    Untyped,
}

#[derive(Debug)]
struct HCol {
    path: Vec<String>,
    text: String,
    kind: DKind,
}

#[derive(Debug)]
struct Header {
    name: String,
    /// Join column of a child table; `true` when it is the ordinal `_`.
    join: Option<(HCol, bool)>,
    cols: Vec<HCol>,
    rest: bool,
    scalar_rows: bool,
    /// Column index by path text, for keyed rows. The join column is `usize::MAX`.
    by_text: HashMap<String, usize>,
}

enum Cell {
    Val(Value),
    Count(usize),
}

struct Row {
    header: Rc<Header>,
    join: Option<Value>,
    cells: Vec<Option<Cell>>,
    rest: Object,
}

struct Cursor<'a> {
    s: &'a str,
    b: &'a [u8],
    pos: usize,
    line: usize,
}

impl<'a> Cursor<'a> {
    fn err<T>(&self, message: impl Into<String>) -> Result<T> {
        Err(DecodeError {
            line: self.line,
            message: message.into(),
        })
    }
    fn peek(&self) -> Option<u8> {
        self.b.get(self.pos).copied()
    }
    fn rest(&self) -> &'a str {
        &self.s[self.pos..]
    }
    fn at_cell_end(&self, d: u8) -> bool {
        matches!(self.peek(), None | Some(b'\n')) || self.peek() == Some(d)
    }
    /// Bare text up to the delimiter or line end. With backslash quoting an
    /// escaped byte never ends the cell.
    fn bare(&mut self, d: u8, escapes: bool) -> &'a str {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c == d || c == b'\n' {
                break;
            }
            if escapes && c == b'\\' && self.pos + 1 < self.b.len() {
                self.pos += 1;
            }
            self.pos += 1;
        }
        &self.s[start..self.pos]
    }
    fn json_value(&mut self) -> Result<Value> {
        let mut it = serde_json::Deserializer::from_str(self.rest()).into_iter::<Value>();
        match it.next() {
            Some(Ok(v)) => {
                self.pos += it.byte_offset();
                Ok(v)
            }
            Some(Err(e)) => self.err(format!("bad JSON cell: {e}")),
            None => self.err("empty JSON cell"),
        }
    }
    fn json_string(&mut self) -> Result<String> {
        let Some(end) = json_string_end(self.rest()) else {
            return self.err("unterminated string");
        };
        let lit = &self.rest()[..end];
        self.pos += end;
        serde_json::from_str(lit).map_err(|e| DecodeError {
            line: self.line,
            message: format!("bad string literal: {e}"),
        })
    }
    fn csv_string(&mut self) -> Result<String> {
        self.pos += 1;
        let mut out = String::new();
        loop {
            let rest = self.rest();
            let Some(q) = rest.find('"') else {
                return self.err("unterminated quoted cell");
            };
            out.push_str(&rest[..q]);
            self.line += rest[..q].matches('\n').count();
            self.pos += q + 1;
            if self.peek() == Some(b'"') {
                out.push('"');
                self.pos += 1;
            } else {
                return Ok(out);
            }
        }
    }
}

fn unescape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut it = raw.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('_') => {}
            Some(o) => out.push(o),
            None => out.push('\\'),
        }
    }
    out
}

enum Got {
    Absent,
    Val(Value),
    Count(usize),
}

/// Reads one cell of the given kind; the cursor stops at the delimiter or line end.
fn read_cell(cur: &mut Cursor, kind: &DKind, cfg: &Config) -> Result<Got> {
    let d = cfg.delim.byte();
    let first = cur.peek();
    let jsonish = matches!(kind, DKind::Json | DKind::Hoisted(_) | DKind::Untyped);
    if jsonish && matches!(first, Some(b'[' | b'{')) || *kind == DKind::Json && first == Some(b'"')
    {
        let v = cur.json_value()?;
        return match kind {
            DKind::Hoisted(keys) => unhoist(cur, keys, v).map(Got::Val),
            _ => Ok(Got::Val(v)),
        };
    }
    let stringy = matches!(kind, DKind::Str | DKind::Untyped);
    if stringy {
        match (cfg.quoting, first) {
            (Quoting::Json, Some(b'"')) => return Ok(Got::Val(Value::String(cur.json_string()?))),
            (Quoting::Csv, Some(b'"')) => return Ok(Got::Val(Value::String(cur.csv_string()?))),
            (Quoting::Backslash, Some(b'\\')) if cur.rest().starts_with("\\_") => {
                let raw = cur.bare(d, true);
                return Ok(Got::Val(Value::String(unescape(raw))));
            }
            _ => {}
        }
    }
    let escapes = stringy && cfg.quoting == Quoting::Backslash;
    let raw = cur.bare(d, escapes);
    if raw == cfg.nulls.absent() {
        return Ok(Got::Absent);
    }
    if raw == cfg.nulls.null() {
        return Ok(Got::Val(Value::Null));
    }
    let bad = |cur: &Cursor, what: &str| cur.err(format!("bad {what} cell {raw:?}"));
    let v = match kind {
        DKind::Int => match parse_json_number(raw) {
            Some(n @ (Number::I64(_) | Number::U64(_))) => Value::Number(n),
            _ => return bad(cur, "integer"),
        },
        DKind::Float => match raw.parse::<f64>() {
            Ok(f) => Value::Number(Number::F64(f)),
            Err(_) => return bad(cur, "float"),
        },
        DKind::Bool => {
            if raw == cfg.bools.text(true) {
                Value::Bool(true)
            } else if raw == cfg.bools.text(false) {
                Value::Bool(false)
            } else {
                return bad(cur, "boolean");
            }
        }
        DKind::Table => match raw.parse::<usize>() {
            Ok(n) => return Ok(Got::Count(n)),
            Err(_) => return bad(cur, "child count"),
        },
        DKind::Str => Value::String(if escapes {
            unescape(raw)
        } else {
            raw.to_string()
        }),
        DKind::Untyped => match raw {
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            "null" => Value::Null,
            _ => match parse_json_number(raw) {
                Some(n) => Value::Number(n),
                None if escapes => Value::String(unescape(raw)),
                None => Value::String(raw.to_string()),
            },
        },
        DKind::Json | DKind::Hoisted(_) => match serde_json::from_str::<Value>(raw) {
            Ok(v) => v,
            Err(_) => return bad(cur, "JSON"),
        },
    };
    Ok(Got::Val(v))
}

fn unhoist(cur: &Cursor, keys: &[String], v: Value) -> Result<Value> {
    let Value::Array(rows) = v else {
        return cur.err("hoisted cell is not an array");
    };
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let Value::Array(vals) = r else {
            return cur.err("hoisted element is not an array");
        };
        if vals.len() != keys.len() {
            return cur.err("hoisted element has the wrong arity");
        }
        out.push(Value::Object(keys.iter().cloned().zip(vals).collect()));
    }
    Ok(Value::Array(out))
}

/// Splits on `sep` outside JSON strings and brackets.
fn split_outside(s: &str, sep: u8) -> Vec<&str> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let (mut start, mut i, mut depth) = (0, 0, 0i32);
    while i < b.len() {
        match b[i] {
            b'"' => {
                i += json_string_end(&s[i..]).unwrap_or(1);
                continue;
            }
            b'[' => depth += 1,
            b']' => depth -= 1,
            c if c == sep && depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    out.push(&s[start..]);
    out
}

fn parse_path(cur: &Cursor, text: &str) -> Result<Vec<String>> {
    let mut segs = Vec::new();
    for part in split_outside(text, b'.') {
        if part.starts_with('"') {
            match serde_json::from_str::<String>(part) {
                Ok(s) => segs.push(s),
                Err(e) => return cur.err(format!("bad key {part:?}: {e}")),
            }
        } else if part.is_empty() {
            return cur.err(format!("empty key segment in {text:?}"));
        } else {
            segs.push(part.to_string());
        }
    }
    Ok(segs)
}

fn parse_header(cur: &Cursor, line: &str, cfg: &Config) -> Result<Header> {
    let body = &line[1..];
    let mut i = 0;
    let b = body.as_bytes();
    while i < b.len() && b[i] != b' ' {
        if b[i] == b'"' {
            i += json_string_end(&body[i..]).unwrap_or(1);
        } else {
            i += 1;
        }
    }
    let name = body[..i].to_string();
    let entries_text = body.get(i + 1..).unwrap_or("");
    let mut header = Header {
        name,
        join: None,
        cols: Vec::new(),
        rest: false,
        scalar_rows: false,
        by_text: HashMap::new(),
    };
    if entries_text.is_empty() {
        return Ok(header);
    }
    for (n, entry) in split_outside(entries_text, cfg.delim.byte())
        .into_iter()
        .enumerate()
    {
        if entry == "~" || entry.starts_with("~:") {
            header.rest = true;
            continue;
        }
        // The type is after the last ':' outside quotes and brackets.
        let parts = split_outside(entry, b':');
        let (text, ty) = if parts.len() > 1 {
            let ty = parts[parts.len() - 1];
            (&entry[..entry.len() - ty.len() - 1], Some(ty))
        } else {
            (entry, None)
        };
        let ty = ty.map(|t| t.strip_suffix('?').unwrap_or(t));
        let kind = match ty {
            None => DKind::Untyped,
            Some("i") => DKind::Int,
            Some("f") => DKind::Float,
            Some("s") => DKind::Str,
            Some("b") => DKind::Bool,
            Some("j") => DKind::Json,
            Some("t") => DKind::Table,
            Some(t) if t.starts_with('[') && t.ends_with(']') => {
                let mut keys = Vec::new();
                for k in split_outside(&t[1..t.len() - 1], b',') {
                    keys.extend(parse_path(cur, k)?);
                }
                DKind::Hoisted(keys)
            }
            Some(t) => return cur.err(format!("unknown type {t:?}")),
        };
        let col = HCol {
            path: parse_path(cur, text)?,
            text: text.to_string(),
            kind,
        };
        if n == 0 && header.name != ROOT {
            let ordinal = col.text == "_";
            header.by_text.insert(col.text.clone(), usize::MAX);
            header.join = Some((col, ordinal));
        } else {
            header.by_text.insert(col.text.clone(), header.cols.len());
            header.cols.push(col);
        }
    }
    header.scalar_rows = header.name == ROOT
        && !header.rest
        && header.cols.len() == 1
        && header.cols[0].text == "_"
        && matches!(header.cols[0].kind, DKind::Json | DKind::Untyped);
    Ok(header)
}

/// After a cell: a delimiter (more cells follow) or the end of the line.
fn next_cell(cur: &mut Cursor, d: u8) -> Result<bool> {
    match cur.peek() {
        Some(c) if c == d => {
            cur.pos += 1;
            Ok(true)
        }
        Some(b'\n') => {
            cur.pos += 1;
            cur.line += 1;
            Ok(false)
        }
        None => Ok(false),
        Some(c) => cur.err(format!("unexpected {:?} after a cell", c as char)),
    }
}

fn got_cell(g: Got) -> Option<Cell> {
    match g {
        Got::Absent => None,
        Got::Val(v) => Some(Cell::Val(v)),
        Got::Count(n) => Some(Cell::Count(n)),
    }
}

/// `key=` at the cursor; returns the key text.
fn read_key<'a>(cur: &mut Cursor<'a>) -> Result<&'a str> {
    let rest = cur.rest();
    let b = rest.as_bytes();
    let mut i = 0;
    while i < b.len() && b[i] != b'=' && b[i] != b'\n' {
        if b[i] == b'"' {
            i += json_string_end(&rest[i..]).unwrap_or(1);
        } else {
            i += 1;
        }
    }
    if b.get(i) != Some(&b'=') {
        return cur.err("expected key=value");
    }
    cur.pos += i + 1;
    Ok(&rest[..i])
}

fn parse_row(cur: &mut Cursor, h: &Rc<Header>, cfg: &Config) -> Result<Row> {
    let d = cfg.delim.byte();
    let mut row = Row {
        header: Rc::clone(h),
        join: None,
        cells: (0..h.cols.len()).map(|_| None).collect(),
        rest: Object::new(),
    };
    if h.scalar_rows {
        let v = match read_cell(cur, &h.cols[0].kind, cfg)? {
            Got::Val(v) => v,
            _ => return cur.err("scalar row without a value"),
        };
        row.cells[0] = Some(Cell::Val(v));
        next_cell(cur, d)?;
        return Ok(row);
    }
    if cfg.keys == Keys::Row {
        if cur.at_cell_end(d) {
            next_cell(cur, d)?;
            return Ok(row);
        }
        loop {
            let key = read_key(cur)?;
            let Some(&idx) = h.by_text.get(key) else {
                return cur.err(format!("unknown key {key:?}"));
            };
            let kind = if idx == usize::MAX {
                &h.join.as_ref().expect("join").0.kind
            } else {
                &h.cols[idx].kind
            };
            match read_cell(cur, kind, cfg)? {
                Got::Val(v) if idx == usize::MAX => row.join = Some(v),
                g => row.cells[idx] = got_cell(g),
            }
            if !next_cell(cur, d)? {
                return Ok(row);
            }
        }
    }
    let jn = usize::from(h.join.is_some());
    let dense = jn + h.cols.len();
    if dense == 0 && cur.at_cell_end(d) {
        next_cell(cur, d)?;
        return Ok(row);
    }
    let mut idx = 0;
    loop {
        if idx < jn {
            let (col, _) = h.join.as_ref().expect("join");
            if let Got::Val(v) = read_cell(cur, &col.kind, cfg)? {
                row.join = Some(v);
            }
        } else if idx < dense {
            row.cells[idx - jn] = got_cell(read_cell(cur, &h.cols[idx - jn].kind, cfg)?);
        } else if h.rest && cfg.rest == RestForm::Json && idx == dense {
            match read_cell(cur, &DKind::Json, cfg)? {
                Got::Val(Value::Object(o)) => row.rest = o,
                Got::Absent | Got::Val(Value::Null) => {}
                _ => return cur.err("rest cell is not an object"),
            }
        } else if h.rest && cfg.rest == RestForm::Pairs {
            let key = read_key(cur)?;
            let mut segs = parse_path(cur, key)?;
            if segs.len() != 1 {
                return cur.err(format!("rest key {key:?} is a path"));
            }
            let v = match read_cell(cur, &DKind::Untyped, cfg)? {
                Got::Val(v) => v,
                _ => Value::Null,
            };
            row.rest.push((segs.remove(0), v));
        } else {
            return cur.err("more cells than columns");
        }
        idx += 1;
        if !next_cell(cur, d)? {
            return Ok(row);
        }
    }
}

pub fn decode(text: &str, cfg: &Config) -> Result<Vec<Value>> {
    let mut cur = Cursor {
        s: text,
        b: text.as_bytes(),
        pos: 0,
        line: 1,
    };
    let mut order: Vec<String> = Vec::new();
    let mut tables: HashMap<String, Vec<Row>> = HashMap::new();
    let mut current: Option<Rc<Header>> = None;
    while cur.pos < cur.b.len() {
        if cur.peek() == Some(b'@') {
            let end = cur.rest().find('\n').unwrap_or(cur.rest().len());
            let line = &cur.rest()[..end];
            let h = parse_header(&cur, line, cfg)?;
            if !tables.contains_key(&h.name) {
                order.push(h.name.clone());
                tables.insert(h.name.clone(), Vec::new());
            }
            current = Some(Rc::new(h));
            cur.pos += end;
            next_cell(&mut cur, b'\n')?;
            continue;
        }
        let Some(h) = current.clone() else {
            return cur.err("row before any header");
        };
        let row = parse_row(&mut cur, &h, cfg)?;
        tables.get_mut(&h.name).expect("table").push(row);
    }
    Assembler::new(&tables)?.root()
}

/// Rebuilds records: child rows attach to their parent by ordinal or by
/// the parent's first column.
struct Assembler<'a> {
    tables: &'a HashMap<String, Vec<Row>>,
    by_ordinal: HashMap<&'a str, HashMap<usize, Vec<usize>>>,
    by_key: HashMap<&'a str, HashMap<String, Vec<usize>>>,
}

fn key_text(v: &Value) -> String {
    serde_json::to_string(v).unwrap_or_default()
}

fn derr<T>(message: impl Into<String>) -> Result<T> {
    Err(DecodeError {
        line: 0,
        message: message.into(),
    })
}

impl<'a> Assembler<'a> {
    fn new(tables: &'a HashMap<String, Vec<Row>>) -> Result<Self> {
        let mut a = Assembler {
            tables,
            by_ordinal: HashMap::new(),
            by_key: HashMap::new(),
        };
        for (name, rows) in tables {
            for (i, r) in rows.iter().enumerate() {
                let Some((_, ordinal)) = &r.header.join else {
                    continue;
                };
                let Some(j) = &r.join else {
                    return derr(format!("child row {} of {name} has no join", i + 1));
                };
                if *ordinal {
                    let Some(n) = j.as_number().and_then(Number::as_i128) else {
                        return derr("ordinal join is not an integer");
                    };
                    a.by_ordinal
                        .entry(name)
                        .or_default()
                        .entry(n as usize)
                        .or_default()
                        .push(i);
                } else {
                    a.by_key
                        .entry(name)
                        .or_default()
                        .entry(key_text(j))
                        .or_default()
                        .push(i);
                }
            }
        }
        Ok(a)
    }

    fn root(&self) -> Result<Vec<Value>> {
        let Some(rows) = self.tables.get(ROOT) else {
            return Ok(Vec::new());
        };
        rows.iter()
            .enumerate()
            .map(|(i, r)| self.record(r, i + 1))
            .collect()
    }

    fn record(&self, row: &Row, ordinal: usize) -> Result<Value> {
        let h = &row.header;
        if h.scalar_rows {
            return match &row.cells[0] {
                Some(Cell::Val(v)) => Ok(v.clone()),
                _ => derr("scalar row without a value"),
            };
        }
        let mut obj = Object::new();
        for (col, cell) in h.cols.iter().zip(&row.cells) {
            match cell {
                None => {}
                Some(Cell::Val(v)) => insert_path(&mut obj, &col.path, v.clone()),
                Some(Cell::Count(n)) => {
                    let child = format!("{}.{}", h.name, col.text);
                    let ids = match self.by_ordinal.get(child.as_str()) {
                        Some(m) => m.get(&ordinal),
                        None => {
                            let key = match row.cells.first() {
                                Some(Some(Cell::Val(v))) => key_text(v),
                                _ => return derr("parent key missing"),
                            };
                            self.by_key.get(child.as_str()).and_then(|m| m.get(&key))
                        }
                    };
                    let ids: &[usize] = ids.map(Vec::as_slice).unwrap_or(&[]);
                    if ids.len() != *n {
                        return derr(format!(
                            "{child}: parent says {n} rows, found {}",
                            ids.len()
                        ));
                    }
                    let rows = &self.tables[&child];
                    let mut items = Vec::with_capacity(ids.len());
                    for &i in ids {
                        items.push(self.record(&rows[i], i + 1)?);
                    }
                    insert_path(&mut obj, &col.path, Value::Array(items));
                }
            }
        }
        obj.extend(row.rest.iter().cloned());
        Ok(Value::Object(obj))
    }
}

fn insert_path(obj: &mut Object, path: &[String], v: Value) {
    let (first, rest) = path.split_first().expect("non-empty path");
    if rest.is_empty() {
        obj.push((first.clone(), v));
        return;
    }
    let idx = match obj.iter().position(|(k, _)| k == first) {
        Some(i) => i,
        None => {
            obj.push((first.clone(), Value::Object(Object::new())));
            obj.len() - 1
        }
    };
    if let Value::Object(inner) = &mut obj[idx].1 {
        insert_path(inner, rest, v);
    }
}
