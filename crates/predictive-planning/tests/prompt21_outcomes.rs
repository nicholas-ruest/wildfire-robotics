#![allow(missing_docs, clippy::unwrap_used)]
use predictive_planning::*;
#[test]
fn dataset_alignment_blocks_leakage_and_records_bias_and_censoring() {
    let rows = [
        AlignedOutcome::new(
            "p1",
            "verified-media-1",
            "incident-a",
            "geo-a",
            2025,
            Some(false),
            true,
            "targeted",
            Some("patrol"),
        )
        .unwrap(),
        AlignedOutcome::new(
            "p2",
            "verified-media-2",
            "incident-a",
            "geo-b",
            2026,
            None,
            false,
            "none",
            None,
        )
        .unwrap(),
    ];
    assert_eq!(
        DatasetSnapshot::freeze("ds", &rows[..1], &rows[1..]).unwrap_err(),
        OutcomeError::Leakage("incident")
    );
    assert!(!rows[0].censored);
    assert!(rows[1].censored);
}
