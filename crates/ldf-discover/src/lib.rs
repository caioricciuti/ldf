//! Format discovery engine, step 1: a parameterized table encoder whose
//! every layout choice is a knob ([`knobs`]), a decoder for every point of
//! the space, and Stage A of the search ([`stage_a`]): exact o200k token
//! counts over the fixture datasets, lossy candidates dropped.
//!
//! LDF v0.1 and v0 are points in the space ([`knobs::Config::LDF_V01`]);
//! JSON, CSV and TOON are reference rows from `ldf-formats`.

pub mod candidate;
pub mod decode;
pub mod encode;
pub mod knobs;
pub mod plan;
pub mod stage_a;
pub mod text;

pub use candidate::Candidate;
pub use decode::decode;
pub use encode::encode;
pub use knobs::{Config, space};
