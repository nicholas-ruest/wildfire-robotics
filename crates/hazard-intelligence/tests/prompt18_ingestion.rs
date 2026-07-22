//! Prompt 18 ingestion, provenance, quarantine, and picture invariants.
#![allow(clippy::unwrap_used)]
use chrono::{Duration, Utc};
use hazard_intelligence::*;
use sha2::{Digest as _, Sha256};
use shared_kernel::{CoordinateReferenceSystem, EntityId, GeoPoint, Quantity, Unit};
use std::fmt::Write as _;
fn source() -> Source {
    let mut s = Source::register(
        EntityId::new(),
        "authority",
        "ops-only",
        [1; 32],
        "region-a",
    )
    .unwrap();
    s.activate(true).unwrap();
    s
}
fn observation(source: &Source, identity: &str, version: &str, digest: u8) -> Observation {
    let now = Utc::now();
    Observation {
        id: EntityId::new(),
        provider_identity: identity.into(),
        source_id: source.id.clone(),
        source_version: version.into(),
        license: source.license.clone(),
        event_time: now - Duration::seconds(1),
        ingested_at: now,
        geometry: GeoPoint::new(-120.0, 50.0, CoordinateReferenceSystem::WGS84, None).unwrap(),
        quantity: Quantity::new(5.0, Unit::Celsius).unwrap(),
        uncertainty: 2,
        quality_bps: 9_000,
        lineage: vec![[2; 32]],
        content_digest: [digest; 32],
        correction_of: None,
        operational: true,
    }
}

#[test]
fn duplicate_is_idempotent_but_same_provider_version_different_content_conflicts() {
    let s = source();
    let mut set = ObservationSet::open(EntityId::new());
    let first = observation(&s, "obs-1", "v1", 3);
    assert!(set.append(first.clone(), &s).unwrap());
    assert!(!set.append(first, &s).unwrap());
    assert_eq!(
        set.append(observation(&s, "obs-1", "v1", 4), &s),
        Err(HazardError::ContradictoryDuplicate)
    );
}
#[test]
fn unlicensed_invalid_and_nonoperational_data_is_quarantined_and_never_pictured() {
    let s = source();
    let mut set = ObservationSet::open(EntityId::new());
    for mutation in 0_u8..3 {
        let mut o = observation(&s, &format!("bad-{mutation}"), "v1", mutation + 3);
        match mutation {
            0 => o.license = "forbidden".into(),
            1 => o.quality_bps = 10_001,
            _ => o.operational = false,
        }
        assert_eq!(set.append(o, &s), Err(HazardError::Quarantined));
    }
    assert_eq!(set.quarantined_count(), 3);
    assert_eq!(set.seal([9; 32]), Err(HazardError::InvalidTransition));
}
#[test]
fn correction_appends_and_preserves_original_while_picture_uses_successor() {
    let s = source();
    let mut set = ObservationSet::open(EntityId::new());
    let original = observation(&s, "obs", "v1", 3);
    let original_id = original.id.clone();
    set.append(original, &s).unwrap();
    let mut correction = observation(&s, "obs", "v2", 4);
    correction.correction_of = Some(original_id.clone());
    set.correct(&original_id, correction, &s).unwrap();
    set.seal([8; 32]).unwrap();
    assert_eq!(set.operational().count(), 1);
    let now = Utc::now();
    let picture = HazardPicture::build(
        EntityId::new(),
        &set,
        now - Duration::seconds(1),
        now + Duration::minutes(5),
        60,
        vec![],
        "canonical-v1",
    )
    .unwrap();
    assert!(picture.observation_digests.contains(&[4; 32]));
    assert!(!picture.observation_digests.contains(&[3; 32]));
}
#[test]
fn late_data_remains_source_claim_and_picture_freshness_expires_exclusively() {
    let s = source();
    let mut set = ObservationSet::open(EntityId::new());
    let mut late = observation(&s, "late", "v1", 5);
    late.event_time -= Duration::days(1);
    set.append(late, &s).unwrap();
    set.seal([7; 32]).unwrap();
    let now = Utc::now();
    let mut picture = HazardPicture::build(
        EntityId::new(),
        &set,
        now - Duration::seconds(1),
        now + Duration::seconds(1),
        1,
        vec![Gap {
            area: "north".into(),
            reason: "provider outage".into(),
            since: now,
        }],
        "v1",
    )
    .unwrap();
    picture.publish(now).unwrap();
    picture.project_freshness(picture.valid_until);
    assert_eq!(picture.state, PictureState::Stale);
    assert_eq!(picture.gaps.len(), 1);
}
#[test]
fn visual_similarity_cannot_become_verified_without_alignment_method_and_confidence() {
    let now = Utc::now();
    let mut visual = VisualEvidenceSet::new(EntityId::new());
    visual
        .register(ObjectEvidence {
            object_key: "raw/frame.tif".into(),
            media_type: "image/tiff".into(),
            checksum: [1; 32],
            captured_at: now,
            footprint_digest: [2; 32],
            sensor_calibration_digest: [3; 32],
        })
        .unwrap();
    visual.index("fixture-index-v1").unwrap();
    assert_eq!(
        visual.verify("candidate", false, "review", 8_000),
        Err(HazardError::UnverifiedVisual)
    );
    assert_eq!(
        visual.verify("candidate", true, "", 8_000),
        Err(HazardError::UnverifiedVisual)
    );
    visual
        .verify("candidate", true, "independent-geospatial-review", 8_000)
        .unwrap();
    assert_eq!(visual.state, VisualState::Verified);
    assert_eq!(visual.raw.len(), 1);
}
#[test]
fn canonical_file_adapter_is_deterministic_and_provider_outage_preserves_records() {
    let checksum = Sha256::digest(b"payload")
        .iter()
        .fold(String::new(), |mut output, byte| {
            write!(&mut output, "{byte:02x}").unwrap();
            output
        });
    let input = format!("id-1|v1|payload|{checksum}|ops-only\n");
    let mut adapter = CanonicalFileAdapter::parse(&input).unwrap();
    let first = adapter.fetch(None).unwrap();
    assert_eq!(first, adapter.fetch(None).unwrap());
    adapter.set_outage(true);
    assert_eq!(adapter.fetch(None), Err(HazardError::InvalidTransition));
    adapter.set_outage(false);
    assert_eq!(adapter.fetch(None).unwrap(), first);
}
#[test]
fn suspended_or_withdrawn_source_cannot_feed_a_picture() {
    let mut s = source();
    s.suspend();
    let mut set = ObservationSet::open(EntityId::new());
    assert_eq!(
        set.append(observation(&s, "id", "v1", 3), &s),
        Err(HazardError::Quarantined)
    );
    assert_eq!(set.quarantined_count(), 1);
}
