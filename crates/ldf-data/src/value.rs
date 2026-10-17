//! The [`Value`] tree, its serde bridge, and equality notions.

use std::fmt;

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};

use crate::number::Number;

/// Ordered object: insertion order is preserved and duplicate keys are not
/// deduplicated at this level (readers reject or keep-last, documented per
/// reader).
pub type Object = Vec<(String, Value)>;

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(Object),
}

impl Value {
    pub fn is_primitive(&self) -> bool {
        !matches!(self, Value::Array(_) | Value::Object(_))
    }

    pub fn as_object(&self) -> Option<&Object> {
        match self {
            Value::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<Number> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Field lookup on an object (linear scan; records are small).
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.as_object()?
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    /// Dotted-path lookup, e.g. `customer.address.city`. Array indices are
    /// numeric segments.
    pub fn get_path(&self, path: &str) -> Option<&Value> {
        let mut cur = self;
        for seg in path.split('.') {
            cur = match cur {
                Value::Object(_) => cur.get(seg)?,
                Value::Array(a) => a.get(seg.parse::<usize>().ok()?)?,
                _ => return None,
            };
        }
        Some(cur)
    }

    /// Name of the JSON type, for error messages and schema inference.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }

    /// Semantic equality: numbers compare by mathematical value, everything
    /// else structurally, key order significant.
    pub fn semantic_eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a.semantic_eq(*b),
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.semantic_eq(y))
            }
            (Value::Object(a), Value::Object(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .zip(b)
                        .all(|((ka, va), (kb, vb))| ka == kb && va.semantic_eq(vb))
            }
            _ => false,
        }
    }

    /// Like [`Value::semantic_eq`] but object key order is ignored. Formats
    /// such as TOON's tabular form legitimately reorder keys to the header
    /// order; the benchmark reports this equality level separately.
    pub fn semantic_eq_unordered(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Array(a), Value::Array(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.semantic_eq_unordered(y))
            }
            (Value::Object(a), Value::Object(b)) => {
                a.len() == b.len()
                    && a.iter().all(|(k, va)| {
                        b.iter()
                            .find(|(kb, _)| kb == k)
                            .is_some_and(|(_, vb)| va.semantic_eq_unordered(vb))
                    })
            }
            _ => self.semantic_eq(other),
        }
    }

    /// First path (dotted) at which two values differ semantically, for
    /// diagnostics. `None` when equal.
    pub fn first_difference(&self, other: &Value) -> Option<String> {
        fn walk(a: &Value, b: &Value, path: &mut Vec<String>) -> Option<String> {
            match (a, b) {
                (Value::Array(x), Value::Array(y)) => {
                    if x.len() != y.len() {
                        return Some(format!(
                            "{}: array length {} vs {}",
                            path.join("."),
                            x.len(),
                            y.len()
                        ));
                    }
                    for (i, (p, q)) in x.iter().zip(y).enumerate() {
                        path.push(i.to_string());
                        if let Some(d) = walk(p, q, path) {
                            return Some(d);
                        }
                        path.pop();
                    }
                    None
                }
                (Value::Object(x), Value::Object(y)) => {
                    if x.len() != y.len() {
                        return Some(format!(
                            "{}: object size {} vs {}",
                            path.join("."),
                            x.len(),
                            y.len()
                        ));
                    }
                    for ((ka, p), (kb, q)) in x.iter().zip(y) {
                        if ka != kb {
                            return Some(format!("{}: key {ka:?} vs {kb:?}", path.join(".")));
                        }
                        path.push(ka.clone());
                        if let Some(d) = walk(p, q, path) {
                            return Some(d);
                        }
                        path.pop();
                    }
                    None
                }
                _ => {
                    if a.semantic_eq(b) {
                        None
                    } else {
                        Some(format!("{}: {a} vs {b}", path.join(".")))
                    }
                }
            }
        }
        walk(self, other, &mut Vec::new())
    }

    /// Approximate heap footprint, used only to sanity check memory numbers.
    pub fn approx_size(&self) -> usize {
        std::mem::size_of::<Value>()
            + match self {
                Value::String(s) => s.capacity(),
                Value::Array(a) => a.iter().map(Value::approx_size).sum(),
                Value::Object(o) => o.iter().map(|(k, v)| k.capacity() + v.approx_size()).sum(),
                _ => 0,
            }
    }
}

/// Debug-friendly display: compact JSON.
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_json::to_string(self).map_err(|_| fmt::Error)?;
        f.write_str(&s)
    }
}

impl PartialEq for Value {
    /// Strict structural equality: `1` and `1.0` are *not* equal here.
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => match (a, b) {
                (Number::I64(x), Number::I64(y)) => x == y,
                (Number::U64(x), Number::U64(y)) => x == y,
                (Number::F64(x), Number::F64(y)) => x.to_bits() == y.to_bits(),
                _ => false,
            },
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            _ => false,
        }
    }
}

