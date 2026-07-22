#![allow(missing_docs, clippy::unwrap_used)]
use hazard_intelligence::*;
fn d(n: u8) -> [u8; 32] {
    [n; 32]
}
#[test]
fn exact_fallback_is_deterministic() {
    let items = vec![
        VisualItem::new("b", d(2), 10, vec![2, 0]).unwrap(),
        VisualItem::new("a", d(1), 10, vec![1, 0]).unwrap(),
    ];
    let index =
        ExactVisualIndex::build(IndexManifest::new("i1", d(9), d(8), items.clone()).unwrap())
            .unwrap();
    assert_eq!(
        index
            .search(&[1, 0], 2)
            .unwrap()
            .iter()
            .map(|x| x.id.as_str())
            .collect::<Vec<_>>(),
        vec!["a", "b"]
    );
}
#[test]
fn manifests_are_immutable_and_calibrated() {
    let m = IndexManifest::new(
        "i",
        d(1),
        d(2),
        vec![VisualItem::new("a", d(3), 0, vec![1]).unwrap()],
    )
    .unwrap();
    assert_ne!(m.digest(), [0; 32]);
    assert_eq!(m.calibration_digest(), d(2));
}
#[test]
fn rebuild_is_atomic_and_recovers_previous_generation() {
    let first = IndexManifest::new(
        "v1",
        d(1),
        d(2),
        vec![VisualItem::new("a", d(3), 0, vec![1]).unwrap()],
    )
    .unwrap();
    let mut store = AtomicIndexStore::activate(first.clone()).unwrap();
    assert!(
        store
            .rebuild(Err(
                IndexManifest::new("bad", d(1), d(2), vec![]).unwrap_err()
            ))
            .is_err()
    );
    assert_eq!(store.active_digest(), first.digest());
}
#[test]
fn candidate_never_becomes_observation_without_independent_verification() {
    let mut c = RetrievalCandidate::new("media", "obs", 8000);
    c.georegister(GeoRegistration {
        method: "control-points".into(),
        error_m: 20,
    });
    assert!(!c.is_verified_observation());
    c.verify(VerificationLabel {
        reviewer: "analyst".into(),
        method: "source-comparison".into(),
        confidence_bps: 9000,
    })
    .unwrap();
    assert!(c.is_verified_observation());
}
#[test]
fn rupixel_remains_disabled_without_evaluation() {
    assert_eq!(
        ExternalRupixel::status(),
        ExternalIndexStatus::DisabledPendingEvaluation
    );
}
