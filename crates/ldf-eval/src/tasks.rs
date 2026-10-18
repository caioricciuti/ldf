//! Task generation with programmatic ground truth.
//!
//! Tasks are generated from the records themselves using a seeded RNG so a
//! bundle is reproducible. Every task records the kind, the natural-language
//! question, the canonical answer, the tolerance, and the record indices it
//! depends on (for later analysis of position effects).

use ldf_data::{Number, Value};
use ldf_fixtures::Rng;
use serde::{Deserialize, Serialize};

use crate::answer::{Answer, Scalar, Tolerance};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    /// Value of one field of one record identified by its key field.
    Lookup,
    /// Ids of records matching a predicate on one field.
    Filter,
    /// Count, sum, min, max or mean over one numeric field.
    Aggregate,
    /// Which of two records has the larger value of a field.
    Compare,
    /// Value of an optional field that is missing or null for a record.
    MissingValue,
    /// Value at a dotted path inside a nested object.
    NestedExtract,
    /// Top-k by a numeric field, or records sharing a value.
    MultiRecord,
    /// Number of elements in one record's array of objects.
    ChildCount,
    /// Sum or count over the array elements of every record that match a value.
    ChildAggregate,
    /// Count, sum, min or max over the records whose categorical field has a value.
    AggregateWhere,
    /// Records with the lowest values of a field, or the single highest or
    /// top 4 and 5 (multi_record asks top 2 and 3).
    Rank,
}

impl TaskKind {
    /// Generation order. New kinds go last: the generator draws from one
    /// RNG stream, so appending keeps every earlier bundle reproducible.
    pub fn all() -> [TaskKind; 11] {
        [
            TaskKind::Lookup,
            TaskKind::Filter,
            TaskKind::Aggregate,
            TaskKind::Compare,
            TaskKind::MissingValue,
            TaskKind::NestedExtract,
            TaskKind::MultiRecord,
            TaskKind::ChildCount,
            TaskKind::ChildAggregate,
            TaskKind::AggregateWhere,
            TaskKind::Rank,
        ]
    }

    pub fn name(self) -> &'static str {
        match self {
            TaskKind::Lookup => "lookup",
            TaskKind::Filter => "filter",
            TaskKind::Aggregate => "aggregate",
            TaskKind::Compare => "compare",
            TaskKind::MissingValue => "missing_value",
            TaskKind::NestedExtract => "nested_extract",
            TaskKind::MultiRecord => "multi_record",
            TaskKind::ChildCount => "child_count",
            TaskKind::ChildAggregate => "child_aggregate",
            TaskKind::AggregateWhere => "aggregate_where",
            TaskKind::Rank => "rank",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub kind: TaskKind,
    pub question: String,
    pub answer: Answer,
    pub tolerance: Tolerance,
    /// Zero-based positions of the records the answer depends on.
    pub depends_on: Vec<usize>,
}

/// Picks the field that identifies records: the first field whose values
/// are all distinct scalars, preferring names containing "id".
fn key_field(records: &[Value]) -> Option<String> {
    let first = records.first()?.as_object()?;
    let candidates: Vec<&str> = first.iter().map(|(k, _)| k.as_str()).collect();
    let mut ordered: Vec<&str> = candidates
        .iter()
        .copied()
        .filter(|k| k.to_lowercase().contains("id"))
        .collect();
    ordered.extend(
        candidates
            .iter()
            .copied()
            .filter(|k| !k.to_lowercase().contains("id")),
    );
    'outer: for k in ordered {
        let mut seen: Vec<String> = Vec::with_capacity(records.len());
        for r in records {
            let Some(v) = r.get(k) else { continue 'outer };
            if !v.is_primitive() || v.is_null() {
                continue 'outer;
            }
            let s = v.to_string();
            if seen.contains(&s) {
                continue 'outer;
            }
            seen.push(s);
        }
        return Some(k.to_string());
    }
    None
}

/// Top-level keys whose values are primitive (or absent) in every record,
/// in first-seen order. Nested records still expose their scalar columns.
fn scalar_columns(records: &[Value]) -> Vec<String> {
    let mut cols: Vec<String> = Vec::new();
    for r in records {
        if let Some(o) = r.as_object() {
            for (k, _) in o {
                if !cols.contains(k) {
                    cols.push(k.clone());
                }
            }
        }
    }
    cols.retain(|c| {
        records
            .iter()
            .all(|r| r.get(c).is_none_or(Value::is_primitive))
    });
    cols
}

fn numeric_fields(records: &[Value], columns: &[String]) -> Vec<String> {
    columns
        .iter()
        .filter(|c| {
            let mut any = false;
            records.iter().all(|r| match r.get(c) {
                Some(Value::Number(_)) => {
                    any = true;
                    true
                }
                Some(Value::Null) | None => true,
                _ => false,
            }) && any
        })
        .cloned()
        .collect()
}

fn describe(v: &Value) -> String {
    match v {
        Value::String(s) => format!("\"{s}\""),
        other => other.to_string(),
    }
}

pub struct Generator<'a> {
    pub records: &'a [Value],
    pub rng: Rng,
    pub per_kind: usize,
}

