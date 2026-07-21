#![forbid(unsafe_code)]
//! Tamper-evident, signed audit ledger primitives governed by ADR-011/039.
//!
//! This crate owns deterministic chain verification, not storage or a signing
//! algorithm. Production storage and HSM/KMS implementations enter through
//! ports and must preserve the encoded entries verbatim.

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{future::Future, pin::Pin};
use thiserror::Error;

/// SHA-256 digest length.
pub const HASH_LENGTH: usize = 32;

/// Fixed-size content digest used for payloads and chain links.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AuditHash([u8; HASH_LENGTH]);

impl AuditHash {
    /// Genesis link used only for sequence one.
    pub const GENESIS: Self = Self([0; HASH_LENGTH]);
    /// Builds a hash from an adapter-provided byte array.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; HASH_LENGTH]) -> Self {
        Self(bytes)
    }
    /// Returns the exact digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; HASH_LENGTH] {
        &self.0
    }
}

/// Audit stream classification with independently durable safety/security paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AuditStream {
    /// Safety decisions, constraints, occurrences, and physical-control actions.
    Safety,
    /// Authentication, authorization, key, policy, and incident-response actions.
    Security,
    /// Governed configuration, model, release, approval, and administrative actions.
    Governance,
}

impl AuditStream {
    const fn canonical_byte(self) -> u8 {
        match self {
            Self::Safety => 1,
            Self::Security => 2,
            Self::Governance => 3,
        }
    }
}

/// Declared quality of the source wall clock used by an audit producer.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AuditClockQuality {
    /// Synchronized within the deployment's approved bound.
    Synchronized,
    /// Known to exceed the nominal uncertainty bound.
    Degraded,
    /// No trustworthy synchronization assessment is available.
    Unknown,
}

/// Immutable semantic content of one audit entry.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuditRecord {
    /// Independent ledger stream.
    pub stream: AuditStream,
    /// Tenant boundary; never inferred from actor or network location.
    pub tenant_id: String,
    /// Stable actor identity.
    pub actor_id: String,
    /// Stable action name.
    pub action: String,
    /// Resource type and identity, encoded by the owning adapter.
    pub resource: String,
    /// Auditable reason, redacted according to classification policy.
    pub reason: String,
    /// UTC source time and declared clock quality are part of the signed payload.
    pub occurred_at: DateTime<Utc>,
    /// Quality of the source clock at `occurred_at`.
    pub clock_quality: AuditClockQuality,
    /// End-to-end correlation ID.
    pub correlation_id: String,
    /// Direct causal predecessor when applicable.
    pub causation_id: Option<String>,
    /// Version of policy used to permit or deny the action.
    pub policy_version: String,
    /// Digest of immutable detail stored outside this bounded record.
    pub payload_digest: AuditHash,
}

impl AuditRecord {
    /// Validates attribution and bounded textual fields before signing.
    pub fn validate(&self) -> Result<(), AuditError> {
        for value in [
            &self.tenant_id,
            &self.actor_id,
            &self.action,
            &self.resource,
            &self.reason,
            &self.correlation_id,
            &self.policy_version,
        ] {
            if value.trim().is_empty() || value.len() > 4096 {
                return Err(AuditError::InvalidRecord);
            }
        }
        if self
            .causation_id
            .as_ref()
            .is_some_and(|value| value.trim().is_empty() || value.len() > 512)
        {
            return Err(AuditError::InvalidRecord);
        }
        Ok(())
    }
}

/// Signed chain element suitable for immutable durable storage.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Strictly increasing stream-local sequence.
    pub sequence: u64,
    /// Digest of the prior entry, or genesis for sequence one.
    pub previous_hash: AuditHash,
    /// Immutable audit semantics.
    pub record: AuditRecord,
    /// Digest over canonical entry bytes excluding signature.
    pub entry_hash: AuditHash,
    /// Identifier of the signing key/version.
    pub signing_key_id: String,
    /// Detached signature produced over `entry_hash`.
    pub signature: Vec<u8>,
}

/// Signing boundary implemented by an approved KMS/HSM adapter.
pub trait AuditSigner: Send + Sync {
    /// Active signing-key identifier included in the entry.
    fn key_id(&self) -> &str;
    /// Signs an entry hash without exposing private key material.
    fn sign(&self, hash: &AuditHash) -> Result<Vec<u8>, AuditError>;
}

/// Verification boundary supporting historical keys and root rotation.
pub trait AuditSignatureVerifier: Send + Sync {
    /// Verifies a signature under the entry's historical key identifier.
    fn verify(&self, key_id: &str, hash: &AuditHash, signature: &[u8]) -> Result<bool, AuditError>;
}

