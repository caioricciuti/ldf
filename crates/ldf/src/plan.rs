//! Schema analysis for the batch encoder (spec section 7).
//!
//! A [`TablePlan`] is computed from the whole dataset: which fields flatten
//! into dotted columns, which become child tables, which stay JSON cells,
//! and how child rows join their parents.

use std::collections::HashSet;

use ldf_data::{Number, Object, Value};

use crate::ROOT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColType {
    Int,
    Float,
    Str,
    Bool,
    Json,
    Table,
}

impl ColType {
    pub fn code(self) -> char {
        match self {
            ColType::Int => 'i',
            ColType::Float => 'f',
            ColType::Str => 's',
            ColType::Bool => 'b',
            ColType::Json => 'j',
            ColType::Table => 't',
        }
    }

    pub fn from_code(c: char) -> Option<ColType> {
        Some(match c {
            'i' => ColType::Int,
            'f' => ColType::Float,
            's' => ColType::Str,
            'b' => ColType::Bool,
            'j' => ColType::Json,
            't' => ColType::Table,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Column {
    /// Path from the row object to the value.
    pub path: Vec<String>,
    pub ty: ColType,
    pub nullable: bool,
    /// Present for `Table` columns: the child table plan.
    pub child: Option<Box<TablePlan>>,
    /// Position of the first path segment in the first row that had it.
    pub hint: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Join {
    /// Root table: no join column.
    None,
    /// First column of the parent table (its path and type are copied).
    ParentKey,
    /// `_:i`, 1-based ordinal of the parent row within the parent table.
    Ordinal,
}

#[derive(Debug, Clone)]
pub struct TablePlan {
    pub name: String,
    pub join: Join,
    /// The join column (for child tables), kept separate from `columns`.
    pub join_col: Option<(Vec<String>, ColType)>,
    pub columns: Vec<Column>,
    /// Records that are not objects make a single `_:j` column.
    pub scalar_rows: bool,
    /// Top-level keys that are too sparse for a column of their own
    /// (encoder option `sparse_threshold`). When non-empty the table ends
    /// with a `~:j?` column whose cell is a JSON object of those keys
    /// present in the row, in record order.
    pub rest_keys: Vec<String>,
}

/// Planner knobs. `Default` is the v0.1 layout; [`Options::V0`] is the
/// original layout without the sparse-key rule.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Options {
    /// A top-level key whose share of rows with a non-null value is below
    /// this moves from a positional column into the row's `~` object
    /// (spec section 7, rule 4). `0.0` disables the rule.
    pub sparse_threshold: f64,
}

impl Options {
    /// The default threshold, chosen on 2026-10-18 from the Haiku runs in
    /// `docs/design-candidates.md`: JSON's accuracy at 25 percent fewer
    /// tokens on sparse data, dense data unchanged.
    pub const DEFAULT_SPARSE_THRESHOLD: f64 = 0.5;
    /// v0 layout: every key is a column.
    pub const V0: Options = Options {
        sparse_threshold: 0.0,
    };
}

impl Default for Options {
    fn default() -> Self {
        Options {
            sparse_threshold: Options::DEFAULT_SPARSE_THRESHOLD,
        }
    }
}

/// Looks a dotted path up in an object. `None` means absent at some step.
pub fn lookup<'a>(obj: &'a Object, path: &[String]) -> Option<&'a Value> {
    lookup_hint(obj, path, usize::MAX)
}

/// Like [`lookup`], trying position `hint` for the first segment before
/// scanning. Records usually share key order, so this is O(1) in practice.
pub fn lookup_hint<'a>(obj: &'a Object, path: &[String], hint: usize) -> Option<&'a Value> {
    let (first, rest) = path.split_first()?;
    let v = match obj.get(hint) {
        Some((k, v)) if k == first => v,
        _ => obj.iter().find(|(k, _)| k == first).map(|(_, v)| v)?,
    };
    if rest.is_empty() {
        return Some(v);
    }
    match v {
        Value::Object(o) => lookup(o, rest),
        _ => None,
    }
}

/// Plans the root table for a dataset with the default options.
pub fn plan(records: &[Value]) -> TablePlan {
    plan_with(records, &Options::default())
}