impl<'a> Generator<'a> {
    pub fn new(records: &'a [Value], seed: u64, per_kind: usize) -> Self {
        Generator {
            records,
            rng: Rng::new(seed),
            per_kind,
        }
    }

    /// Tasks for every kind in [`TaskKind::all`] order. The first seven
    /// kinds keep their original attempt budget (20 per wanted task) and the
    /// later kinds get 100; a final top-up pass gives a short early kind the
    /// difference. Everything new draws from the RNG after the original
    /// stream, so earlier bundles regenerate as an exact prefix.
    pub fn generate(&mut self) -> Vec<Task> {
        let mut tasks = Vec::new();
        let Some(key) = key_field(self.records) else {
            return tasks;
        };
        let columns = scalar_columns(self.records);
        let mut made = [0usize; 11];
        let original = 7;
        for (i, kind) in TaskKind::all().into_iter().enumerate() {
            let budget = self.per_kind * if i < original { 20 } else { 100 };
            self.fill(kind, budget, &key, &columns, &mut made[i], &mut tasks);
        }
        for (i, kind) in TaskKind::all().into_iter().enumerate().take(original) {
            if made[i] < self.per_kind {
                let budget = self.per_kind * 80;
                self.fill(kind, budget, &key, &columns, &mut made[i], &mut tasks);
            }
        }
        tasks
    }

    fn fill(
        &mut self,
        kind: TaskKind,
        budget: usize,
        key: &str,
        columns: &[String],
        made: &mut usize,
        tasks: &mut Vec<Task>,
    ) {
        let n = self.records.len();
        let mut attempts = 0;
        while *made < self.per_kind && attempts < budget {
            attempts += 1;
            let idx = self.rng.below(n as u64) as usize;
            let task = match kind {
                TaskKind::Lookup => self.lookup(key, idx),
                TaskKind::Filter => self.filter(key, columns),
                TaskKind::Aggregate => self.aggregate(columns),
                TaskKind::Compare => self.compare(key, columns, idx),
                TaskKind::MissingValue => self.missing(key, columns, idx),
                TaskKind::NestedExtract => self.nested(key, idx),
                TaskKind::MultiRecord => self.multi(key, columns),
                TaskKind::ChildCount => self.child_count(key, idx),
                TaskKind::ChildAggregate => self.child_aggregate(),
                TaskKind::AggregateWhere => self.aggregate_where(columns),
                TaskKind::Rank => self.rank(key, columns),
            };
            if let Some(mut t) = task {
                t.id = format!("{}-{:02}", kind.name(), *made + 1);
                if tasks.iter().any(|x: &Task| x.question == t.question) {
                    continue;
                }
                tasks.push(t);
                *made += 1;
            }
        }
    }

