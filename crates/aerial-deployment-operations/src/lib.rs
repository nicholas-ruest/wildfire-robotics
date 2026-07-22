#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! Experimental aerial deployment domain boundary (ADR-069–ADR-074).
pub mod airborne;
pub mod commands;
pub mod configuration;
pub mod domain;
pub mod errors;
pub mod events;
pub mod ground;
pub mod ids;
pub mod mission;
pub mod payload;
pub mod recovery;
pub mod value;
pub use airborne::*;
pub use commands::*;
pub use configuration::*;
pub use domain::*;
pub use errors::*;
pub use events::*;
pub use ground::*;
pub use ids::*;
pub use mission::*;
pub use payload::*;
pub use recovery::*;
pub use value::*;
pub const CONTEXT_NAME: &str = "aerial-deployment-operations";
