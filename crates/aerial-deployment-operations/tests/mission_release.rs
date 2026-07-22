#![allow(clippy::expect_used)]
#![allow(missing_docs)]
use aerial_deployment_operations::*;
use chrono::{Duration, TimeZone, Utc};
use std::collections::BTreeSet;

fn at() -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(10_000, 0).single().expect("time")
}
fn digest(byte: char) -> String {
    format!("sha256:{}", byte.to_string().repeat(64))
}
fn evidence(name: &str) -> EvidenceRef {
    EvidenceRef::new(
        EvidenceId::new(name).expect("id"),
        &digest('a'),
        "evidence://signed",
        at() - Duration::hours(1),
        Some(at() + Duration::hours(1)),
    )
    .expect("evidence")
}
fn bindings(tail: &str) -> MissionBindings {
    MissionBindings {
        payload: PayloadManifestId::new("manifest-1").expect("id"),
        payload_digest: digest('b'),
        aircraft: AircraftBinding::new(
            AircraftConfigurationId::new("aircraft-rev-7").expect("id"),
            tail,
        )
        .expect("aircraft"),
        route: evidence("route"),
        corridor: ReleaseCorridorId::new("corridor").expect("id"),
        nominal_footprint: FootprintId::new("nominal").expect("id"),
        failed_component_footprints: vec![FootprintId::new("failed-parafoil").expect("id")],
        exclusion_volume: ExclusionZoneId::new("exclusion-3d").expect("id"),
        jettison_sectors: vec![JettisonZoneId::new("jettison-east").expect("id")],
        emergency_landing_zones: vec![EmergencyLandingZoneId::new("elz-west").expect("id")],
        ground_boundary: FootprintId::new("ground-boundary").expect("id"),
        point_of_no_return: evidence("pnr"),
        alternate_abort_plan: evidence("abort-plan"),
        odd: OddId::new("odd-rev-4").expect("id"),
        odd_evidence: evidence("odd-evidence"),
    }
}
fn mission(tail: &str) -> AerialDropMission {
    AerialDropMission::new(
        AerialDropMissionId::new("mission-1").expect("id"),
        bindings(tail),
        BTreeSet::from([
            LeastHarmContingency::Retain,
            LeastHarmContingency::EmergencyLand,
        ]),
    )
    .expect("mission")
}
fn current_observation(condition: ReleaseCondition) -> SourceObservation {
    SourceObservation::new(
        condition,
        evidence(&format!("obs-{condition:?}")),
        at() - Duration::minutes(1),
        at() + Duration::minutes(1),
        true,
        9_500,
    )
    .expect("observation")
}
fn make_current(mission: &mut AerialDropMission) {
    for condition in [
        ReleaseCondition::AircraftHealth,
        ReleaseCondition::PayloadHealth,
        ReleaseCondition::Weather,
        ReleaseCondition::Wind,
        ReleaseCondition::Turbulence,
        ReleaseCondition::Smoke,
        ReleaseCondition::Icing,
        ReleaseCondition::Airspace,
        ReleaseCondition::Terrain,
        ReleaseCondition::FirePosition,
        ReleaseCondition::PeopleVehiclesAircraft,
        ReleaseCondition::NavigationTime,
        ReleaseCondition::Communications,
        ReleaseCondition::SurveillanceConfidence,
        ReleaseCondition::GroundReadiness,
    ] {
        mission
            .observe(mission.version(), current_observation(condition))
            .expect("observe");
    }
}
fn decisions(mission: &AerialDropMission) -> (AuthorityDecision, AuthorityDecision) {
    let command_digest = mission.canonical_release_digest();
    let base = AuthorityDecision {
        authority: AuthorityRole::AircraftAuthority,
        outcome: DecisionOutcome::Approve,
        command_digest: command_digest.clone(),
        decided_at: at() - Duration::seconds(5),
        expires_at: at() + Duration::seconds(30),
        evidence: evidence("aircraft-key"),
    };
    let incident = AuthorityDecision {
        authority: AuthorityRole::IncidentSafety,
        outcome: DecisionOutcome::Approve,
        command_digest,
        decided_at: at() - Duration::seconds(4),
        expires_at: at() + Duration::seconds(30),
        evidence: evidence("incident-key"),
    };
    (base, incident)
}

#[test]
fn exact_aircraft_tail_is_in_canonical_digest_and_both_keys_must_match() {
    let alpha = mission("TAIL-ALPHA");
    let bravo = mission("TAIL-BRAVO");
    assert_ne!(
        alpha.canonical_release_digest(),
        bravo.canonical_release_digest()
    );
    let (aircraft, mut incident) = decisions(&alpha);
    incident.command_digest = bravo.canonical_release_digest();
    let mut alpha = alpha;
    make_current(&mut alpha);
    assert_eq!(
        alpha.commit_release(alpha.version(), at(), 9_000, &aircraft, &incident),
        Err(DomainError::ReleaseDigestMismatch)
    );
}

