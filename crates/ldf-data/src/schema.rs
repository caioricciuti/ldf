//! Flat schema inference.
//!
//! Row-oriented formats (CSV, TSV, Markdown tables, TOON's tabular form)
//! need to know whether a dataset is a flat table. Inference is strict and
//! explicit: a dataset is flat only if every record is an object whose
//! values are all primitives. Column order is the union of keys in first-seen
//! order, so sparse records are allowed (missing cells are reported as such
//! and are *not* the same as nulls; formats decide how to render each).

use std::collections::HashMap;
use std::fmt;

use crate::value::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaError {
    /// A record is not an object.
    NonObjectRecord { index: usize, found: &'static str },
    /// A field holds an array or object.
    NestedField {
        index: usize,
        key: String,
        found: &'static str,
    },
    /// A record has the same key twice.
    DuplicateKey { index: usize, key: String },
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchemaError::NonObjectRecord { index, found } => {
                write!(f, "record {index} is a {found}, not an object")
            }
            SchemaError::NestedField { index, key, found } => {
                write!(f, "record {index} field {key:?} is a {found} (nested)")
            }
            SchemaError::DuplicateKey { index, key } => {
                write!(f, "record {index} repeats key {key:?}")
            }
        }
    }
}

impl std::error::Error for SchemaError {}

#[derive(Debug, Clone, Default)]
pub struct FlatSchema {
    pub columns: Vec<String>,
    /// True when every record has every column (no missing cells).
    pub uniform: bool,
    /// True when every record lists the columns in the same order.
    pub same_order: bool,
    index: HashMap<String, usize>,
}

impl FlatSchema {
    /// Infers the flat schema or reports the first violation.
    pub fn infer(records: &[Value]) -> Result<FlatSchema, SchemaError> {
        let mut schema = FlatSchema {
            uniform: true,
            same_order: true,
            ..Default::default()
        };
        let mut seen_in_record: Vec<bool> = Vec::new();
        for (index, rec) in records.iter().enumerate() {
            let Value::Object(fields) = rec else {
                return Err(SchemaError::NonObjectRecord {
                    index,
                    found: rec.type_name(),
                });
            };
            seen_in_record.clear();
            seen_in_record.resize(schema.columns.len(), false);
            for (expected_pos, (key, val)) in fields.iter().enumerate() {
                if !val.is_primitive() {
                    return Err(SchemaError::NestedField {
                        index,
                        key: key.clone(),
                        found: val.type_name(),
                    });
                }
                let pos = match schema.index.get(key) {
                    Some(&p) => p,
                    None => {
                        if index > 0 {
                            schema.uniform = false;
                        }
                        let p = schema.columns.len();
                        schema.columns.push(key.clone());
                        schema.index.insert(key.clone(), p);
                        seen_in_record.push(false);
                        p
                    }
                };
                if seen_in_record[pos] {
                    return Err(SchemaError::DuplicateKey {
                        index,
                        key: key.clone(),
                    });
                }
                seen_in_record[pos] = true;
                if pos != expected_pos {
                    schema.same_order = false;
                }
            }
            if fields.len() != schema.columns.len() {
                schema.uniform = false;
            }
        }
        Ok(schema)
    }

    pub fn column_index(&self, key: &str) -> Option<usize> {
        self.index.get(key).copied()
    }

    pub fn len(&self) -> usize {
        self.columns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obj;

    #[test]
    fn infers_union_in_first_seen_order() {
        let recs = vec![obj! {"a" => 1, "b" => "x"}, obj! {"b" => "y", "c" => true}];
        let s = FlatSchema::infer(&recs).unwrap();
        assert_eq!(s.columns, ["a", "b", "c"]);
        assert!(!s.uniform);
        assert!(!s.same_order);
    }

    #[test]
    fn rejects_nested() {
        let recs = vec![obj! {"a" => Value::Array(vec![])}];
        assert!(matches!(
            FlatSchema::infer(&recs),
            Err(SchemaError::NestedField { .. })
        ));
    }
}