/// Plans the root table for a dataset.
pub fn plan_with(records: &[Value], opts: &Options) -> TablePlan {
    let objs: Option<Vec<&Object>> = records.iter().map(Value::as_object).collect();
    match objs {
        Some(objs) if !records.is_empty() => {
            plan_table(ROOT.to_string(), &objs, Join::None, None, opts)
        }
        _ => TablePlan {
            name: ROOT.to_string(),
            join: Join::None,
            join_col: None,
            columns: vec![Column {
                path: vec!["_".into()],
                ty: ColType::Json,
                nullable: records.iter().any(Value::is_null),
                child: None,
                hint: 0,
            }],
            scalar_rows: true,
            rest_keys: Vec::new(),
        },
    }
}

fn plan_table(
    name: String,
    rows: &[&Object],
    join: Join,
    join_col: Option<(Vec<String>, ColType)>,
    opts: &Options,
) -> TablePlan {
    let mut columns = Vec::new();
    // Top-level keys in first-seen order. Rows usually repeat the key list
    // of the first row exactly, which the zip check catches in O(columns);
    // the set is only consulted for rows that differ.
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
    let mut rest_keys: Vec<String> = Vec::new();
    for (pos, key) in keys.iter().enumerate() {
        let values: Vec<Option<&Value>> = rows
            .iter()
            .map(|o| match o.get(pos) {
                Some((k, v)) if k == key => Some(v),
                _ => o.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            })
            .collect();
        if opts.sparse_threshold > 0.0 {
            let filled = values
                .iter()
                .filter(|v| v.is_some_and(|v| !v.is_null()))
                .count();
            if (filled as f64) < opts.sparse_threshold * rows.len() as f64 {
                rest_keys.push(key.to_string());
                continue;
            }
        }
        let before = columns.len();
        plan_field(&name, vec![key.to_string()], &values, &mut columns, opts);
        for c in &mut columns[before..] {
            c.hint = pos;
        }
    }
    let mut plan = TablePlan {
        name,
        join,
        join_col,
        columns,
        scalar_rows: false,
        rest_keys,
    };
    // Child tables need to know how to join: decide now that the parent's
    // first column is known.
    let parent_key = parent_join_key(&plan, rows);
    for col in plan.columns.iter_mut() {
        if let Some(child) = col.child.as_mut() {
            match &parent_key {
                Some((path, ty)) => {
                    child.join = Join::ParentKey;
                    child.join_col = Some((path.clone(), *ty));
                }
                None => {
                    child.join = Join::Ordinal;
                    child.join_col = Some((vec!["_".into()], ColType::Int));
                }
            }
        }
    }
    plan
}

/// The parent's first column qualifies as a join key when it is a scalar
/// column present and non-null in every row with unique values.
fn parent_join_key(plan: &TablePlan, rows: &[&Object]) -> Option<(Vec<String>, ColType)> {
    let first = plan.columns.first()?;
    if !matches!(
        first.ty,
        ColType::Int | ColType::Str | ColType::Float | ColType::Bool
    ) {
        return None;
    }
    let mut seen: HashSet<String> = HashSet::with_capacity(rows.len());
    for o in rows {
        let v = lookup_hint(o, &first.path, first.hint)?;
        let key = match v {
            Value::Null => return None,
            Value::String(s) => s.clone(),
            Value::Number(n) => n.canonical_text()?,
            Value::Bool(b) => b.to_string(),
            _ => return None,
        };
        if !seen.insert(key) {
            return None;
        }
    }
    Some((first.path.clone(), first.ty))
}

fn plan_field(
    table: &str,
    path: Vec<String>,
    values: &[Option<&Value>],
    out: &mut Vec<Column>,
    opts: &Options,
) {
    let present: Vec<&Value> = values
        .iter()
        .flatten()
        .copied()
        .filter(|v| !v.is_null())
        .collect();
    let nullable = present.len() != values.len();

    // 1. Uniform non-empty objects flatten.
    if !present.is_empty()
        && present
            .iter()
            .all(|v| matches!(v, Value::Object(o) if !o.is_empty()))
        && same_key_set(&present)
    {
        let first = present[0].as_object().unwrap();
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
            plan_field(table, p, &sub, out, opts);
        }
        return;
    }

    // 2. Arrays of objects become a child table.
    if !present.is_empty()
        && present
            .iter()
            .all(|v| matches!(v, Value::Array(a) if a.iter().all(|x| matches!(x, Value::Object(o) if !o.is_empty()))))
        && present.iter().any(|v| !v.as_array().unwrap().is_empty())
    {
        let elements: Vec<&Object> = present
            .iter()
            .flat_map(|v| v.as_array().unwrap().iter().map(|x| x.as_object().unwrap()))
            .collect();
        let mut child_name = String::with_capacity(table.len() + 16);
        child_name.push_str(table);
        for seg in &path {
            child_name.push('.');
            child_name.push_str(seg);
        }
        let child = plan_table(child_name, &elements, Join::Ordinal, None, opts);
        out.push(Column {
            path,
            ty: ColType::Table,
            nullable,
            child: Some(Box::new(child)),
            hint: 0,
        });
        return;
    }

    // 3. Scalars of one kind, else JSON.
    let ty = scalar_type(&present);
    out.push(Column {
        path,
        ty,
        nullable,
        child: None,
        hint: 0,
    });
}