/// Builds a signed successor while enforcing optimistic sequence/link state.
pub fn build_entry(
    record: AuditRecord,
    expected_sequence: u64,
    expected_previous_hash: AuditHash,
    signer: &dyn AuditSigner,
) -> Result<AuditEntry, AuditError> {
    record.validate()?;
    let sequence = expected_sequence
        .checked_add(1)
        .ok_or(AuditError::SequenceExhausted)?;
    if sequence == 1 && expected_previous_hash != AuditHash::GENESIS {
        return Err(AuditError::InvalidGenesis);
    }
    if signer.key_id().trim().is_empty() || signer.key_id().len() > 512 {
        return Err(AuditError::InvalidSigningKey);
    }
    let entry_hash = calculate_hash(sequence, &expected_previous_hash, &record);
    let signature = signer.sign(&entry_hash)?;
    if signature.is_empty() {
        return Err(AuditError::EmptySignature);
    }
    Ok(AuditEntry {
        sequence,
        previous_hash: expected_previous_hash,
        record,
        entry_hash,
        signing_key_id: signer.key_id().to_owned(),
        signature,
    })
}

/// Verifies sequence, links, deterministic hashes, and historical signatures.
pub fn verify_chain(
    entries: &[AuditEntry],
    verifier: &dyn AuditSignatureVerifier,
) -> Result<(), AuditError> {
    let mut expected_sequence = 1_u64;
    let mut expected_previous = AuditHash::GENESIS;
    for entry in entries {
        if entry.sequence != expected_sequence {
            return Err(AuditError::SequenceGap {
                expected: expected_sequence,
                actual: entry.sequence,
            });
        }
        if entry.previous_hash != expected_previous {
            return Err(AuditError::BrokenLink {
                sequence: entry.sequence,
            });
        }
        entry.record.validate()?;
        let calculated = calculate_hash(entry.sequence, &entry.previous_hash, &entry.record);
        if calculated != entry.entry_hash {
            return Err(AuditError::ContentTampered {
                sequence: entry.sequence,
            });
        }
        if !verifier.verify(&entry.signing_key_id, &entry.entry_hash, &entry.signature)? {
            return Err(AuditError::InvalidSignature {
                sequence: entry.sequence,
            });
        }
        expected_previous = entry.entry_hash;
        expected_sequence = expected_sequence
            .checked_add(1)
            .ok_or(AuditError::SequenceExhausted)?;
    }
    Ok(())
}

fn calculate_hash(sequence: u64, previous_hash: &AuditHash, record: &AuditRecord) -> AuditHash {
    let mut bytes = Vec::with_capacity(512);
    bytes.extend_from_slice(b"wildfire.audit.v1\0");
    bytes.extend_from_slice(&sequence.to_be_bytes());
    bytes.extend_from_slice(previous_hash.as_bytes());
    bytes.push(record.stream.canonical_byte());
    encode_text(&mut bytes, &record.tenant_id);
    encode_text(&mut bytes, &record.actor_id);
    encode_text(&mut bytes, &record.action);
    encode_text(&mut bytes, &record.resource);
    encode_text(&mut bytes, &record.reason);
    encode_text(
        &mut bytes,
        &record
            .occurred_at
            .to_rfc3339_opts(SecondsFormat::Nanos, true),
    );
    bytes.push(match record.clock_quality {
        AuditClockQuality::Synchronized => 1,
        AuditClockQuality::Degraded => 2,
        AuditClockQuality::Unknown => 3,
    });
    encode_text(&mut bytes, &record.correlation_id);
    match &record.causation_id {
        Some(value) => {
            bytes.push(1);
            encode_text(&mut bytes, value);
        }
        None => bytes.push(0),
    }
    encode_text(&mut bytes, &record.policy_version);
    bytes.extend_from_slice(record.payload_digest.as_bytes());
    AuditHash(Sha256::digest(bytes).into())
}

fn encode_text(output: &mut Vec<u8>, value: &str) {
    let length = u64::try_from(value.len()).unwrap_or(u64::MAX);
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(value.as_bytes());
}

/// Expected ledger head used as an optimistic append precondition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LedgerHead {
    /// Last committed sequence, or zero for empty stream.
    pub sequence: u64,
    /// Last committed hash, or genesis for empty stream.
    pub hash: AuditHash,
}

/// Asynchronous durable append result.
pub type AuditFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, AuditError>> + Send + 'a>>;

