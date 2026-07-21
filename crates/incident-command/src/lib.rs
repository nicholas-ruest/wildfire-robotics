#![forbid(unsafe_code)]
//! Incident Command authority aggregates and activation process.

mod domain;
mod process;
pub use domain::*;
pub use process::*;

/// Stable bounded-context name used in diagnostics and contract metadata.
pub const CONTEXT_NAME: &str = "incident-command";
