//! Key and number text shared with LDF v0.1, plus the quoting rules that
//! the [`Quoting`](crate::knobs::Quoting) knob opens up.

pub use ldf::text::{
    is_bare_segment, json_string_end, push_json_string, push_json_value, push_path, push_segment,
};
