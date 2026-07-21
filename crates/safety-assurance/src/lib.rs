#![forbid(unsafe_code)]
//! Safety Assurance aggregates and fail-closed physical-release promotion.

mod assurance;
mod promotion;

pub use assurance::*;
pub use promotion::*;
