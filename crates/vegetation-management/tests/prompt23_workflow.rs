#![allow(missing_docs, clippy::unwrap_used)]
use std::collections::BTreeSet;
use vegetation_management::*;
fn polygon() -> Polygon {
    Polygon::new(vec![
        Point { x_mm: 0, y_mm: 0 },
        Point {
            x_mm: 10000,
            y_mm: 0,
        },
        Point {
            x_mm: 10000,
            y_mm: 10000,
        },
    ])
    .unwrap()
}
fn package() -> WorkPackage {
    let unit = TreatmentUnit::new(
        "u",
        polygon(),
        3,
        "survey",
        "fuel",
        vec![Exclusion {
            kind: ExclusionKind::Cultural,
            area: Polygon::new(vec![
                Point {
                    x_mm: 100,
                    y_mm: 100,
                },
                Point {
                    x_mm: 200,
                    y_mm: 100,
                },
                Point {
                    x_mm: 200,
                    y_mm: 200,
                },
            ])
            .unwrap(),
        }],
    )
    .unwrap();
    let rx = Prescription::new(
        "rx",
        Method::Mechanical,
        ToolKind::Mulcher,
        2500,
        OperationalEnvelope {
            max_fire_danger_bps: 4000,
            min_localization_quality_bps: 8000,
        },
        "auth",
        3,
    )
    .unwrap();
    WorkPackage::plan("wp", unit, rx, 1000).unwrap()
}
fn mission() -> MissionSnapshot {
    MissionSnapshot {
        authority_id: "auth".into(),
        lease_id: "lease".into(),
        fence_digest: [8; 32],
        geometry_revision: 3,
        issued_tick: 10,
        expires_tick: 20,
    }
}
fn prepare() -> WorkPackage {
    let mut w = package();
    w.authorize(&mission(), 15).unwrap();
    w.admit_resources(
        &LogisticsSnapshot {
            capacity_kg: 2000,
            custody_ready: true,
        },
        &CareSnapshot {
            robot_serviceable: true,
        },
    )
    .unwrap();
    w.start().unwrap();
    w
}
#[test]
fn mission_expiry_is_exclusive_and_resources_cannot_be_bypassed() {
    let mut w = package();
    assert_eq!(
        w.authorize(&mission(), 20).unwrap_err(),
        VegetationError::MissingAuthority
    );
    w.authorize(&mission(), 15).unwrap();
    assert_eq!(w.start().unwrap_err(), VegetationError::InvalidTransition);
}
#[test]
fn resource_evidence_is_required_and_stored() {
    let mut w = package();
    w.authorize(&mission(), 15).unwrap();
    assert_eq!(
        w.admit_resources(
            &LogisticsSnapshot {
                capacity_kg: 999,
                custody_ready: true
            },
            &CareSnapshot {
                robot_serviceable: true
            }
        )
        .unwrap_err(),
        VegetationError::InsufficientBiomassCapacity
    );
    w.admit_resources(
        &LogisticsSnapshot {
            capacity_kg: 1000,
            custody_ready: true,
        },
        &CareSnapshot {
            robot_serviceable: true,
        },
    )
    .unwrap();
    assert_ne!(w.custody_digest().unwrap(), [0; 32]);
}
#[test]
fn inhibits_latch_and_structured_reset_is_one_use() {
    let mut w = prepare();
    w.observe(SafetyObservation::PersonDetected);
    w.observe(SafetyObservation::Clear);
    assert_eq!(w.tool_state(), ToolState::Inhibited);
    let reset = ResetAuthorization {
        id: "r1".into(),
        work_package_id: "wp".into(),
        actor: "inspector".into(),
        inspection_digest: [7; 32],
        authority_id: "auth".into(),
        fence_digest: [8; 32],
        issued_tick: 14,
        expires_tick: 18,
        cleared_causes: BTreeSet::from([InhibitReason::Person]),
    };
    w.rearm(reset.clone(), &mission(), 15).unwrap();
    w.observe(SafetyObservation::PersonDetected);
    assert_eq!(
        w.rearm(reset, &mission(), 15).unwrap_err(),
        VegetationError::IndependentReleaseRequired
    );
}
#[test]
fn expired_wrong_or_incomplete_reset_fails() {
    let mut w = prepare();
    w.observe(SafetyObservation::UtilityDetected);
    let reset = ResetAuthorization {
        id: "r".into(),
        work_package_id: "other".into(),
        actor: "i".into(),
        inspection_digest: [1; 32],
        authority_id: "auth".into(),
        fence_digest: [8; 32],
        issued_tick: 10,
        expires_tick: 15,
        cleared_causes: BTreeSet::new(),
    };
    assert_eq!(
        w.rearm(reset, &mission(), 15).unwrap_err(),
        VegetationError::IndependentReleaseRequired
    );
}
#[test]
fn every_hazard_inhibits_with_evidence() {
    for f in [
        SafetyObservation::InsideExclusion,
        SafetyObservation::PersonDetected,
        SafetyObservation::WildlifeDetected,
        SafetyObservation::UtilityDetected,
        SafetyObservation::FireDanger { bps: 5000 },
        SafetyObservation::LocalizationQuality { bps: 7000 },
        SafetyObservation::ToolFault,
        SafetyObservation::CommunicationsLost,
    ] {
        let mut w = prepare();
        w.observe(f);
        assert_eq!(w.tool_state(), ToolState::Inhibited);
        assert_eq!(w.evidence().len(), 1);
    }
}
#[test]
fn simulator_requires_running_armed_and_accounts_exclusions() {
    assert_eq!(
        DeterministicGroundRobot.execute(&package(), 1).unwrap_err(),
        VegetationError::SimulationUnavailable
    );
    let w = prepare();
    let e = DeterministicGroundRobot.execute(&w, 1).unwrap();
    assert!(e.missed_area_mm2 > 0);
    assert_eq!(e.planned_biomass_kg, 1000);
}
#[test]
fn completion_bypass_fails_and_effectiveness_is_separate() {
    let mut fresh = package();
    let fake = ExecutionEvidence {
        planned_biomass_kg: 1000,
        actual_biomass_kg: 900,
        geometry_revision: 3,
        envelope: fresh.prescription().envelope,
        trace_digest: [1; 32],
        custody_digest: [2; 32],
        missed_area_mm2: 1,
    };
    assert_eq!(
        fresh.record_completion(fake).unwrap_err(),
        VegetationError::InvalidTransition
    );
    let mut w = prepare();
    let e = DeterministicGroundRobot.execute(&w, 2).unwrap();
    w.end_execution().unwrap();
    w.record_completion(e).unwrap();
    assert!(w.is_complete());
    assert!(w.effectiveness().is_none());
    let a = EffectivenessAssessment::assess(
        "a",
        &w,
        vec![LongitudinalObservation {
            days_after: 90,
            residual_fuel_bps: 2400,
            evidence_digest: [2; 32],
        }],
    )
    .unwrap();
    assert!(a.target_met());
}
#[test]
fn drone_support_is_advisory() {
    let r = DeterministicDroneSupport
        .survey(&package().unit, 3)
        .unwrap();
    assert_eq!(r.authority, SupportAuthority::AdvisoryOnly);
}
