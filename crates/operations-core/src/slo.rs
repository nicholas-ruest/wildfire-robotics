//! Validated service-level indicators, objectives, and error budgets.

use std::time::Duration;
use thiserror::Error;

/// Direction in which an indicator satisfies its objective.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectiveDirection {
    /// Higher values are better, such as availability.
    AtLeast,
    /// Lower values are better, such as latency or loss.
    AtMost,
}

/// Accountable service-level indicator descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SliDescriptor {
    /// Stable machine-readable name.
    pub name: String,
    /// Plain-language measurement definition.
    pub description: String,
    /// Unit used by measurements.
    pub unit: String,
    /// Accountable team.
    pub owner: String,
    /// Dashboard reference.
    pub dashboard: String,
    /// Actionable alert reference.
    pub alert: String,
    /// Operator runbook reference.
    pub runbook: String,
}

impl SliDescriptor {
    /// Rejects incomplete operational ownership.
    pub fn validate(&self) -> Result<(), SloError> {
        let required = [
            &self.name,
            &self.description,
            &self.unit,
            &self.owner,
            &self.dashboard,
            &self.alert,
            &self.runbook,
        ];
        if required.iter().any(|value| value.trim().is_empty()) {
            Err(SloError::IncompleteIndicator)
        } else {
            Ok(())
        }
    }
}

/// Service-level objective and its rolling error-budget policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SloDescriptor {
    /// Measured indicator.
    pub indicator: SliDescriptor,
    /// Comparison direction.
    pub direction: ObjectiveDirection,
    /// Objective in millionths, avoiding floating-point boundary ambiguity.
    pub target_millionths: u32,
    /// Rolling measurement window.
    pub window: Duration,
    /// Whether exhaustion blocks risky promotion.
    pub blocks_promotion_on_exhaustion: bool,
}

impl SloDescriptor {
    /// Validates objective range, ownership, and non-zero window.
    pub fn validate(&self) -> Result<(), SloError> {
        self.indicator.validate()?;
        if self.target_millionths > 1_000_000 || self.window.is_zero() {
            return Err(SloError::InvalidObjective);
        }
        Ok(())
    }

    /// Computes the permitted bad events for a count-based at-least objective.
    pub fn permitted_bad_events(&self, total_events: u64) -> Result<u64, SloError> {
        self.validate()?;
        if self.direction != ObjectiveDirection::AtLeast {
            return Err(SloError::NotCountBased);
        }
        let bad_millionths = 1_000_000_u64 - u64::from(self.target_millionths);
        Ok(total_events.saturating_mul(bad_millionths) / 1_000_000)
    }

    /// Returns whether consumed bad events exhaust the budget.
    pub fn budget_exhausted(&self, total_events: u64, bad_events: u64) -> Result<bool, SloError> {
        Ok(bad_events >= self.permitted_bad_events(total_events)?)
    }

    /// Evaluates a normalized measurement against either objective direction.
    pub fn objective_met(&self, measured_millionths: u32) -> Result<bool, SloError> {
        self.validate()?;
        if measured_millionths > 1_000_000 {
            return Err(SloError::InvalidMeasurement);
        }
        Ok(match self.direction {
            ObjectiveDirection::AtLeast => measured_millionths >= self.target_millionths,
            ObjectiveDirection::AtMost => measured_millionths <= self.target_millionths,
        })
    }
}

/// Stable SLI/SLO descriptor failures.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum SloError {
    /// Indicator lacks definition, owner, dashboard, alert, or runbook.
    #[error("SLI descriptor is incomplete")]
    IncompleteIndicator,
    /// Target or rolling window is invalid.
    #[error("SLO target or window is invalid")]
    InvalidObjective,
    /// Count-based error budget was requested for a threshold objective.
    #[error("objective is not a count-based at-least objective")]
    NotCountBased,
    /// A normalized measurement exceeded one million millionths.
    #[error("SLI measurement is outside the normalized range")]
    InvalidMeasurement,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn computes_exact_integer_error_budget_and_exhaustion() -> Result<(), SloError> {
        let slo = SloDescriptor {
            indicator: SliDescriptor {
                name: "availability".into(),
                description: "successful requests".into(),
                unit: "ratio".into(),
                owner: "on-call".into(),
                dashboard: "dash".into(),
                alert: "alert".into(),
                runbook: "runbook".into(),
            },
            direction: ObjectiveDirection::AtLeast,
            target_millionths: 999_000,
            window: Duration::from_hours(720),
            blocks_promotion_on_exhaustion: true,
        };
        assert_eq!(slo.permitted_bad_events(10_000)?, 10);
        assert!(slo.budget_exhausted(10_000, 10)?);
        assert!(slo.budget_exhausted(10_000, 11)?);
        Ok(())
    }
}
