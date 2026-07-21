#![forbid(unsafe_code)]
//! Boundary marker for the Robot Care and Recovery bounded context.
//!
//! Domain behavior is intentionally deferred to Main Prompt 17.

/// Stable bounded-context name used by ownership and deployment tooling.
pub const CONTEXT_NAME: &str = "robot-care-recovery";
