#![forbid(unsafe_code)]
//! Tenant-isolated immutable object manifests and orphan reconciliation.
//!
//! This crate implements the object-storage half of ADR-019. It deliberately
//! contains no database, cloud SDK, or runtime dependency: adapters implement
//! [`ObjectStorage`] and persist [`ObjectManifest`] in the owning context's
//! transaction.

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use shared_kernel::{ContentDigest, DataClassification, TenantId};
use std::fmt;
use thiserror::Error;

const MAX_IDENTIFIER_BYTES: usize = 128;
const MAX_STORAGE_KEY_BYTES: usize = 1024;
const MAX_MEDIA_TYPE_BYTES: usize = 127;
const MAX_POLICY_TEXT_BYTES: usize = 256;

/// Stable identifier for an immutable object manifest.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ManifestId(String);

impl ManifestId {
    /// Validates a caller-supplied manifest identifier.
    pub fn parse(value: impl Into<String>) -> Result<Self, ManifestError> {
        let value = value.into();
        validate_token(&value, MAX_IDENTIFIER_BYTES)
            .map_err(|()| ManifestError::InvalidManifestId)?;
        Ok(Self(value))
    }

    /// Returns the stable identifier text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Validated tenant-relative immutable storage key.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ObjectKey(String);

impl ObjectKey {
    /// Validates a key. Absolute paths, traversal segments, and control bytes are rejected.
    pub fn parse(value: impl Into<String>) -> Result<Self, ManifestError> {
        let value = value.into();
        let invalid = value.is_empty()
            || value.len() > MAX_STORAGE_KEY_BYTES
            || value.starts_with('/')
            || value.ends_with('/')
            || value
                .split('/')
                .any(|part| part.is_empty() || matches!(part, "." | ".."))
            || value
                .bytes()
                .any(|byte| byte.is_ascii_control() || byte == b'\\');
        if invalid {
            return Err(ManifestError::InvalidObjectKey);
        }
        Ok(Self(value))
    }

    /// Returns the tenant-relative key.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Validated media type without parameters.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MediaType(String);

impl MediaType {
    /// Validates an IANA-style `type/subtype` value.
    pub fn parse(value: impl Into<String>) -> Result<Self, ManifestError> {
        let value = value.into();
        let valid_token = |part: &str| {
            !part.is_empty()
                && part.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric()
                        || matches!(
                            byte,
                            b'!' | b'#' | b'$' | b'&' | b'^' | b'_' | b'.' | b'+' | b'-'
                        )
                })
        };
        let valid = value.len() <= MAX_MEDIA_TYPE_BYTES
            && value.split_once('/').is_some_and(|(kind, subtype)| {
                !subtype.contains('/') && valid_token(kind) && valid_token(subtype)
            });
        if !valid {
            return Err(ManifestError::InvalidMediaType);
        }
        Ok(Self(value))
    }

    /// Returns the canonical media type.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Storage lifecycle class assigned at object creation (ADR-028).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RetentionClass {
    /// Short-lived derived data that may be deleted after its declared deadline.
    Temporary,
    /// Operational records retained according to the owning context's policy.
    Operational,
    /// Immutable safety, audit, or regulatory evidence.
    Evidence,
    /// Records retained indefinitely unless a superseding approved policy exists.
    Permanent,
}

/// Explicit retention and deletion policy snapshot bound to an object.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionPolicy {
    class: RetentionClass,
    policy_id: String,
    retain_until: Option<DateTime<Utc>>,
    deletion_policy: String,
}

impl RetentionPolicy {
    /// Creates a validated retention policy snapshot.
    pub fn new(
        class: RetentionClass,
        policy_id: impl Into<String>,
        retain_until: Option<DateTime<Utc>>,
        deletion_policy: impl Into<String>,
    ) -> Result<Self, ManifestError> {
        let policy_id = policy_id.into();
        let deletion_policy = deletion_policy.into();
        validate_token(&policy_id, MAX_IDENTIFIER_BYTES)
            .map_err(|()| ManifestError::InvalidRetentionPolicy)?;
        validate_text(&deletion_policy, MAX_POLICY_TEXT_BYTES)
            .map_err(|()| ManifestError::InvalidRetentionPolicy)?;
        if matches!(class, RetentionClass::Temporary) && retain_until.is_none() {
            return Err(ManifestError::InvalidRetentionPolicy);
        }
        Ok(Self {
            class,
            policy_id,
            retain_until,
            deletion_policy,
        })
    }

