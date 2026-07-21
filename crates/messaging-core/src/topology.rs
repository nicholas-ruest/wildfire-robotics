//! Context-owned stream and station leaf/store-forward topology policy.

use crate::subject::Subject;
use std::time::Duration;
use thiserror::Error;

/// Reviewed immutable stream policy owned by one bounded context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextStreamSpec {
    /// Stable broker stream name.
    pub name: String,
    /// Exact environment boundary.
    pub environment: String,
    /// Exact region boundary.
    pub region: String,
    /// Exact tenant boundary.
    pub tenant: String,
    /// Owning bounded context.
    pub context: String,
    /// File-storage byte bound.
    pub maximum_bytes: u64,
    /// Message-count bound.
    pub maximum_messages: u64,
    /// Retention age.
    pub maximum_age: Duration,
    /// Duplicate-detection window.
    pub duplicate_window: Duration,
    /// Production replication count.
    pub replicas: u8,
    /// Consumer-count bound.
    pub maximum_consumers: u32,
}

impl ContextStreamSpec {
    /// Validates ownership, retention, capacity, deduplication, and replication bounds.
    pub fn validate(&self, production: bool) -> Result<(), TopologyError> {
        let probe = format!(
            "wr.{}.{}.{}.{}.aggregate.Event.v1",
            self.environment, self.region, self.tenant, self.context
        );
        Subject::parse(&probe).map_err(|_| TopologyError::InvalidStreamSpec)?;
        if self.name.is_empty()
            || self.name.len() > 128
            || !self
                .name
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
            || self.maximum_bytes == 0
            || self.maximum_messages == 0
            || self.maximum_age.is_zero()
            || self.duplicate_window.is_zero()
            || self.duplicate_window > self.maximum_age
            || self.replicas == 0
            || self.replicas > 5
            || (production && self.replicas < 3)
            || self.maximum_consumers == 0
        {
            return Err(TopologyError::InvalidStreamSpec);
        }
        Ok(())
    }

    /// Sole wildcard owned by this context; callers cannot supply arbitrary patterns.
    #[must_use]
    pub fn owned_subject_pattern(&self) -> String {
        format!(
            "wr.{}.{}.{}.{}.>",
            self.environment, self.region, self.tenant, self.context
        )
    }
}

/// Station leaf import/export boundary for disconnected store-forward operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeafBoundary {
    /// Station identity.
    pub station_id: String,
    /// Environment boundary.
    pub environment: String,
    /// Region boundary.
    pub region: String,
    /// Tenant boundary.
    pub tenant: String,
    /// Maximum durable offline bytes.
    pub spool_bytes: u64,
    /// Raw local telemetry retention window.
    pub local_retention: Duration,
}

impl LeafBoundary {
    /// Validates a non-global, bounded leaf policy.
    pub fn validate(&self) -> Result<(), TopologyError> {
        let probe = format!(
            "wr.{}.{}.{}.station-operations.station.Event.v1",
            self.environment, self.region, self.tenant
        );
        Subject::parse(&probe).map_err(|_| TopologyError::InvalidLeafBoundary)?;
        if self.station_id.trim().is_empty()
            || self.station_id.len() > 128
            || self.spool_bytes == 0
            || self.local_retention.is_zero()
        {
            return Err(TopologyError::InvalidLeafBoundary);
        }
        Ok(())
    }
    /// Exact tenant/region prefix imported and exported by the leaf.
    #[must_use]
    pub fn subject_prefix(&self) -> String {
        format!("wr.{}.{}.{}.", self.environment, self.region, self.tenant)
    }
}

/// Stream/leaf validation failures.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum TopologyError {
    /// Context stream ownership/capacity/retention is unsafe.
    #[error("context stream specification is invalid")]
    InvalidStreamSpec,
    /// Station leaf scope or durable bounds are unsafe.
    #[error("station leaf boundary is invalid")]
    InvalidLeafBoundary,
}
