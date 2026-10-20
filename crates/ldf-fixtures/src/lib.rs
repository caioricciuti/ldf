//! Deterministic synthetic datasets for correctness fixtures and
//! performance runs.
//!
//! Every dataset is a pure function of `(name, seed, row index)`; see
//! [`datasets::catalog`] for the schema of each one and
//! `site/src/content/docs/docs/methodology.md` for the rationale. Records are generated
//! one at a time so GB-scale files stream to disk with bounded memory.

pub mod datasets;
pub mod rng;
pub mod writer;

pub use datasets::{DatasetSpec, Scale, catalog, find};
pub use rng::Rng;

/// Default seed used for the committed fixtures.
pub const DEFAULT_SEED: u64 = 20_261_017;