// ---- serde bridge -----------------------------------------------------

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::Null => s.serialize_unit(),
            Value::Bool(b) => s.serialize_bool(*b),
            Value::Number(Number::I64(v)) => s.serialize_i64(*v),
            Value::Number(Number::U64(v)) => s.serialize_u64(*v),
            Value::Number(Number::F64(v)) => s.serialize_f64(*v),
            Value::String(v) => s.serialize_str(v),
            Value::Array(a) => {
                let mut seq = s.serialize_seq(Some(a.len()))?;
                for v in a {
                    seq.serialize_element(v)?;
                }
                seq.end()
            }
            Value::Object(o) => {
                let mut map = s.serialize_map(Some(o.len()))?;
                for (k, v) in o {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Value, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Value;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("any JSON value")
            }
            fn visit_bool<E>(self, v: bool) -> Result<Value, E> {
                Ok(Value::Bool(v))
            }
            fn visit_i64<E>(self, v: i64) -> Result<Value, E> {
                Ok(Value::Number(Number::I64(v)))
            }
            fn visit_u64<E>(self, v: u64) -> Result<Value, E> {
                Ok(Value::Number(Number::from(v)))
            }
            fn visit_i128<E: de::Error>(self, v: i128) -> Result<Value, E> {
                // Out of documented domain; nearest f64 (documented loss).
                Ok(Value::Number(Number::F64(v as f64)))
            }
            fn visit_u128<E: de::Error>(self, v: u128) -> Result<Value, E> {
                Ok(Value::Number(Number::F64(v as f64)))
            }
            fn visit_f64<E>(self, v: f64) -> Result<Value, E> {
                Ok(Value::Number(Number::F64(v)))
            }
            fn visit_str<E>(self, v: &str) -> Result<Value, E> {
                Ok(Value::String(v.to_owned()))
            }
            fn visit_string<E>(self, v: String) -> Result<Value, E> {
                Ok(Value::String(v))
            }
            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }
            fn visit_none<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }
            fn visit_some<D2: Deserializer<'de>>(self, d: D2) -> Result<Value, D2::Error> {
                Deserialize::deserialize(d)
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
                let mut out = Vec::with_capacity(seq.size_hint().unwrap_or(0).min(1 << 16));
                while let Some(v) = seq.next_element()? {
                    out.push(v);
                }
                Ok(Value::Array(out))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
                let mut out = Vec::with_capacity(map.size_hint().unwrap_or(0).min(1 << 12));
                while let Some((k, v)) = map.next_entry::<String, Value>()? {
                    out.push((k, v));
                }
                Ok(Value::Object(out))
            }
        }
        d.deserialize_any(V)
    }
}

// ---- constructors -----------------------------------------------------

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_owned())
    }
}
impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}
impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}
impl<T: Into<Number>> From<T> for Value {
    fn from(v: T) -> Self {
        Value::Number(v.into())
    }
}
impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::Array(v)
    }
}
impl From<Object> for Value {
    fn from(v: Object) -> Self {
        Value::Object(v)
    }
}

/// Small helper to build objects in tests and generators.
#[macro_export]
macro_rules! obj {
    ($($k:expr => $v:expr),* $(,)?) => {
        $crate::Value::Object(vec![$(($k.to_string(), $crate::Value::from($v))),*])
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip_preserves_key_order_and_number_kinds() {
        let src = r#"{"z":1,"a":1.0,"m":[true,null,"x"],"big":18446744073709551615}"#;
        let v: Value = serde_json::from_str(src).unwrap();
        assert_eq!(serde_json::to_string(&v).unwrap(), src);
        assert!(matches!(v.get("z"), Some(Value::Number(Number::I64(1)))));
        assert!(matches!(v.get("a"), Some(Value::Number(Number::F64(_)))));
        assert!(matches!(v.get("big"), Some(Value::Number(Number::U64(_)))));
    }

    #[test]
    fn equality_levels() {
        let a: Value = serde_json::from_str(r#"{"x":1,"y":{"p":2,"q":3}}"#).unwrap();
        let b: Value = serde_json::from_str(r#"{"x":1.0,"y":{"p":2,"q":3}}"#).unwrap();
        let c: Value = serde_json::from_str(r#"{"y":{"q":3,"p":2},"x":1}"#).unwrap();
        assert_ne!(a, b);
        assert!(a.semantic_eq(&b));
        assert!(!a.semantic_eq(&c));
        assert!(a.semantic_eq_unordered(&c));
        assert_eq!(a.first_difference(&b), None);
        assert!(a.first_difference(&c).is_some());
    }

    #[test]
    fn paths() {
        let v: Value = serde_json::from_str(r#"{"a":{"b":[{"c":5}]}}"#).unwrap();
        assert_eq!(v.get_path("a.b.0.c").unwrap().to_string(), "5");
        assert!(v.get_path("a.b.1.c").is_none());
    }
}
