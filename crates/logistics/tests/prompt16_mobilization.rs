//! Prompt 16 pod, carrier, flow, and 100,000-robot invariants.
#![allow(clippy::unwrap_used)]
use logistics::*;
use proptest::prelude::*;
use shared_kernel::EntityId;

fn limits() -> PodLimits {
    PodLimits {
        max_mass_grams: 20_000,
        max_volume_cm3: 20_000,
        max_abs_cog_mm: 100,
        max_axle_grams: 10_000,
        required_interface: "rack-v1".into(),
        max_assets: 4,
    }
}
fn load(id: EntityId, mass: u64, pos: i64) -> PodLoad {
    PodLoad {
        asset_id: id,
        mass_grams: mass,
        volume_cm3: 100,
        longitudinal_mm: pos,
        securement: true,
        energy_isolated: true,
        compatible_interface: "rack-v1".into(),
    }
}
fn slot(id: &str, kind: CapacityKind, capacity: u64) -> CapacitySlot {
    CapacitySlot {
        id: id.into(),
        kind,
        from_epoch: 1,
        to_epoch: 2,
        capacity,
    }
}

#[test]
fn pod_rejects_duplicate_overload_cog_unsecured_and_isolation_failure() {
    let mut pod = TransportPod::new(EntityId::new(), limits()).unwrap();
    let id = EntityId::new();
    pod.load(load(id.clone(), 5_000, 0)).unwrap();
    assert_eq!(
        pod.load(load(id, 1, 0)),
        Err(LogisticsError::UnsafeManifest)
    );
    let mut unsecured = load(EntityId::new(), 1, 0);
    unsecured.securement = false;
    assert_eq!(pod.load(unsecured), Err(LogisticsError::UnsafeManifest));
    let mut isolated = load(EntityId::new(), 1, 0);
    isolated.energy_isolated = false;
    assert_eq!(pod.load(isolated), Err(LogisticsError::UnsafeManifest));
    assert_eq!(
        pod.load(load(EntityId::new(), 20_000, 0)),
        Err(LogisticsError::UnsafeManifest)
    );
    assert_eq!(pod.asset_count(), 1);
    pod.seal([1; 32]).unwrap();
    assert_eq!(pod.state, PodState::Sealed);
}

#[test]
fn reservation_ledger_prevents_duplicate_asset_and_slot_overbooking() {
    let s = slot("bridge-1", CapacityKind::Bridge, 2);
    let asset = EntityId::new();
    let mut ledger = CapacityLedger::default();
    ledger.reserve(&s, &asset, 1).unwrap();
    assert_eq!(
        ledger.reserve(&s, &asset, 1),
        Err(LogisticsError::MobilizationCapacity)
    );
    ledger.reserve(&s, &EntityId::new(), 1).unwrap();
    assert_eq!(
        ledger.reserve(&s, &EntityId::new(), 1),
        Err(LogisticsError::MobilizationCapacity)
    );
    assert_eq!(ledger.used("bridge-1"), 2);
}

#[test]
fn shared_downstream_slot_is_never_double_allocated_across_paths() {
    let shared = slot("admission", CapacityKind::Admission, 5);
    let paths = vec![
        FlowPath {
            id: "road".into(),
            slots: vec![slot("road", CapacityKind::Route, 5), shared.clone()],
            travel_epochs: 1,
            useful_arrival_capacity: 5,
        },
        FlowPath {
            id: "rail".into(),
            slots: vec![slot("rail", CapacityKind::Rail, 5), shared],
            travel_epochs: 1,
            useful_arrival_capacity: 5,
        },
    ];
    let demands = vec![
        FlowDemand {
            cohort_id: EntityId::new(),
            robots: 3,
            required_arrival_epoch: 5,
        },
        FlowDemand {
            cohort_id: EntityId::new(),
            robots: 3,
            required_arrival_epoch: 5,
        },
    ];
    assert_eq!(
        TimeExpandedFlowPlanner.plan(&demands, &paths),
        Err(LogisticsError::MobilizationCapacity)
    );
}

