#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! Experimental aerial deployment domain boundary (ADR-069–ADR-074).
pub mod airborne;
pub mod commands;
pub mod configuration;
pub mod domain;
pub mod effectiveness;
pub mod errors;
pub mod events;
pub mod ground;
pub mod ids;
pub mod mission;
pub mod operator;
pub mod payload;
pub mod recovery;
pub mod simulation;
pub mod value;
pub use airborne::*;
pub use commands::*;
pub use configuration::*;
pub use domain::*;
pub use effectiveness::*;
pub use errors::*;
pub use events::*;
pub use ground::*;
pub use ids::*;
pub use mission::*;
pub use operator::*;
pub use payload::*;
pub use recovery::*;
pub use simulation::*;
pub use value::*;
pub const CONTEXT_NAME: &str = "aerial-deployment-operations";
