#![allow(missing_docs)]
#![allow(clippy::expect_used)]
use aerial_deployment_operations::*;
use chrono::{Duration, TimeZone, Utc};

fn now() -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(10_000, 0).single().expect("time")
}
fn prerequisites() -> GroundPrerequisites {
    let at = now();
    GroundPrerequisites::new(
        PrerequisiteKind::ALL
            .into_iter()
            .map(|kind| PrerequisiteAssessment {
                kind,
                satisfied: true,
                uncertainty_bps: 25,
                observed_at: at - Duration::seconds(1),
                valid_until: at + Duration::minutes(1),
            })
            .collect(),
    )
}
fn sensors() -> SensorSuite {
    let at = now();
    SensorSuite::new(
        SensorKind::ALL
            .into_iter()
            .map(|kind| SensorReading {
                kind,
                value: 0,
                uncertainty: 2,
                observed_at: at - Duration::seconds(1),
                valid_until: at + Duration::minutes(1),
            })
            .collect(),
    )
}
fn installation() -> (GroundInstallation, GroundZoneId, GroundZoneId) {
    let a = GroundZoneId::new("zone-a").expect("zone");
    let b = GroundZoneId::new("zone-b").expect("zone");
    (
        GroundInstallation::new(
            GroundInstallationId::new("ground-1").expect("id"),
            vec![a.clone(), b.clone()],
            100,
            10,
        )
        .expect("installation"),
        a,
        b,
    )
}
fn activate(subject: &mut GroundInstallation, zone: &GroundZoneId) {
    let evidence = prerequisites();
    subject
        .advance(
            zone,
            InstallationPhase::Transitioning,
            &evidence,
            None,
            now(),
        )
        .expect("transition");
    subject
        .advance(zone, InstallationPhase::Anchoring, &evidence, None, now())
        .expect("anchor");
    subject
        .advance(zone, InstallationPhase::Sealing, &evidence, None, now())
        .expect("seal");
    subject
        .advance(
            zone,
            InstallationPhase::Active,
            &evidence,
            Some(sensors()),
            now(),
        )
        .expect("active");
}

#[test]
fn should_execute_full_ground_lifecycle_per_bounded_zone() {
    let (mut subject, zone, _) = installation();
    activate(&mut subject, &zone);
    subject
        .apply_fault(&zone, GroundFault::Gap)
        .expect("degrade");
    subject
        .advance(
            &zone,
            InstallationPhase::Recovering,
            &prerequisites(),
            None,
            now(),
        )
        .expect("recover");
    subject
        .advance(
            &zone,
            InstallationPhase::TemporarilyLeft,
            &prerequisites(),
            None,
            now(),
        )
        .expect("left");
    assert_eq!(
        subject.zone(&zone).expect("zone").phase,
        InstallationPhase::TemporarilyLeft
    );
}

#[test]
fn should_require_every_current_certain_prerequisite_before_work() {
    for missing in PrerequisiteKind::ALL {
        let (mut subject, zone, _) = installation();
        let at = now();
        let assessments = PrerequisiteKind::ALL
            .into_iter()
            .filter(|kind| *kind != missing)
            .map(|kind| PrerequisiteAssessment {
                kind,
                satisfied: true,
                uncertainty_bps: 0,
                observed_at: at,
                valid_until: at + Duration::seconds(1),
            })
            .collect();
        assert_eq!(
            subject.advance(
                &zone,
                InstallationPhase::Transitioning,
                &GroundPrerequisites::new(assessments),
                None,
                at
            ),
            Err(DomainError::GroundWorkInhibited)
        );
    }
}

