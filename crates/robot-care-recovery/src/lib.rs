#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! Robot maintenance, recovery, quarantine, repair, and retirement.
mod domain;
mod simulation;
pub use domain::*;
pub use simulation::*;
pub const CONTEXT_NAME: &str = "robot-care-recovery";