    fn key_text(&self, key: &str, idx: usize) -> String {
        describe(self.records[idx].get(key).unwrap())
    }

    fn lookup(&mut self, key: &str, idx: usize) -> Option<Task> {
        let rec = self.records[idx].as_object()?;
        let candidates: Vec<&(String, Value)> = rec
            .iter()
            .filter(|(k, v)| k != key && v.is_primitive())
            .collect();
        if candidates.is_empty() {
            return None;
        }
        let (field, value) = self.rng.pick(&candidates);
        Some(Task {
            id: String::new(),
            kind: TaskKind::Lookup,
            question: format!(
                "What is the value of the field {field:?} for the record whose {key:?} is {}?",
                self.key_text(key, idx)
            ),
            answer: Answer::from_value(value)?,
            tolerance: Tolerance::default(),
            depends_on: vec![idx],
        })
    }

    fn filter(&mut self, key: &str, columns: &[String]) -> Option<Task> {
        let nums = numeric_fields(self.records, columns);
        if nums.is_empty() {
            return None;
        }
        let field = self.rng.pick(&nums).clone();
        let values: Vec<(usize, f64)> = self
            .records
            .iter()
            .enumerate()
            .filter_map(|(i, r)| {
                r.get(&field)
                    .and_then(Value::as_number)
                    .map(|n| (i, n.as_f64()))
            })
            .collect();
        if values.len() < 3 {
            return None;
        }
        // Threshold at a random observed value so the result set is 1..n-1.
        let pivot = self.rng.pick(&values).1;
        let greater = self.rng.chance(0.5);
        let matching: Vec<&(usize, f64)> = values
            .iter()
            .filter(|(_, v)| if greater { *v > pivot } else { *v < pivot })
            .collect();
        if matching.is_empty() || matching.len() > 12 {
            return None;
        }
        let ids: Vec<Scalar> = matching
            .iter()
            .map(|(i, _)| Scalar::from_value(self.records[*i].get(key).unwrap()).unwrap())
            .collect();
        let pivot_text = Number::F64(pivot).canonical_text()?;
        Some(Task {
            id: String::new(),
            kind: TaskKind::Filter,
            question: format!(
                "List the {key:?} of every record whose {field:?} is {} {pivot_text}. Answer with a JSON array.",
                if greater { "greater than" } else { "less than" }
            ),
            answer: Answer::Set(ids),
            tolerance: Tolerance::default(),
            depends_on: matching.iter().map(|(i, _)| *i).collect(),
        })
    }

    fn aggregate(&mut self, columns: &[String]) -> Option<Task> {
        let nums = numeric_fields(self.records, columns);
        if nums.is_empty() {
            return None;
        }
        let field = self.rng.pick(&nums).clone();
        let values: Vec<(usize, f64)> = self
            .records
            .iter()
            .enumerate()
            .filter_map(|(i, r)| {
                r.get(&field)
                    .and_then(Value::as_number)
                    .map(|n| (i, n.as_f64()))
            })
            .collect();
        if values.is_empty() {
            return None;
        }
        let depends: Vec<usize> = values.iter().map(|(i, _)| *i).collect();
        let (op, answer, tol) = match self.rng.below(5) {
            0 => (
                "How many records have a non-null",
                Answer::Number(values.len() as f64),
                Tolerance::default(),
            ),
            1 => (
                "What is the sum of",
                Answer::Number(values.iter().map(|v| v.1).sum()),
                Tolerance::aggregate(),
            ),
            2 => (
                "What is the minimum of",
                Answer::Number(values.iter().map(|v| v.1).fold(f64::INFINITY, f64::min)),
                Tolerance::default(),
            ),
            3 => (
                "What is the maximum of",
                Answer::Number(values.iter().map(|v| v.1).fold(f64::NEG_INFINITY, f64::max)),
                Tolerance::default(),
            ),
            _ => (
                "What is the mean of",
                Answer::Number(values.iter().map(|v| v.1).sum::<f64>() / values.len() as f64),
                Tolerance::aggregate(),
            ),
        };
        let question = if op.starts_with("How many") {
            format!("{op} {field:?} value? Answer with a number.")
        } else {
            format!("{op} the field {field:?} across all records? Answer with a number.")
        };
        Some(Task {
            id: String::new(),
            kind: TaskKind::Aggregate,
            question,
            answer,
            tolerance: tol,
            depends_on: depends,
        })
    }

