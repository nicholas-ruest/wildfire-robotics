#![forbid(unsafe_code)]
//! Stable vehicle gateway contracts, deterministic simulation, and normalized telemetry.
mod coordination;
mod domain;
#[cfg(feature = "mavlink-adapter")]
pub mod mavlink;
#[cfg(feature = "ros2-adapter")]
pub mod ros2;
mod simulator;
pub use coordination::*;
pub use domain::*;
pub use simulator::*;
/// Stable bounded-context name.
pub const CONTEXT_NAME: &str = "vehicle-integration";
