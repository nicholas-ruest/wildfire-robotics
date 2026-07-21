//! Integration tests for tamper evidence and durable audit failure policy.

use audit_ledger::*;
use chrono::{DateTime, Utc};
use std::sync::Mutex;

struct TestKeys;

impl AuditSigner for TestKeys {
    fn key_id(&self) -> &'static str {
        "root-2/key-7"
    }
    fn sign(&self, hash: &AuditHash) -> Result<Vec<u8>, AuditError> {
        Ok(hash.as_bytes().iter().map(|byte| byte ^ 0x5a).collect())
    }
}

impl AuditSignatureVerifier for TestKeys {
    fn verify(&self, key_id: &str, hash: &AuditHash, signature: &[u8]) -> Result<bool, AuditError> {
        Ok(matches!(key_id, "root-1/key-3" | "root-2/key-7")
            && signature
                == hash
                    .as_bytes()
                    .iter()
                    .map(|byte| byte ^ 0x5a)
                    .collect::<Vec<_>>())
    }
}

fn record(action: &str) -> AuditRecord {
    AuditRecord {
        stream: AuditStream::Safety,
        tenant_id: "tenant-1".into(),
        actor_id: "principal-1".into(),
        action: action.into(),
        resource: "mission/mission-1".into(),
        reason: "safety constraint changed".into(),
        occurred_at: DateTime::<Utc>::UNIX_EPOCH,
        clock_quality: AuditClockQuality::Synchronized,
        correlation_id: "correlation-1".into(),
        causation_id: None,
        policy_version: "policy-3".into(),
        payload_digest: AuditHash::from_bytes([7; HASH_LENGTH]),
    }
}

fn two_entries() -> Result<Vec<AuditEntry>, AuditError> {
    let first = build_entry(
        record("constraint.publish"),
        0,
        AuditHash::GENESIS,
        &TestKeys,
    )?;
    let second = build_entry(
        record("mission.stop"),
        first.sequence,
        first.entry_hash,
        &TestKeys,
    )?;
    Ok(vec![first, second])
}

#[test]
fn valid_chain_survives_historical_root_rotation() -> Result<(), AuditError> {
    let mut entries = two_entries()?;
    entries[0].signing_key_id = "root-1/key-3".into();
    verify_chain(&entries, &TestKeys)
}

#[test]
fn content_mutation_is_detected() -> Result<(), AuditError> {
    let mut entries = two_entries()?;
    entries[0].record.reason = "concealed mutation".into();
    assert!(matches!(
        verify_chain(&entries, &TestKeys),
        Err(AuditError::ContentTampered { sequence: 1 })
    ));
    Ok(())
}

#[test]
fn deletion_reordering_and_signature_corruption_are_detected() -> Result<(), AuditError> {
    let entries = two_entries()?;
    assert!(matches!(
        verify_chain(&entries[1..], &TestKeys),
        Err(AuditError::SequenceGap { .. })
    ));
    let mut reversed = entries.clone();
    reversed.reverse();
    assert!(matches!(
        verify_chain(&reversed, &TestKeys),
        Err(AuditError::SequenceGap { .. })
    ));
    let mut corrupted = entries;
    corrupted[0].signature[0] ^= 1;
    assert!(matches!(
        verify_chain(&corrupted, &TestKeys),
        Err(AuditError::InvalidSignature { sequence: 1 })
    ));
    Ok(())
}

struct MemoryDurable {
    head: Mutex<LedgerHead>,
}

impl DurableAuditPort for MemoryDurable {
    fn append<'a>(
        &'a self,
        expected_head: LedgerHead,
        entry: &'a AuditEntry,
    ) -> AuditFuture<'a, LedgerHead> {
        Box::pin(async move {
            let mut head = self
                .head
                .lock()
                .map_err(|_| AuditError::Provider("test lock poisoned".into()))?;
            if *head != expected_head {
                return Err(AuditError::ConcurrentAppend);
            }
            let committed = LedgerHead {
                sequence: entry.sequence,
                hash: entry.entry_hash,
            };
            *head = committed;
            Ok(committed)
        })
    }
}

struct FailedTelemetry;
impl AuditTelemetryPort for FailedTelemetry {
    fn try_emit(&self, _entry: &AuditEntry) -> Result<(), TelemetryError> {
        Err(TelemetryError::Unavailable)
    }
}

#[test]
fn concurrent_head_is_rejected_and_telemetry_cannot_block_durable_audit() -> Result<(), AuditError>
{
    let durable = MemoryDurable {
        head: Mutex::new(LedgerHead {
            sequence: 0,
            hash: AuditHash::GENESIS,
        }),
    };
    let entry = build_entry(record("emergency.stop"), 0, AuditHash::GENESIS, &TestKeys)?;
    let genesis = LedgerHead {
        sequence: 0,
        hash: AuditHash::GENESIS,
    };
    let committed = futures::executor::block_on(append_durable_then_telemetry(
        &durable,
        &FailedTelemetry,
        genesis,
        &entry,
    ))?;
    assert_eq!(committed.sequence, 1);
    let stale = futures::executor::block_on(durable.append(genesis, &entry));
    assert!(matches!(stale, Err(AuditError::ConcurrentAppend)));
    Ok(())
}

struct FailedDurable;
impl DurableAuditPort for FailedDurable {
    fn append<'a>(
        &'a self,
        _expected_head: LedgerHead,
        _entry: &'a AuditEntry,
    ) -> AuditFuture<'a, LedgerHead> {
        Box::pin(async { Err(AuditError::Provider("durable path unavailable".into())) })
    }
}

#[test]
fn durable_failure_fails_closed() -> Result<(), AuditError> {
    let entry = build_entry(record("emergency.stop"), 0, AuditHash::GENESIS, &TestKeys)?;
    let result = futures::executor::block_on(append_durable_then_telemetry(
        &FailedDurable,
        &FailedTelemetry,
        LedgerHead {
            sequence: 0,
            hash: AuditHash::GENESIS,
        },
        &entry,
    ));
    assert!(matches!(result, Err(AuditError::Provider(_))));
    Ok(())
}
