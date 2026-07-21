//! Serialization-independent property tests for Prompt 02 technical values.

use chrono::{DateTime, Duration, Utc};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use shared_kernel::{
    CoordinateReferenceSystem, Finite, GeoPoint, IncidentId, NumericError, Probability, Quantity,
    TimeWindow, Unit, UtcInstant,
};
use uuid::Uuid;

proptest! {
    #[test]
    fn typed_identifier_round_trips_uuid_bits(bits in any::<u128>()) {
        let uuid = Uuid::from_u128(bits);
        let identifier = IncidentId::from_uuid(uuid);
        prop_assert_eq!(identifier.as_uuid(), uuid);
    }

    #[test]
    fn finite_acceptance_matches_ieee_finiteness(value in any::<f64>()) {
        prop_assert_eq!(Finite::new(value).is_ok(), value.is_finite());
    }

    #[test]
    fn probability_acceptance_matches_closed_unit_interval(value in any::<f64>()) {
        let expected = value.is_finite() && (0.0..=1.0).contains(&value);
        prop_assert_eq!(Probability::new(value).is_ok(), expected);
    }

    #[test]
    fn length_conversion_round_trips_without_implicit_units(value in -1.0e9_f64..1.0e9_f64) {
        let original = Quantity::new(value, Unit::Metre)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        let restored = original.convert_to(Unit::Kilometre)
            .and_then(|quantity| quantity.convert_to(Unit::Metre))
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        prop_assert!((restored.value().get() - value).abs() <= value.abs().max(1.0) * 1.0e-12);
    }

    #[test]
    fn utc_windows_are_closed_open(start in -1_000_000_i64..1_000_000_i64, width in 1_i64..100_000_i64) {
        let epoch = DateTime::<Utc>::UNIX_EPOCH;
        let start = UtcInstant::new(epoch + Duration::seconds(start));
        let end = start.checked_add(Duration::seconds(width))
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        let window = TimeWindow::new(start, end)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        prop_assert!(window.contains(start));
        prop_assert!(!window.contains(end));
    }

    #[test]
    fn wgs84_accepts_all_in_range_points(longitude in -180.0_f64..=180.0, latitude in -90.0_f64..=90.0) {
        prop_assert!(GeoPoint::new(
            longitude,
            latitude,
            CoordinateReferenceSystem::WGS84,
            None,
        ).is_ok());
    }
}

#[test]
fn incompatible_dimensions_are_stably_rejected() -> Result<(), NumericError> {
    let distance = Quantity::new(1.0, Unit::Metre)?;
    assert_eq!(
        distance.convert_to(Unit::Second),
        Err(NumericError::IncompatibleUnits)
    );
    Ok(())
}
