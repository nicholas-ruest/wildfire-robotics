#![allow(missing_docs, clippy::unwrap_used)]
use fleet_operations::*;
fn ev_as(
    a: &str,
    b: &str,
    kind: EvidenceKind,
    tick: u64,
    value: u16,
    witness: &str,
    event: &str,
) -> RelationshipEvidence {
    RelationshipEvidence::sign_event(
        a,
        b,
        "wildfire-recon",
        kind,
        tick,
        value,
        witness,
        event,
        "legacy-fixture",
        [7; 32],
    )
    .unwrap()
}

struct Keys;
impl EvidenceKeyResolver for Keys {
    fn resolve(&self, key_id: &str, witness: &str) -> Option<[u8; 32]> {
        (key_id == "trusted-1" && witness == "witness").then_some([9; 32])
    }
}
fn ev(a: &str, b: &str, kind: EvidenceKind, tick: u64, value: u16) -> RelationshipEvidence {
    ev_as(
        a,
        b,
        kind,
        tick,
        value,
        "witness",
        &format!("event-{tick}-{}", kind as u8),
    )
}
#[test]
fn should_verify_context_and_decay_monotonically() {
    let e = ev("a", "b", EvidenceKind::Communication, 10, 9000);
    assert!(e.verify_with_key([7; 32]));
    let p = CollaborationProfile::build("p", vec![e], 10).unwrap();
    assert!(p.score_at(20) < p.score_at(10));
    assert_eq!(p.score_at(9), 0);
}

