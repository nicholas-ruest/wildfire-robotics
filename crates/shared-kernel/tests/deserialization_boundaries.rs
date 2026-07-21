//! Adversarial serialization-boundary tests; constructors remain format-independent.

use shared_kernel::{ClockQuality, EntityId, ErrorCode, FencingToken, SemanticVersion, TimeWindow};

#[test]
fn invalid_scalar_values_cannot_bypass_constructors() {
    assert!(serde_json::from_str::<FencingToken>("0").is_err());
    assert!(serde_json::from_str::<EntityId>(r#"""#).is_err());
    assert!(serde_json::from_str::<ErrorCode>(r#""lowercase""#).is_err());
    assert!(serde_json::from_str::<SemanticVersion>(r#""01.0.0""#).is_err());
}

#[test]
fn invalid_temporal_composites_cannot_bypass_constructors() {
    let reversed = r#"{
        "starts_at":"1970-01-01T00:00:02Z",
        "ends_at":"1970-01-01T00:00:01Z"
    }"#;
    assert!(serde_json::from_str::<TimeWindow>(reversed).is_err());

    let negative_uncertainty = r#"{
        "source":"Ntp",
        "offset":[0,0],
        "uncertainty":[-1,0],
        "observed_at":"1970-01-01T00:00:00Z"
    }"#;
    assert!(serde_json::from_str::<ClockQuality>(negative_uncertainty).is_err());
}
