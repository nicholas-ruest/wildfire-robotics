#![forbid(unsafe_code)]
//! Boundary marker for the experimental Aerial Deployment Operations context.
//!
//! Domain behavior is intentionally deferred to the AFB promptbook.

/// Stable bounded-context name used by ownership and deployment tooling.
pub const CONTEXT_NAME: &str = "aerial-deployment-operations";
