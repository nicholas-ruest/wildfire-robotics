//! UTC, monotonic deadline, and clock-quality primitives (ADR-027).

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;
use thiserror::Error;

/// An externally meaningful instant normalized to UTC.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UtcInstant(DateTime<Utc>);

impl UtcInstant {
    /// Wraps an explicitly UTC timestamp.
    #[must_use]
    pub const fn new(value: DateTime<Utc>) -> Self {
        Self(value)
    }
    /// Returns the UTC timestamp.
    #[must_use]
    pub const fn get(self) -> DateTime<Utc> {
        self.0
    }
    /// Computes a checked UTC offset.
    pub fn checked_add(self, duration: Duration) -> Result<Self, TimeError> {
        self.0
            .checked_add_signed(duration)
            .map(Self)
            .ok_or(TimeError::UtcOverflow)
    }
}

impl From<DateTime<Utc>> for UtcInstant {
    fn from(value: DateTime<Utc>) -> Self {
        Self::new(value)
    }
}

/// Non-empty closed-open UTC interval `[start, end)`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct TimeWindow {
    starts_at: UtcInstant,
    ends_at: UtcInstant,
}

#[derive(Deserialize)]
struct TimeWindowWire {
    starts_at: UtcInstant,
    ends_at: UtcInstant,
}

impl<'de> Deserialize<'de> for TimeWindow {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = TimeWindowWire::deserialize(deserializer)?;
        Self::new(wire.starts_at, wire.ends_at).map_err(serde::de::Error::custom)
    }
}

impl TimeWindow {
    /// Creates a window whose exclusive end is later than its start.
    pub fn new(
        starts_at: impl Into<UtcInstant>,
        ends_at: impl Into<UtcInstant>,
    ) -> Result<Self, TimeError> {
        let starts_at = starts_at.into();
        let ends_at = ends_at.into();
        if starts_at >= ends_at {
            Err(TimeError::InvalidTimeWindow)
        } else {
            Ok(Self { starts_at, ends_at })
        }
    }
    /// Inclusive start.
    #[must_use]
    pub const fn starts_at(self) -> UtcInstant {
        self.starts_at
    }
    /// Exclusive end.
    #[must_use]
    pub const fn ends_at(self) -> UtcInstant {
        self.ends_at
    }
    /// Whether the instant belongs to this closed-open interval.
    #[must_use]
    pub fn contains(self, instant: impl Into<UtcInstant>) -> bool {
        let instant = instant.into();
        instant >= self.starts_at && instant < self.ends_at
    }
    /// Whether two windows have any shared instant.
    #[must_use]
    pub fn overlaps(self, other: Self) -> bool {
        self.starts_at < other.ends_at && other.starts_at < self.ends_at
    }
}

/// Identifies a monotonic clock epoch. Values from different epochs cannot be compared.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClockEpoch(u64);

impl ClockEpoch {
    /// Builds an epoch identifier supplied by the clock adapter.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
    /// Returns the adapter-defined epoch value.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Clock-independent representation of a reading from an injected monotonic clock.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct MonotonicInstant {
    epoch: ClockEpoch,
    elapsed_nanos: u64,
}

impl MonotonicInstant {
    /// Creates a reading; this does not access the system clock.
    #[must_use]
    pub const fn new(epoch: ClockEpoch, elapsed_nanos: u64) -> Self {
        Self {
            epoch,
            elapsed_nanos,
        }
    }
    /// Clock epoch for safe comparison.
    #[must_use]
    pub const fn epoch(self) -> ClockEpoch {
        self.epoch
    }
    /// Nanoseconds elapsed in this epoch.
    #[must_use]
    pub const fn elapsed_nanos(self) -> u64 {
        self.elapsed_nanos
    }
    /// Produces a deadline relative to this reading.
    pub fn checked_deadline_after(
        self,
        duration: StdDuration,
    ) -> Result<MonotonicDeadline, TimeError> {
        let nanos = u64::try_from(duration.as_nanos()).map_err(|_| TimeError::MonotonicOverflow)?;
        let elapsed_nanos = self
            .elapsed_nanos
            .checked_add(nanos)
            .ok_or(TimeError::MonotonicOverflow)?;
        Ok(MonotonicDeadline {
            epoch: self.epoch,
            elapsed_nanos,
        })
    }
}

