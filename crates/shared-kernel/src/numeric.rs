//! Validated numerical value objects used at trust boundaries.
//!
//! These types implement the finite-value and explicit-unit rules from ADR-006
//! and the tactical domain model standard. They deliberately do not perform
//! implicit unit conversion.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// A finite IEEE-754 value.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "f64", into = "f64")]
pub struct Finite(f64);

impl Finite {
    /// Validates a floating-point value at a boundary.
    pub fn new(value: f64) -> Result<Self, NumericError> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(NumericError::NotFinite)
        }
    }

    /// Returns the validated scalar.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for Finite {
    type Error = NumericError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Finite> for f64 {
    fn from(value: Finite) -> Self {
        value.get()
    }
}

/// Dimension of a physical quantity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Dimension {
    /// One-dimensional spatial extent.
    Length,
    /// Two-dimensional spatial extent.
    Area,
    /// Elapsed time.
    Duration,
    /// Distance travelled per unit time.
    Speed,
    /// Angular displacement.
    Angle,
    /// Thermodynamic temperature.
    Temperature,
    /// Matter quantity measured by mass.
    Mass,
    /// Three-dimensional capacity.
    Volume,
    /// Dimensionless proportion.
    Ratio,
}

/// Canonical units accepted by shared technical contracts.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Unit {
    /// Metres (`m`).
    Metre,
    /// Kilometres (`km`).
    Kilometre,
    /// Square metres (`m²`).
    SquareMetre,
    /// Hectares (`ha`).
    Hectare,
    /// Seconds (`s`).
    Second,
    /// Milliseconds (`ms`).
    Millisecond,
    /// Metres per second (`m/s`).
    MetresPerSecond,
    /// Kilometres per hour (`km/h`).
    KilometresPerHour,
    /// Degrees of arc.
    Degree,
    /// Radians.
    Radian,
    /// Degrees Celsius.
    Celsius,
    /// Kelvin.
    Kelvin,
    /// Kilograms (`kg`).
    Kilogram,
    /// Litres (`L`).
    Litre,
    /// A percentage in the scale 0 to 100.
    Percent,
    /// A dimensionless ratio in the scale 0 to 1.
    Unitless,
}

impl Unit {
    /// Returns the physical dimension represented by the unit.
    #[must_use]
    pub const fn dimension(self) -> Dimension {
        match self {
            Self::Metre | Self::Kilometre => Dimension::Length,
            Self::SquareMetre | Self::Hectare => Dimension::Area,
            Self::Second | Self::Millisecond => Dimension::Duration,
            Self::MetresPerSecond | Self::KilometresPerHour => Dimension::Speed,
            Self::Degree | Self::Radian => Dimension::Angle,
            Self::Celsius | Self::Kelvin => Dimension::Temperature,
            Self::Kilogram => Dimension::Mass,
            Self::Litre => Dimension::Volume,
            Self::Percent | Self::Unitless => Dimension::Ratio,
        }
    }
}

/// A scalar paired with its explicit unit.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Quantity {
    value: Finite,
    unit: Unit,
}

impl Quantity {
    /// Creates a quantity and rejects non-finite input.
    pub fn new(value: f64, unit: Unit) -> Result<Self, NumericError> {
        Ok(Self {
            value: Finite::new(value)?,
            unit,
        })
    }

    #[must_use]
    /// Returns the finite scalar without changing its unit.
    pub const fn value(self) -> Finite {
        self.value
    }

    #[must_use]
    /// Returns the declared unit.
    pub const fn unit(self) -> Unit {
        self.unit
    }

    /// Converts to another unit of the same dimension.
    ///
    /// Conversion is always explicit at the call site.
    pub fn convert_to(self, target: Unit) -> Result<Self, NumericError> {
        if self.unit.dimension() != target.dimension() {
            return Err(NumericError::IncompatibleUnits);
        }
        let canonical = to_canonical(self.value.get(), self.unit);
        Self::new(from_canonical(canonical, target), target)
    }
}

fn to_canonical(value: f64, unit: Unit) -> f64 {
    match unit {
        Unit::Kilometre => value * 1_000.0,
        Unit::Hectare => value * 10_000.0,
        Unit::Millisecond => value / 1_000.0,
        Unit::KilometresPerHour => value / 3.6,
        Unit::Radian => value.to_degrees(),
        Unit::Kelvin => value - 273.15,
        Unit::Percent => value / 100.0,
        _ => value,
    }
}

fn from_canonical(value: f64, unit: Unit) -> f64 {
    match unit {
        Unit::Kilometre => value / 1_000.0,
        Unit::Hectare => value / 10_000.0,
        Unit::Millisecond => value * 1_000.0,
        Unit::KilometresPerHour => value * 3.6,
        Unit::Radian => value.to_radians(),
        Unit::Kelvin => value + 273.15,
        Unit::Percent => value * 100.0,
        _ => value,
    }
}

/// Probability in the inclusive interval `[0, 1]`.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "f64", into = "f64")]
pub struct Probability(Finite);

