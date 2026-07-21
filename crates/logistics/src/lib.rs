#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! Logistics inventory, custody, delivery, water, and supply planning.

mod delivery;
mod inventory;
mod supply;
mod water;

pub use delivery::*;
pub use inventory::*;
pub use supply::*;
pub use water::*;

/// Stable bounded-context name used in diagnostics and contract metadata.
pub const CONTEXT_NAME: &str = "logistics";

use thiserror::Error;

/// Stable logistics-domain failures.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum LogisticsError {
    #[error("quantity or unit is invalid or incompatible")]
    InvalidQuantity,
    #[error("resource identity, condition, or lifecycle is invalid")]
    InvalidResource,
    #[error("resource is unavailable to promise")]
    Unavailable,
    #[error("reservation exceeds exclusive usable capacity")]
    OverReserved,
    #[error("reservation is stale, expired, or already consumed")]
    StaleReservation,
    #[error("custody transition is incomplete or inconsistent")]
    InvalidCustody,
    #[error("route or operational envelope is unsafe")]
    UnsafeRoute,
    #[error("delivery transition is invalid")]
    InvalidDelivery,
    #[error("water source is stale, restricted, contaminated, or depleted")]
    UnsafeWaterSource,
    #[error("relay cycle is invalid")]
    InvalidRelay,
    #[error("supply plan input or transition is invalid")]
    InvalidSupplyPlan,
    #[error("no feasible supply plan satisfies hard dependencies")]
    InfeasibleSupply,
    #[error("aggregate version exhausted")]
    VersionExhausted,
}
