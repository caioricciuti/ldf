//! Encoder for any [`Config`]. With [`Config::LDF_V01`] the output is LDF
//! v0.1 byte for byte (tested against `crates/ldf`).

use std::fmt::Write as _;

use ldf_data::{Number, Object, Value};

use crate::knobs::{Config, FloatKind, Keys, Quoting, RestForm, Typing};
use crate::plan::{Join, Kind, TablePlan, lookup, plan};
use crate::text::{push_json_string, push_json_value, push_path, push_segment};

pub fn encode(records: &[Value], cfg: &Config) -> String {
    let p = plan(records, cfg);
    let mut w = Writer {
        cfg,
        out: String::with_capacity(records.len() * 64),
    };
    if p.scalar_rows {
        w.header(&p);
        for (i, r) in records.iter().enumerate() {
            w.repeat_header(&p, i);
            w.cell(&Kind::Json, Some(r), true);
            w.out.push('\n');
        }
        return w.out;
    }
    let objs: Vec<&Object> = records.iter().filter_map(Value::as_object).collect();
    w.header(&p);
    for (i, o) in objs.iter().enumerate() {
        w.repeat_header(&p, i);
        w.row(&p, o, None);
    }
    w.children(&p, &objs);
    w.out
}

struct Writer<'a> {
    cfg: &'a Config,
    out: String,
}