    /// Returns the lifecycle class.
    #[must_use]
    pub const fn class(&self) -> RetentionClass {
        self.class
    }

    /// Returns the policy snapshot identifier.
    #[must_use]
    pub fn policy_id(&self) -> &str {
        &self.policy_id
    }

    /// Returns the earliest policy deadline after which deletion may be considered.
    #[must_use]
    pub const fn retain_until(&self) -> Option<DateTime<Utc>> {
        self.retain_until
    }

    /// Returns the declared deletion behavior.
    #[must_use]
    pub fn deletion_policy(&self) -> &str {
        &self.deletion_policy
    }
}

/// Attributable legal hold that prevents object deletion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegalHold {
    hold_id: String,
    authority: String,
    reason: String,
    placed_at: DateTime<Utc>,
}

impl LegalHold {
    /// Creates a validated legal-hold record.
    pub fn new(
        hold_id: impl Into<String>,
        authority: impl Into<String>,
        reason: impl Into<String>,
        placed_at: DateTime<Utc>,
    ) -> Result<Self, ManifestError> {
        let hold_id = hold_id.into();
        let authority = authority.into();
        let reason = reason.into();
        validate_token(&hold_id, MAX_IDENTIFIER_BYTES)
            .map_err(|()| ManifestError::InvalidLegalHold)?;
        validate_text(&authority, MAX_POLICY_TEXT_BYTES)
            .map_err(|()| ManifestError::InvalidLegalHold)?;
        validate_text(&reason, MAX_POLICY_TEXT_BYTES)
            .map_err(|()| ManifestError::InvalidLegalHold)?;
        Ok(Self {
            hold_id,
            authority,
            reason,
            placed_at,
        })
    }

    /// Returns the legal-hold identifier.
    #[must_use]
    pub fn hold_id(&self) -> &str {
        &self.hold_id
    }

    /// Returns the authority that placed the hold.
    #[must_use]
    pub fn authority(&self) -> &str {
        &self.authority
    }

    /// Returns the hold reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Returns when the hold was placed.
    #[must_use]
    pub const fn placed_at(&self) -> DateTime<Utc> {
        self.placed_at
    }
}

/// Immutable metadata record for one content-addressed object.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectManifest {
    id: ManifestId,
    tenant_id: TenantId,
    key: ObjectKey,
    digest: ContentDigest,
    size_bytes: u64,
    media_type: MediaType,
    classification: DataClassification,
    owner: String,
    residency: String,
    retention: RetentionPolicy,
    legal_hold: Option<LegalHold>,
    created_at: DateTime<Utc>,
}

impl ObjectManifest {
    /// Creates an immutable manifest after validating policy metadata and time relationships.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: ManifestId,
        tenant_id: TenantId,
        key: ObjectKey,
        digest: ContentDigest,
        size_bytes: u64,
        media_type: MediaType,
        classification: DataClassification,
        owner: impl Into<String>,
        residency: impl Into<String>,
        retention: RetentionPolicy,
        legal_hold: Option<LegalHold>,
        created_at: DateTime<Utc>,
    ) -> Result<Self, ManifestError> {
        let owner = owner.into();
        let residency = residency.into();
        validate_text(&owner, MAX_POLICY_TEXT_BYTES).map_err(|()| ManifestError::InvalidOwner)?;
        validate_token(&residency, MAX_IDENTIFIER_BYTES)
            .map_err(|()| ManifestError::InvalidResidency)?;
        if retention
            .retain_until()
            .is_some_and(|deadline| deadline < created_at)
            || legal_hold
                .as_ref()
                .is_some_and(|hold| hold.placed_at() < created_at)
        {
            return Err(ManifestError::InvalidPolicyTime);
        }
        Ok(Self {
            id,
            tenant_id,
            key,
            digest,
            size_bytes,
            media_type,
            classification,
            owner,
            residency,
            retention,
            legal_hold,
            created_at,
        })
    }

    /// Returns the manifest identifier.
    #[must_use]
    pub const fn id(&self) -> &ManifestId {
        &self.id
    }
    /// Returns the owning tenant.
    #[must_use]
    pub const fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }
    /// Returns the tenant-relative storage key.
    #[must_use]
    pub const fn key(&self) -> &ObjectKey {
        &self.key
    }
    /// Returns the expected SHA-256 digest.
    #[must_use]
    pub const fn digest(&self) -> ContentDigest {
        self.digest
    }
    /// Returns the exact expected byte count.
    #[must_use]
    pub const fn size_bytes(&self) -> u64 {
        self.size_bytes
    }
    /// Returns the media type.
    #[must_use]
    pub const fn media_type(&self) -> &MediaType {
        &self.media_type
    }
    /// Returns the assigned classification.
    #[must_use]
    pub const fn classification(&self) -> DataClassification {
        self.classification
    }
    /// Returns the accountable owner.
    #[must_use]
    pub fn owner(&self) -> &str {
        &self.owner
    }
    /// Returns the residency policy identifier.
    #[must_use]
    pub fn residency(&self) -> &str {
        &self.residency
    }
    /// Returns the retention policy snapshot.
    #[must_use]
    pub const fn retention(&self) -> &RetentionPolicy {
        &self.retention
    }
    /// Returns the optional legal hold.
    #[must_use]
    pub const fn legal_hold(&self) -> Option<&LegalHold> {
        self.legal_hold.as_ref()
    }
    /// Returns creation time.
    #[must_use]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Reports whether policy permits deletion at `now`.
    #[must_use]
    pub fn deletion_allowed_at(&self, now: DateTime<Utc>) -> bool {
        self.legal_hold.is_none()
            && !matches!(
                self.retention.class(),
                RetentionClass::Evidence | RetentionClass::Permanent
            )
            && self
                .retention
                .retain_until()
                .is_none_or(|deadline| now >= deadline)
    }
}

