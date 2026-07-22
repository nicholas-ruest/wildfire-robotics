#![allow(missing_docs, clippy::unwrap_used)]
use chrono::{TimeZone, Utc};
use predictive_planning::*;

fn record(id: &str, incident: &str, geo: &str, year: i32, label: Option<bool>) -> LearningRecord {
    LearningRecord {
        id: id.into(),
        incident_id: incident.into(),
        geography: geo.into(),
        fire_year: year,
        event_time: Utc.with_ymd_and_hms(year, 7, 1, 0, 0, 0).unwrap(),
        features: LightningFeatures {
            strike_observed: true,
            age_minutes: 60,
            weather_dryness_bps: 7000,
            fuel_receptivity_bps: 6000,
            terrain_slope_bps: 1000,
            hotspot_observed: false,
        },
        ignition_outcome: label,
        location_uncertainty_m: 250,
        time_uncertainty_minutes: 15,
        reconnaissance_selected: false,
        censored: label.is_none(),
        intervention: None,
    }
}

#[test]
fn should_assemble_only_complete_authoritative_snapshot() {
    let parts = vec![
        SnapshotPart::authoritative(DataKind::Lightning, "lightning", [1; 32]),
        SnapshotPart::authoritative(DataKind::Weather, "weather", [2; 32]),
        SnapshotPart::authoritative(DataKind::Fuels, "fuels", [3; 32]),
        SnapshotPart::authoritative(DataKind::Terrain, "terrain", [4; 32]),
        SnapshotPart::authoritative(DataKind::Hotspots, "hotspots", [5; 32]),
    ];
    let snapshot = AuthoritativeSnapshot::assemble("S1", parts).unwrap();
    assert_eq!(snapshot.parts().len(), 5);
}
#[test]
fn should_produce_transparent_reproducible_baseline_with_abstention() {
    let baseline = OperationalBaseline::new([9; 32], BaselineThresholds::default());
    let uncertain = record("1", "i1", "g1", 2025, None);
    let a = baseline.infer(
        &uncertain.features,
        uncertain.location_uncertainty_m,
        uncertain.time_uncertainty_minutes,
    );
    let b = baseline.infer(
        &uncertain.features,
        uncertain.location_uncertainty_m,
        uncertain.time_uncertainty_minutes,
    );
    assert_eq!(a, b);
    assert!(a.explanation.contains("dryness"));
    assert!(a.abstained);
}
#[test]
fn should_block_incident_geography_time_and_fire_year_leakage() {
    let rows = [
        record("1", "incident", "geo", 2025, Some(true)),
        record("2", "incident", "other", 2026, Some(false)),
    ];
    assert_eq!(
        LeakageGuard::validate(&rows[..1], &rows[1..]).unwrap_err(),
        LearningError::Leakage("incident")
    );
}
#[test]
fn should_require_rare_event_calibration_and_prospective_shadow_evidence() {
    let mut study = LightningEvaluation::evaluate(&[
        PredictionOutcome {
            probability_bps: 9000,
            outcome: true,
        },
        PredictionOutcome {
            probability_bps: 1000,
            outcome: false,
        },
    ])
    .unwrap();
    assert!(!study.can_promote());
    study.attach_calibration([7; 32], 200);
    study.record_shadow(ShadowEvidence {
        prospective_cases: 100,
        observed_ignitions: 5,
        baseline_brier_bps: 2000,
        candidate_brier_bps: 1500,
    });
    assert!(study.can_promote());
    assert!(study.metrics().precision_bps > 0);
}
#[test]
fn should_record_sampling_censoring_intervention_and_negative_outcomes() {
    let mut row = record("1", "i", "g", 2025, Some(false));
    row.reconnaissance_selected = true;
    row.intervention = Some("patrol dispatched".into());
    assert!(row.validate().is_ok());
    assert_eq!(row.ignition_outcome, Some(false));
}
#[test]
fn should_withdraw_candidate_on_material_drift_or_broken_lineage() {
    let mut monitor = ModelMonitor::new([1; 32], 100);
    assert_eq!(monitor.observe(250, [1; 32]), MonitoringDecision::Withdraw);
    assert_eq!(monitor.observe(10, [2; 32]), MonitoringDecision::Withdraw);
}