impl Probability {
    /// Validates a probability in the inclusive interval `[0, 1]`.
    pub fn new(value: f64) -> Result<Self, NumericError> {
        let value = Finite::new(value)?;
        if (0.0..=1.0).contains(&value.get()) {
            Ok(Self(value))
        } else {
            Err(NumericError::OutsideUnitInterval)
        }
    }

    #[must_use]
    /// Returns the probability scalar.
    pub const fn get(self) -> f64 {
        self.0.get()
    }
}

impl TryFrom<f64> for Probability {
    type Error = NumericError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Probability> for f64 {
    fn from(value: Probability) -> Self {
        value.get()
    }
}

/// Confidence score in the inclusive interval `[0, 1]`.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "f64", into = "f64")]
pub struct Confidence(Probability);

impl Confidence {
    /// Validates a confidence score in the inclusive interval `[0, 1]`.
    pub fn new(value: f64) -> Result<Self, NumericError> {
        Ok(Self(Probability::new(value)?))
    }

    #[must_use]
    /// Returns the confidence scalar.
    pub const fn get(self) -> f64 {
        self.0.get()
    }
}

impl TryFrom<f64> for Confidence {
    type Error = NumericError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Confidence> for f64 {
    fn from(value: Confidence) -> Self {
        value.get()
    }
}

/// Maximum permitted age for an observation, in milliseconds.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "u64", into = "u64")]
pub struct Freshness(u64);

impl Freshness {
    /// Creates a strictly positive freshness limit.
    pub fn from_millis(max_age_millis: u64) -> Result<Self, NumericError> {
        if max_age_millis == 0 {
            Err(NumericError::ZeroFreshness)
        } else {
            Ok(Self(max_age_millis))
        }
    }

    #[must_use]
    /// Returns the maximum permitted observation age in milliseconds.
    pub const fn max_age_millis(self) -> u64 {
        self.0
    }

    /// Uses closed-boundary semantics: an observation exactly at the limit is fresh.
    #[must_use]
    pub const fn permits_age_millis(self, age_millis: u64) -> bool {
        age_millis <= self.0
    }
}

impl TryFrom<u64> for Freshness {
    type Error = NumericError;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::from_millis(value)
    }
}

impl From<Freshness> for u64 {
    fn from(value: Freshness) -> Self {
        value.max_age_millis()
    }
}

/// Stable numeric boundary failures.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum NumericError {
    /// The input was `NaN` or positive/negative infinity.
    #[error("numeric value must be finite")]
    NotFinite,
    /// The input was outside the inclusive interval `[0, 1]`.
    #[error("value must be between zero and one inclusive")]
    OutsideUnitInterval,
    /// A conversion was requested between different physical dimensions.
    #[error("units belong to different dimensions")]
    IncompatibleUnits,
    /// A zero-duration freshness policy was requested.
    #[error("freshness duration must be greater than zero")]
    ZeroFreshness,
}

impl fmt::Display for Finite {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_reject_every_non_finite_boundary_value() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(Finite::new(value), Err(NumericError::NotFinite));
        }
    }

    #[test]
    fn should_accept_probability_interval_endpoints() {
        assert!(Probability::new(0.0).is_ok());
        assert!(Probability::new(1.0).is_ok());
    }

    #[test]
    fn should_reject_probability_values_outside_interval() {
        for value in [-f64::EPSILON, 1.0 + f64::EPSILON] {
            assert_eq!(
                Probability::new(value),
                Err(NumericError::OutsideUnitInterval)
            );
        }
    }

    #[test]
    fn should_round_trip_supported_unit_conversions() -> Result<(), NumericError> {
        let pairs = [
            (Unit::Metre, Unit::Kilometre),
            (Unit::SquareMetre, Unit::Hectare),
            (Unit::Second, Unit::Millisecond),
            (Unit::MetresPerSecond, Unit::KilometresPerHour),
            (Unit::Degree, Unit::Radian),
            (Unit::Celsius, Unit::Kelvin),
            (Unit::Unitless, Unit::Percent),
        ];
        for (source, target) in pairs {
            for sample in [-100.0, -1.0, 0.0, 1.0, 1234.5] {
                let round_trip = Quantity::new(sample, source)?
                    .convert_to(target)?
                    .convert_to(source)?;
                assert!((round_trip.value().get() - sample).abs() < 1.0e-9);
            }
        }
        Ok(())
    }

    #[test]
    fn should_reject_conversion_across_dimensions() -> Result<(), NumericError> {
        let length = Quantity::new(1.0, Unit::Metre)?;
        assert_eq!(
            length.convert_to(Unit::Second),
            Err(NumericError::IncompatibleUnits)
        );
        Ok(())
    }

    #[test]
    fn should_treat_freshness_limit_as_inclusive() -> Result<(), NumericError> {
        let freshness = Freshness::from_millis(10)?;
        assert!(freshness.permits_age_millis(10));
        assert!(!freshness.permits_age_millis(11));
        Ok(())
    }
}
