//! Table planning under a [`Config`]: the rules of LDF v0.1 section 7 with
//! the nesting and sparse knobs open.

use std::collections::HashSet;

use ldf_data::{Number, Object, Value};

use crate::knobs::{Arrays, Config, Objects};

pub const ROOT: &str = "records";

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Int,
    Float,
    Str,
    Bool,
    Json,
    /// Child table; the cell is the element count.
    Table(Box<TablePlan>),
    /// Array of objects with these keys, as positional arrays in a JSON cell.
    Hoisted(Vec<String>),
}

impl Kind {
    pub fn code(&self) -> char {
        match self {
            Kind::Int => 'i',
            Kind::Float => 'f',
            Kind::Str => 's',
            Kind::Bool => 'b',
            Kind::Json => 'j',
            Kind::Table(_) => 't',
            Kind::Hoisted(_) => 'h',
        }
    }
    pub fn is_scalar_key(&self) -> bool {
        matches!(self, Kind::Int | Kind::Float | Kind::Str | Kind::Bool)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    pub path: Vec<String>,
    pub kind: Kind,
    pub nullable: bool,
    /// Position of the first path segment in the first row that had it.
    pub hint: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Join {
    None,
    /// The parent's first column: its path and kind.
    ParentKey(Vec<String>, Kind),
    Ordinal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TablePlan {
    pub name: String,
    pub join: Join,
    pub columns: Vec<Column>,
    /// Records that are not objects: one `_:j` column.
    pub scalar_rows: bool,
    /// Top-level keys that travel in the rest of the row.
    pub rest_keys: Vec<String>,
}

pub fn lookup<'a>(obj: &'a Object, path: &[String], hint: usize) -> Option<&'a Value> {
    let (first, rest) = path.split_first()?;
    let v = match obj.get(hint) {
        Some((k, v)) if k == first => v,
        _ => obj.iter().find(|(k, _)| k == first).map(|(_, v)| v)?,
    };
    if rest.is_empty() {
        return Some(v);
    }
    match v {
        Value::Object(o) => lookup(o, rest, usize::MAX),
        _ => None,
    }
}

pub fn plan(records: &[Value], cfg: &Config) -> TablePlan {
    let objs: Option<Vec<&Object>> = records.iter().map(Value::as_object).collect();
    match objs {
        Some(objs) if !records.is_empty() => plan_table(ROOT.to_string(), &objs, Join::None, cfg),
        _ => TablePlan {
            name: ROOT.to_string(),
            join: Join::None,
            columns: vec![Column {
                path: vec!["_".into()],
                kind: Kind::Json,
                nullable: records.iter().any(Value::is_null),
                hint: 0,
            }],
            scalar_rows: true,
            rest_keys: Vec::new(),
        },
    }
}

fn plan_table(name: String, rows: &[&Object], join: Join, cfg: &Config) -> TablePlan {
    let mut keys: Vec<&str> = Vec::new();
    let mut seen: HashSet<&str> = HashSet::new();
    for o in rows {
        if o.len() == keys.len() && o.iter().zip(&keys).all(|((k, _), kk)| k == kk) {
            continue;
        }
        for (k, _) in o.iter() {
            if seen.insert(k.as_str()) {
                keys.push(k);
            }
        }
    }
    let threshold = f64::from(cfg.sparse) / 100.0;
    let mut columns = Vec::new();
    let mut rest_keys = Vec::new();
    for (pos, key) in keys.iter().enumerate() {
        let values: Vec<Option<&Value>> = rows
            .iter()
            .map(|o| match o.get(pos) {
                Some((k, v)) if k == key => Some(v),
                _ => o.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            })
            .collect();
        if threshold > 0.0 {
            let filled = values
                .iter()
                .filter(|v| v.is_some_and(|v| !v.is_null()))
                .count();
            if (filled as f64) < threshold * rows.len() as f64 {
                rest_keys.push(key.to_string());
                continue;
            }
        }
        let before = columns.len();
        plan_field(&name, vec![key.to_string()], &values, &mut columns, cfg);
        for c in &mut columns[before..] {
            c.hint = pos;
        }
    }
    let mut plan = TablePlan {
        name,
        join,
        columns,
        scalar_rows: false,
        rest_keys,
    };
    let parent_key = match cfg.arrays {
        Arrays::ChildKey => parent_join_key(&plan, rows),
        _ => None,
    };
    for col in plan.columns.iter_mut() {
        if let Kind::Table(child) = &mut col.kind {
            child.join = match &parent_key {
                Some((p, k)) => Join::ParentKey(p.clone(), k.clone()),
                None => Join::Ordinal,
            };
        }
    }
    plan
}

fn parent_join_key(plan: &TablePlan, rows: &[&Object]) -> Option<(Vec<String>, Kind)> {
    let first = plan.columns.first()?;
    if !first.kind.is_scalar_key() {
        return None;
    }
    let mut seen: HashSet<String> = HashSet::with_capacity(rows.len());
    for o in rows {
        let key = match lookup(o, &first.path, first.hint)? {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.canonical_text()?,
            Value::Bool(b) => b.to_string(),
            _ => return None,
        };
        if !seen.insert(key) {
            return None;
        }
    }
    Some((first.path.clone(), first.kind.clone()))
}

fn plan_field(
    table: &str,
    path: Vec<String>,
    values: &[Option<&Value>],
    out: &mut Vec<Column>,
    cfg: &Config,
) {
    let present: Vec<&Value> = values
        .iter()
        .flatten()
        .copied()
        .filter(|v| !v.is_null())
        .collect();
    let has_null = values.iter().any(|v| v.is_some_and(Value::is_null));
    let nullable = present.len() != values.len();
    let col = |kind| Column {
        path: path.clone(),
        kind,
        nullable,
        hint: 0,
    };

    if cfg.objects == Objects::Dotted
        && !has_null
        && !present.is_empty()
        && present
            .iter()
            .all(|v| matches!(v, Value::Object(o) if !o.is_empty()))
        && same_key_set(&present)
    {
        let first = present[0].as_object().expect("object");
        for (k, _) in first {
            let sub: Vec<Option<&Value>> = values
                .iter()
                .map(|v| match v {
                    Some(Value::Object(o)) => o.iter().find(|(kk, _)| kk == k).map(|(_, vv)| vv),
                    _ => None,
                })
                .collect();
            let mut p = path.clone();
            p.push(k.clone());
            plan_field(table, p, &sub, out, cfg);
        }
        return;
    }

    let arrays_of_objects = !present.is_empty()
        && present.iter().all(|v| {
            matches!(v, Value::Array(a) if a.iter().all(|x| matches!(x, Value::Object(o) if !o.is_empty())))
        })
        && present
            .iter()
            .any(|v| v.as_array().is_some_and(|a| !a.is_empty()));
    if arrays_of_objects {
        let elements: Vec<&Object> = present
            .iter()
            .flat_map(|v| {
                v.as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_object)
            })
            .collect();
        match cfg.arrays {
            Arrays::ChildKey | Arrays::ChildOrdinal => {
                let mut name = String::with_capacity(table.len() + 16);
                name.push_str(table);
                name.push('.');
                crate::text::push_path(&path, &mut name);
                let child = plan_table(name, &elements, Join::Ordinal, cfg);
                out.push(col(Kind::Table(Box::new(child))));
                return;
            }
            Arrays::Hoisted => {
                let keys: Vec<&str> = elements[0].iter().map(|(k, _)| k.as_str()).collect();
                if elements.iter().all(|o| {
                    o.len() == keys.len() && o.iter().zip(&keys).all(|((k, _), kk)| k == kk)
                }) {
                    let keys = keys.into_iter().map(str::to_string).collect();
                    out.push(col(Kind::Hoisted(keys)));
                    return;
                }
            }
            Arrays::Json => {}
        }
        out.push(col(Kind::Json));
        return;
    }

    out.push(col(scalar_kind(&present)));
}

fn same_key_set(objs: &[&Value]) -> bool {
    let first: Vec<&str> = objs[0]
        .as_object()
        .into_iter()
        .flatten()
        .map(|(k, _)| k.as_str())
        .collect();
    objs.iter().all(|v| {
        v.as_object().is_some_and(|o| {
            o.len() == first.len() && o.iter().all(|(k, _)| first.contains(&k.as_str()))
        })
    })
}

fn scalar_kind(present: &[&Value]) -> Kind {
    if present.is_empty() {
        return Kind::Json;
    }
    let (mut ints, mut floats, mut strs, mut bools) = (0, 0, 0, 0);
    for v in present {
        match v {
            Value::Number(Number::F64(_)) => floats += 1,
            Value::Number(_) => ints += 1,
            Value::String(_) => strs += 1,
            Value::Bool(_) => bools += 1,
            _ => return Kind::Json,
        }
    }
    let n = present.len();
    if ints == n {
        Kind::Int
    } else if floats == n {
        Kind::Float
    } else if strs == n {
        Kind::Str
    } else if bools == n {
        Kind::Bool
    } else {
        Kind::Json
    }
}