    fn compare(&mut self, key: &str, columns: &[String], idx: usize) -> Option<Task> {
        let nums = numeric_fields(self.records, columns);
        if nums.is_empty() {
            return None;
        }
        let field = self.rng.pick(&nums).clone();
        let other = self.rng.below(self.records.len() as u64) as usize;
        if other == idx {
            return None;
        }
        let a = self.records[idx].get(&field)?.as_number()?.as_f64();
        let b = self.records[other].get(&field)?.as_number()?.as_f64();
        if a == b {
            return None;
        }
        let winner = if a > b { idx } else { other };
        Some(Task {
            id: String::new(),
            kind: TaskKind::Compare,
            question: format!(
                "Which record has the larger {field:?}: the one whose {key:?} is {} or the one whose {key:?} is {}? Answer with that record's {key:?}.",
                self.key_text(key, idx),
                self.key_text(key, other)
            ),
            answer: Answer::from_value(self.records[winner].get(key)?)?,
            tolerance: Tolerance::default(),
            depends_on: vec![idx, other],
        })
    }

    fn missing(&mut self, key: &str, columns: &[String], idx: usize) -> Option<Task> {
        let rec = self.records[idx].as_object()?;
        let absent: Vec<&String> = columns
            .iter()
            .filter(|c| rec.iter().all(|(k, _)| k != *c))
            .collect();
        let nulls: Vec<&String> = rec
            .iter()
            .filter(|(_, v)| v.is_null())
            .map(|(k, _)| k)
            .collect();
        let field = if !absent.is_empty() && (nulls.is_empty() || self.rng.chance(0.5)) {
            (*self.rng.pick(&absent)).clone()
        } else if !nulls.is_empty() {
            (*self.rng.pick(&nulls)).clone()
        } else {
            return None;
        };
        Some(Task {
            id: String::new(),
            kind: TaskKind::MissingValue,
            question: format!(
                "What is the value of the field {field:?} for the record whose {key:?} is {}? If the field is absent or null for that record, answer null.",
                self.key_text(key, idx)
            ),
            answer: Answer::Null,
            tolerance: Tolerance::default(),
            depends_on: vec![idx],
        })
    }

    fn nested(&mut self, key: &str, idx: usize) -> Option<Task> {
        fn paths(v: &Value, prefix: &str, out: &mut Vec<(String, Value)>) {
            match v {
                Value::Object(o) => {
                    for (k, child) in o {
                        let p = if prefix.is_empty() {
                            k.clone()
                        } else {
                            format!("{prefix}.{k}")
                        };
                        paths(child, &p, out);
                    }
                }
                Value::Array(a) => {
                    for (i, child) in a.iter().enumerate() {
                        paths(child, &format!("{prefix}.{i}"), out);
                    }
                }
                prim => {
                    if prefix.contains('.') {
                        out.push((prefix.to_string(), prim.clone()));
                    }
                }
            }
        }
        let mut leaves = Vec::new();
        paths(&self.records[idx], "", &mut leaves);
        if leaves.is_empty() {
            return None;
        }
        let (path, value) = self.rng.pick(&leaves).clone();
        Some(Task {
            id: String::new(),
            kind: TaskKind::NestedExtract,
            question: format!(
                "For the record whose {key:?} is {}, what is the value at the nested path {path:?} (segments separated by dots, numeric segments are zero-based array indices)?",
                self.key_text(key, idx)
            ),
            answer: Answer::from_value(&value)?,
            tolerance: Tolerance::default(),
            depends_on: vec![idx],
        })
    }

