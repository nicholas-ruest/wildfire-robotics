#![forbid(unsafe_code)]
//! Portable station edge runtime, signed deployment supervision, and safe reconciliation.
mod domain;
mod supervisor;
pub use domain::*;
pub use supervisor::*;
/// Stable bounded-context name.
pub const CONTEXT_NAME: &str = "station-operations";
