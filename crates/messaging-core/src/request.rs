//! Monotonic deadline policy for authenticated request/reply operations.

use shared_kernel::{MonotonicDeadline, MonotonicInstant};
use std::time::Duration;
use thiserror::Error;

/// Validated local request budget independent of UTC clock jumps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequestBudget {
    deadline: MonotonicDeadline,
    maximum_payload_bytes: usize,
}

impl RequestBudget {
    /// Creates a positive bounded request budget.
    pub fn new(
        deadline: MonotonicDeadline,
        maximum_payload_bytes: usize,
    ) -> Result<Self, RequestError> {
        if maximum_payload_bytes == 0 || maximum_payload_bytes > 1_048_576 {
            return Err(RequestError::InvalidRequest);
        }
        Ok(Self {
            deadline,
            maximum_payload_bytes,
        })
    }
    /// Returns remaining wait duration or fails closed for expiry/cross-epoch clocks.
    pub fn remaining(
        &self,
        now: MonotonicInstant,
        payload_bytes: usize,
    ) -> Result<Duration, RequestError> {
        if payload_bytes == 0 || payload_bytes > self.maximum_payload_bytes {
            return Err(RequestError::InvalidRequest);
        }
        let remaining = self
            .deadline
            .remaining_at(now)
            .map_err(|_| RequestError::ClockEpochMismatch)?;
        if remaining.is_zero() {
            Err(RequestError::DeadlineExceeded)
        } else {
            Ok(remaining)
        }
    }
}

/// Explicit command outcome; timeout never implies success or rejection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequestOutcome<T> {
    /// Correlated reply received before the local deadline.
    Reply(T),
    /// Deadline elapsed; the remote command may or may not have executed.
    UnknownAfterDeadline,
}

/// Request validation/deadline failures.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum RequestError {
    /// Payload bound is invalid or exceeded.
    #[error("request is invalid")]
    InvalidRequest,
    /// Local monotonic deadline elapsed.
    #[error("request outcome is unknown after deadline")]
    DeadlineExceeded,
    /// Clock restart prevents deadline comparison.
    #[error("monotonic clock epoch changed")]
    ClockEpochMismatch,
}
