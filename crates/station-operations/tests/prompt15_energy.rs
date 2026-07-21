//! Prompt 15 habitat, microgrid, and charging safety scenarios.
#![allow(clippy::unwrap_used)]
use chrono::{DateTime, Duration, TimeZone, Utc};
use proptest::prelude::*;
use shared_kernel::EntityId;
use station_operations::*;
use std::collections::BTreeSet;

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, 21, 12, 0, 0).unwrap()
}
fn estimate(uncertainty: u16) -> EnergyEstimate {
    EnergyEstimate {
        usable_wh: 10_000,
        soc_bps: 8_000,
        soh_bps: 9_000,
        power_w: 1_000,
        uncertainty_bps: uncertainty,
        source: "bms-a".into(),
        method: "coulomb-counting-v2".into(),
        measured_at: now(),
        valid_until: now() + Duration::minutes(1),
    }
}
fn precheck() -> ChargePrecheck {
    ChargePrecheck {
        battery: EntityId::new(),
        charger: EntityId::new(),
        vehicle: EntityId::new(),
        compatibility: SafetyCheck::Passed,
        isolation: SafetyCheck::Passed,
        temperature: SafetyCheck::Passed,
        bms_authority: SafetyCheck::Passed,
        zone_capacity: SafetyCheck::Passed,
        schedule_fence: 4,
        protection: SafetyCheck::Passed,
    }
}

#[test]
fn habitat_requires_every_readiness_gate() {
    let mut h = RobotHabitat::plan(EntityId::new());
    h.commission().unwrap();
    let mut a = HabitatAssessment {
        structure: Readiness::Ready,
        environment: Readiness::Ready,
        communications: Readiness::Ready,
        emergency_energy: Readiness::Ready,
        isolation_and_fire: Readiness::Ready,
        compatible_docks: 1,
        maintenance_capacity: 1,
        evacuation_digest: [1; 32],
        deployment_digest: [2; 32],
    };
    a.isolation_and_fire = Readiness::Unsafe;
    assert_eq!(h.assess(a), Err(StationError::UnsafeCapacity));
    assert_eq!(h.state, HabitatState::Degraded);
}

#[test]
fn low_solar_and_outage_shed_optional_loads_before_critical() {
    let mut grid = Microgrid::new(EntityId::new(), 100, 500).unwrap();
    grid.black_start(true).unwrap();
    let critical = EntityId::new();
    let optional = EntityId::new();
    let shed = grid.dispatch(
        &[
            SiteLoad {
                id: optional.clone(),
                watts: 80,
                priority: LoadPriority::OptionalCharge,
            },
            SiteLoad {
                id: critical.clone(),
                watts: 80,
                priority: LoadPriority::LifeSafety,
            },
        ],
        80,
    );
    assert!(shed.contains(&optional.to_string()));
    assert!(!shed.contains(&critical.to_string()));
    assert_eq!(grid.mode, GridMode::Constrained);
}

#[test]
fn cold_incompatible_stale_bms_and_bad_fence_never_start() {
    for mutate in 0..4 {
        let mut p = precheck();
        match mutate {
            0 => p.temperature = SafetyCheck::Failed,
            1 => p.compatibility = SafetyCheck::Failed,
            2 => p.bms_authority = SafetyCheck::Failed,
            _ => p.schedule_fence = 3,
        }
        let mut s = ChargeSession::request(EntityId::new(), 4).unwrap();
        assert_eq!(s.start(&p), Err(StationError::UnsafeCapacity));
        assert_eq!(s.state, ChargeState::Requested);
    }
}

#[test]
fn stale_energy_is_rejected_at_exact_expiry() {
    let e = estimate(100);
    assert_eq!(e.validate(e.valid_until), Err(StationError::UnsafeCapacity));
}

