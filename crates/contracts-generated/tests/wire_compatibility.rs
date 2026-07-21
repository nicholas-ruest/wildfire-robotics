//! Wire-compatibility and checked-in golden fixture tests.

use prost::{Enumeration, Message};
use serde::Serialize;
use wildfire_contracts_generated::conformance::{
    ConformanceError, canonical_json, decode_bounded, encode_bounded, verify_golden,
};

#[derive(Clone, Copy, Debug, Enumeration)]
#[repr(i32)]
enum KnownState {
    Unspecified = 0,
    Ready = 1,
}

#[derive(Clone, PartialEq, Message)]
struct OlderConsumer {
    #[prost(string, tag = "1")]
    id: String,
    #[prost(enumeration = "KnownState", tag = "2")]
    state: i32,
}

#[derive(Clone, PartialEq, Message)]
struct NewerProducer {
    #[prost(string, tag = "1")]
    id: String,
    #[prost(enumeration = "KnownState", tag = "2")]
    state: i32,
    #[prost(string, tag = "3")]
    additive_field: String,
}

#[test]
fn unknown_enum_number_is_preserved_without_assuming_meaning() -> Result<(), ConformanceError> {
    let bytes = NewerProducer {
        id: "event-1".into(),
        state: 77,
        additive_field: String::new(),
    }
    .encode_to_vec();
    let decoded: OlderConsumer = decode_bounded(&bytes, 1024)?;
    assert_eq!(decoded.state, 77);
    assert!(KnownState::try_from(decoded.state).is_err());
    Ok(())
}

#[test]
fn additive_field_is_tolerated_by_older_consumer() -> Result<(), ConformanceError> {
    let bytes = NewerProducer {
        id: "event-2".into(),
        state: KnownState::Ready as i32,
        additive_field: "new".into(),
    }
    .encode_to_vec();
    let decoded: OlderConsumer = decode_bounded(&bytes, 1024)?;
    assert_eq!(decoded.id, "event-2");
    Ok(())
}

#[test]
fn decoder_and_encoder_enforce_exact_size_boundary() {
    let message = OlderConsumer {
        id: "bounded".into(),
        state: 0,
    };
    let bytes = message.encode_to_vec();
    assert!(decode_bounded::<OlderConsumer>(&bytes, bytes.len()).is_ok());
    assert!(matches!(
        decode_bounded::<OlderConsumer>(&bytes, bytes.len() - 1),
        Err(ConformanceError::MessageTooLarge { .. })
    ));
    assert!(encode_bounded(&message, bytes.len()).is_ok());
}

#[derive(Serialize)]
struct Fixture<'a> {
    z_field: u32,
    a_field: &'a str,
}

#[test]
fn binary_and_json_goldens_are_byte_deterministic() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let json_path = root.join("contracts/examples/device_trust_changed.v1.json");
    let binary_path = root.join("contracts/fixtures/device_trust_changed.v1.binpb");
    let reviewed_json = include_bytes!("../../../contracts/examples/device_trust_changed.v1.json");
    let reviewed_binary =
        include_bytes!("../../../contracts/fixtures/device_trust_changed.v1.binpb");
    assert_eq!(verify_golden(&json_path, reviewed_json)?.len(), 64);
    assert_eq!(verify_golden(&binary_path, reviewed_binary)?.len(), 64);
    let first = canonical_json(&Fixture {
        z_field: 7,
        a_field: "stable",
    })?;
    let second = canonical_json(&Fixture {
        z_field: 7,
        a_field: "stable",
    })?;
    assert_eq!(first, second);
    Ok(())
}
