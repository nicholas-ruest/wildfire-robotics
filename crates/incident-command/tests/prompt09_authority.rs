//! Prompt 09 acceptance, concurrency, expiry, replay, and property tests.
#![allow(clippy::expect_used)]

use chrono::{DateTime, Duration, Utc};
use incident_command::*;
use proptest::prelude::*;
use shared_kernel::{EntityId, TimeWindow};

fn now() -> DateTime<Utc> {
    DateTime::<Utc>::UNIX_EPOCH + Duration::days(20)
}
fn envelope(
    zones: &[&str],
    capabilities: &[&str],
    resources: u32,
    hours: i64,
) -> AuthorityEnvelope {
    AuthorityEnvelope::new(
        zones.iter().map(|v| (*v).into()),
        capabilities.iter().map(|v| (*v).into()),
        TimeWindow::new(now() - Duration::hours(1), now() + Duration::hours(hours))
            .expect("window"),
        resources,
    )
    .expect("envelope")
}

fn active_fixture() -> (Incident, OperationalPeriod, Objective) {
    let incident = Incident::open(
        EntityId::new(),
        envelope(&["north", "south"], &["survey", "suppression"], 20, 8),
        EntityId::new(),
        now(),
    )
    .expect("incident");
    let mut objective = Objective::propose(
        EntityId::new(),
        EntityId::new(),
        "contain the north flank",
        1,
    )
    .expect("objective");
    objective
        .approve(1, EntityId::new())
        .expect("approve objective");
    objective.activate(2).expect("activate objective");
    let mut period = OperationalPeriod::define(
        EntityId::new(),
        incident.id().clone(),
        envelope(&["north"], &["survey", "suppression"], 10, 4),
        incident.commander().clone(),
    );
    period
        .add_objective(1, objective.id())
        .expect("add objective");
    period
        .approve(2, EntityId::new(), 3)
        .expect("approve period");
    period
        .activate(3, &incident, now())
        .expect("activate period");
    assert!(incident.is_active_at(now()));
    (incident, period, objective)
}

#[test]
fn concurrent_command_transfers_have_one_versioned_winner() {
    let (incident, _, _) = active_fixture();
    let expected = incident.version();
    let prior = incident.commander().clone();
    let mut first = incident.clone();
    let mut second = incident;
    assert!(
        first
            .transfer_command(
                expected,
                &prior,
                EntityId::new(),
                now(),
                now(),
                "shift change"
            )
            .is_ok()
    );
    // A repository compare-and-swap accepts only the first aggregate at `expected`.
    assert_eq!(first.version(), expected + 1);
    assert_eq!(
        second.transfer_command(
            expected + 1,
            &prior,
            EntityId::new(),
            now(),
            now(),
            "conflicting transfer"
        ),
        Err(IncidentError::ConcurrencyConflict)
    );
}

#[test]
fn assignment_cannot_exceed_incident_or_period_authority_and_contains_no_actuator_payload() {
    let (incident, period, objective) = active_fixture();
    let issuer = EntityId::new();
    let mut valid = Assignment::draft(
        EntityId::new(),
        incident.id().clone(),
        period.id().clone(),
        objective.id().clone(),
        envelope(&["north"], &["survey"], 2, 2),
        ResourceRequest {
            capability: "survey".into(),
            quantity: 2,
        },
        ["CTL-1".into()],
    )
    .expect("draft");
    valid
        .approve(1, &issuer, EntityId::new(), &incident, &period, &objective)
        .expect("bounded approval");
    valid
        .issue(2, now(), period.restriction_version())
        .expect("issue");
    assert_eq!(valid.state(), AssignmentState::Issued);
    let mut excessive = Assignment::draft(
        EntityId::new(),
        incident.id().clone(),
        period.id().clone(),
        objective.id().clone(),
        envelope(&["south"], &["suppression"], 20, 6),
        ResourceRequest {
            capability: "suppression".into(),
            quantity: 20,
        },
        [],
    )
    .expect("draft");
    assert_eq!(
        excessive.approve(1, &issuer, EntityId::new(), &incident, &period, &objective),
        Err(IncidentError::InvalidAuthority)
    );
}

#[test]
fn activation_blocks_until_every_policy_distribution_acknowledgement() {
    let mut flow = IncidentActivation::start(
        EntityId::new(),
        EntityId::new(),
        EntityId::new(),
        ["station-a".into(), "station-b".into()],
    );
    flow.period_established("event-1").expect("period");
    flow.period_established("event-1").expect("idempotent");
    flow.authority_validated("event-2").expect("authority");
    flow.restrictions_published("event-3", 7)
        .expect("restriction");
    flow.record_ack("station-a");
    assert_eq!(
        flow.permit_assignments(),
        Err(IncidentError::PolicyDistributionGap)
    );
    assert_eq!(
        flow.acknowledgement_gaps(),
        ["station-b".to_owned()].into_iter().collect()
    );
    flow.record_ack("station-b");
    flow.permit_assignments().expect("permit");
    assert!(flow.can_issue());
}

#[test]
fn offline_restrictions_are_idempotent_monotonic_and_detect_contradiction() {
    let mut cache = OfflineRestrictionCache::default();
    assert_eq!(cache.apply(4, [4; 32]), Ok(true));
    assert_eq!(cache.apply(4, [4; 32]), Ok(false));
    assert_eq!(cache.apply(3, [3; 32]), Ok(false));
    assert_eq!(
        cache.apply(4, [9; 32]),
        Err(IncidentError::AmbiguousAuthority)
    );
    assert_eq!(cache.highest_sequence(), 4);
}

#[test]
fn emergency_restriction_narrows_immediately_without_waiting_for_ack() {
    let (incident, _, _) = active_fixture();
    let mut restriction = Restriction::propose(
        EntityId::new(),
        incident.id().clone(),
        envelope(&["north"], &["survey"], 1, 1),
        "airspace conflict",
        incident.commander().clone(),
        8,
    )
    .expect("restriction");
    restriction
        .make_effective(1, &incident, now())
        .expect("effective before distribution ack");
    let effective = effective_restriction([&restriction], now())
        .expect("unambiguous")
        .expect("active");
    assert!(incident.envelope().contains(effective));
    assert!(!effective.contains(incident.envelope()));
}

#[test]
fn expiry_is_exclusive_and_immediately_removes_assignment_authority() {
    let (incident, period, objective) = active_fixture();
    let issuer = EntityId::new();
    let mut assignment = Assignment::draft(
        EntityId::new(),
        incident.id().clone(),
        period.id().clone(),
        objective.id().clone(),
        AuthorityEnvelope::new(
            ["north".into()],
            ["survey".into()],
            TimeWindow::new(now(), now() + Duration::seconds(1)).expect("window"),
            1,
        )
        .expect("scope"),
        ResourceRequest {
            capability: "survey".into(),
            quantity: 1,
        },
        [],
    )
    .expect("draft");
    assignment
        .approve(1, &issuer, EntityId::new(), &incident, &period, &objective)
        .expect("approve");
    assignment
        .issue(2, now(), period.restriction_version())
        .expect("issue");
    assert_eq!(
        assignment.expire_if_due(now() + Duration::seconds(1)),
        Ok(true)
    );
    assert_eq!(assignment.state(), AssignmentState::Expired);
}

proptest! {
    #[test]
    fn authority_containment_never_accepts_more_resources(parent in 1u32..100, extra in 1u32..100){let outer=envelope(&["north"],&["survey"],parent,4);let inner=envelope(&["north"],&["survey"],parent.saturating_add(extra),2);prop_assert!(!outer.contains(&inner));}
}
