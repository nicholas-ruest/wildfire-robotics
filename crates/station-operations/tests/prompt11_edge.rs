//! Prompt 11 fault campaigns for station isolation, reconciliation, pressure, and recovery.
#![allow(clippy::expect_used)]
use chrono::{DateTime, Duration, Utc};
use shared_kernel::EntityId;
use station_operations::*;

fn now() -> DateTime<Utc> {
    DateTime::<Utc>::UNIX_EPOCH + Duration::days(40)
}
fn budget() -> ResourceBudget {
    ResourceBudget {
        cpu_millis: 4_000,
        memory_mib: 8_192,
        disk_mib: 100_000,
    }
}
fn manifest() -> DeploymentManifest {
    DeploymentManifest {
        deployment_id: EntityId::new(),
        release_digest: [1; 32],
        configuration_digest: [2; 32],
        policy_digest: [3; 32],
        identity_bundle_digest: [4; 32],
        map_digest: [5; 32],
        schema_version: 1,
        required: ResourceBudget {
            cpu_millis: 1_000,
            memory_mib: 1_024,
            disk_mib: 10_000,
        },
        minimum_runtime_version: 1,
        created_at: now() - Duration::minutes(1),
        expires_at: now() + Duration::days(1),
        signature: "valid-detached-signature".into(),
    }
}
struct Verifier(bool);
impl ManifestVerifier for Verifier {
    type Error = ();
    fn verify(&self, _: &DeploymentManifest) -> Result<bool, Self::Error> {
        Ok(self.0)
    }
}

#[test]
fn signed_compatible_deployment_activates_but_bad_signature_clock_and_upgrade_fail_closed() {
    let mut good = EdgeDeployment::stage(manifest(), Some([9; 32]));
    let mut supervisor = EdgeRuntimeSupervisor::new(
        Verifier(true),
        LightweightRuntimeAdapter::default(),
        1,
        budget(),
        100,
    );
    supervisor
        .verify_and_activate(&mut good, now(), 10)
        .expect("activate");
    assert_eq!(good.state(), DeploymentState::Active);
    let mut bad = EdgeDeployment::stage(manifest(), None);
    let mut invalid = EdgeRuntimeSupervisor::new(
        Verifier(false),
        LightweightRuntimeAdapter::default(),
        1,
        budget(),
        100,
    );
    assert_eq!(
        invalid.verify_and_activate(&mut bad, now(), 10),
        Err(StationError::InvalidSignature)
    );
    assert_eq!(bad.state(), DeploymentState::Staged);
    let mut uncertain = EdgeDeployment::stage(manifest(), None);
    assert_eq!(
        supervisor.verify_and_activate(&mut uncertain, now(), 101),
        Err(StationError::ClockUncertain)
    );
    assert_eq!(uncertain.state(), DeploymentState::Staged);
    let mut failed = EdgeDeployment::stage(manifest(), Some([9; 32]));
    let mut runtime = LightweightRuntimeAdapter::default();
    runtime.inject_activation_failure();
    let mut failing = EdgeRuntimeSupervisor::new(Verifier(true), runtime, 1, budget(), 100);
    assert_eq!(
        failing.verify_and_activate(&mut failed, now(), 1),
        Err(StationError::IncompatibleDeployment)
    );
    assert_ne!(failed.state(), DeploymentState::Active);
}

