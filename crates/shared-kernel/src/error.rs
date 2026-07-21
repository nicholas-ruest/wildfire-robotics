//! Stable, caller-safe technical error categories and retry guidance.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

const MAX_ERROR_CODE_CHARS: usize = 64;

/// Stable top-level error categories required by the tactical model standard.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCategory {
    /// Schema, unit, CRS, range, or semantic validation failed.
    InvalidArgument,
    /// Identity or signature could not be established.
    Unauthenticated,
    /// The authenticated principal lacks authority.
    Forbidden,
    /// Authority, policy, approval, ODD, lease, or command expired.
    StaleAuthority,
    /// Aggregate version or exclusive resource conflict.
    Conflict,
    /// A domain invariant prevents the action until state changes.
    FailedPrecondition,
    /// A bounded quota or capacity was reached.
    ResourceExhausted,
    /// A required dependency is unavailable.
    DependencyUnavailable,
    /// An unexpected controlled failure occurred.
    Internal,
}

impl ErrorCategory {
    /// Returns stable retry guidance for this category.
    #[must_use]
    pub const fn retry_classification(self) -> RetryClassification {
        match self {
            Self::InvalidArgument
            | Self::Unauthenticated
            | Self::Forbidden
            | Self::StaleAuthority => RetryClassification::Never,
            Self::Conflict => RetryClassification::Reread,
            Self::FailedPrecondition => RetryClassification::AfterChange,
            Self::ResourceExhausted => RetryClassification::Backoff,
            Self::DependencyUnavailable => RetryClassification::Bounded,
            Self::Internal => RetryClassification::Controlled,
        }
    }

    /// Returns the stable `SCREAMING_SNAKE_CASE` wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidArgument => "INVALID_ARGUMENT",
            Self::Unauthenticated => "UNAUTHENTICATED",
            Self::Forbidden => "FORBIDDEN",
            Self::StaleAuthority => "STALE_AUTHORITY",
            Self::Conflict => "CONFLICT",
            Self::FailedPrecondition => "FAILED_PRECONDITION",
            Self::ResourceExhausted => "RESOURCE_EXHAUSTED",
            Self::DependencyUnavailable => "DEPENDENCY_UNAVAILABLE",
            Self::Internal => "INTERNAL",
        }
    }
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Machine-actionable retry behavior; never infer this from free-form text.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RetryClassification {
    /// Repeating the same request is unsafe or cannot succeed.
    Never,
    /// Re-read authoritative state, then reevaluate the command.
    Reread,
    /// Retry only after the blocking domain state changes.
    AfterChange,
    /// Retry with bounded exponential backoff and jitter at the adapter boundary.
    Backoff,
    /// A bounded dependency retry may be attempted under degraded-mode policy.
    Bounded,
    /// Retry only through a controlled internal recovery policy.
    Controlled,
}

/// Validation failure for a stable context-specific error code.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("error code must contain 1 to 64 uppercase ASCII letters, digits, or underscores")]
pub struct InvalidErrorCode;

/// Stable, context-specific machine-readable error code.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ErrorCode(String);

impl ErrorCode {
    /// Validates a stable code such as `MISSION_VERSION_MISMATCH`.
    pub fn parse(value: impl Into<String>) -> Result<Self, InvalidErrorCode> {
        let value = value.into();
        if value.is_empty()
            || value.len() > MAX_ERROR_CODE_CHARS
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
            || !value
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_uppercase())
        {
            return Err(InvalidErrorCode);
        }
        Ok(Self(value))
    }

    /// Returns the stable code.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl TryFrom<String> for ErrorCode {
    type Error = InvalidErrorCode;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl From<ErrorCode> for String {
    fn from(value: ErrorCode) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_categories_have_normative_retry_classifications() {
        let mappings = [
            (ErrorCategory::InvalidArgument, RetryClassification::Never),
            (ErrorCategory::Unauthenticated, RetryClassification::Never),
            (ErrorCategory::Forbidden, RetryClassification::Never),
            (ErrorCategory::StaleAuthority, RetryClassification::Never),
            (ErrorCategory::Conflict, RetryClassification::Reread),
            (
                ErrorCategory::FailedPrecondition,
                RetryClassification::AfterChange,
            ),
            (
                ErrorCategory::ResourceExhausted,
                RetryClassification::Backoff,
            ),
            (
                ErrorCategory::DependencyUnavailable,
                RetryClassification::Bounded,
            ),
            (ErrorCategory::Internal, RetryClassification::Controlled),
        ];
        for (category, expected) in mappings {
            assert_eq!(category.retry_classification(), expected);
        }
    }

    #[test]
    fn category_display_is_stable() {
        assert_eq!(
            ErrorCategory::DependencyUnavailable.to_string(),
            "DEPENDENCY_UNAVAILABLE"
        );
    }

    #[test]
    fn error_code_accepts_machine_readable_values() -> Result<(), InvalidErrorCode> {
        let code = ErrorCode::parse("MISSION_VERSION_MISMATCH_2")?;
        assert_eq!(code.as_str(), "MISSION_VERSION_MISMATCH_2");
        Ok(())
    }

    #[test]
    fn error_code_rejects_ambiguous_boundary_values() {
        for invalid in [
            "",
            "lowercase",
            "1_STARTS_WITH_DIGIT",
            "HAS-HYPHEN",
            "HAS SPACE",
        ] {
            assert_eq!(ErrorCode::parse(invalid), Err(InvalidErrorCode));
        }
    }
}
