#![forbid(unsafe_code)]
//! Tenant-safe commercial administration. This context is deliberately outside
//! the operational safety control path (ADR-015, ADR-050).

mod domain;
pub use domain::*;

/// Stable bounded-context name used in diagnostics and contract metadata.
pub const CONTEXT_NAME: &str = "commercial-operations";