    fn multi(&mut self, key: &str, columns: &[String]) -> Option<Task> {
        let nums = numeric_fields(self.records, columns);
        if nums.is_empty() {
            return None;
        }
        let field = self.rng.pick(&nums).clone();
        let mut values: Vec<(usize, f64)> = self
            .records
            .iter()
            .enumerate()
            .filter_map(|(i, r)| {
                r.get(&field)
                    .and_then(Value::as_number)
                    .map(|n| (i, n.as_f64()))
            })
            .collect();
        if values.len() < 4 {
            return None;
        }
        let k = 2 + self.rng.below(2) as usize;
        values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        // Require a strict gap after position k so the answer is unambiguous.
        if values[k - 1].1 == values[k].1 {
            return None;
        }
        let top: Vec<&(usize, f64)> = values.iter().take(k).collect();
        Some(Task {
            id: String::new(),
            kind: TaskKind::MultiRecord,
            question: format!(
                "Which {k} records have the highest {field:?}? Answer with a JSON array of their {key:?} values, highest first."
            ),
            answer: Answer::List(
                top.iter()
                    .map(|(i, _)| Scalar::from_value(self.records[*i].get(key).unwrap()).unwrap())
                    .collect(),
            ),
            tolerance: Tolerance::default(),
            depends_on: top.iter().map(|(i, _)| *i).collect(),
        })
    }
}

/// Top-level fields whose present values are arrays of objects, with at
/// least one non-empty array, in first-seen order.
fn child_fields(records: &[Value]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for r in records {
        for (k, v) in r.as_object().into_iter().flatten() {
            if !out.contains(k)
                && matches!(v, Value::Array(a) if !a.is_empty() && a.iter().all(|x| x.as_object().is_some()))
            {
                out.push(k.clone());
            }
        }
    }
    out.retain(|f| {
        records.iter().all(|r| match r.get(f) {
            None | Some(Value::Null) => true,
            Some(Value::Array(a)) => a.iter().all(|x| x.as_object().is_some()),
            _ => false,
        })
    });
    out
}

/// Scalar string or boolean columns with 2 to 20 distinct values.
fn categorical_fields(records: &[Value], columns: &[String]) -> Vec<String> {
    columns
        .iter()
        .filter(|c| {
            let mut seen: Vec<String> = Vec::new();
            for r in records {
                match r.get(c) {
                    Some(v @ (Value::String(_) | Value::Bool(_))) => {
                        let t = v.to_string();
                        if !seen.contains(&t) {
                            seen.push(t);
                        }
                        if seen.len() > 20 {
                            return false;
                        }
                    }
                    None | Some(Value::Null) => {}
                    _ => return false,
                }
            }
            seen.len() >= 2
        })
        .cloned()
        .collect()
}

