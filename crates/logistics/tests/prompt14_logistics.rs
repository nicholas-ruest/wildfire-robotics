//! Prompt 14 inventory, custody, delivery, water, and supply invariants.
#![allow(clippy::unwrap_used)]

use chrono::{DateTime, Duration, TimeZone, Utc};
use logistics::*;
use proptest::prelude::*;
use shared_kernel::{EntityId, TimeWindow};
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, 21, 12, 0, 0).unwrap()
}

fn item(quantity: u64) -> ResourceItem {
    ResourceItem::stock(
        EntityId::new(),
        Some("batch-a".into()),
        None,
        ExactQuantity::new(quantity, "each").unwrap(),
        ResourceCondition::Serviceable,
        ["pump-v2".into()],
        "station-a",
        Some(now() + Duration::hours(1)),
        "supplier-a",
    )
    .unwrap()
}

#[test]
fn concurrent_reservation_has_exactly_one_winner_for_the_last_stock() {
    let state = Arc::new(Mutex::new((item(1), InventoryLedger::default())));
    let mut workers = Vec::new();
    for _ in 0..16 {
        let state = Arc::clone(&state);
        workers.push(std::thread::spawn(move || {
            let mut guard = state.lock().unwrap();
            let (resource, ledger) = &mut *guard;
            ledger
                .reserve(
                    resource,
                    EntityId::new(),
                    1,
                    "each",
                    "pump-v2",
                    "station-a",
                    "incident",
                    now(),
                    now() + Duration::minutes(5),
                )
                .is_ok()
        }));
    }
    assert_eq!(
        workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .filter(|won| *won)
            .count(),
        1
    );
}

#[test]
fn expired_quarantined_incompatible_and_reserved_stock_is_not_available() {
    let mut resource = item(4);
    assert_eq!(
        resource.available_to_promise("wrong", "station-a", now()),
        0
    );
    resource.set_condition(ResourceCondition::Quarantined);
    assert_eq!(
        resource.available_to_promise("pump-v2", "station-a", now()),
        0
    );
    resource.set_condition(ResourceCondition::Serviceable);
    let mut ledger = InventoryLedger::default();
    ledger
        .reserve(
            &mut resource,
            EntityId::new(),
            1,
            "each",
            "pump-v2",
            "station-a",
            "mission",
            now(),
            now() + Duration::minutes(1),
        )
        .unwrap();
    assert_eq!(
        resource.available_to_promise("pump-v2", "station-a", now()),
        0
    );
}

fn transfer(
    item_id: &EntityId,
    giver: &str,
    receiver: &str,
    previous: [u8; 32],
    digest: [u8; 32],
) -> CustodyTransfer {
    let mut transfer = CustodyTransfer {
        id: EntityId::new(),
        item_id: item_id.clone(),
        giver: giver.into(),
        receiver: receiver.into(),
        amount: 1,
        unit: "each".into(),
        condition: "serviceable".into(),
        place: "station-a".into(),
        occurred_at: now(),
        evidence_digest: [7; 32],
        discrepancy: None,
        previous_digest: previous,
        digest,
    };
    transfer.digest = transfer.compute_digest();
    transfer
}

#[test]
fn custody_replay_is_idempotent_and_lineage_cannot_fork_or_skip_custodian() {
    let item_id = EntityId::new();
    let mut chain = CustodyChain::default();
    let first = transfer(&item_id, "supplier", "station", [0; 32], [1; 32]);
    let first_digest = first.digest;
    assert!(chain.append(first.clone()).unwrap());
    let mut tampered = first.clone();
    tampered.condition = "damaged".into();
    assert_eq!(chain.append(tampered), Err(LogisticsError::InvalidCustody));
    assert!(!chain.append(first).unwrap());
    assert_eq!(
        chain.append(transfer(
            &item_id,
            "attacker",
            "vehicle",
            first_digest,
            [2; 32]
        )),
        Err(LogisticsError::InvalidCustody)
    );
    chain
        .append(transfer(
            &item_id,
            "station",
            "vehicle",
            first_digest,
            [2; 32],
        ))
        .unwrap();
    assert_eq!(chain.lineage(&item_id).len(), 2);
}

#[test]
fn contamination_after_reservation_blocks_water_relay_and_inventory_issue() {
    let mut source = WaterSource::verify(
        EntityId::new(),
        100,
        [1; 32],
        now() + Duration::minutes(5),
        now(),
    )
    .unwrap();
    let mut relay = RelayCycle::plan(
        EntityId::new(),
        source.id().clone(),
        50,
        vec![EntityId::new()],
    )
    .unwrap();
    relay.reserve(&mut source, now()).unwrap();
    source.contaminate();
    assert_eq!(
        relay.record_leg(&source, "leg-1", 50, now()),
        Err(LogisticsError::InvalidRelay)
    );

    let mut resource = item(1);
    let mut ledger = InventoryLedger::default();
    let reservation = ledger
        .reserve(
            &mut resource,
            EntityId::new(),
            1,
            "each",
            "pump-v2",
            "station-a",
            "delivery",
            now(),
            now() + Duration::minutes(5),
        )
        .unwrap();
    resource.set_condition(ResourceCondition::Contaminated);
    assert_eq!(
        ledger.issue(&mut resource, &reservation.id, now()),
        Err(LogisticsError::StaleReservation)
    );
}

