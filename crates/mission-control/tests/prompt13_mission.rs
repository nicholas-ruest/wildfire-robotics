//! Prompt 13 mission-control safety and concurrency invariants.
#![allow(clippy::unwrap_used)]

use chrono::{DateTime, Duration, TimeZone, Utc};
use mission_control::*;
use proptest::prelude::*;
use shared_kernel::{EntityId, TimeWindow};

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, 21, 12, 0, 0).unwrap()
}

fn window() -> TimeWindow {
    TimeWindow::new(now() - Duration::minutes(1), now() + Duration::hours(1)).unwrap()
}

fn versioned(name: &str, byte: u8) -> VersionedSnapshot {
    VersionedSnapshot {
        name: name.into(),
        digest: [byte; 32],
        version: 1,
        valid_until: now() + Duration::hours(1),
    }
}

fn snapshots(plan: [u8; 32]) -> AuthorizationSnapshot {
    AuthorizationSnapshot {
        assignment: versioned("assignment", 1),
        policy: versioned("policy", 2),
        restriction: versioned("restriction", 3),
        constraint: versioned("constraint", 4),
        odd: versioned("odd", 5),
        hazard: versioned("hazard", 6),
        fleet: versioned("fleet", 7),
        plan: VersionedSnapshot {
            digest: plan,
            ..versioned("plan", 8)
        },
    }
}

fn committed_allocation(mission_id: &EntityId) -> Allocation {
    let mut book = ReservationBook::default();
    let mut allocation = Allocation::propose(
        EntityId::new(),
        mission_id.clone(),
        ["vehicle-a".into()],
        [9; 32],
        EntityId::new(),
        4,
        window(),
    )
    .unwrap();
    allocation.reserve(&mut book, now()).unwrap();
    allocation.commit(&book, 4, [9; 32], now()).unwrap();
    allocation
}

fn active_lease(mission_id: &EntityId, holder: &EntityId) -> MissionLease {
    let mut lease = MissionLease::offer(
        EntityId::new(),
        mission_id.clone(),
        holder.clone(),
        1,
        window(),
    )
    .unwrap();
    lease.acquire(now()).unwrap();
    lease
}

#[test]
fn planner_is_deterministic_and_consumes_only_bounded_cell_summaries() {
    let request = PlanningRequest {
        capability: "survey".into(),
        quantity: 2,
        energy_wh_per_asset: 100,
        relay: None,
    };
    let preferred = EntityId::new();
    let summaries = vec![
        CellSummary {
            cell_id: EntityId::new(),
            epoch: 2,
            capability: "survey".into(),
            eligible_count: 2,
            energy_wh_lower_bound: 200,
            valid_until_epoch_seconds: 10,
            digest: [1; 32],
        },
        CellSummary {
            cell_id: preferred.clone(),
            epoch: 3,
            capability: "survey".into(),
            eligible_count: 4,
            energy_wh_lower_bound: 400,
            valid_until_epoch_seconds: 10,
            digest: [2; 32],
        },
    ];
    let planner = DeterministicReferencePlanner;
    let first = planner.plan(&request, &summaries).unwrap();
    assert_eq!(first, planner.plan(&request, &summaries).unwrap());
    assert_eq!(first.selected_cell, preferred);
    let oversized = vec![summaries[0].clone(); 10_001];
    assert_eq!(
        planner.plan(&request, &oversized),
        Err(MissionError::InvalidPlan)
    );
}

#[test]
fn reservation_is_exclusive_and_commit_rejects_stale_fleet_facts() {
    let mission = EntityId::new();
    let cell = EntityId::new();
    let mut book = ReservationBook::default();
    let make = || {
        Allocation::propose(
            EntityId::new(),
            mission.clone(),
            ["asset-7".into()],
            [3; 32],
            cell.clone(),
            8,
            window(),
        )
        .unwrap()
    };
    let mut winner = make();
    let mut loser = make();
    winner.reserve(&mut book, now()).unwrap();
    assert_eq!(
        loser.reserve(&mut book, now()),
        Err(MissionError::DoubleAllocation)
    );
    assert_eq!(
        winner.commit(&book, 7, [3; 32], now()),
        Err(MissionError::StaleAllocation)
    );
    assert_eq!(
        winner.commit(&book, 8, [4; 32], now()),
        Err(MissionError::StaleAllocation)
    );
    winner.commit(&book, 8, [3; 32], now()).unwrap();
}

#[test]
fn stale_fence_cannot_advance_and_renewal_is_monotonic() {
    let mission = EntityId::new();
    let holder = EntityId::new();
    let mut lease = active_lease(&mission, &holder);
    lease.renew(1, 2, window(), now()).unwrap();
    assert!(!lease.permits(&mission, &holder, 1, now()));
    assert!(lease.permits(&mission, &holder, 2, now()));
    assert_eq!(
        lease.renew(1, 3, window(), now()),
        Err(MissionError::StaleLease)
    );
}