impl Generator<'_> {
    fn aggregate_where(&mut self, columns: &[String]) -> Option<Task> {
        let cats = categorical_fields(self.records, columns);
        if cats.is_empty() {
            return None;
        }
        let cat = self.rng.pick(&cats).clone();
        let idx = self.rng.below(self.records.len() as u64) as usize;
        let value = self.records[idx].get(&cat)?.clone();
        if value.is_null() {
            return None;
        }
        let rows: Vec<usize> = (0..self.records.len())
            .filter(|&i| self.records[i].get(&cat) == Some(&value))
            .collect();
        let nums: Vec<String> = numeric_fields(self.records, columns)
            .into_iter()
            .filter(|n| *n != cat && key_field(self.records).as_deref() != Some(n.as_str()))
            .collect();
        let op = if nums.is_empty() {
            0
        } else {
            self.rng.below(4)
        };
        let vtext = describe(&value);
        if op == 0 {
            return Some(Task {
                id: String::new(),
                kind: TaskKind::AggregateWhere,
                question: format!(
                    "How many records have {cat:?} equal to {vtext}? Answer with a number."
                ),
                answer: Answer::Number(rows.len() as f64),
                tolerance: Tolerance::default(),
                depends_on: rows,
            });
        }
        let num = self.rng.pick(&nums).clone();
        let vals: Vec<(usize, Number)> = rows
            .iter()
            .filter_map(|&i| Some((i, self.records[i].get(&num)?.as_number()?)))
            .collect();
        if vals.is_empty() {
            return None;
        }
        let all_int = vals.iter().all(|(_, n)| !matches!(n, Number::F64(_)));
        let fs = vals.iter().map(|(_, n)| n.as_f64());
        let (what, answer, tol) = match op {
            1 => (
                "sum",
                fs.sum::<f64>(),
                if all_int {
                    Tolerance::default()
                } else {
                    Tolerance::aggregate()
                },
            ),
            2 => (
                "minimum",
                fs.fold(f64::INFINITY, f64::min),
                Tolerance::default(),
            ),
            _ => (
                "maximum",
                fs.fold(f64::NEG_INFINITY, f64::max),
                Tolerance::default(),
            ),
        };
        Some(Task {
            id: String::new(),
            kind: TaskKind::AggregateWhere,
            question: format!(
                "Among the records whose {cat:?} is {vtext}, what is the {what} of {num:?}? Answer with a number."
            ),
            answer: Answer::Number(answer),
            tolerance: tol,
            depends_on: vals.iter().map(|(i, _)| *i).collect(),
        })
    }

    fn rank(&mut self, key: &str, columns: &[String]) -> Option<Task> {
        // The key field is excluded: its order is often the row order.
        let nums: Vec<String> = numeric_fields(self.records, columns)
            .into_iter()
            .filter(|n| n != key)
            .collect();
        if nums.is_empty() {
            return None;
        }
        let field = self.rng.pick(&nums).clone();
        let low = self.rng.chance(0.5);
        let k = if low {
            1 + self.rng.below(5) as usize
        } else {
            *self.rng.pick(&[1usize, 4, 5])
        };
        let mut values: Vec<(usize, f64)> = self
            .records
            .iter()
            .enumerate()
            .filter_map(|(i, r)| {
                r.get(&field)
                    .and_then(Value::as_number)
                    .map(|n| (i, n.as_f64()))
            })
            .collect();
        if values.len() <= k {
            return None;
        }
        values.sort_by(|a, b| {
            if low {
                a.1.total_cmp(&b.1)
            } else {
                b.1.total_cmp(&a.1)
            }
        });
        if values[k - 1].1 == values[k].1 {
            return None;
        }
        let top = &values[..k];
        let (dir, order) = if low {
            ("lowest", "lowest first")
        } else {
            ("highest", "highest first")
        };
        let question = if k == 1 {
            format!(
                "Which record has the {dir} {field:?}? Answer with a JSON array holding its {key:?} value."
            )
        } else {
            format!(
                "List the {k} records with the {dir} {field:?}. Answer with a JSON array of their {key:?} values, {order}."
            )
        };
        Some(Task {
            id: String::new(),
            kind: TaskKind::Rank,
            question,
            answer: Answer::List(
                top.iter()
                    .map(|(i, _)| Scalar::from_value(self.records[*i].get(key)?))
                    .collect::<Option<Vec<_>>>()?,
            ),
            tolerance: Tolerance::default(),
            depends_on: top.iter().map(|(i, _)| *i).collect(),
        })
    }

    fn child_count(&mut self, key: &str, idx: usize) -> Option<Task> {
        let fields = child_fields(self.records);
        if fields.is_empty() {
            return None;
        }
        let field = self.rng.pick(&fields).clone();
        let n = self.records[idx].get(&field)?.as_array()?.len();
        Some(Task {
            id: String::new(),
            kind: TaskKind::ChildCount,
            question: format!(
                "For the record whose {key:?} is {}, how many elements does its {field:?} array have? Answer with a number.",
                self.key_text(key, idx)
            ),
            answer: Answer::Number(n as f64),
            tolerance: Tolerance::default(),
            depends_on: vec![idx],
        })
    }

    /// Matches array elements across all records on a string field shared
    /// by several elements, then counts them or sums a numeric field.
    fn child_aggregate(&mut self) -> Option<Task> {
        let fields = child_fields(self.records);
        if fields.is_empty() {
            return None;
        }
        let field = self.rng.pick(&fields).clone();
        let elements: Vec<(usize, &ldf_data::Object)> = self
            .records
            .iter()
            .enumerate()
            .flat_map(|(i, r)| {
                r.get(&field)
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(move |e| e.as_object().map(|o| (i, o)))
            })
            .collect();
        let (_, sample) = *self.rng.pick(&elements);
        let strings: Vec<(String, String)> = sample
            .iter()
            .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
            .collect();
        if strings.is_empty() {
            return None;
        }
        let (match_key, match_val) = self.rng.pick(&strings).clone();
        let matching: Vec<&(usize, &ldf_data::Object)> = elements
            .iter()
            .filter(|(_, o)| {
                o.iter()
                    .any(|(k, v)| *k == match_key && v.as_str() == Some(match_val.as_str()))
            })
            .collect();
        if matching.len() < 2 {
            return None;
        }
        let mut depends: Vec<usize> = matching.iter().map(|(i, _)| *i).collect();
        depends.dedup();
        let numeric: Vec<String> = sample
            .iter()
            .filter(|(_, v)| matches!(v, Value::Number(_)))
            .map(|(k, _)| k.clone())
            .collect();
        let sum_of = if numeric.is_empty() || self.rng.below(2) == 0 {
            None
        } else {
            Some(self.rng.pick(&numeric).clone())
        };
        let (question, answer, tolerance) = match sum_of {
            None => (
                format!(
                    "Across all records, how many elements of the {field:?} arrays have {match_key:?} equal to {match_val:?}? Answer with a number."
                ),
                Answer::Number(matching.len() as f64),
                Tolerance::default(),
            ),
            Some(num) => {
                let mut all_int = true;
                let mut total = 0.0;
                for (_, o) in &matching {
                    match o.iter().find(|(k, _)| *k == num).map(|(_, v)| v) {
                        Some(Value::Number(n)) => {
                            all_int &= !matches!(n, Number::F64(_));
                            total += n.as_f64();
                        }
                        _ => return None,
                    }
                }
                (
                    format!(
                        "Across all records, what is the sum of {num:?} over the elements of the {field:?} arrays whose {match_key:?} is {match_val:?}? Answer with a number."
                    ),
                    Answer::Number(total),
                    if all_int {
                        Tolerance::default()
                    } else {
                        Tolerance::aggregate()
                    },
                )
            }
        };
        Some(Task {
            id: String::new(),
            kind: TaskKind::ChildAggregate,
            question,
            answer,
            tolerance,
            depends_on: depends,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldf_fixtures::{DEFAULT_SEED, find};

    #[test]
    fn generates_every_kind_for_flat_and_nested_data() {
        let recs: Vec<Value> = find("flat_users")
            .unwrap()
            .records(DEFAULT_SEED, 30)
            .collect();
        let tasks = Generator::new(&recs, 1, 2).generate();
        let kinds: std::collections::HashSet<TaskKind> = tasks.iter().map(|t| t.kind).collect();
        assert!(kinds.contains(&TaskKind::Lookup));
        assert!(kinds.contains(&TaskKind::Filter));
        assert!(kinds.contains(&TaskKind::Aggregate));
        assert!(kinds.contains(&TaskKind::Compare));
        assert!(kinds.contains(&TaskKind::MultiRecord));

        let recs: Vec<Value> = find("nested_orders")
            .unwrap()
            .records(DEFAULT_SEED, 30)
            .collect();
        let tasks = Generator::new(&recs, 1, 2).generate();
        assert!(tasks.iter().any(|t| t.kind == TaskKind::NestedExtract));
        assert!(tasks.iter().any(|t| t.kind == TaskKind::ChildCount));
        assert!(tasks.iter().any(|t| t.kind == TaskKind::ChildAggregate));

        let recs: Vec<Value> = find("sparse_events")
            .unwrap()
            .records(DEFAULT_SEED, 30)
            .collect();
        let tasks = Generator::new(&recs, 1, 2).generate();
        assert!(tasks.iter().any(|t| t.kind == TaskKind::MissingValue));
    }

    #[test]
    fn deterministic() {
        let recs: Vec<Value> = find("flat_users")
            .unwrap()
            .records(DEFAULT_SEED, 30)
            .collect();
        let a = Generator::new(&recs, 5, 3).generate();
        let b = Generator::new(&recs, 5, 3).generate();
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
    }
}

#[cfg(test)]
mod stream_tests {
    use super::*;
    use ldf_fixtures::{DEFAULT_SEED, find};

    /// Built by the generator before child_count existed (commit 959ebb6),
    /// same records, seed 1, three per kind.
    const GOLDEN: [&str; 15] = [
        "What is the value of the field \"active\" for the record whose \"id\" is 22?",
        "What is the value of the field \"age\" for the record whose \"id\" is 18?",
        "What is the value of the field \"email\" for the record whose \"id\" is 21?",
        "List the \"id\" of every record whose \"score\" is less than 18.31. Answer with a JSON array.",
        "List the \"id\" of every record whose \"score\" is greater than 80.33. Answer with a JSON array.",
        "List the \"id\" of every record whose \"age\" is greater than 76. Answer with a JSON array.",
        "What is the maximum of the field \"score\" across all records? Answer with a number.",
        "What is the mean of the field \"id\" across all records? Answer with a number.",
        "What is the maximum of the field \"age\" across all records? Answer with a number.",
        "Which record has the larger \"id\": the one whose \"id\" is 8 or the one whose \"id\" is 26? Answer with that record's \"id\".",
        "Which record has the larger \"age\": the one whose \"id\" is 12 or the one whose \"id\" is 3? Answer with that record's \"id\".",
        "Which record has the larger \"age\": the one whose \"id\" is 1 or the one whose \"id\" is 20? Answer with that record's \"id\".",
        "Which 2 records have the highest \"id\"? Answer with a JSON array of their \"id\" values, highest first.",
        "Which 3 records have the highest \"age\"? Answer with a JSON array of their \"id\" values, highest first.",
        "Which 2 records have the highest \"age\"? Answer with a JSON array of their \"id\" values, highest first.",
    ];

    /// The first seven kinds draw the same RNG stream as before the later
    /// kinds existed, so bundles built then regenerate as a prefix.
    #[test]
    fn original_kinds_keep_their_stream() {
        let recs: Vec<Value> = find("flat_users")
            .unwrap()
            .records(DEFAULT_SEED, 30)
            .collect();
        let tasks = Generator::new(&recs, 1, 3).generate();
        let head: Vec<&str> = tasks
            .iter()
            .take(GOLDEN.len())
            .map(|t| t.question.as_str())
            .collect();
        assert_eq!(head, GOLDEN);
    }

    #[test]
    fn new_kinds_apply_to_flat_data_without_the_key_field() {
        let recs: Vec<Value> = find("flat_users")
            .unwrap()
            .records(DEFAULT_SEED, 200)
            .collect();
        let tasks = Generator::new(&recs, 1, 15).generate();
        for kind in [TaskKind::AggregateWhere, TaskKind::Rank] {
            let n = tasks.iter().filter(|t| t.kind == kind).count();
            assert!(n >= 5, "{kind:?}: {n}");
        }
        assert!(tasks.iter().filter(|t| t.kind == TaskKind::Rank).all(|t| {
            !t.question.contains("highest \"id\"") && !t.question.contains("lowest \"id\"")
        }));
    }
}