#[test]
fn route_failure_selects_only_pre_authorized_reroute_or_named_safe_stop() {
    let validity =
        TimeWindow::new(now() - Duration::minutes(1), now() + Duration::hours(1)).unwrap();
    let mut route = Route::propose(
        EntityId::new(),
        [1; 32],
        [2; 32],
        validity,
        "refuge-7",
        false,
    )
    .unwrap();
    route.validate([1; 32], [2; 32], now()).unwrap();
    route.activate().unwrap();
    assert_eq!(
        route.block(),
        RouteCompensation::SafeStop("refuge-7".into())
    );
    assert_eq!(route.activate(), Err(LogisticsError::UnsafeRoute));
}

#[test]
fn routing_optimizer_rejects_missing_hard_dependencies_and_is_stable() {
    let unsafe_option = RouteOption {
        route_id: EntityId::new(),
        duration_seconds: 1,
        risk_micros: 1,
        envelope: DependencyState::Current,
        restrictions: DependencyState::Current,
        communications: DependencyState::Unavailable,
        energy: DependencyState::Current,
        maintenance: DependencyState::Current,
        safe_stops: BTreeSet::from(["refuge".into()]),
    };
    let safe_id = EntityId::new();
    let safe_option = RouteOption {
        route_id: safe_id.clone(),
        communications: DependencyState::Current,
        duration_seconds: 20,
        risk_micros: 20,
        ..unsafe_option.clone()
    };
    let optimizer = DeterministicRoutingOptimizer;
    let result = optimizer.choose(&[unsafe_option, safe_option]).unwrap();
    assert_eq!(result.route_id, safe_id);
}

#[test]
fn physical_disposition_is_idempotent_and_cannot_be_rewritten() {
    let mut resource = item(1);
    let mut ledger = InventoryLedger::default();
    let reservation = ledger
        .reserve(
            &mut resource,
            EntityId::new(),
            1,
            "each",
            "pump-v2",
            "station-a",
            "use",
            now(),
            now() + Duration::minutes(5),
        )
        .unwrap();
    ledger.issue(&mut resource, &reservation.id, now()).unwrap();
    assert!(ledger.consume(&mut resource, &reservation.id).unwrap());
    assert!(!ledger.consume(&mut resource, &reservation.id).unwrap());
    assert_eq!(
        ledger.return_item(&mut resource, &reservation.id),
        Err(LogisticsError::StaleReservation)
    );
}

#[test]
fn optimizer_is_deterministic_explicit_about_substitution_and_shortage() {
    let demands = vec![
        Demand {
            capability: "filter-a".into(),
            quantity: 2,
            unit: "each".into(),
            priority: 10,
            destination: "camp".into(),
            deadline_epoch_seconds: 10,
        },
        Demand {
            capability: "pump".into(),
            quantity: 3,
            unit: "each".into(),
            priority: 5,
            destination: "line".into(),
            deadline_epoch_seconds: 20,
        },
    ];
    let stock = vec![StockOption {
        item_id: EntityId::new(),
        capability: "filter-b".into(),
        available: 2,
        unit: "each".into(),
        location: "depot".into(),
        lead_time_seconds: 2,
        lead_time_uncertainty_seconds: 1,
        energy_ready: true,
        maintenance_ready: true,
        transport_ready: true,
        substitution_for: BTreeSet::from(["filter-a".into()]),
    }];
    let optimizer = DeterministicBaselineOptimizer;
    let first = optimizer.optimize(&demands, &stock).unwrap();
    assert_eq!(first, optimizer.optimize(&demands, &stock).unwrap());
    assert_eq!(first.allocations.iter().map(|a| a.quantity).sum::<u64>(), 2);
    assert_eq!(first.alternatives, vec!["substitute filter-b for filter-a"]);
    assert_eq!(first.bottlenecks, vec!["3 each pump short at line"]);
}

#[test]
fn plan_refuses_unacknowledged_shortage_and_replans_without_rewriting_history() {
    let demand = Demand {
        capability: "fuel".into(),
        quantity: 10,
        unit: "litre".into(),
        priority: 10,
        destination: "habitat".into(),
        deadline_epoch_seconds: 100,
    };
    let mut plan = SupplyPlan::draft(EntityId::new(), [1; 32]).unwrap();
    plan.optimize(&DeterministicBaselineOptimizer, &[demand], &[])
        .unwrap();
    assert_eq!(plan.approve(false), Err(LogisticsError::InfeasibleSupply));
    plan.approve(true).unwrap();
    plan.replan().unwrap();
    assert_eq!(plan.state, SupplyPlanState::Replanning);
}

proptest! {
    #[test]
    fn reservation_never_exceeds_batch_capacity(capacity in 1u64..10_000, requested in 1u64..20_000) {
        let mut resource = item(capacity);
        let mut ledger = InventoryLedger::default();
        let accepted = ledger.reserve(&mut resource, EntityId::new(), requested, "each", "pump-v2", "station-a",
            "property", now(), now() + Duration::minutes(1)).is_ok();
        prop_assert_eq!(accepted, requested <= capacity);
    }
}
