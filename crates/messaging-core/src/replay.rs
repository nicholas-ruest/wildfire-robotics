//! Authorized bounded replay and quarantine repair requests.

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

/// Bounded replay request that always targets an isolated consumer identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayRequest {
    /// New identity for this replay operation.
    pub replay_id: Uuid,
    /// Human operator requesting replay.
    pub operator_id: String,
    /// Independently approving human.
    pub approver_id: String,
    /// Tenant boundary.
    pub tenant_id: String,
    /// Source stream.
    pub stream: String,
    /// Isolated target durable consumer.
    pub target_consumer: String,
    /// Inclusive start sequence.
    pub starts_at_sequence: u64,
    /// Inclusive end sequence.
    pub ends_at_sequence: u64,
    /// Maximum replay rate.
    pub maximum_messages_per_second: u32,
    /// Dry-run validates without delivery.
    pub dry_run: bool,
    /// Request expiry.
    pub expires_at: DateTime<Utc>,
}

impl ReplayRequest {
    /// Validates separation of duties, scope, finite range/rate, and expiry.
    pub fn validate(&self, now: DateTime<Utc>) -> Result<(), ReplayError> {
        let fields = [
            &self.operator_id,
            &self.approver_id,
            &self.tenant_id,
            &self.stream,
            &self.target_consumer,
        ];
        if fields
            .iter()
            .any(|value| value.trim().is_empty() || value.len() > 128)
            || self.operator_id == self.approver_id
            || self.starts_at_sequence == 0
            || self.ends_at_sequence < self.starts_at_sequence
            || self
                .ends_at_sequence
                .saturating_sub(self.starts_at_sequence)
                > 1_000_000
            || self.maximum_messages_per_second == 0
            || self.maximum_messages_per_second > 10_000
            || now >= self.expires_at
        {
            return Err(ReplayError::UnauthorizedOrUnbounded);
        }
        Ok(())
    }
}

/// Replay request validation failure.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ReplayError {
    /// Request lacks separation, expiry, or bounded range/rate.
    #[error("replay request is unauthorized or unbounded")]
    UnauthorizedOrUnbounded,
}