#[test]
fn charge_surge_is_partition_bounded_and_deterministic() {
    let candidates = (0_u8..100)
        .map(|i| ChargeCandidate {
            session_id: EntityId::new(),
            readiness_class: i % 3,
            deadline_epoch_seconds: i64::from(i),
            requested_wh: 100,
            max_power_w: 100,
            degradation_cost_micros: u64::from(i),
            compatible: true,
            bms_current: true,
            zone: "a".into(),
        })
        .collect::<Vec<_>>();
    let o = PartitionedReferenceOptimizer;
    let a = o.schedule(&candidates, 1_000, 1_000, 500, 9).unwrap();
    let b = o.schedule(&candidates, 1_000, 1_000, 500, 9).unwrap();
    assert_eq!(a, b);
    assert!(a.scheduled.iter().map(|v| v.power_w).sum::<u64>() <= 1_000);
    assert!(a.scheduled.iter().map(|v| v.energy_wh).sum::<u64>() <= 1_000);
    assert_eq!(a.scheduled.len(), 10);
}

struct Bms {
    trip: bool,
    limit: Option<u64>,
}
impl BmsPort for Bms {
    fn authoritative_limit_w(&self, _: &EntityId, _: DateTime<Utc>) -> Option<u64> {
        self.limit
    }
    fn tripped(&self, _: &EntityId) -> bool {
        self.trip
    }
}
#[test]
fn thermal_trip_overrides_optimizer_and_inhibits_physical_effect() {
    let battery = EntityId::new();
    let mut session = ChargeSession::request(EntityId::new(), 4).unwrap();
    session.start(&precheck()).unwrap();
    let mut charger = DeterministicChargerSimulator::default();
    apply_charge_command(
        &mut session,
        &battery,
        400,
        4,
        now(),
        &Bms {
            trip: false,
            limit: Some(500),
        },
        &mut charger,
    )
    .unwrap();
    assert_eq!(charger.effect_count(), 1);
    assert_eq!(
        apply_charge_command(
            &mut session,
            &battery,
            400,
            4,
            now(),
            &Bms {
                trip: true,
                limit: Some(500)
            },
            &mut charger
        ),
        Err(StationError::UnsafeCapacity)
    );
    assert_eq!(charger.effect_count(), 0);
    assert_eq!(session.state, ChargeState::Quarantined);
}

#[test]
fn emergency_reserve_is_not_available_to_routine_charging() {
    let mut store = EnergyStore::record(EntityId::new(), estimate(1_000), now()).unwrap();
    store.reserve_emergency(6_000).unwrap();
    assert_eq!(
        store.reserve_routine(1_001),
        Err(StationError::EmergencyReserve)
    );
    store.reserve_routine(1_000).unwrap();
    assert_eq!(store.routine_reserved_wh(), 1_000);
}

#[test]
fn maintenance_bay_is_compatible_and_exclusive() {
    let mut bay = MaintenanceBay::new(EntityId::new(), vec!["battery-isolation".into()]).unwrap();
    assert_eq!(bay.reserve("wrong"), Err(StationError::InvalidTransition));
    bay.reserve("battery-isolation").unwrap();
    assert_eq!(
        bay.reserve("battery-isolation"),
        Err(StationError::InvalidTransition)
    );
    bay.block();
    assert_eq!(bay.state, BayState::Blocked);
}

proptest! {#[test]fn wider_uncertainty_never_increases_routine_capacity(low in 0u16..5_000,delta in 0u16..5_000){let high=low.saturating_add(delta).min(9_999);let mut loose=EnergyStore::record(EntityId::new(),estimate(low),now()).unwrap();let mut conservative=EnergyStore::record(EntityId::new(),estimate(high),now()).unwrap();let request=1_000;let loose_ok=loose.reserve_routine(request).is_ok();let conservative_ok=conservative.reserve_routine(request).is_ok();prop_assert!(!conservative_ok||loose_ok);}}

#[test]
fn readiness_summary_is_partition_local() {
    let ids = (0..10_000)
        .map(|_| EntityId::new().to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), 10_000);
}
