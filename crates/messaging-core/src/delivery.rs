//! Bounded delivery retry and quarantine policy.

use thiserror::Error;

/// Stable failure class decided by a schema/handler boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureClass {
    /// Transient dependency or capacity failure eligible for bounded retry.
    Retryable,
    /// Schema, authorization, integrity, or other non-retryable poison failure.
    Permanent,
}

/// Explicit transport disposition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeliveryDisposition {
    /// Explicit acknowledgement after durable business effect or deduplication.
    Ack,
    /// Negative acknowledgement with bounded delay.
    Nak {
        /// Delay before the next delivery attempt.
        delay_millis: u64,
    },
    /// Durable quarantine followed by terminal broker acknowledgement.
    Quarantine {
        /// Stable policy reason stored with the durable quarantine record.
        reason: &'static str,
    },
}

/// Exponential, capped, deterministic-jitter retry policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetryPolicy {
    maximum_attempts: u32,
    base_millis: u64,
    cap_millis: u64,
    jitter_percent: u8,
}

impl RetryPolicy {
    /// Creates a bounded policy; jitter may be at most 100 percent.
    pub const fn new(
        maximum_attempts: u32,
        base_millis: u64,
        cap_millis: u64,
        jitter_percent: u8,
    ) -> Result<Self, DeliveryError> {
        if maximum_attempts == 0
            || base_millis == 0
            || cap_millis < base_millis
            || jitter_percent > 100
        {
            return Err(DeliveryError::InvalidRetryPolicy);
        }
        Ok(Self {
            maximum_attempts,
            base_millis,
            cap_millis,
            jitter_percent,
        })
    }

    /// Returns NAK or quarantine without exceeding attempt/overflow bounds.
    #[must_use]
    pub fn disposition(
        self,
        message_id: &str,
        attempt: u32,
        class: FailureClass,
    ) -> DeliveryDisposition {
        if class == FailureClass::Permanent {
            return DeliveryDisposition::Quarantine {
                reason: "permanent failure",
            };
        }
        if attempt == 0 || attempt >= self.maximum_attempts {
            return DeliveryDisposition::Quarantine {
                reason: "retry budget exhausted",
            };
        }
        let shift = attempt.saturating_sub(1).min(63);
        let base = self
            .base_millis
            .saturating_mul(1_u64 << shift)
            .min(self.cap_millis);
        let jitter_bound = base.saturating_mul(u64::from(self.jitter_percent)) / 100;
        let hash = message_id.bytes().fold(0_u64, |value, byte| {
            value
                .wrapping_mul(1_099_511_628_211)
                .wrapping_add(u64::from(byte))
        });
        let jitter = if jitter_bound == 0 {
            0
        } else {
            hash % jitter_bound.saturating_add(1)
        };
        DeliveryDisposition::Nak {
            delay_millis: base.saturating_add(jitter).min(self.cap_millis),
        }
    }
}

/// Retry policy validation failures.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DeliveryError {
    /// Attempts/delays/jitter are zero, inverted, or unbounded.
    #[error("delivery retry policy is invalid")]
    InvalidRetryPolicy,
}