#[test]
fn should_reject_stale_or_uncertain_sensor_suite_without_claiming_effectiveness() {
    let (mut subject, zone, _) = installation();
    let evidence = prerequisites();
    for phase in [
        InstallationPhase::Transitioning,
        InstallationPhase::Anchoring,
        InstallationPhase::Sealing,
    ] {
        subject
            .advance(&zone, phase, &evidence, None, now())
            .expect("phase");
    }
    let mut readings: Vec<_> = SensorKind::ALL
        .into_iter()
        .map(|kind| SensorReading {
            kind,
            value: 0,
            uncertainty: 2,
            observed_at: now() - Duration::minutes(2),
            valid_until: now() - Duration::minutes(1),
        })
        .collect();
    readings[0].uncertainty = 100;
    assert_eq!(
        subject.advance(
            &zone,
            InstallationPhase::Active,
            &evidence,
            Some(SensorSuite::new(readings)),
            now()
        ),
        Err(DomainError::GroundSensingInhibited)
    );
    assert_eq!(
        subject.zone(&zone).expect("zone").effectiveness,
        EffectivenessAssessment::Unknown
    );
}

#[test]
fn should_inhibit_only_affected_zone_for_each_named_fault() {
    let faults = [
        GroundFault::TerrainIntrusion,
        GroundFault::UtilityIntrusion,
        GroundFault::StaleSensing,
        GroundFault::AnchorPullout,
        GroundFault::Uplift,
        GroundFault::Tear,
        GroundFault::HeatTransfer,
        GroundFault::Gap,
        GroundFault::RobotTool,
        GroundFault::LostCommunications,
    ];
    for fault in faults {
        let (mut subject, affected, neighbor) = installation();
        activate(&mut subject, &affected);
        activate(&mut subject, &neighbor);
        subject
            .apply_fault(&affected, fault)
            .expect("bounded policy");
        assert!(subject.zone(&affected).expect("affected").inhibited);
        assert_eq!(
            subject.zone(&affected).expect("affected").effectiveness,
            EffectivenessAssessment::Unknown
        );
        assert!(!subject.zone(&neighbor).expect("neighbor").inhibited);
        assert_eq!(
            subject.zone(&neighbor).expect("neighbor").phase,
            InstallationPhase::Active
        );
    }
}

#[test]
fn should_emit_authority_free_requests_for_externally_owned_work() {
    let (mut subject, zone, _) = installation();
    for kind in [
        ExternalWorkKind::SuppressantApplication,
        ExternalWorkKind::VegetationWork,
        ExternalWorkKind::MissionExpansion,
        ExternalWorkKind::RobotCapability,
    ] {
        let request = subject
            .request_external_work(&zone, kind, "owner review required")
            .expect("request");
        assert_eq!(request.kind, kind);
    }
    assert_eq!(
        subject.zone(&zone).expect("zone").external_requests.len(),
        4
    );
}

#[test]
fn should_bound_local_policies_and_reject_local_suppression_authority() {
    let (mut subject, zone, neighbor) = installation();
    for action in [
        GroundAction::Pause,
        GroundAction::Vent,
        GroundAction::Isolate,
        GroundAction::Reposition,
        GroundAction::Escalate,
    ] {
        subject
            .apply_local_policy(&zone, action)
            .expect("owned action");
    }
    assert_eq!(
        subject.apply_local_policy(&zone, GroundAction::RequestSuppression),
        Err(DomainError::GroundPolicyInhibited)
    );
    for action in [
        GroundAction::Pause,
        GroundAction::Vent,
        GroundAction::Isolate,
        GroundAction::Reposition,
        GroundAction::Escalate,
    ] {
        assert!(subject.has_applied_action(&zone, action));
    }
    assert!(!subject.zone(&neighbor).expect("neighbor").inhibited);
}

#[test]
fn should_continuously_degrade_only_zone_whose_sensor_suite_expires() {
    let (mut subject, affected, neighbor) = installation();
    activate(&mut subject, &affected);
    activate(&mut subject, &neighbor);
    let after_expiry = now() + Duration::minutes(2);
    assert_eq!(
        subject.reevaluate_sensing(&affected, after_expiry),
        Err(DomainError::GroundSensingInhibited)
    );
    assert_eq!(
        subject.zone(&affected).expect("affected").phase,
        InstallationPhase::Degraded
    );
    assert_eq!(
        subject.zone(&neighbor).expect("neighbor").phase,
        InstallationPhase::Active
    );
}
