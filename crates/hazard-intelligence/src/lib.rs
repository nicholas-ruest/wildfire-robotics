#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! Provenance-aware hazard ingestion and immutable operational pictures.
mod adapter;
mod domain;
pub use adapter::*;
pub use domain::*;
pub const CONTEXT_NAME: &str = "hazard-intelligence";