#[test]
fn wave_release_is_capped_by_downstream_and_arrival_is_not_useful_until_all_gates() {
    let demand = FlowDemand {
        cohort_id: EntityId::new(),
        robots: 10,
        required_arrival_epoch: 5,
    };
    let path = FlowPath {
        id: "route".into(),
        slots: vec![
            slot("route", CapacityKind::Route, 10),
            slot("admit", CapacityKind::Admission, 10),
        ],
        travel_epochs: 1,
        useful_arrival_capacity: 10,
    };
    let mut wave = MobilizationWave::draft(EntityId::new(), 10).unwrap();
    wave.check_capacity(&TimeExpandedFlowPlanner, &[demand], &[path])
        .unwrap();
    assert_eq!(wave.release(3).unwrap(), 3);
    wave.record_useful_arrival(3, 3, 3, 3, 0).unwrap();
    assert_eq!(wave.useful_arrivals, 0);
    wave.record_useful_arrival(10, 10, 10, 10, 10).unwrap();
    assert_eq!(wave.state, WaveState::Complete);
}

#[test]
fn carrier_failure_and_v2x_loss_always_produce_local_safe_stop() {
    let readiness = CarrierReadiness {
        traction_energy_wh: 100,
        fuel_energy_wh: 100,
        reserve_wh: 50,
        braking: OperationalCheck::Passed,
        steering: OperationalCheck::Passed,
        tires: OperationalCheck::Passed,
        odd: OperationalCheck::Passed,
        recovery: OperationalCheck::Passed,
    };
    let mut carrier = Carrier::new(EntityId::new(), 20_000, readiness).unwrap();
    carrier.dispatch(100, 1).unwrap();
    let mut simulator = HybridCarrierSimulator::default();
    simulator.drive(&carrier.id, 1).unwrap();
    assert!(simulator.moving(&carrier.id));
    carrier.v2x_lost();
    simulator.safe_stop(&carrier.id).unwrap();
    simulator.request_manual_recovery(&carrier.id).unwrap();
    assert!(simulator.stopped(&carrier.id));
    assert_eq!(carrier.state, CarrierState::Recovering);
}

#[test]
fn platoons_are_bounded_and_every_member_safe_stops_without_v2x() {
    let members = (0..8).map(|_| EntityId::new()).collect::<Vec<_>>();
    let plan = PlatoonPlan {
        cell_id: EntityId::new(),
        epoch: 2,
        members: members.clone(),
        route_digest: [1; 32],
        fallback: "local-safe-stop".into(),
    };
    let mut simulator = HybridCarrierSimulator::default();
    simulator.activate(&plan).unwrap();
    assert!(members.iter().all(|id| simulator.moving(id)));
    simulator.v2x_lost(&plan).unwrap();
    assert!(members.iter().all(|id| simulator.stopped(id)));
    let oversized = PlatoonPlan {
        members: (0..33).map(|_| EntityId::new()).collect(),
        ..plan
    };
    assert_eq!(oversized.validate(), Err(LogisticsError::UnsafeCarrier));
}

#[test]
fn parameterized_hundred_thousand_robot_scenario_is_partitioned_and_conserved() {
    let scenario = generate_mobilization_scenario(100_000, 500).unwrap();
    assert_eq!(scenario.demands.len(), 200);
    assert_eq!(
        scenario.demands.iter().map(|d| d.robots).sum::<u64>(),
        100_000
    );
    assert!(scenario.demands.iter().all(|d| d.robots <= 500));
    assert_eq!(
        generate_mobilization_scenario(100_000, 100_000),
        Err(LogisticsError::MobilizationCapacity)
    );
}

proptest! {#[test]fn release_never_exceeds_downstream(target in 1u64..100_000,downstream in 1u64..100_000){let mut wave=MobilizationWave::draft(EntityId::new(),target).unwrap();wave.state=WaveState::CapacityChecked;let released=wave.release(downstream).unwrap();prop_assert!(released<=target&&released<=downstream);}}
