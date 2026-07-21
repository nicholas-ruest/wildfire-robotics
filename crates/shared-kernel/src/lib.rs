#![forbid(unsafe_code)]
//! Small technical and identity primitives shared across bounded contexts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

/// Identifies an aggregate without allowing blank or ambiguous values.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntityId(String);

impl EntityId {
    /// Creates a random globally unique identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Validates an identifier received at a boundary.
    pub fn parse(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        if value.trim().is_empty() || value.len() > 128 {
            return Err(ValidationError::InvalidIdentifier);
        }
        Ok(Self(value))
    }

    /// Returns the stable string representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A closed-open UTC time interval `[start, end)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TimeWindow {
    /// Inclusive start.
    pub starts_at: DateTime<Utc>,
    /// Exclusive end.
    pub ends_at: DateTime<Utc>,
}

impl TimeWindow {
    /// Creates a non-empty bounded interval.
    pub fn new(starts_at: DateTime<Utc>, ends_at: DateTime<Utc>) -> Result<Self, ValidationError> {
        if starts_at >= ends_at {
            return Err(ValidationError::InvalidTimeWindow);
        }
        Ok(Self { starts_at, ends_at })
    }

    /// Returns whether a timestamp is authorized by this interval.
    #[must_use]
    pub fn contains(self, instant: DateTime<Utc>) -> bool {
        instant >= self.starts_at && instant < self.ends_at
    }
}

/// Validation failures safe to return to callers.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum ValidationError {
    /// Identifier is empty or exceeds the boundary limit.
    #[error("identifier must contain 1 to 128 characters")]
    InvalidIdentifier,
    /// A time window is empty or reversed.
    #[error("time window end must be after its start")]
    InvalidTimeWindow,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn time_window_is_closed_open() -> Result<(), ValidationError> {
        let start = Utc::now();
        let window = TimeWindow::new(start, start + Duration::seconds(5))?;
        assert!(window.contains(start));
        assert!(!window.contains(window.ends_at));
        Ok(())
    }

    #[test]
    fn identifier_rejects_blank_input() {
        assert_eq!(
            EntityId::parse("  "),
            Err(ValidationError::InvalidIdentifier)
        );
    }
}
