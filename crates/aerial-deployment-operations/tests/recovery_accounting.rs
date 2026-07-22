#![allow(missing_docs, clippy::unwrap_used)]
use aerial_deployment_operations::*;
use chrono::{TimeZone, Utc};

fn id(value: &str) -> ComponentId {
    ComponentId::new(value).unwrap()
}
fn at(seconds: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(seconds, 0).single().unwrap()
}

fn observation(
    component: &str,
    scan: &str,
    custody: Custodian,
    location: LocationStatus,
) -> RecoveryObservation {
    RecoveryObservation {
        observation_id: ObservationId::new(scan).unwrap(),
        component: id(component),
        observed_at: at(100),
        custody,
        location,
        exposure: ExposureRecord::default(),
        damage: DamageAssessment::None,
        contamination: ContaminationStatus::Clear,
        energized_hazards: vec![],
    }
}

#[test]
fn accounts_every_required_serialized_kind_without_losing_partial_recovery() {
    let kinds = [
        SerializedKind::Robot,
        SerializedKind::Panel,
        SerializedKind::Joint,
        SerializedKind::Parafoil,
        SerializedKind::Tether,
        SerializedKind::Reel,
        SerializedKind::Anchor,
        SerializedKind::CradleSection,
        SerializedKind::ChemicalPayload,
    ];
    let items = kinds
        .into_iter()
        .enumerate()
        .map(|(n, kind)| {
            SerializedItem::new(id(&format!("item-{n}")), kind, &format!("serial-{n}"))
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let mut ledger = RecoveryLedger::new(items).unwrap();
    ledger
        .record(observation(
            "item-0",
            "scan-1",
            Custodian::RecoveryTeam("team-a".into()),
            LocationStatus::Known(LocationFix::new("yard-a", 8000).unwrap()),
        ))
        .unwrap();
    assert_eq!(ledger.items().len(), 9);
    assert_eq!(ledger.summary().recovered, 1);
    assert_eq!(ledger.summary().unlocated, 8);
}

#[test]
fn duplicate_scans_are_idempotent_and_conflicting_replays_are_rejected() {
    let mut ledger = RecoveryLedger::new(vec![
        SerializedItem::new(id("panel-1"), SerializedKind::Panel, "p-1").unwrap(),
    ])
    .unwrap();
    let scan = observation(
        "panel-1",
        "scan-1",
        Custodian::RecoveryTeam("team-a".into()),
        LocationStatus::Known(LocationFix::new("yard", 9000).unwrap()),
    );
    assert_eq!(ledger.record(scan.clone()).unwrap(), RecordOutcome::Applied);
    assert_eq!(ledger.record(scan).unwrap(), RecordOutcome::Duplicate);
    let conflicting = observation(
        "panel-1",
        "scan-1",
        Custodian::Logistics,
        LocationStatus::Unknown,
    );
    assert_eq!(ledger.record(conflicting), Err(DomainError::ReplayConflict));
}

#[test]
fn disconnected_conflicting_custody_converges_to_disputed_without_dropping_observations() {
    let mut ledger = RecoveryLedger::new(vec![
        SerializedItem::new(id("robot-1"), SerializedKind::Robot, "r-1").unwrap(),
    ])
    .unwrap();
    ledger
        .record(observation(
            "robot-1",
            "offline-a",
            Custodian::RecoveryTeam("alpha".into()),
            LocationStatus::Known(LocationFix::new("sector-1", 7000).unwrap()),
        ))
        .unwrap();
    let mut other = observation(
        "robot-1",
        "offline-b",
        Custodian::RecoveryTeam("bravo".into()),
        LocationStatus::Known(LocationFix::new("sector-2", 7500).unwrap()),
    );
    other.observed_at = at(100);
    ledger.record(other).unwrap();
    assert!(matches!(
        ledger.item(&id("robot-1")).unwrap().custody(),
        CustodyState::Disputed { .. }
    ));
    assert_eq!(ledger.item(&id("robot-1")).unwrap().observation_count(), 2);
}

#[test]
fn late_older_scan_cannot_roll_back_newer_custody() {
    let mut ledger = RecoveryLedger::new(vec![
        SerializedItem::new(id("robot-4"), SerializedKind::Robot, "serial-4").unwrap(),
    ])
    .unwrap();
    ledger
        .record(observation(
            "robot-4",
            "newer",
            Custodian::RobotCare,
            LocationStatus::Known(LocationFix::new("bay-1", 9_900).unwrap()),
        ))
        .unwrap();
    let mut older = observation(
        "robot-4",
        "older",
        Custodian::Aircraft,
        LocationStatus::Unknown,
    );
    older.observed_at = at(5);
    ledger.record(older).unwrap();
    assert_eq!(
        ledger.item(&id("robot-4")).unwrap().custody(),
        &CustodyState::Held(Custodian::RobotCare)
    );
}

#[test]
fn quarantine_handoff_is_idempotent_and_adapter_outage_does_not_clear_hazards() {
    let mut contaminated = observation(
        "reel-1",
        "scan-1",
        Custodian::RecoveryTeam("alpha".into()),
        LocationStatus::Known(LocationFix::new("hot-zone", 6000).unwrap()),
    );
    contaminated.contamination = ContaminationStatus::Confirmed {
        agents: vec!["retardant".into()],
    };
    contaminated.energized_hazards = vec![EnergizedHazard::Electrical { volts: 48 }];
    let mut ledger = RecoveryLedger::new(vec![
        SerializedItem::new(id("reel-1"), SerializedKind::Reel, "reel-s").unwrap(),
    ])
    .unwrap();
    ledger.record(contaminated).unwrap();
    let request = HandoffRequest::new(
        HandoffRequestId::new("handoff-1").unwrap(),
        id("reel-1"),
        HandoffTarget::RobotCare,
        RequestedTreatment::Decontamination,
        at(110),
    )
    .unwrap();
    assert_eq!(
        ledger.request_handoff(request.clone()).unwrap(),
        HandoffOutcome::Queued
    );
    assert_eq!(
        ledger.request_handoff(request).unwrap(),
        HandoffOutcome::Duplicate
    );
    assert!(
        ledger
            .closure_report()
            .blocking_hazards
            .contains(&id("reel-1"))
    );
    assert_eq!(
        ledger
            .acknowledge_handoff(
                &HandoffAck::accepted(
                    HandoffRequestId::new("handoff-1").unwrap(),
                    "ack-1",
                    at(120)
                )
                .unwrap()
            )
            .unwrap(),
        HandoffOutcome::Acknowledged
    );
    assert!(
        ledger
            .closure_report()
            .blocking_hazards
            .contains(&id("reel-1"))
    );
}

#[test]
fn closure_requires_authorized_evidenced_missing_disposition_and_continuing_hazard_notice() {
    let mut ledger = RecoveryLedger::new(vec![
        SerializedItem::new(id("anchor-1"), SerializedKind::Anchor, "a-1").unwrap(),
    ])
    .unwrap();
    let search = SearchRecord::new(
        SearchId::new("search-1").unwrap(),
        id("anchor-1"),
        "grid complete",
        at(200),
    )
    .unwrap();
    ledger.record_search(search).unwrap();
    assert!(!ledger.closure_report().can_close);
    let acceptance = MissingDisposition::new(
        id("anchor-1"),
        MissingReason::Unlocated,
        DispositionAuthority::new("incident-commander", "authority-ref").unwrap(),
        vec![SearchId::new("search-1").unwrap()],
        HazardNotice::new(
            "notice-1",
            "possible energized anchor remains",
            at(210),
            None,
        )
        .unwrap(),
    )
    .unwrap();
    ledger.accept_missing(acceptance).unwrap();
    assert!(ledger.closure_report().can_close);
    assert_eq!(
        ledger.item(&id("anchor-1")).unwrap().disposition(),
        RecoveryDisposition::AcceptedUnlocated
    );
}

#[test]
fn sacrifice_and_temporary_abandonment_require_formal_authority_and_notice() {
    let mut ledger = RecoveryLedger::new(vec![
        SerializedItem::new(id("panel-9"), SerializedKind::Panel, "p-9").unwrap(),
    ])
    .unwrap();
    let release = SacrificialRelease::new(
        "release-9",
        id("panel-9"),
        DispositionAuthority::new("safety", "auth-9").unwrap(),
        at(9),
        HazardNotice::new("notice-9", "panel remains in exclusion zone", at(9), None).unwrap(),
    )
    .unwrap();
    ledger.record_sacrifice(release).unwrap();
    assert!(ledger.closure_report().can_close);
    assert_eq!(
        ledger.item(&id("panel-9")).unwrap().disposition(),
        RecoveryDisposition::Sacrificed
    );
}

#[test]
fn unknown_location_and_conflicting_custody_block_closure() {
    let mut ledger = RecoveryLedger::new(vec![
        SerializedItem::new(id("joint-1"), SerializedKind::Joint, "j-1").unwrap(),
    ])
    .unwrap();
    ledger
        .record(observation(
            "joint-1",
            "scan-unknown",
            Custodian::Unknown,
            LocationStatus::Unknown,
        ))
        .unwrap();
    assert_eq!(ledger.summary().unlocated, 1);
    assert!(!ledger.closure_report().can_close);
}

#[test]
fn accepted_handoff_records_external_custody_and_treatment_but_keeps_contamination_blocking() {
    let mut scan = observation(
        "robot-2",
        "scan-r2",
        Custodian::RecoveryTeam("alpha".into()),
        LocationStatus::Known(LocationFix::new("quarantine", 9900).unwrap()),
    );
    scan.contamination = ContaminationStatus::Suspected {
        agents: vec!["smoke-residue".into()],
    };
    let mut ledger = RecoveryLedger::new(vec![
        SerializedItem::new(id("robot-2"), SerializedKind::Robot, "r-2").unwrap(),
    ])
    .unwrap();
    ledger.record(scan).unwrap();
    ledger
        .request_handoff(
            HandoffRequest::new(
                HandoffRequestId::new("h-r2").unwrap(),
                id("robot-2"),
                HandoffTarget::RobotCare,
                RequestedTreatment::Inspection,
                at(300),
            )
            .unwrap(),
        )
        .unwrap();
    ledger
        .acknowledge_handoff(
            &HandoffAck::accepted(HandoffRequestId::new("h-r2").unwrap(), "ack-r2", at(301))
                .unwrap(),
        )
        .unwrap();
    let item = ledger.item(&id("robot-2")).unwrap();
    assert_eq!(item.custody(), &CustodyState::Held(Custodian::RobotCare));
    assert_eq!(item.disposition(), RecoveryDisposition::Inspection);
    assert!(!ledger.closure_report().can_close);
    let retry = HandoffRequest::new(
        HandoffRequestId::new("h-r2").unwrap(),
        id("robot-2"),
        HandoffTarget::RobotCare,
        RequestedTreatment::Inspection,
        at(300),
    )
    .unwrap();
    assert_eq!(
        ledger.request_handoff(retry).unwrap(),
        HandoffOutcome::Duplicate
    );
}

#[test]
fn temporary_abandonment_needs_search_authority_and_a_continuing_notice() {
    let component = id("tether-left");
    let authority = DispositionAuthority::new("incident-command", "approval-44").unwrap();
    let notice = HazardNotice::new(
        "notice-left",
        "tether remains marked in sector 4",
        at(400),
        None,
    )
    .unwrap();
    assert_eq!(
        MissingDisposition::new(
            component.clone(),
            MissingReason::TemporarilyAbandoned,
            authority.clone(),
            vec![],
            notice.clone()
        ),
        Err(DomainError::RecoveryClosureInhibited)
    );
    let mut ledger = RecoveryLedger::new(vec![
        SerializedItem::new(component.clone(), SerializedKind::Tether, "t-left").unwrap(),
    ])
    .unwrap();
    ledger
        .record_search(
            SearchRecord::new(
                SearchId::new("search-left").unwrap(),
                component.clone(),
                "remote visual and grid search",
                at(399),
            )
            .unwrap(),
        )
        .unwrap();
    ledger
        .accept_missing(
            MissingDisposition::new(
                component.clone(),
                MissingReason::TemporarilyAbandoned,
                authority,
                vec![SearchId::new("search-left").unwrap()],
                notice,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(
        ledger.item(&component).unwrap().disposition(),
        RecoveryDisposition::TemporarilyAbandoned
    );
    assert!(ledger.closure_report().can_close);
}