/// Metadata independently reported by an object-store adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StoredObjectMetadata {
    /// Tenant boundary reported by the adapter, never inferred from a key.
    pub tenant_id: TenantId,
    /// Server-computed or independently verified content digest.
    pub digest: ContentDigest,
    /// Stored content length.
    pub size_bytes: u64,
    /// Whether storage-level retention or legal hold prevents deletion.
    pub deletion_protected: bool,
}

/// Port implemented by versioned, encrypted, tenant-isolated object storage adapters.
pub trait ObjectStorage {
    /// Adapter-specific failure, mapped at the application boundary.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Returns independently obtained metadata for a tenant-scoped key.
    fn metadata(
        &self,
        tenant_id: TenantId,
        key: &ObjectKey,
    ) -> Result<Option<StoredObjectMetadata>, Self::Error>;

    /// Deletes a known tenant-scoped orphan using an adapter precondition.
    fn delete_orphan(
        &self,
        tenant_id: TenantId,
        key: &ObjectKey,
        expected_digest: ContentDigest,
    ) -> Result<(), Self::Error>;
}

/// Verifies a manifest against metadata independently reported by storage.
pub fn verify_stored_object(
    manifest: &ObjectManifest,
    actual: StoredObjectMetadata,
) -> Result<(), VerificationError> {
    if actual.tenant_id != manifest.tenant_id() {
        return Err(VerificationError::TenantMismatch);
    }
    if actual.digest != manifest.digest() {
        return Err(VerificationError::DigestMismatch);
    }
    if actual.size_bytes != manifest.size_bytes() {
        return Err(VerificationError::SizeMismatch);
    }
    Ok(())
}

/// Computes SHA-256 locally and verifies exact bytes against a manifest.
pub fn verify_bytes(manifest: &ObjectManifest, bytes: &[u8]) -> Result<(), VerificationError> {
    let size_bytes = u64::try_from(bytes.len()).map_err(|_| VerificationError::SizeMismatch)?;
    let digest = ContentDigest::from_sha256_bytes(Sha256::digest(bytes).into());
    verify_stored_object(
        manifest,
        StoredObjectMetadata {
            tenant_id: manifest.tenant_id(),
            digest,
            size_bytes,
            deletion_protected: false,
        },
    )
}

/// Verification failures safe for deterministic adapter policy.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum VerificationError {
    /// Storage metadata crossed a tenant boundary.
    #[error("stored object tenant does not match manifest tenant")]
    TenantMismatch,
    /// Stored bytes do not match the immutable manifest digest.
    #[error("stored object digest does not match manifest")]
    DigestMismatch,
    /// Stored byte length does not match the immutable manifest.
    #[error("stored object size does not match manifest")]
    SizeMismatch,
}

