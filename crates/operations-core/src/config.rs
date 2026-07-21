//! Signed typed configuration and narrow-only feature isolation (ADR-042).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shared_kernel::{ContentDigest, DataClassification, SemanticVersion};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// Declares whether applying a configuration requires process restart.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RestartSemantics {
    /// May be atomically activated at runtime.
    HotReload,
    /// Requires a controlled process restart.
    ProcessRestart,
    /// Requires a coordinated vehicle/station restart and safety review.
    ControlledAssetRestart,
}

/// Replaceable signature verification port backed by approved PKI/KMS adapters.
pub trait ConfigurationSignatureVerifier {
    /// Verifies the detached signature over exact canonical artifact bytes.
    fn verify(&self, key_id: &str, canonical: &[u8], signature: &[u8]) -> Result<(), ConfigError>;
}

/// A typed configuration payload owns its validation and safe default.
pub trait TypedConfiguration: Clone + Eq + Serialize {
    /// Returns the fail-closed safe default.
    fn safe_default() -> Self;
    /// Validates all semantic and numeric bounds.
    fn validate(&self) -> Result<(), ConfigError>;
}

/// Signed, schema-versioned configuration artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedConfiguration<T> {
    /// Configuration schema version.
    pub schema_version: SemanticVersion,
    /// Accountable configuration owner.
    pub owner: String,
    /// Rollout scope fingerprint.
    pub rollout_scope: String,
    /// Data classification.
    pub classification: DataClassification,
    /// Restart behavior.
    pub restart: RestartSemantics,
    /// Exclusive expiry for temporary configuration.
    pub expires_at: Option<DateTime<Utc>>,
    /// Typed payload.
    pub payload: T,
    /// Digest computed over canonical payload bytes.
    pub payload_digest: ContentDigest,
    /// Signing key identifier.
    pub signing_key_id: String,
    /// Canonical artifact bytes excluding signature.
    pub canonical_bytes: Vec<u8>,
    /// Detached signature.
    pub signature: Vec<u8>,
}

impl<T: TypedConfiguration> SignedConfiguration<T> {
    /// Resolves a verified payload, returning its declared safe default on any failure.
    ///
    /// The rejection is returned alongside the fallback so callers can audit it without
    /// accidentally applying unverified content.
    pub fn resolve_or_safe_default(
        &self,
        now: DateTime<Utc>,
        independently_computed_digest: ContentDigest,
        verifier: &dyn ConfigurationSignatureVerifier,
    ) -> (T, Option<ConfigError>) {
        match self.verify(now, independently_computed_digest, verifier) {
            Ok(()) => (self.payload.clone(), None),
            Err(error) => (T::safe_default(), Some(error)),
        }
    }

    /// Produces the deterministic unsigned artifact representation that must be signed.
    pub fn canonical_unsigned_bytes(&self) -> Result<Vec<u8>, ConfigError> {
        #[derive(Serialize)]
        struct Unsigned<'a, T> {
            schema_version: &'a SemanticVersion,
            owner: &'a str,
            rollout_scope: &'a str,
            classification: DataClassification,
            restart: RestartSemantics,
            expires_at: Option<DateTime<Utc>>,
            payload: &'a T,
            payload_digest: ContentDigest,
            signing_key_id: &'a str,
        }
        serde_json::to_vec(&Unsigned {
            schema_version: &self.schema_version,
            owner: &self.owner,
            rollout_scope: &self.rollout_scope,
            classification: self.classification,
            restart: self.restart,
            expires_at: self.expires_at,
            payload: &self.payload,
            payload_digest: self.payload_digest,
            signing_key_id: &self.signing_key_id,
        })
        .map_err(|_| ConfigError::CanonicalizationFailed)
    }

    /// Verifies typed bounds, digest, expiry, metadata, and detached signature.
    pub fn verify(
        &self,
        now: DateTime<Utc>,
        independently_computed_digest: ContentDigest,
        verifier: &dyn ConfigurationSignatureVerifier,
    ) -> Result<(), ConfigError> {
        if self.owner.trim().is_empty()
            || self.rollout_scope.trim().is_empty()
            || self.signing_key_id.trim().is_empty()
            || self.canonical_bytes.is_empty()
            || self.signature.is_empty()
        {
            return Err(ConfigError::InvalidMetadata);
        }
        self.payload.validate()?;
        let payload_bytes =
            serde_json::to_vec(&self.payload).map_err(|_| ConfigError::CanonicalizationFailed)?;
        let computed_payload_digest =
            ContentDigest::from_sha256_bytes(Sha256::digest(payload_bytes).into());
        if self.payload_digest != independently_computed_digest
            || self.payload_digest != computed_payload_digest
        {
            return Err(ConfigError::DigestMismatch);
        }
        if self.expires_at.is_some_and(|expiry| now >= expiry) {
            return Err(ConfigError::Expired);
        }
        if self.canonical_bytes != self.canonical_unsigned_bytes()? {
            return Err(ConfigError::CanonicalBytesMismatch);
        }
        verifier.verify(&self.signing_key_id, &self.canonical_bytes, &self.signature)
    }
}