/// Deadline in a particular monotonic-clock epoch.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct MonotonicDeadline {
    epoch: ClockEpoch,
    elapsed_nanos: u64,
}

impl MonotonicDeadline {
    /// Creates a deadline from an adapter reading without accessing a clock.
    #[must_use]
    pub const fn new(epoch: ClockEpoch, elapsed_nanos: u64) -> Self {
        Self {
            epoch,
            elapsed_nanos,
        }
    }
    /// Determines expiry. Cross-epoch comparisons fail closed with an error.
    pub fn is_expired_at(self, now: MonotonicInstant) -> Result<bool, TimeError> {
        if self.epoch == now.epoch {
            Ok(now.elapsed_nanos >= self.elapsed_nanos)
        } else {
            Err(TimeError::ClockEpochMismatch)
        }
    }
    /// Computes remaining duration, saturating at zero after expiry.
    pub fn remaining_at(self, now: MonotonicInstant) -> Result<StdDuration, TimeError> {
        if self.epoch != now.epoch {
            return Err(TimeError::ClockEpochMismatch);
        }
        Ok(StdDuration::from_nanos(
            self.elapsed_nanos.saturating_sub(now.elapsed_nanos),
        ))
    }
}

/// Authenticated source used to estimate UTC clock quality.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ClockSource {
    /// Precision Time Protocol.
    Ptp,
    /// Network Time Protocol.
    Ntp,
    /// Global Navigation Satellite System.
    Gnss,
    /// Local oscillator during loss of synchronization.
    Holdover,
    /// Source is absent or untrusted.
    Unknown,
}

/// Quality metadata accompanying a UTC observation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct ClockQuality {
    source: ClockSource,
    offset: Duration,
    uncertainty: Duration,
    observed_at: UtcInstant,
}

#[derive(Deserialize)]
struct ClockQualityWire {
    source: ClockSource,
    offset: Duration,
    uncertainty: Duration,
    observed_at: UtcInstant,
}

impl<'de> Deserialize<'de> for ClockQuality {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = ClockQualityWire::deserialize(deserializer)?;
        Self::new(wire.source, wire.offset, wire.uncertainty, wire.observed_at)
            .map_err(serde::de::Error::custom)
    }
}

impl ClockQuality {
    /// Creates quality metadata. Uncertainty must be finite in chrono and non-negative.
    pub fn new(
        source: ClockSource,
        offset: Duration,
        uncertainty: Duration,
        observed_at: UtcInstant,
    ) -> Result<Self, TimeError> {
        if uncertainty < Duration::zero() {
            return Err(TimeError::NegativeClockUncertainty);
        }
        Ok(Self {
            source,
            offset,
            uncertainty,
            observed_at,
        })
    }
    /// Synchronization source.
    #[must_use]
    pub const fn source(self) -> ClockSource {
        self.source
    }
    /// Estimated signed UTC offset.
    #[must_use]
    pub const fn offset(self) -> Duration {
        self.offset
    }
    /// Non-negative error bound.
    #[must_use]
    pub const fn uncertainty(self) -> Duration {
        self.uncertainty
    }
    /// UTC instant at which quality was observed.
    #[must_use]
    pub const fn observed_at(self) -> UtcInstant {
        self.observed_at
    }
    /// Whether the uncertainty is within a caller's safety threshold.
    #[must_use]
    pub fn is_within(self, maximum_uncertainty: Duration) -> bool {
        maximum_uncertainty >= Duration::zero() && self.uncertainty <= maximum_uncertainty
    }
}