impl Writer<'_> {
    fn delim(&mut self) {
        self.out.push(self.cfg.delim.char());
    }

    fn header(&mut self, p: &TablePlan) {
        self.out.push('@');
        self.out.push_str(&p.name);
        self.out.push(' ');
        let mut first = true;
        match &p.join {
            Join::None => {}
            Join::Ordinal => {
                self.out.push('_');
                self.type_suffix(&Kind::Int, false);
                first = false;
            }
            Join::ParentKey(path, kind) => {
                push_path(path, &mut self.out);
                self.type_suffix(kind, false);
                first = false;
            }
        }
        for c in &p.columns {
            if !first {
                self.delim();
            }
            first = false;
            push_path(&c.path, &mut self.out);
            self.type_suffix(&c.kind, c.nullable);
        }
        if !p.rest_keys.is_empty() {
            if !first {
                self.delim();
            }
            self.out.push('~');
            self.type_suffix(&Kind::Json, true);
        }
        self.out.push('\n');
    }

    fn type_suffix(&mut self, kind: &Kind, nullable: bool) {
        let typed = self.cfg.typing == Typing::Letters;
        match kind {
            Kind::Table(_) => self.out.push_str(":t"),
            Kind::Hoisted(keys) => {
                self.out.push_str(":[");
                for (i, k) in keys.iter().enumerate() {
                    if i > 0 {
                        self.out.push(',');
                    }
                    push_segment(k, &mut self.out);
                }
                self.out.push(']');
            }
            k if typed => {
                self.out.push(':');
                self.out.push(k.code());
            }
            _ => return,
        }
        if nullable && typed && self.cfg.nullable_mark {
            self.out.push('?');
        }
    }

    fn repeat_header(&mut self, p: &TablePlan, i: usize) {
        if let Keys::Every(n) = self.cfg.keys
            && i > 0
            && i.is_multiple_of(usize::from(n))
        {
            self.header(p);
        }
    }

    fn row(&mut self, p: &TablePlan, o: &Object, join: Option<&Value>) {
        if self.cfg.keys == Keys::Row {
            return self.keyed_row(p, o, join);
        }
        let line_start = self.out.len();
        let mut keep = line_start;
        let mut first = true;
        if let Some(j) = join {
            let kind = match &p.join {
                Join::ParentKey(_, k) => k.clone(),
                _ => Kind::Int,
            };
            self.cell(&kind, Some(j), true);
            first = false;
            keep = self.out.len();
        }
        for c in &p.columns {
            if !first {
                self.delim();
            }
            let v = lookup(o, &c.path, c.hint);
            self.cell(&c.kind, v, first);
            first = false;
            if v.is_some() {
                keep = self.out.len();
            }
        }
        if !p.rest_keys.is_empty() {
            let rest: Vec<&(String, Value)> = o
                .iter()
                .filter(|(k, _)| p.rest_keys.iter().any(|r| r == k))
                .collect();
            match self.cfg.rest {
                RestForm::Json => {
                    if !first {
                        self.delim();
                    }
                    if rest.is_empty() {
                        self.out.push_str(self.cfg.nulls.absent());
                    } else {
                        let obj: Object = rest.into_iter().cloned().collect();
                        push_json_value(&Value::Object(obj), &mut self.out);
                        keep = self.out.len();
                    }
                }
                RestForm::Pairs => {
                    for (k, v) in rest {
                        if !first {
                            self.delim();
                        }
                        first = false;
                        push_segment(k, &mut self.out);
                        self.out.push('=');
                        self.self_describing(v, false);
                        keep = self.out.len();
                    }
                }
            }
        }
        if self.cfg.trim {
            self.out.truncate(keep);
        }
        // An empty line reads as one empty cell, which is null under some markers.
        let cells = join.is_some()
            || !p.columns.is_empty()
            || (!p.rest_keys.is_empty() && self.cfg.rest == RestForm::Json);
        if self.out.len() == line_start && cells {
            self.out.push_str(self.cfg.nulls.absent());
        }
        self.out.push('\n');
    }

    fn keyed_row(&mut self, p: &TablePlan, o: &Object, join: Option<&Value>) {
        let mut first = true;
        if let Some(j) = join {
            match &p.join {
                Join::ParentKey(path, kind) => {
                    push_path(path, &mut self.out);
                    self.out.push('=');
                    self.cell(kind, Some(j), false);
                }
                _ => {
                    self.out.push_str("_=");
                    self.cell(&Kind::Int, Some(j), false);
                }
            }
            first = false;
        }
        for c in &p.columns {
            let Some(v) = lookup(o, &c.path, c.hint) else {
                continue;
            };
            if !first {
                self.delim();
            }
            first = false;
            push_path(&c.path, &mut self.out);
            self.out.push('=');
            self.cell(&c.kind, Some(v), false);
        }
        self.out.push('\n');
    }

    /// One cell under a column kind. `line_start` is set for the first cell
    /// of a line, where a bare `@` would read as a header.
    fn cell(&mut self, kind: &Kind, v: Option<&Value>, line_start: bool) {
        let Some(v) = v else {
            self.out.push_str(self.cfg.nulls.absent());
            return;
        };
        if v.is_null() {
            self.out.push_str(self.cfg.nulls.null());
            return;
        }
        match (kind, v) {
            (Kind::Table(_), Value::Array(a)) => {
                let _ = write!(self.out, "{}", a.len());
            }
            (Kind::Hoisted(keys), Value::Array(a)) => {
                let rows: Vec<Value> = a
                    .iter()
                    .map(|e| match e {
                        Value::Object(o) => Value::Array(
                            keys.iter()
                                .map(|k| {
                                    o.iter()
                                        .find(|(kk, _)| kk == k)
                                        .map(|(_, v)| v.clone())
                                        .unwrap_or(Value::Null)
                                })
                                .collect(),
                        ),
                        other => other.clone(),
                    })
                    .collect();
                push_json_value(&Value::Array(rows), &mut self.out);
            }
            _ if self.cfg.typing == Typing::None => self.self_describing(v, line_start),
            (Kind::Int, Value::Number(n)) => push_number(*n, false, &mut self.out),
            (Kind::Float, Value::Number(n)) => {
                push_number(*n, self.cfg.float_kind == FloatKind::Typed, &mut self.out)
            }
            (Kind::Str, Value::String(s)) => self.string(s, false, line_start),
            (Kind::Bool, Value::Bool(b)) => self.out.push_str(self.cfg.bools.text(*b)),
            _ => push_json_value(v, &mut self.out),
        }
    }

    /// A value whose cell must say what it is: no column type to lean on.
    fn self_describing(&mut self, v: &Value, line_start: bool) {
        match v {
            Value::Null => self.out.push_str(self.cfg.nulls.null()),
            Value::Bool(b) => self.out.push_str(if *b { "true" } else { "false" }),
            Value::Number(n) => push_number(*n, false, &mut self.out),
            Value::String(s) => self.string(s, true, line_start),
            Value::Array(_) | Value::Object(_) => push_json_value(v, &mut self.out),
        }
    }

    /// A string cell. `untyped` adds the strings that would otherwise read
    /// as numbers or JSON.
    fn string(&mut self, s: &str, untyped: bool, line_start: bool) {
        let cfg = self.cfg;
        let d = cfg.delim.char();
        let collides = s.is_empty()
            || s == "null"
            || s == "true"
            || s == "false"
            || s == cfg.nulls.null()
            || s == cfg.nulls.absent()
            || (line_start && s.starts_with('@'))
            || (untyped
                && (s.starts_with('[')
                    || s.starts_with('{')
                    || ldf_data::number::parse_json_number(s).is_some()));
        match cfg.quoting {
            Quoting::Json => {
                let edge = |c: Option<char>| matches!(c, Some(' ' | '\t'));
                if collides
                    || edge(s.chars().next())
                    || edge(s.chars().next_back())
                    || s.chars()
                        .any(|c| c == d || matches!(c, '"' | '\\') || c < '\u{20}')
                {
                    push_json_string(s, &mut self.out);
                } else {
                    self.out.push_str(s);
                }
            }
            Quoting::Csv => {
                if collides || s.chars().any(|c| c == d || matches!(c, '"' | '\r' | '\n')) {
                    self.out.push('"');
                    for c in s.chars() {
                        if c == '"' {
                            self.out.push('"');
                        }
                        self.out.push(c);
                    }
                    self.out.push('"');
                } else {
                    self.out.push_str(s);
                }
            }
            Quoting::Backslash => {
                if collides {
                    self.out.push_str("\\_");
                }
                for c in s.chars() {
                    match c {
                        '\\' => self.out.push_str("\\\\"),
                        '\n' => self.out.push_str("\\n"),
                        '\r' => self.out.push_str("\\r"),
                        '\t' if d == '\t' => self.out.push_str("\\t"),
                        c if c == d => {
                            self.out.push('\\');
                            self.out.push(c);
                        }
                        c => self.out.push(c),
                    }
                }
            }
        }
    }

    fn children(&mut self, p: &TablePlan, parents: &[&Object]) {
        for c in &p.columns {
            let Kind::Table(child) = &c.kind else {
                continue;
            };
            self.header(child);
            let mut elements: Vec<&Object> = Vec::new();
            let mut n = 0;
            for (idx, parent) in parents.iter().enumerate() {
                let Some(Value::Array(items)) = lookup(parent, &c.path, c.hint) else {
                    continue;
                };
                let ordinal = Value::Number(Number::I64(idx as i64 + 1));
                let join = match &child.join {
                    Join::ParentKey(path, _) => lookup(parent, path, usize::MAX),
                    _ => Some(&ordinal),
                };
                for item in items {
                    let Some(o) = item.as_object() else { continue };
                    self.repeat_header(child, n);
                    self.row(child, o, join);
                    elements.push(o);
                    n += 1;
                }
            }
            self.children(child, &elements);
        }
    }
}

/// Integer text, or shortest round-trip float text with `.0` kept unless
/// `drop_point` (the column type already says float).
fn push_number(n: Number, drop_point: bool, out: &mut String) {
    match n {
        Number::I64(v) => {
            let _ = write!(out, "{v}");
        }
        Number::U64(v) => {
            let _ = write!(out, "{v}");
        }
        Number::F64(v) if v.is_finite() => {
            let start = out.len();
            let _ = write!(out, "{v:?}");
            if drop_point && out.ends_with(".0") {
                out.truncate(out.len() - 2);
            }
            debug_assert!(out.len() > start);
        }
        Number::F64(_) => out.push_str("null"),
    }
}