/// Only effects that reduce exposure are expressible as runtime flags.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum FeatureEffect {
    /// Disable or isolate a capability.
    Isolate,
    /// Narrow an already approved rollout population or behavior.
    Narrow,
}

/// Temporary, expiring feature control.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FeatureFlag {
    /// Stable flag key.
    pub key: String,
    /// Whether the narrowing/isolation effect is active.
    pub active: bool,
    /// Explicit non-expanding effect.
    pub effect: FeatureEffect,
    /// Exclusive mandatory expiry.
    pub expires_at: DateTime<Utc>,
    /// Approved rollout scopes to which this restriction applies.
    pub scopes: BTreeSet<String>,
}

/// Immutable feature flag inventory.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FeatureFlagSet(BTreeMap<String, FeatureFlag>);

impl FeatureFlagSet {
    /// Validates keys, expiry, scope, and duplicate-free inventory.
    pub fn new(
        flags: impl IntoIterator<Item = FeatureFlag>,
        now: DateTime<Utc>,
    ) -> Result<Self, ConfigError> {
        let mut indexed = BTreeMap::new();
        for flag in flags {
            if flag.key.trim().is_empty() || flag.scopes.is_empty() || now >= flag.expires_at {
                return Err(ConfigError::InvalidFeatureFlag);
            }
            if indexed.insert(flag.key.clone(), flag).is_some() {
                return Err(ConfigError::DuplicateFeatureFlag);
            }
        }
        Ok(Self(indexed))
    }

    /// Validates that an offline/runtime update retains or increases restrictions.
    pub fn ensure_narrower_than(&self, previous: &Self) -> Result<(), ConfigError> {
        for (key, prior) in &previous.0 {
            if prior.active {
                let candidate = self.0.get(key).ok_or(ConfigError::AuthorityExpansion)?;
                if !candidate.active
                    || !candidate.scopes.is_subset(&prior.scopes)
                    || (prior.effect == FeatureEffect::Isolate
                        && candidate.effect != FeatureEffect::Isolate)
                {
                    return Err(ConfigError::AuthorityExpansion);
                }
            }
        }
        Ok(())
    }

    /// Returns whether a restriction is active at an instant; expired flags fail closed as inactive data.
    #[must_use]
    pub fn is_active(&self, key: &str, now: DateTime<Utc>) -> bool {
        self.0
            .get(key)
            .is_some_and(|flag| flag.active && now < flag.expires_at)
    }
}