fn same_key_set(objs: &[&Value]) -> bool {
    let first: Vec<&str> = objs[0]
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, _)| k.as_str())
        .collect();
    objs.iter().all(|v| {
        let o = v.as_object().unwrap();
        o.len() == first.len() && o.iter().all(|(k, _)| first.contains(&k.as_str()))
    })
}

fn scalar_type(present: &[&Value]) -> ColType {
    if present.is_empty() {
        return ColType::Json;
    }
    let mut ints = 0;
    let mut floats = 0;
    let mut strs = 0;
    let mut bools = 0;
    for v in present {
        match v {
            Value::Number(Number::F64(_)) => floats += 1,
            Value::Number(_) => ints += 1,
            Value::String(_) => strs += 1,
            Value::Bool(_) => bools += 1,
            _ => return ColType::Json,
        }
    }
    let n = present.len();
    if ints == n {
        ColType::Int
    } else if floats == n {
        ColType::Float
    } else if strs == n {
        ColType::Str
    } else if bools == n {
        ColType::Bool
    } else {
        ColType::Json
    }
}

impl TablePlan {
    /// Header line text, without the trailing newline.
    pub fn header(&self) -> String {
        let mut s = String::with_capacity(64);
        s.push('@');
        s.push_str(&self.name);
        s.push(' ');
        let mut first = true;
        if let Some((path, ty)) = &self.join_col {
            crate::text::push_path(path, &mut s);
            s.push(':');
            s.push(ty.code());
            first = false;
        }
        for c in &self.columns {
            if !first {
                s.push(',');
            }
            first = false;
            crate::text::push_path(&c.path, &mut s);
            s.push(':');
            s.push(c.ty.code());
            if c.nullable {
                s.push('?');
            }
        }
        if !self.rest_keys.is_empty() {
            if !first {
                s.push(',');
            }
            s.push_str("~:j?");
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldf_data::obj;

    #[test]
    fn plans_flatten_children_and_json() {
        let recs = vec![
            obj! {"id" => 1, "c" => obj! {"x" => 1, "y" => "a"}, "items" => vec![obj! {"sku" => "A", "n" => 1.5}], "tags" => vec![Value::from("t")], "m" => 1},
            obj! {"id" => 2, "c" => obj! {"x" => 2, "y" => "b"}, "items" => Vec::<Value>::new(), "tags" => vec![Value::from("u")], "m" => 2.5},
        ];
        let p = plan(&recs);
        assert_eq!(p.header(), "@records id:i,c.x:i,c.y:s,items:t,tags:j,m:j");
        let child = p.columns[3].child.as_ref().unwrap();
        assert_eq!(child.header(), "@records.items id:i,sku:s,n:f");
        assert_eq!(child.join, Join::ParentKey);
    }

    #[test]
    fn ordinal_join_when_key_not_unique_and_irregular_objects_stay_json() {
        let recs = vec![
            obj! {"k" => 1, "o" => obj! {"a" => 1}, "items" => vec![obj! {"v" => 1}]},
            obj! {"k" => 1, "o" => obj! {"b" => 2}, "items" => vec![obj! {"v" => 2}]},
        ];
        let p = plan(&recs);
        assert_eq!(p.header(), "@records k:i,o:j,items:t");
        let child = p.columns[2].child.as_ref().unwrap();
        assert_eq!(child.header(), "@records.items _:i,v:i");
    }

    #[test]
    fn scalar_records() {
        let p = plan(&[Value::from(1), Value::from("x")]);
        assert!(p.scalar_rows);
        assert_eq!(p.header(), "@records _:j");
    }
}
