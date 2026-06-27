//! nitrodb
//!
//! The fast, lightweight distribution of NEDB: append-only op-log lineage, minimal footprint, built for speed.
//!
//! Identical to `nedb-engine` today; this crate is the distribution seam where
//! nitrodb-specific defaults will land (no flags) in a later release.
pub use nedb_engine::*;