/// Durable orphan-reconciliation state. Transitions are monotonic and explicit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OrphanState {
    /// First unreferenced observation; no deletion is permitted yet.
    Candidate {
        /// First observation time used to enforce the grace period.
        first_observed_at: DateTime<Utc>,
        /// Number of independent reconciliation observations.
        observations: u32,
    },
    /// Repeated observations and grace period make deletion eligible.
    Eligible {
        /// Time at which deletion first became eligible.
        eligible_at: DateTime<Utc>,
        /// Number of observations supporting the decision.
        observations: u32,
    },
    /// Object became referenced; the orphan workflow is terminal.
    Reconciled,
    /// Object was deleted using tenant and digest preconditions.
    Deleted {
        /// Confirmed deletion time.
        deleted_at: DateTime<Utc>,
    },
    /// Storage protection or inconsistent observations require operator review.
    Quarantined {
        /// Stable non-sensitive reason.
        reason: OrphanQuarantineReason,
    },
}

/// Stable reasons an orphan cannot be automatically deleted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrphanQuarantineReason {
    /// The object store reports retention or legal-hold protection.
    StorageProtected,
    /// Observation time moved backward, making grace-period evidence unreliable.
    ClockRegression,
    /// Object identity changed while reconciliation was in progress.
    ObjectChanged,
}

/// Immutable identity and current durable state for a potential orphan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrphanRecord {
    tenant_id: TenantId,
    key: ObjectKey,
    digest: ContentDigest,
    state: OrphanState,
}

impl OrphanRecord {
    /// Starts reconciliation after the first independently observed unreferenced object.
    #[must_use]
    pub const fn candidate(
        tenant_id: TenantId,
        key: ObjectKey,
        digest: ContentDigest,
        observed_at: DateTime<Utc>,
    ) -> Self {
        Self {
            tenant_id,
            key,
            digest,
            state: OrphanState::Candidate {
                first_observed_at: observed_at,
                observations: 1,
            },
        }
    }

    /// Applies another reconciliation observation.
    #[must_use]
    pub fn observe(
        self,
        observed: Option<StoredObjectMetadata>,
        manifest_exists: bool,
        observed_at: DateTime<Utc>,
        grace_period: chrono::Duration,
    ) -> Self {
        if manifest_exists {
            return Self {
                state: OrphanState::Reconciled,
                ..self
            };
        }
        let Some(actual) = observed else {
            return Self {
                state: OrphanState::Reconciled,
                ..self
            };
        };
        if actual.tenant_id != self.tenant_id || actual.digest != self.digest {
            return Self {
                state: OrphanState::Quarantined {
                    reason: OrphanQuarantineReason::ObjectChanged,
                },
                ..self
            };
        }
        if actual.deletion_protected {
            return Self {
                state: OrphanState::Quarantined {
                    reason: OrphanQuarantineReason::StorageProtected,
                },
                ..self
            };
        }
        match self.state {
            OrphanState::Candidate {
                first_observed_at,
                observations,
            } => {
                if observed_at < first_observed_at {
                    Self {
                        state: OrphanState::Quarantined {
                            reason: OrphanQuarantineReason::ClockRegression,
                        },
                        ..self
                    }
                } else if observations >= 1 && observed_at - first_observed_at >= grace_period {
                    Self {
                        state: OrphanState::Eligible {
                            eligible_at: observed_at,
                            observations: observations.saturating_add(1),
                        },
                        ..self
                    }
                } else {
                    Self {
                        state: OrphanState::Candidate {
                            first_observed_at,
                            observations: observations.saturating_add(1),
                        },
                        ..self
                    }
                }
            }
            _ => self,
        }
    }

    /// Marks an eligible record deleted after the adapter confirms preconditioned deletion.
    pub fn mark_deleted(self, deleted_at: DateTime<Utc>) -> Result<Self, ReconciliationError> {
        if !matches!(self.state, OrphanState::Eligible { .. }) {
            return Err(ReconciliationError::NotEligible);
        }
        Ok(Self {
            state: OrphanState::Deleted { deleted_at },
            ..self
        })
    }

    /// Returns the tenant used for every adapter operation.
    #[must_use]
    pub const fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }
    /// Returns the tenant-relative key.
    #[must_use]
    pub const fn key(&self) -> &ObjectKey {
        &self.key
    }
    /// Returns the content precondition used for deletion.
    #[must_use]
    pub const fn digest(&self) -> ContentDigest {
        self.digest
    }
    /// Returns current reconciliation state.
    #[must_use]
    pub const fn state(&self) -> &OrphanState {
        &self.state
    }
}

