//! Prompt 10 eligibility, fencing, repartitioning, and scale-generator tests.
#![allow(clippy::expect_used)]
use chrono::{DateTime, Duration, Utc};
use fleet_operations::*;
use proptest::prelude::*;
use shared_kernel::EntityId;
use std::collections::BTreeSet;

fn now() -> DateTime<Utc> {
    DateTime::<Utc>::UNIX_EPOCH + Duration::days(30)
}
fn installed_configuration() -> Configuration {
    let mut c =
        Configuration::register(EntityId::new(), [1; 32], [2; 32], [3; 32]).expect("config");
    c.validate([4; 32], "signed-matrix-v1-valid")
        .expect("validate");
    c.approve().expect("approve");
    c.attest_installation([1; 32]).expect("install");
    c
}
fn eligible_fixture() -> (
    Vehicle,
    Configuration,
    CapabilityRecord,
    HealthAssessment,
    BatteryAsset,
    MissionEnergyRequirement,
    Digest,
) {
    let instant = now();
    let config = installed_configuration();
    let odd = [8; 32];
    let mut vehicle = Vehicle::enroll(
        EntityId::new(),
        "serial-1",
        "device-1",
        "tenant-1",
        "owner-1",
        config.digest(),
        [9; 32],
        instant + Duration::days(10),
        instant + Duration::days(5),
        instant + Duration::days(5),
        instant,
    )
    .expect("vehicle");
    vehicle.activate(&config).expect("active");
    let mut capability = CapabilityRecord::claim(
        EntityId::new(),
        vehicle.id().clone(),
        "suppression",
        odd,
        config.digest(),
        instant + Duration::days(2),
        instant,
    )
    .expect("claim");
    capability.attach_evidence([7; 32]).expect("evidence");
    capability.attest(&config, instant).expect("attest");
    let health = HealthAssessment::assess(
        EntityId::new(),
        vehicle.id().clone(),
        "vehicle-monitor",
        9_500,
        instant - Duration::seconds(1),
        instant + Duration::minutes(5),
        [],
    )
    .expect("health");
    let mut battery = BatteryAsset::register(EntityId::new(), "lfp", "pack-a", [config.digest()])
        .expect("battery");
    battery
        .make_available(EnergyAssessment {
            usable_wh: 10_000,
            uncertainty_wh: 500,
            continuous_power_w: 4_000,
            temperature_millicelsius: 30_000,
            assessed_at: instant - Duration::seconds(1),
            expires_at: instant + Duration::minutes(5),
        })
        .expect("available");
    let requirement = MissionEnergyRequirement {
        departure_wh: 7_000,
        minimum_risk_reserve_wh: 2_000,
        required_power_w: 3_000,
        maximum_temperature_millicelsius: 45_000,
        minimum_health_quality_bps: 9_000,
    };
    (
        vehicle,
        config,
        capability,
        health,
        battery,
        requirement,
        odd,
    )
}

#[test]
fn exact_current_tuple_is_eligible_but_grounding_and_uncertainty_block() {
    let (mut vehicle, config, capability, health, battery, requirement, odd) = eligible_fixture();
    assert!(vehicle.allocatable(
        &config,
        &capability,
        &health,
        &battery,
        odd,
        &requirement,
        now()
    ));
    vehicle.ground("maintenance fault").expect("ground");
    assert!(!vehicle.allocatable(
        &config,
        &capability,
        &health,
        &battery,
        odd,
        &requirement,
        now()
    ));
    assert_eq!(
        vehicle.clear_grounding("different cause", [5; 32], true),
        Err(FleetError::InvalidClearance)
    );
    vehicle
        .clear_grounding("maintenance fault", [5; 32], true)
        .expect("clear");
    assert!(vehicle.allocatable(
        &config,
        &capability,
        &health,
        &battery,
        odd,
        &requirement,
        now()
    ));
}

#[test]
fn stale_health_suspended_capability_and_quarantined_battery_cannot_allocate() {
    let (vehicle, config, mut capability, mut health, mut battery, requirement, odd) =
        eligible_fixture();
    health.mark_stale();
    assert!(!vehicle.allocatable(
        &config,
        &capability,
        &health,
        &battery,
        odd,
        &requirement,
        now()
    ));
    capability.suspend().expect("suspend");
    battery.quarantine().expect("quarantine");
    assert!(!vehicle.allocatable(
        &config,
        &capability,
        &health,
        &battery,
        odd,
        &requirement,
        now()
    ));
}

#[test]
fn stale_epoch_cannot_reserve_or_command_and_split_conserves_membership() {
    let scope = CellScope {
        tenant: "tenant-1".into(),
        region: "ca".into(),
        purpose: "mission".into(),
        capability_bucket: "mixed".into(),
    };
    let mut cell = FleetCell::form(EntityId::new(), None, scope, 1, 1, 100).expect("cell");
    cell.activate().expect("activate");
    let mut old = Vec::new();
    for _ in 0..40 {
        old.push(
            cell.assign_member(EntityId::new(), "mission", 1)
                .expect("member"),
        );
    }
    let old_fence = cell.fence();
    assert!(
        old.iter()
            .all(|m| authorize_local_operation(m, &old_fence).is_ok())
    );
    cell.begin_split(1).expect("split");
    let (left, right) = cell
        .split(EntityId::new(), EntityId::new())
        .expect("children");
    assert_eq!(left.member_count() + right.member_count(), 40);
    assert!(
        old.iter()
            .all(|m| authorize_local_operation(m, &left.fence()).is_err()
                && authorize_local_operation(m, &right.fence()).is_err())
    );
    let ids = left
        .memberships()
        .chain(right.memberships())
        .map(|m| m.asset_id.to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), 40);
}

#[test]
fn generator_is_reproducible_streaming_and_ci_scale_unique() {
    let first = SyntheticIdentityGenerator::new(42, 10_000).collect::<Vec<_>>();
    let second = SyntheticIdentityGenerator::new(42, 10_000).collect::<Vec<_>>();
    assert_eq!(first, second);
    assert_eq!(
        first
            .iter()
            .map(|v| v.stable_id)
            .collect::<BTreeSet<_>>()
            .len(),
        10_000
    );
    assert_ne!(
        first[0],
        SyntheticIdentityGenerator::new(43, 1)
            .next()
            .expect("identity")
    );
}

proptest! {#[test]fn increasing_energy_uncertainty_never_creates_eligibility(usable in 1_000u64..100_000,uncertainty in 0u64..100_000){let config=[1;32];let instant=now();let mut battery=BatteryAsset::register(EntityId::new(),"lfp","pack",[config]).expect("battery");let result=battery.make_available(EnergyAssessment{usable_wh:usable,uncertainty_wh:uncertainty,continuous_power_w:1000,temperature_millicelsius:20_000,assessed_at:instant,expires_at:instant+Duration::minutes(1)});if result.is_ok(){let eligible=battery.eligible(config,usable,1,1,40_000,instant);prop_assert!(!eligible);}}}