/// Stable configuration rejection reasons.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum ConfigError {
    /// Artifact ownership, scope, key, or signed bytes were incomplete.
    #[error("configuration metadata is invalid")]
    InvalidMetadata,
    /// Typed configuration violated declared bounds.
    #[error("configuration value is outside approved bounds")]
    InvalidValue,
    /// Independently computed content digest differed.
    #[error("configuration payload digest mismatch")]
    DigestMismatch,
    /// Temporary configuration or flag expired.
    #[error("configuration is expired")]
    Expired,
    /// Detached signature did not verify.
    #[error("configuration signature is invalid")]
    InvalidSignature,
    /// Cryptographic verifier was unavailable.
    #[error("configuration signature dependency is unavailable")]
    SignatureDependencyUnavailable,
    /// Feature flag omitted required key, expiry, or scopes.
    #[error("feature flag is invalid")]
    InvalidFeatureFlag,
    /// Feature flag key appeared more than once.
    #[error("feature flag key is duplicated")]
    DuplicateFeatureFlag,
    /// Runtime update removed a restriction or broadened its scope.
    #[error("feature flags may narrow or isolate but never expand authority")]
    AuthorityExpansion,
    /// Typed payload or metadata could not be deterministically encoded.
    #[error("configuration canonicalization failed")]
    CanonicalizationFailed,
    /// Signed bytes do not exactly represent the typed payload and metadata.
    #[error("configuration signed bytes do not match typed content")]
    CanonicalBytesMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[derive(Clone, Debug, Eq, PartialEq, Serialize)]
    struct Limits {
        maximum_vehicles: u32,
    }
    impl TypedConfiguration for Limits {
        fn safe_default() -> Self {
            Self {
                maximum_vehicles: 0,
            }
        }
        fn validate(&self) -> Result<(), ConfigError> {
            if self.maximum_vehicles <= 100 {
                Ok(())
            } else {
                Err(ConfigError::InvalidValue)
            }
        }
    }
    struct Signature;
    impl ConfigurationSignatureVerifier for Signature {
        fn verify(&self, _: &str, bytes: &[u8], signature: &[u8]) -> Result<(), ConfigError> {
            if bytes == signature {
                Ok(())
            } else {
                Err(ConfigError::InvalidSignature)
            }
        }
    }
    fn now() -> DateTime<Utc> {
        DateTime::<Utc>::UNIX_EPOCH
    }

    #[test]
    fn signed_configuration_checks_bounds_digest_expiry_and_signature() -> Result<(), ConfigError> {
        let payload = Limits {
            maximum_vehicles: 10,
        };
        let digest = ContentDigest::from_sha256_bytes(
            Sha256::digest(
                serde_json::to_vec(&payload).map_err(|_| ConfigError::CanonicalizationFailed)?,
            )
            .into(),
        );
        let mut config = SignedConfiguration {
            schema_version: SemanticVersion::new(1, 0, 0),
            owner: "operations".into(),
            rollout_scope: "tenant:a".into(),
            classification: DataClassification::Restricted,
            restart: RestartSemantics::ProcessRestart,
            expires_at: Some(now() + Duration::hours(1)),
            payload,
            payload_digest: digest,
            signing_key_id: "key-1".into(),
            canonical_bytes: Vec::new(),
            signature: Vec::new(),
        };
        config.canonical_bytes = config.canonical_unsigned_bytes()?;
        config.signature.clone_from(&config.canonical_bytes);
        assert!(config.verify(now(), digest, &Signature).is_ok());
        assert_eq!(
            config.verify(now(), ContentDigest::from_sha256_bytes([2; 32]), &Signature),
            Err(ConfigError::DigestMismatch)
        );
        assert_eq!(
            config.verify(now() + Duration::hours(1), digest, &Signature),
            Err(ConfigError::Expired)
        );
        config.owner = "attacker".into();
        assert_eq!(
            config.verify(now(), digest, &Signature),
            Err(ConfigError::CanonicalBytesMismatch)
        );
        Ok(())
    }

    #[test]
    fn flag_update_cannot_remove_restriction_or_broaden_scope() -> Result<(), ConfigError> {
        let expiry = now() + Duration::hours(1);
        let previous = FeatureFlagSet::new(
            [FeatureFlag {
                key: "isolate-release".into(),
                active: true,
                effect: FeatureEffect::Isolate,
                expires_at: expiry,
                scopes: BTreeSet::from(["a".into()]),
            }],
            now(),
        )?;
        let removed = FeatureFlagSet::new([], now())?;
        assert_eq!(
            removed.ensure_narrower_than(&previous),
            Err(ConfigError::AuthorityExpansion)
        );
        Ok(())
    }
}
