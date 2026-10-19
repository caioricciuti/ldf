//! LDF v0: Language Data Format.
//!
//! Lossless, line-oriented, token-efficient text encoding of JSON records.
//! Specification: `docs/ldf-spec-v0.md`. This crate has no opinion about
//! benchmarks; `ldf-formats` wraps it in the common `Format` trait.
//!
//! ```text
//! @records order_id:s,customer.id:i,items:t,total:f
//! ORD-000001,4654,2,715.94
//! @records.items order_id:s,sku:s,qty:i,price:f
//! ORD-000001,E5,3,28.78
//! ORD-000001,C3,4,157.4
//! ```

#[cfg(feature = "arrow")]
pub mod arrow;
pub mod decode;
pub mod encode;
pub mod plan;
pub mod text;

pub use decode::{DecodeError, decode};
pub use encode::{BlockEncoder, Encoder, StreamEncoder, encode, encode_with};
pub use plan::Options;

/// Name of the root table.
pub const ROOT: &str = "records";
