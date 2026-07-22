#![forbid(unsafe_code)]
#![allow(missing_docs)]
#![allow(clippy::must_use_candidate, clippy::struct_excessive_bools)]
//! Reproducible, advisory-only predictive planning.

mod domain;
mod lightning;
mod outcomes;
mod runner;
mod scenario;

pub use domain::*;
pub use lightning::*;
pub use outcomes::*;
pub use runner::*;
pub use scenario::*;

/// Stable bounded-context name used in diagnostics and contract metadata.
pub const CONTEXT_NAME: &str = "predictive-planning";