#[test]
fn stale_data_intrusion_comms_and_surveillance_each_inhibit_release() {
    for condition in [
        ReleaseCondition::PeopleVehiclesAircraft,
        ReleaseCondition::Communications,
        ReleaseCondition::SurveillanceConfidence,
        ReleaseCondition::Wind,
    ] {
        let mut subject = mission("TAIL-1");
        make_current(&mut subject);
        subject
            .observe(
                subject.version(),
                SourceObservation::new(
                    condition,
                    evidence("unsafe"),
                    at() - Duration::minutes(2),
                    at() + Duration::minutes(1),
                    false,
                    9_900,
                )
                .expect("observation"),
            )
            .expect("observe");
        let (a, i) = decisions(&subject);
        assert_eq!(
            subject.commit_release(subject.version(), at(), 9_000, &a, &i),
            Err(DomainError::UnsafeOrStaleObservation)
        );
    }
    let mut stale = mission("TAIL-1");
    make_current(&mut stale);
    stale
        .observe(
            stale.version(),
            SourceObservation::new(
                ReleaseCondition::Weather,
                evidence("stale"),
                at() - Duration::minutes(3),
                at() - Duration::minutes(1),
                true,
                9_900,
            )
            .expect("observation"),
        )
        .expect("observe");
    let (a, i) = decisions(&stale);
    assert_eq!(
        stale.commit_release(stale.version(), at(), 9_000, &a, &i),
        Err(DomainError::UnsafeOrStaleObservation)
    );
}

#[test]
fn hold_veto_abort_are_never_inferred_from_mission_approval() {
    for outcome in [
        DecisionOutcome::Hold,
        DecisionOutcome::Veto,
        DecisionOutcome::Abort,
    ] {
        let mut subject = mission("TAIL-1");
        make_current(&mut subject);
        let (mut aircraft, incident) = decisions(&subject);
        aircraft.outcome = outcome;
        assert_eq!(
            subject.commit_release(subject.version(), at(), 9_000, &aircraft, &incident),
            Err(DomainError::ReleaseInhibited)
        );
    }
}

#[test]
fn optimistic_race_and_identical_replay_cannot_broaden_release() {
    let mut subject = mission("TAIL-1");
    make_current(&mut subject);
    let stale_version = subject.version();
    subject
        .observe(
            subject.version(),
            current_observation(ReleaseCondition::Wind),
        )
        .expect("observe");
    let (a, i) = decisions(&subject);
    assert_eq!(
        subject.commit_release(stale_version, at(), 9_000, &a, &i),
        Err(DomainError::VersionConflict)
    );
    let version = subject.version();
    subject
        .commit_release(version, at(), 9_000, &a, &i)
        .expect("release");
    subject
        .commit_release(version, at(), 9_000, &a, &i)
        .expect("idempotent replay");
    let mut contradictory = a.clone();
    contradictory.outcome = DecisionOutcome::Hold;
    assert_eq!(
        subject.commit_release(version, at(), 9_000, &contradictory, &i),
        Err(DomainError::ReplayConflict)
    );
    assert!(subject.released());
    assert_eq!(
        subject.execute_post_release(LeastHarmContingency::SafeSectorJettison),
        Err(DomainError::ContingencyNotAuthorized)
    );
    assert!(
        subject
            .execute_post_release(LeastHarmContingency::EmergencyLand)
            .is_ok()
    );
    assert_eq!(
        subject.broaden_or_reroute(),
        Err(DomainError::OperationalBoundaryCrossed)
    );
}

#[test]
fn authority_keys_must_be_distinct_and_confidence_floor_cannot_be_weakened() {
    let mut subject = mission("TAIL-1");
    make_current(&mut subject);
    let (aircraft, mut incident) = decisions(&subject);
    incident.evidence = aircraft.evidence.clone();
    assert_eq!(
        subject.commit_release(subject.version(), at(), 9_000, &aircraft, &incident),
        Err(DomainError::ReleaseDigestMismatch)
    );
    let (aircraft, incident) = decisions(&subject);
    assert_eq!(
        subject.commit_release(subject.version(), at(), 0, &aircraft, &incident),
        Err(DomainError::UnsafeOrStaleObservation)
    );
}

#[test]
fn pnr_blocks_reroute_but_does_not_compel_release() {
    let mut subject = mission("TAIL-1");
    subject
        .cross_point_of_no_return(subject.version())
        .expect("cross pnr");
    assert_eq!(
        subject.broaden_or_reroute(),
        Err(DomainError::OperationalBoundaryCrossed)
    );
    let (a, i) = decisions(&subject);
    assert_eq!(
        subject.commit_release(subject.version(), at(), 9_000, &a, &i),
        Err(DomainError::UnsafeOrStaleObservation)
    );
}

#[test]
fn expired_odd_binding_cannot_be_overridden_by_current_observations() {
    let mut bindings = bindings("TAIL-1");
    bindings.odd_evidence = EvidenceRef::new(
        EvidenceId::new("expired-odd").expect("id"),
        &digest('c'),
        "evidence://signed",
        at() - Duration::hours(2),
        Some(at() - Duration::hours(1)),
    )
    .expect("historic evidence");
    let mut subject = AerialDropMission::new(
        AerialDropMissionId::new("mission-expired-odd").expect("id"),
        bindings,
        BTreeSet::from([LeastHarmContingency::Retain]),
    )
    .expect("mission");
    make_current(&mut subject);
    let (aircraft, incident) = decisions(&subject);
    assert_eq!(
        subject.commit_release(subject.version(), at(), 9_000, &aircraft, &incident),
        Err(DomainError::UnsafeOrStaleObservation)
    );
}