#[test]
fn conflicts_require_resolution_and_hard_conflicts_cannot_be_accepted() {
    let mission_id = EntityId::new();
    let plan = [42; 32];
    let mut mission = Mission::plan(mission_id.clone(), window(), plan, None).unwrap();
    mission.begin_validation().unwrap();
    let allocation = committed_allocation(&mission_id);
    let mut conflicts = ConflictSet::open(EntityId::new(), mission_id);
    let conflict_id = EntityId::new();
    conflicts
        .detect(
            conflict_id.clone(),
            ConflictKind::Collision,
            "trajectory overlap",
        )
        .unwrap();
    assert_eq!(
        mission.authorize(snapshots(plan), &allocation, &conflicts, now()),
        Err(MissionError::AuthorizationDenied)
    );
    conflicts
        .mitigate(&conflict_id, "separate trajectories")
        .unwrap();
    assert_eq!(
        conflicts.accept_residual(&conflict_id, &EntityId::new(), &EntityId::new(), true),
        Err(MissionError::ResidualConflictNotApproved)
    );
    conflicts.close(&conflict_id).unwrap();
    mission
        .authorize(snapshots(plan), &allocation, &conflicts, now())
        .unwrap();
}

#[derive(Default)]
struct FakeGateway {
    outcome: Option<GatewayOutcome>,
    minimum_risk_calls: usize,
}
impl CommandGateway for FakeGateway {
    type Error = ();
    fn dispatch(&mut self, _: &EntityId, _: Digest, _: u64) -> Result<GatewayOutcome, Self::Error> {
        Ok(self.outcome.unwrap_or(GatewayOutcome::Unknown))
    }
    fn minimum_risk(&mut self, _: &EntityId, _: &str) -> Result<(), Self::Error> {
        self.minimum_risk_calls += 1;
        Ok(())
    }
}

fn authorized() -> (Mission, MissionLease, EntityId, AuthorizationSnapshot) {
    let mission_id = EntityId::new();
    let holder = EntityId::new();
    let plan = [42; 32];
    let mut mission = Mission::plan(mission_id.clone(), window(), plan, None).unwrap();
    mission.begin_validation().unwrap();
    let allocation = committed_allocation(&mission_id);
    let conflicts = ConflictSet::open(EntityId::new(), mission_id.clone());
    let snapshot = snapshots(plan);
    mission
        .authorize(snapshot.clone(), &allocation, &conflicts, now())
        .unwrap();
    let lease = active_lease(&mission_id, &holder);
    (mission, lease, holder, snapshot)
}

#[test]
fn dispatch_is_bound_to_exact_current_snapshot_and_unknown_outcome_is_compensated() {
    let (mut mission, lease, holder, snapshot) = authorized();
    let mut changed = snapshot.clone();
    changed.restriction.version += 1;
    assert_eq!(
        mission.dispatch(&lease, &holder, 1, &changed, now()),
        Err(MissionError::DispatchDenied)
    );
    let mut gateway = FakeGateway::default();
    assert_eq!(
        dispatch_command(
            &mut mission,
            &lease,
            &holder,
            1,
            &snapshot,
            now(),
            &mut gateway
        )
        .unwrap(),
        GatewayOutcome::Unknown
    );
    assert_eq!(gateway.minimum_risk_calls, 1);
}

#[test]
fn abort_revokes_authority_invokes_minimum_risk_and_dominates_progress() {
    let (mut mission, mut lease, holder, snapshot) = authorized();
    mission
        .dispatch(&lease, &holder, 1, &snapshot, now())
        .unwrap();
    let mut gateway = FakeGateway::default();
    abort_with_compensation(&mut mission, &mut lease, "grounded", &mut gateway).unwrap();
    assert_eq!(mission.state(), MissionState::Aborted);
    assert_eq!(gateway.minimum_risk_calls, 1);
    assert_eq!(
        mission.advance(&lease, &holder, 1, now()),
        Err(MissionError::StaleLease)
    );
}

#[test]
fn relay_requirements_are_complete_and_do_not_relax_local_safety() {
    let mut relay = RelayRequirement {
        service_class: "incident".into(),
        coverage_digest: [1; 32],
        duration_seconds: 60,
        spectrum_profile: "licensed".into(),
        energy_wh: 100,
        return_reserve_wh: 50,
        airspace_digest: [2; 32],
        handoff: "make-before-break".into(),
        fallback: "return".into(),
    };
    relay.validate().unwrap();
    relay.fallback.clear();
    assert_eq!(relay.validate(), Err(MissionError::InvalidRelayPlan));
}

proptest! {
    #[test]
    fn arbitrary_stale_fences_never_permit(delta in 1u64..u64::MAX) {
        let mission = EntityId::new();
        let holder = EntityId::new();
        let mut lease = active_lease(&mission, &holder);
        lease.renew(1, 1 + delta, window(), now()).unwrap();
        prop_assert!(!lease.permits(&mission, &holder, 1, now()));
    }
}
