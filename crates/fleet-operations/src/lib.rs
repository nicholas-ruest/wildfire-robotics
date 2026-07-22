#![forbid(unsafe_code)]
//! Fleet identity, eligibility, energy, health, and hierarchical cell ownership.
mod assets;
mod cells;
mod collaboration;
pub use assets::*;
pub use cells::*;
pub use collaboration::*;
/// Stable bounded-context name.
pub const CONTEXT_NAME: &str = "fleet-operations";