#[test]
fn cloud_loss_and_partial_sync_never_extend_expired_authority_or_skip_cursor() {
    let instant = now();
    let mut cache = OfflineCache::default();
    cache
        .reconcile(
            CacheKind::Policy,
            CacheEntry {
                key: "policy".into(),
                digest: [1; 32],
                version: 1,
                expires_at: Some(instant + Duration::seconds(1)),
                tombstone: false,
                authority_rank: 10,
            },
            instant,
        )
        .expect("cache");
    assert!(cache.usable(CacheKind::Policy, "policy", instant));
    assert!(!cache.usable(CacheKind::Policy, "policy", instant + Duration::seconds(1)));
    let renewed = cache.reconcile(
        CacheKind::Policy,
        CacheEntry {
            key: "policy".into(),
            digest: [2; 32],
            version: 2,
            expires_at: Some(instant + Duration::days(1)),
            tombstone: false,
            authority_rank: 10,
        },
        instant + Duration::seconds(1),
    );
    assert_eq!(renewed, Err(StationError::ReconciliationConflict));
    let mut cursors = ReconciliationCursors::default();
    cursors.advance("policy", 0, 1).expect("first");
    assert_eq!(
        cursors.advance("policy", 1, 3),
        Err(StationError::ReconciliationConflict)
    );
    assert_eq!(cursors.get("policy"), 1);
}

#[test]
fn disk_pressure_sheds_optional_work_and_preserves_reserved_audit_records() {
    let mut station =
        Station::commission(EntityId::new(), budget(), 1_000, 5_000).expect("station");
    station.attest().expect("attest");
    let shed = station
        .apply_pressure(ResourcePressure {
            disk_used_bps: 9_600,
            memory_used_bps: 5_000,
            cpu_used_bps: 5_000,
            power_available_bps: 8_000,
            thermal_margin_bps: 8_000,
        })
        .expect("shed");
    assert!(
        shed.contains(&WorkloadClass::OptionalIndexing)
            && shed.contains(&WorkloadClass::OptionalMl)
    );
    for class in [
        WorkloadClass::Command,
        WorkloadClass::Safety,
        WorkloadClass::Identity,
        WorkloadClass::Audit,
    ] {
        assert!(station.enabled(class));
    }
    assert_eq!(
        station.reserve_routine_energy(4_001),
        Err(StationError::EmergencyReserve)
    );
    let mut buffer = DurableLocalBuffer::new(4, 2).expect("buffer");
    buffer.append(CacheKind::Mission, [1; 32]).expect("event");
    buffer.append(CacheKind::Map, [2; 32]).expect("event");
    assert_eq!(
        buffer.append(CacheKind::Map, [3; 32]),
        Err(StationError::CorruptLog)
    );
    buffer
        .append(CacheKind::Audit, [4; 32])
        .expect("audit reserve");
    buffer
        .append(CacheKind::Audit, [5; 32])
        .expect("audit reserve");
    buffer.verify().expect("chain");
}

#[test]
fn corrupt_log_and_restart_are_detected_without_rebasing_audit_chain() {
    let mut original = DurableLocalBuffer::new(8, 4).expect("buffer");
    original.append(CacheKind::Audit, [1; 32]).expect("audit");
    original.append(CacheKind::Mission, [2; 32]).expect("event");
    let snapshot = original.snapshot();
    let restored = DurableLocalBuffer::restore(snapshot.clone(), 8, 4).expect("restart");
    assert_eq!(restored.len(), 2);
    let mut corrupt = snapshot;
    corrupt[1].payload_digest = [9; 32];
    assert!(matches!(
        DurableLocalBuffer::restore(corrupt, 8, 4),
        Err(StationError::CorruptLog)
    ));
}

#[test]
fn contradictory_equal_version_and_tombstone_resurrection_enter_quarantine() {
    let instant = now();
    let mut cache = OfflineCache::default();
    cache
        .reconcile(
            CacheKind::Identity,
            CacheEntry {
                key: "device".into(),
                digest: [1; 32],
                version: 3,
                expires_at: None,
                tombstone: true,
                authority_rank: 255,
            },
            instant,
        )
        .expect("revoke");
    assert_eq!(
        cache.reconcile(
            CacheKind::Identity,
            CacheEntry {
                key: "device".into(),
                digest: [2; 32],
                version: 3,
                expires_at: None,
                tombstone: false,
                authority_rank: 1
            },
            instant
        ),
        Err(StationError::ReconciliationConflict)
    );
    assert_eq!(cache.quarantine_len(), 1);
    assert!(!cache.usable(CacheKind::Identity, "device", instant));
}
