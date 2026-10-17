//! Shared data model for the LDF research workspace.
//!
//! Everything in this workspace moves data as [`Value`] trees. The model is
//! deliberately the JSON data model (null, bool, number, string, array,
//! object) with two refinements:
//!
//! * object key order is preserved (objects are ordered vectors, not maps);
//! * numbers keep their integer-ness: a value parsed from `1` is an integer,
//!   a value parsed from `1.0` is a float. Semantic equality treats them as
//!   equal, byte-level round trips do not.
//!
//! A *dataset* is a sequence of records, each record a [`Value`]. Readers in
//! [`input`] stream records so that memory stays bounded by the largest
//! record, not the file.

pub mod input;
pub mod number;
pub mod schema;
pub mod value;

pub use number::Number;
pub use schema::{FlatSchema, SchemaError};
pub use value::{Object, Value};

/// A fully materialised dataset: one [`Value`] per record.
pub type Records = Vec<Value>;