/// Manifest boundary validation failures.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ManifestError {
    /// Manifest identifier is empty, too long, or malformed.
    #[error("invalid manifest identifier")]
    InvalidManifestId,
    /// Object key is unsafe or malformed.
    #[error("invalid tenant-relative object key")]
    InvalidObjectKey,
    /// Media type is malformed.
    #[error("invalid media type")]
    InvalidMediaType,
    /// Retention metadata is incomplete or inconsistent.
    #[error("invalid retention policy")]
    InvalidRetentionPolicy,
    /// Legal-hold metadata is incomplete.
    #[error("invalid legal hold")]
    InvalidLegalHold,
    /// Accountable owner is missing or invalid.
    #[error("invalid object owner")]
    InvalidOwner,
    /// Residency policy identifier is missing or invalid.
    #[error("invalid residency policy")]
    InvalidResidency,
    /// Policy timestamps precede object creation.
    #[error("policy timestamp precedes object creation")]
    InvalidPolicyTime,
}

/// Invalid orphan state transition.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ReconciliationError {
    /// Deletion confirmation is legal only from the eligible state.
    #[error("orphan is not eligible for deletion")]
    NotEligible,
}

fn validate_token(value: &str, maximum: usize) -> Result<(), ()> {
    if value.is_empty()
        || value.len() > maximum
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        return Err(());
    }
    Ok(())
}

fn validate_text(value: &str, maximum: usize) -> Result<(), ()> {
    if value.trim().is_empty() || value.len() > maximum || value.chars().any(char::is_control) {
        return Err(());
    }
    Ok(())
}

