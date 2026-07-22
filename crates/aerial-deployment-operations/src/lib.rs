#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! Experimental aerial deployment domain boundary (ADR-069–ADR-074).
pub mod commands;
pub mod domain;
pub mod errors;
pub mod events;
pub mod ids;
pub mod value;
pub use commands::*;
pub use domain::*;
pub use errors::*;
pub use events::*;
pub use ids::*;
pub use value::*;
pub const CONTEXT_NAME: &str = "aerial-deployment-operations";
