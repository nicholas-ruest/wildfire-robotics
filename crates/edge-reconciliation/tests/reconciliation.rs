//! Outside-in edge reconciliation acceptance tests.

use edge_reconciliation::{
    AggregateDisposition, AggregateTracker, AuthorityFact, AuthorityState, CausalRelation,
    ReconcileDecision, ReplicaVersion, SyncCursor, VersionVector,
};

#[test]
fn should_apply_stricter_authority_regardless_of_arrival_order()
-> Result<(), Box<dyn std::error::Error>> {
    let permissive =
        AuthorityFact::new(ReplicaVersion::new("cloud", 8)?, AuthorityState::Permitted);
    let grounded = AuthorityFact::new(ReplicaVersion::new("station", 3)?, AuthorityState::Grounded);
    assert_eq!(
        permissive.reconcile(&grounded),
        ReconcileDecision::ApplyRemote(grounded.clone())
    );
    assert_eq!(
        grounded.reconcile(&permissive),
        ReconcileDecision::KeepLocal
    );
    Ok(())
}

#[test]
fn should_surface_gaps_duplicates_contradictions_and_cursor_rollback()
-> Result<(), Box<dyn std::error::Error>> {
    let mut tracker = AggregateTracker::default();
    assert_eq!(
        tracker.classify(2, [2; 32]),
        AggregateDisposition::Gap {
            expected: 1,
            actual: 2
        }
    );
    tracker.record_committed(1, [1; 32])?;
    assert_eq!(
        tracker.classify(1, [1; 32]),
        AggregateDisposition::Duplicate
    );
    assert_eq!(
        tracker.classify(1, [9; 32]),
        AggregateDisposition::Contradiction { version: 1 }
    );
    let mut cursor = SyncCursor::new("cloud", 4)?;
    assert!(cursor.advance_after_commit(6).is_err());
    cursor.advance_after_commit(5)?;
    assert_eq!(cursor.applied_sequence(), 5);
    Ok(())
}

#[test]
fn should_detect_concurrent_version_vectors_without_wall_clock_order()
-> Result<(), Box<dyn std::error::Error>> {
    let mut cloud = VersionVector::default();
    cloud.observe(&ReplicaVersion::new("cloud", 2)?)?;
    let mut station = VersionVector::default();
    station.observe(&ReplicaVersion::new("station", 3)?)?;
    assert_eq!(cloud.relation(&station), CausalRelation::Concurrent);
    Ok(())
}

#[test]
fn should_suspend_incomparable_equal_authority_facts() -> Result<(), Box<dyn std::error::Error>> {
    let first = AuthorityFact::new(
        ReplicaVersion::new("cloud", 2)?,
        AuthorityState::Restricted(4),
    );
    let second = AuthorityFact::new(
        ReplicaVersion::new("station", 7)?,
        AuthorityState::Restricted(4),
    );
    assert!(matches!(
        first.reconcile(&second),
        ReconcileDecision::SuspendAmbiguous { .. }
    ));
    Ok(())
}
