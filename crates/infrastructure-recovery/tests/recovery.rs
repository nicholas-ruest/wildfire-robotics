#![allow(
    missing_docs,
    clippy::unwrap_used,
    clippy::semicolon_if_nothing_returned
)]
use infrastructure_recovery::*;
use std::collections::BTreeSet;
struct Keys(bool);
impl KeyRecoveryPort for Keys {
    fn verify_key_available(&self, _: &str) -> Result<(), RecoveryError> {
        if self.0 {
            Ok(())
        } else {
            Err(RecoveryError::KeyUnavailable)
        }
    }
    fn verify_manifest_signature(&self, _: [u8; 32]) -> Result<(), RecoveryError> {
        if self.0 {
            Ok(())
        } else {
            Err(RecoveryError::KeyUnavailable)
        }
    }
}
fn manifest() -> BackupManifest {
    BackupManifest::seal(
        "b1",
        "tenant-a",
        "ca-central",
        "ca-west",
        1_000,
        900,
        "kms/key/1",
        vec![
            BackupObject {
                resource: ResourceKind::Database,
                object_id: "wal-1".into(),
                checksum: [1; 32],
                bytes: 10,
            },
            BackupObject {
                resource: ResourceKind::Broker,
                object_id: "stream-1".into(),
                checksum: [2; 32],
                bytes: 10,
            },
        ],
        7,
        9,
    )
    .unwrap()
}
fn plan(m: &BackupManifest) -> RecoveryPlan {
    let resources = [
        ResourceKind::Database,
        ResourceKind::Broker,
        ResourceKind::Station,
        ResourceKind::CloudRegion,
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    RecoveryPlan::define(
        "p1",
        m,
        required_actions(&resources),
        200,
        1_000,
        "disaster/1",
        "approval/1",
        [8; 32],
        8,
        [9; 32],
    )
    .unwrap()
}
fn step(action: RecoveryAction, n: u64) -> StepEvidence {
    StepEvidence {
        action,
        started_ms: n,
        completed_ms: n + 1,
        artifact_digest: [3; 32],
    }
}
#[test]
fn immutable_manifest_detects_mutation_and_residency_violation() {
    let mut m = manifest();
    assert!(m.verify());
    m.command_high_watermark += 1;
    assert!(!m.verify());
    assert!(
        BackupManifest::seal(
            "x",
            "t",
            "us-east",
            "ca-west",
            1,
            1,
            "k",
            vec![BackupObject {
                resource: ResourceKind::Database,
                object_id: "x".into(),
                checksum: [1; 32],
                bytes: 1
            }],
            0,
            0
        )
        .is_err()
    )
}
#[test]
fn restore_order_prevents_blind_replay_and_authority_resurrection() {
    let m = manifest();
    let p = plan(&m);
    let mut e = RecoveryEvidence::start(&p, 1_100, 1_000);
    e.resume(&p, &m, &Keys(true)).unwrap();
    assert!(
        e.record_step(&p, step(RecoveryAction::RestoreBrokerQuarantined, 1_101))
            .is_err()
    );
    assert!(!e.authority_restored);
    assert!(!e.commands_replayed)
}
#[test]
fn restart_resumes_from_durable_cursor_and_measures_rpo_rto() {
    let m = manifest();
    let p = plan(&m);
    let mut e = RecoveryEvidence::start(&p, 1_100, 1_000);
    e.resume(&p, &m, &Keys(true)).unwrap();
    e.attach_authority_revalidation(
        &p,
        AuthorityRevalidation {
            evidence_ref: "authority/8".into(),
            authority_epoch: 8,
            valid_until_ms: 5_000,
            signature: [7; 32],
        },
    )
    .unwrap();
    for (i, a) in p.actions.iter().copied().enumerate() {
        if i == 2 {
            let encoded = serde_json::to_vec(&e).unwrap();
            e = serde_json::from_slice(&encoded).unwrap();
            e.resume(&p, &m, &Keys(true)).unwrap()
        }
        e.record_step(&p, step(a, 1_101 + i as u64)).unwrap()
    }
    assert_eq!(e.verify_objectives(&p, &m, 1_200).unwrap(), (100, 100));
    assert_eq!(e.state, ExerciseState::Verified)
}
#[test]
fn key_failure_and_objective_miss_fail_closed_and_rollback() {
    let m = manifest();
    let p = plan(&m);
    let mut e = RecoveryEvidence::start(&p, 1_100, 1_000);
    assert_eq!(
        e.resume(&p, &m, &Keys(false)),
        Err(RecoveryError::KeyUnavailable)
    );
    e.resume(&p, &m, &Keys(true)).unwrap();
    e.attach_authority_revalidation(
        &p,
        AuthorityRevalidation {
            evidence_ref: "authority/8".into(),
            authority_epoch: 8,
            valid_until_ms: 5_000,
            signature: [7; 32],
        },
    )
    .unwrap();
    for (i, a) in p.actions.iter().copied().enumerate() {
        e.record_step(&p, step(a, 1_101 + i as u64)).unwrap()
    }
    assert_eq!(
        e.verify_objectives(&p, &m, 3_000),
        Err(RecoveryError::ObjectiveMissed)
    );
    e.rollback().unwrap();
    assert_eq!(e.state, ExerciseState::RolledBack)
}
#[test]
fn plan_requires_signed_approval_and_new_epoch() {
    let m = manifest();
    let actions = required_actions(&[ResourceKind::Database].into_iter().collect());
    assert!(
        RecoveryPlan::define(
            "p",
            &m,
            actions.clone(),
            1,
            1,
            "",
            "approval",
            [1; 32],
            8,
            [2; 32]
        )
        .is_err()
    );
    assert!(
        RecoveryPlan::define(
            "p", &m, actions, 1, 1, "disaster", "approval", [1; 32], 7, [2; 32]
        )
        .is_err()
    );
}
#[test]
fn required_order_is_foundational_and_command_enable_is_last() {
    let actions = required_actions(
        &[
            ResourceKind::Database,
            ResourceKind::ObjectStore,
            ResourceKind::Broker,
            ResourceKind::Station,
            ResourceKind::CloudRegion,
        ]
        .into_iter()
        .collect(),
    );
    assert_eq!(
        actions.first(),
        Some(&RecoveryAction::RestoreNetworkAndTrustedTime)
    );
    assert_eq!(actions.get(1), Some(&RecoveryAction::RestoreKeyAccess));
    assert_eq!(
        actions.get(2),
        Some(&RecoveryAction::RestoreIdentityPkiAndRevocation)
    );
    assert_eq!(actions.last(), Some(&RecoveryAction::EnableCommandTraffic));
}
#[test]
fn stale_authority_cannot_release_quarantined_commands() {
    let m = manifest();
    let p = plan(&m);
    let mut e = RecoveryEvidence::start(&p, 1_100, 1_000);
    e.resume(&p, &m, &Keys(true)).unwrap();
    assert!(
        e.attach_authority_revalidation(
            &p,
            AuthorityRevalidation {
                evidence_ref: "old".into(),
                authority_epoch: 7,
                valid_until_ms: 5_000,
                signature: [7; 32]
            }
        )
        .is_err()
    );
    assert!(!e.command_traffic_enabled);
    assert!(!e.commands_replayed);
}
