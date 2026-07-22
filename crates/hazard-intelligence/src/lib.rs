#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! Provenance-aware hazard ingestion and immutable operational pictures.
mod adapter;
mod domain;
mod visual_index;
pub use adapter::*;
pub use domain::*;
pub use visual_index::*;
pub const CONTEXT_NAME: &str = "hazard-intelligence";