/// Temporal validation or comparison failure.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum TimeError {
    /// Window end is not later than its start.
    #[error("time window end must be after its start")]
    InvalidTimeWindow,
    /// UTC arithmetic exceeded the representable range.
    #[error("UTC instant overflow")]
    UtcOverflow,
    /// Monotonic duration or instant exceeded `u64` nanoseconds.
    #[error("monotonic instant overflow")]
    MonotonicOverflow,
    /// Readings belong to different monotonic clock epochs.
    #[error("monotonic clock epochs differ")]
    ClockEpochMismatch,
    /// Clock uncertainty cannot be negative.
    #[error("clock uncertainty must be non-negative")]
    NegativeClockUncertainty,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(seconds: i64) -> Result<UtcInstant, TimeError> {
        UtcInstant::new(DateTime::<Utc>::UNIX_EPOCH).checked_add(Duration::seconds(seconds))
    }

    #[test]
    fn time_window_is_closed_open_and_adjacency_is_not_overlap() -> Result<(), TimeError> {
        let first = TimeWindow::new(utc(10)?, utc(20)?)?;
        let second = TimeWindow::new(utc(20)?, utc(30)?)?;
        assert!(first.contains(utc(10)?));
        assert!(!first.contains(utc(20)?));
        assert!(!first.overlaps(second));
        Ok(())
    }

    #[test]
    fn empty_and_reversed_windows_are_rejected() -> Result<(), TimeError> {
        assert_eq!(
            TimeWindow::new(utc(1)?, utc(1)?),
            Err(TimeError::InvalidTimeWindow)
        );
        assert_eq!(
            TimeWindow::new(utc(2)?, utc(1)?),
            Err(TimeError::InvalidTimeWindow)
        );
        Ok(())
    }

    #[test]
    fn monotonic_deadline_has_exact_boundary_semantics() -> Result<(), TimeError> {
        let epoch = ClockEpoch::new(7);
        let deadline =
            MonotonicInstant::new(epoch, 10).checked_deadline_after(StdDuration::from_nanos(5))?;
        assert!(!deadline.is_expired_at(MonotonicInstant::new(epoch, 14))?);
        assert!(deadline.is_expired_at(MonotonicInstant::new(epoch, 15))?);
        assert_eq!(
            deadline.remaining_at(MonotonicInstant::new(epoch, 16))?,
            StdDuration::ZERO
        );
        Ok(())
    }

    #[test]
    fn cross_epoch_deadline_comparison_is_rejected() {
        let deadline = MonotonicDeadline::new(ClockEpoch::new(1), 5);
        assert_eq!(
            deadline.is_expired_at(MonotonicInstant::new(ClockEpoch::new(2), 6)),
            Err(TimeError::ClockEpochMismatch)
        );
    }

    #[test]
    fn monotonic_arithmetic_never_wraps() {
        let instant = MonotonicInstant::new(ClockEpoch::new(1), u64::MAX);
        assert_eq!(
            instant.checked_deadline_after(StdDuration::from_nanos(1)),
            Err(TimeError::MonotonicOverflow)
        );
    }

    #[test]
    fn clock_quality_rejects_negative_uncertainty_and_applies_threshold() -> Result<(), TimeError> {
        assert_eq!(
            ClockQuality::new(
                ClockSource::Ntp,
                Duration::zero(),
                Duration::milliseconds(-1),
                utc(0)?
            ),
            Err(TimeError::NegativeClockUncertainty)
        );
        let quality = ClockQuality::new(
            ClockSource::Gnss,
            Duration::milliseconds(2),
            Duration::milliseconds(5),
            utc(0)?,
        )?;
        assert!(quality.is_within(Duration::milliseconds(5)));
        assert!(!quality.is_within(Duration::milliseconds(4)));
        Ok(())
    }
}