impl fmt::Display for ObjectKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;
    use std::{collections::HashMap, io};

    const TENANT_A: &str = "018f3f22-40d2-7df0-8000-000000000001";
    const TENANT_B: &str = "018f3f22-40d2-7df0-8000-000000000002";

    fn tenant(value: &str) -> TenantId {
        TenantId::parse(value).unwrap_or_else(|error| unreachable!("valid test tenant: {error}"))
    }
    fn at(seconds: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(seconds, 0).unwrap_or_else(|| unreachable!("valid test time"))
    }

    fn manifest(
        bytes: &[u8],
        tenant_id: TenantId,
        legal_hold: Option<LegalHold>,
    ) -> Result<ObjectManifest, ManifestError> {
        let digest = ContentDigest::from_sha256_bytes(Sha256::digest(bytes).into());
        ObjectManifest::new(
            ManifestId::parse("manifest-1")?,
            tenant_id,
            ObjectKey::parse("evidence/flight-1.bin")?,
            digest,
            u64::try_from(bytes.len()).map_err(|_| ManifestError::InvalidObjectKey)?,
            MediaType::parse("application/octet-stream")?,
            DataClassification::Restricted,
            "safety-assurance",
            "ca-central-1",
            RetentionPolicy::new(
                RetentionClass::Operational,
                "RET-7Y",
                Some(at(10_000)),
                "delete-after-review",
            )?,
            legal_hold,
            at(100),
        )
    }

    #[test]
    fn detects_content_digest_mismatch() -> Result<(), Box<dyn std::error::Error>> {
        let manifest = manifest(b"expected", tenant(TENANT_A), None)?;
        assert_eq!(
            verify_bytes(&manifest, b"tampered"),
            Err(VerificationError::DigestMismatch)
        );
        Ok(())
    }

    #[test]
    fn rejects_cross_tenant_storage_metadata_even_when_content_matches()
    -> Result<(), Box<dyn std::error::Error>> {
        let manifest = manifest(b"same", tenant(TENANT_A), None)?;
        let actual = StoredObjectMetadata {
            tenant_id: tenant(TENANT_B),
            digest: manifest.digest(),
            size_bytes: manifest.size_bytes(),
            deletion_protected: false,
        };
        assert_eq!(
            verify_stored_object(&manifest, actual),
            Err(VerificationError::TenantMismatch)
        );
        Ok(())
    }

    #[test]
    fn legal_hold_and_evidence_retention_prevent_deletion() -> Result<(), Box<dyn std::error::Error>>
    {
        let hold = LegalHold::new(
            "HOLD-1",
            "court-order-7",
            "preserve occurrence evidence",
            at(200),
        )?;
        let held_manifest = manifest(b"held", tenant(TENANT_A), Some(hold))?;
        assert!(!held_manifest.deletion_allowed_at(at(20_000)));
        let evidence = ObjectManifest::new(
            ManifestId::parse("manifest-evidence")?,
            tenant(TENANT_A),
            ObjectKey::parse("evidence/permanent.bin")?,
            ContentDigest::from_sha256_bytes([0; 32]),
            0,
            MediaType::parse("application/octet-stream")?,
            DataClassification::Restricted,
            "safety-assurance",
            "ca-central-1",
            RetentionPolicy::new(RetentionClass::Evidence, "RET-EVIDENCE", None, "immutable")?,
            None,
            at(100),
        )?;
        assert!(!evidence.deletion_allowed_at(at(20_000)));
        Ok(())
    }

    #[test]
    fn orphan_requires_repeated_observation_and_grace_period()
    -> Result<(), Box<dyn std::error::Error>> {
        let tenant_id = tenant(TENANT_A);
        let digest = ContentDigest::from_sha256_bytes([9; 32]);
        let metadata = StoredObjectMetadata {
            tenant_id,
            digest,
            size_bytes: 4,
            deletion_protected: false,
        };
        let candidate = OrphanRecord::candidate(
            tenant_id,
            ObjectKey::parse("orphan/object.bin")?,
            digest,
            at(100),
        );
        let still_candidate =
            candidate.observe(Some(metadata), false, at(120), TimeDelta::seconds(60));
        assert!(matches!(
            still_candidate.state(),
            OrphanState::Candidate {
                observations: 2,
                ..
            }
        ));
        let eligible =
            still_candidate.observe(Some(metadata), false, at(161), TimeDelta::seconds(60));
        assert!(matches!(
            eligible.state(),
            OrphanState::Eligible {
                observations: 3,
                ..
            }
        ));
        assert!(matches!(
            eligible.mark_deleted(at(162))?.state(),
            OrphanState::Deleted { .. }
        ));
        Ok(())
    }

    #[test]
    fn referenced_or_protected_objects_are_never_deletion_eligible()
    -> Result<(), Box<dyn std::error::Error>> {
        let tenant_id = tenant(TENANT_A);
        let digest = ContentDigest::from_sha256_bytes([4; 32]);
        let candidate =
            OrphanRecord::candidate(tenant_id, ObjectKey::parse("object.bin")?, digest, at(100));
        let referenced = candidate
            .clone()
            .observe(None, true, at(200), TimeDelta::seconds(60));
        assert_eq!(referenced.state(), &OrphanState::Reconciled);
        let protected = candidate.observe(
            Some(StoredObjectMetadata {
                tenant_id,
                digest,
                size_bytes: 1,
                deletion_protected: true,
            }),
            false,
            at(200),
            TimeDelta::seconds(60),
        );
        assert_eq!(
            protected.state(),
            &OrphanState::Quarantined {
                reason: OrphanQuarantineReason::StorageProtected
            }
        );
        Ok(())
    }

    #[derive(Default)]
    struct MemoryStorage {
        objects: HashMap<(TenantId, String), StoredObjectMetadata>,
    }

    impl ObjectStorage for MemoryStorage {
        type Error = io::Error;
        fn metadata(
            &self,
            tenant_id: TenantId,
            key: &ObjectKey,
        ) -> Result<Option<StoredObjectMetadata>, Self::Error> {
            Ok(self
                .objects
                .get(&(tenant_id, key.as_str().to_owned()))
                .copied())
        }
        fn delete_orphan(
            &self,
            _tenant_id: TenantId,
            _key: &ObjectKey,
            _expected_digest: ContentDigest,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[test]
    fn storage_port_requires_tenant_for_lookup() -> Result<(), Box<dyn std::error::Error>> {
        let key = ObjectKey::parse("shared-name.bin")?;
        let digest = ContentDigest::from_sha256_bytes([1; 32]);
        let mut storage = MemoryStorage::default();
        storage.objects.insert(
            (tenant(TENANT_A), key.as_str().to_owned()),
            StoredObjectMetadata {
                tenant_id: tenant(TENANT_A),
                digest,
                size_bytes: 1,
                deletion_protected: false,
            },
        );
        assert!(storage.metadata(tenant(TENANT_A), &key)?.is_some());
        assert!(storage.metadata(tenant(TENANT_B), &key)?.is_none());
        Ok(())
    }
}