#[test]
fn should_require_trusted_external_key_and_reject_tampering() {
    let signed = RelationshipEvidence::sign_event(
        "a",
        "b",
        "wildfire-recon",
        EvidenceKind::Communication,
        10,
        9000,
        "witness",
        "event-1",
        "trusted-1",
        [9; 32],
    )
    .unwrap();
    assert!(!signed.verify_with_key([8; 32]));
    let tampered = RelationshipEvidence::from_transport(
        "a".into(),
        "b".into(),
        "wildfire-recon".into(),
        EvidenceKind::Communication,
        10,
        1,
        "witness".into(),
        "event-1".into(),
        "trusted-1".into(),
        signed.signature(),
    )
    .unwrap();
    assert!(CollaborationProfile::evaluate_verified("p", vec![tampered], 10, 1, &Keys).is_err());
}
#[test]
fn should_reject_replay_context_mixing_and_source_flood() {
    let duplicate = ev("a", "b", EvidenceKind::Cooperation, 10, 9000);
    assert!(CollaborationProfile::build("p", vec![duplicate.clone(), duplicate], 10).is_err());
    let flood = (0..9)
        .map(|i| {
            ev_as(
                "a",
                "b",
                EvidenceKind::Cooperation,
                10,
                9000,
                "one",
                &format!("e{i}"),
            )
        })
        .collect();
    assert!(CollaborationProfile::build("p", flood, 10).is_err());
}
#[test]
fn should_require_explicit_profile_promotion() {
    let mut p = CollaborationProfile::evaluate(
        "p",
        vec![ev("a", "b", EvidenceKind::Handoff, 10, 8000)],
        10,
        2,
    )
    .unwrap();
    assert_eq!(p.state(), ProfileState::Evaluated);
    p.promote().unwrap();
    assert_eq!(p.state(), ProfileState::Active);
    p.revoke().unwrap();
    assert_eq!(p.privileges(), PlatformPrivileges::None);
}
#[test]
fn should_use_graph_affinity_but_remain_advisory() {
    let p = CollaborationProfile::build(
        "ab",
        vec![ev("a", "b", EvidenceKind::Complementarity, 10, 8000)],
        10,
    )
    .unwrap();
    let r = ConventionalCollaborationRuntime
        .recommend(&["a", "c", "b", "d"], &[p], 2, 10)
        .unwrap();
    assert!(r.cohorts.contains(&vec!["a".into(), "b".into()]));
    assert_eq!(r.authority, RecommendationAuthority::AdvisoryOnly);
}
#[test]
fn should_reject_member_outside_partition_gate() {
    let members = [
        PartitionMember {
            id: "a",
            tenant: "t1",
            capability: "recon",
            epoch: 2,
            eligible: true,
        },
        PartitionMember {
            id: "b",
            tenant: "t2",
            capability: "recon",
            epoch: 2,
            eligible: true,
        },
    ];
    let gate = PartitionGate {
        tenant: "t1",
        capability: "recon",
        epoch: 2,
    };
    assert_eq!(
        ConventionalCollaborationRuntime.recommend_partition(&members, &gate, &[], 2, 10),
        Err(CollaborationError::IneligibleMember)
    );
}
#[test]
fn should_fallback_on_primary_fault_or_stale_profile() {
    let runtime = FaultTolerantRuntime {
        primary: FailingRuntime,
        fallback: ConventionalCollaborationRuntime,
    };
    assert_eq!(
        runtime.recommend(&["a", "b"], &[], 2, 10).unwrap().source,
        RecommendationSource::ConventionalFallback
    );
    let stale =
        CollaborationProfile::build("p", vec![ev("a", "b", EvidenceKind::Handoff, 1, 8000)], 1)
            .unwrap();
    assert_eq!(
        ConventionalCollaborationRuntime
            .recommend(&["a", "b"], &[stale], 2, 1000)
            .unwrap()
            .source,
        RecommendationSource::ConventionalFallback
    );
}
#[test]
fn should_chain_checkpoints_and_support_multi_generation_rollback() {
    let cp1 = GraphCheckpoint::seal(1, vec!["ab".into(), "c".into()], [8; 32]).unwrap();
    let cp2 = GraphCheckpoint::seal_after(
        2,
        vec!["a".into(), "bc".into()],
        [9; 32],
        Some(cp1.digest()),
    )
    .unwrap();
    assert_ne!(
        cp1.digest(),
        GraphCheckpoint::seal(1, vec!["a".into(), "bc".into()], [8; 32])
            .unwrap()
            .digest()
    );
    let mut store = CheckpointStore::activate(cp1).unwrap();
    store.replace(Ok(cp2.clone())).unwrap();
    let cp3 =
        GraphCheckpoint::seal_after(3, vec!["d".into()], [10; 32], Some(cp2.digest())).unwrap();
    store.replace(Ok(cp3)).unwrap();
    store.rollback().unwrap();
    store.rollback().unwrap();
    assert_eq!(store.active_epoch(), 1);
}
#[test]
fn should_bound_inputs_and_never_grant_privilege() {
    let ids = vec!["x"; 4097];
    assert!(
        ConventionalCollaborationRuntime
            .recommend(&ids, &[], 2, 1)
            .is_err()
    );
    let p = CollaborationProfile::build(
        "p",
        vec![ev("a", "b", EvidenceKind::SafetyOutcome, 10, 10000)],
        10,
    )
    .unwrap();
    assert_eq!(p.privileges(), PlatformPrivileges::None);
    assert_eq!(
        RvmAdapter::status(),
        ExternalRuntimeStatus::DisabledPendingEvaluation
    );
}

struct CorruptExternal;
impl ExternalRecommendationPort for CorruptExternal {
    fn external_recommend(
        &self,
        _: &[&str],
        _: &[CollaborationProfile],
        _: usize,
        _: u64,
    ) -> Result<CohortRecommendation, CollaborationError> {
        Ok(CohortRecommendation {
            cohorts: vec![vec!["a".into(), "intruder".into()]],
            split_advised: false,
            merge_advised: false,
            authority: RecommendationAuthority::AdvisoryOnly,
            source: RecommendationSource::External,
        })
    }
}
#[test]
fn should_fallback_when_external_advice_changes_member_set() {
    let runtime = GovernedExternalAdapter {
        external: CorruptExternal,
        fallback: ConventionalCollaborationRuntime,
    };
    let r = runtime.recommend(&["a", "b"], &[], 2, 10).unwrap();
    assert_eq!(r.source, RecommendationSource::ConventionalFallback);
    assert_eq!(r.cohorts, vec![vec!["a".to_owned(), "b".to_owned()]]);
}