/// Independent durable audit path. Implementations use append-only/WORM storage.
pub trait DurableAuditPort: Send + Sync {
    /// Atomically appends only if the durable stream still has `expected_head`.
    fn append<'a>(
        &'a self,
        expected_head: LedgerHead,
        entry: &'a AuditEntry,
    ) -> AuditFuture<'a, LedgerHead>;
}

/// Explicitly distinct durable safety and security sinks.
pub struct IndependentAuditPorts<'a> {
    /// Append-only safety ledger, isolated from operational telemetry.
    pub safety: &'a dyn DurableAuditPort,
    /// Append-only security ledger, isolated from the safety ledger.
    pub security: &'a dyn DurableAuditPort,
}

impl IndependentAuditPorts<'_> {
    /// Selects the independently durable sink for a mandatory audit stream.
    pub fn for_stream(&self, stream: AuditStream) -> Result<&dyn DurableAuditPort, AuditError> {
        match stream {
            AuditStream::Safety => Ok(self.safety),
            AuditStream::Security => Ok(self.security),
            AuditStream::Governance => Err(AuditError::UnsupportedIndependentStream),
        }
    }
}

/// Routes a mandatory safety/security entry to its independent durable path.
pub async fn append_independent(
    ports: &IndependentAuditPorts<'_>,
    telemetry: &dyn AuditTelemetryPort,
    expected_head: LedgerHead,
    entry: &AuditEntry,
) -> Result<LedgerHead, AuditError> {
    let durable = ports.for_stream(entry.record.stream)?;
    append_durable_then_telemetry(durable, telemetry, expected_head, entry).await
}

/// Best-effort telemetry path. It is deliberately synchronous and non-blocking.
pub trait AuditTelemetryPort: Send + Sync {
    /// Attempts enqueue into a bounded buffer; failure means dropped telemetry, not lost audit.
    fn try_emit(&self, entry: &AuditEntry) -> Result<(), TelemetryError>;
}

/// Persists audit first, then independently attempts best-effort telemetry.
pub async fn append_durable_then_telemetry(
    durable: &dyn DurableAuditPort,
    telemetry: &dyn AuditTelemetryPort,
    expected_head: LedgerHead,
    entry: &AuditEntry,
) -> Result<LedgerHead, AuditError> {
    let committed = durable.append(expected_head, entry).await?;
    let _telemetry_result = telemetry.try_emit(entry);
    Ok(committed)
}

/// Telemetry enqueue failure, intentionally excluded from durable audit errors.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum TelemetryError {
    /// Bounded telemetry queue is full.
    #[error("telemetry queue is full")]
    Full,
    /// Telemetry exporter is unavailable.
    #[error("telemetry exporter is unavailable")]
    Unavailable,
}

/// Audit construction, verification, or durability failure.
#[derive(Debug, Error)]
pub enum AuditError {
    /// Required content is blank or exceeds its bound.
    #[error("audit record contains an invalid required value")]
    InvalidRecord,
    /// Sequence cannot advance without wrapping.
    #[error("audit sequence exhausted")]
    SequenceExhausted,
    /// Genesis entry referenced a non-genesis predecessor.
    #[error("first audit entry must reference genesis")]
    InvalidGenesis,
    /// Signing key ID is missing or too long.
    #[error("audit signing key identifier is invalid")]
    InvalidSigningKey,
    /// Signer returned an empty signature.
    #[error("audit signature is empty")]
    EmptySignature,
    /// Ledger contains a missing, duplicate, or reordered sequence.
    #[error("audit sequence gap: expected {expected}, found {actual}")]
    SequenceGap {
        /// Next required sequence number.
        expected: u64,
        /// Sequence number encountered.
        actual: u64,
    },
    /// Previous-hash link does not match.
    #[error("audit chain link is broken at sequence {sequence}")]
    BrokenLink {
        /// Sequence whose predecessor digest mismatched.
        sequence: u64,
    },
    /// Recomputed content digest differs.
    #[error("audit content was modified at sequence {sequence}")]
    ContentTampered {
        /// Sequence whose canonical content changed.
        sequence: u64,
    },
    /// Historical signature failed verification.
    #[error("audit signature is invalid at sequence {sequence}")]
    InvalidSignature {
        /// Sequence whose detached signature was invalid.
        sequence: u64,
    },
    /// Durable adapter rejected a stale optimistic head.
    #[error("audit ledger head changed concurrently")]
    ConcurrentAppend,
    /// Signing, verification, or durable storage adapter failed closed.
    #[error("audit provider failed: {0}")]
    Provider(String),
    /// Governance records require an explicitly selected governed durable store.
    #[error("stream is not safety or security")]
    UnsupportedIndependentStream,
}
