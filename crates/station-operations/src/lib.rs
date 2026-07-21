#![forbid(unsafe_code)]
//! Portable station edge runtime, signed deployment supervision, and safe reconciliation.
mod domain;
mod energy;
mod habitat;
mod simulation;
mod supervisor;
pub use domain::*;
pub use energy::*;
pub use habitat::*;
pub use simulation::*;
pub use supervisor::*;
/// Stable bounded-context name.
pub const CONTEXT_NAME: &str = "station-operations";
