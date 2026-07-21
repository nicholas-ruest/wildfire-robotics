//! Signed, monotonically narrowing offline authority bundles (`IA-INV-005`).

use crate::crypto::{CryptoError, DetachedSignatureVerifier, KeyPurpose, VerificationKeyResolver};
use chrono::{DateTime, Duration, Utc};
use std::collections::BTreeSet;
use thiserror::Error;

/// Signed policy and revocation state usable while authority services are unavailable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OfflineBundle {
    sequence: u64,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    policy_version: String,
    authority_fingerprints: BTreeSet<String>,
    revoked_principals: BTreeSet<String>,
    revoked_keys: BTreeSet<String>,
    signer_key_id: String,
    canonical_bytes: Vec<u8>,
    signature: Vec<u8>,
}

/// Boundary input used to construct an immutable offline bundle.
pub struct OfflineBundleInput {
    /// Monotonically increasing distribution sequence.
    pub sequence: u64,
    /// Bundle issuance time.
    pub issued_at: DateTime<Utc>,
    /// Exclusive expiry time.
    pub expires_at: DateTime<Utc>,
    /// Signed policy release version.
    pub policy_version: String,
    /// Exact authority grants retained by the bundle.
    pub authority_fingerprints: BTreeSet<String>,
    /// Cumulative revoked principal identifiers.
    pub revoked_principals: BTreeSet<String>,
    /// Cumulative revoked key identifiers.
    pub revoked_keys: BTreeSet<String>,
    /// Bundle-signing key identifier.
    pub signer_key_id: String,
    /// Canonical serialization excluding the detached signature.
    pub canonical_bytes: Vec<u8>,
    /// Detached signature.
    pub signature: Vec<u8>,
}

/// Trusted inputs and replaceable ports used during bundle installation.
pub struct OfflineVerificationContext<'a> {
    /// Previously installed verified bundle, when present.
    pub previous: Option<&'a OfflineBundle>,
    /// Whether installation is connected to the authoritative service.
    pub installed_online: bool,
    /// Adapter-provided UTC time.
    pub now: DateTime<Utc>,
    /// Maximum permitted clock uncertainty.
    pub maximum_clock_uncertainty: Duration,
    /// Currently observed clock uncertainty.
    pub observed_clock_uncertainty: Duration,
    /// Trusted key resolver.
    pub keys: &'a dyn VerificationKeyResolver,
    /// Approved detached-signature implementation.
    pub signatures: &'a dyn DetachedSignatureVerifier,
}

impl OfflineBundle {
    /// Validates structural boundary invariants.
    pub fn new(input: OfflineBundleInput) -> Result<Self, OfflineBundleError> {
        if input.sequence == 0
            || input.issued_at >= input.expires_at
            || input.policy_version.trim().is_empty()
            || input.signer_key_id.trim().is_empty()
            || input.canonical_bytes.is_empty()
            || input.signature.is_empty()
        {
            return Err(OfflineBundleError::InvalidBundle);
        }
        Ok(Self {
            sequence: input.sequence,
            issued_at: input.issued_at,
            expires_at: input.expires_at,
            policy_version: input.policy_version,
            authority_fingerprints: input.authority_fingerprints,
            revoked_principals: input.revoked_principals,
            revoked_keys: input.revoked_keys,
            signer_key_id: input.signer_key_id,
            canonical_bytes: input.canonical_bytes,
            signature: input.signature,
        })
    }

    /// Verifies signature, freshness, monotonicity, and non-expansion before installation.
    pub fn verify_for_installation(
        &self,
        context: &OfflineVerificationContext<'_>,
    ) -> Result<(), OfflineBundleError> {
        if context.observed_clock_uncertainty < Duration::zero()
            || context.observed_clock_uncertainty > context.maximum_clock_uncertainty
        {
            return Err(OfflineBundleError::ClockUncertain);
        }
        if context.now < self.issued_at || context.now >= self.expires_at {
            return Err(OfflineBundleError::ExpiredOrNotYetValid);
        }
        let key = context.keys.resolve(&self.signer_key_id)?;
        key.permits(KeyPurpose::OfflinePolicySigning, self.issued_at)?;
        context
            .signatures
            .verify(&key, &self.canonical_bytes, &self.signature)?;
        match context.previous {
            None if !context.installed_online => {
                return Err(OfflineBundleError::OfflineBootstrapForbidden);
            }
            Some(prior) => {
                if self.sequence <= prior.sequence {
                    return Err(OfflineBundleError::NonIncreasingSequence);
                }
                if !self
                    .authority_fingerprints
                    .is_subset(&prior.authority_fingerprints)
                    || !self
                        .revoked_principals
                        .is_superset(&prior.revoked_principals)
                    || !self.revoked_keys.is_superset(&prior.revoked_keys)
                {
                    return Err(OfflineBundleError::AuthorityExpansion);
                }
            }
            None => {}
        }
        Ok(())
    }

    /// Sequence bound into command authorization decisions.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }
    /// Signed policy release version.
    #[must_use]
    pub fn policy_version(&self) -> &str {
        &self.policy_version
    }
    /// Whether the exact pre-approved authority remains available offline.
    #[must_use]
    pub fn retains_authority(&self, fingerprint: &str) -> bool {
        self.authority_fingerprints.contains(fingerprint)
    }
    /// Whether a principal is explicitly revoked.
    #[must_use]
    pub fn revokes_principal(&self, principal_id: &str) -> bool {
        self.revoked_principals.contains(principal_id)
    }
    /// Whether a signing key is explicitly revoked.
    #[must_use]
    pub fn revokes_key(&self, key_id: &str) -> bool {
        self.revoked_keys.contains(key_id)
    }
}

/// Stable offline bundle rejection reasons.
#[derive(Debug, Error)]
pub enum OfflineBundleError {
    /// Structural fields were missing or invalid.
    #[error("offline bundle is structurally invalid")]
    InvalidBundle,
    /// Trusted time could not establish bundle validity safely.
    #[error("clock uncertainty exceeds offline policy")]
    ClockUncertain,
    /// Bundle is expired or not yet valid.
    #[error("offline bundle is outside its validity interval")]
    ExpiredOrNotYetValid,
    /// First-time bundle installation was attempted without the authority service.
    #[error("offline bootstrap cannot create authority")]
    OfflineBootstrapForbidden,
    /// Bundle sequence did not advance.
    #[error("offline bundle sequence must strictly increase")]
    NonIncreasingSequence,
    /// Bundle removed a revocation or introduced new authority while offline.
    #[error("offline bundle may retain or narrow but never expand authority")]
    AuthorityExpansion,
    /// Cryptographic verification failed closed.
    #[error(transparent)]
    Crypto(#[from] CryptoError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::VerificationKey;

    struct Keys(VerificationKey);
    impl VerificationKeyResolver for Keys {
        fn resolve(&self, key_id: &str) -> Result<VerificationKey, CryptoError> {
            if key_id == self.0.key_id() {
                Ok(self.0.clone())
            } else {
                Err(CryptoError::UnknownKey)
            }
        }
    }
    struct Signatures;
    impl DetachedSignatureVerifier for Signatures {
        fn verify(
            &self,
            _: &VerificationKey,
            message: &[u8],
            signature: &[u8],
        ) -> Result<(), CryptoError> {
            if message == signature {
                Ok(())
            } else {
                Err(CryptoError::InvalidSignature)
            }
        }
    }
    fn time(seconds: i64) -> DateTime<Utc> {
        DateTime::<Utc>::UNIX_EPOCH + Duration::seconds(seconds)
    }
    fn key() -> Result<VerificationKey, CryptoError> {
        VerificationKey::new(
            "policy-key",
            "TEST-ONLY",
            vec![7],
            3,
            KeyPurpose::OfflinePolicySigning,
            time(0),
            time(100),
            false,
        )
    }
    fn bundle(
        sequence: u64,
        authorities: &[&str],
        revocations: &[&str],
    ) -> Result<OfflineBundle, OfflineBundleError> {
        OfflineBundle::new(OfflineBundleInput {
            sequence,
            issued_at: time(1),
            expires_at: time(90),
            policy_version: "1.0.0".into(),
            authority_fingerprints: authorities
                .iter()
                .map(|value| (*value).to_owned())
                .collect(),
            revoked_principals: revocations
                .iter()
                .map(|value| (*value).to_owned())
                .collect(),
            revoked_keys: BTreeSet::new(),
            signer_key_id: "policy-key".into(),
            canonical_bytes: vec![1, 2, 3],
            signature: vec![1, 2, 3],
        })
    }

    #[test]
    fn allows_monotonic_narrowing_and_cumulative_revocation()
    -> Result<(), Box<dyn std::error::Error>> {
        let prior = bundle(1, &["a", "b"], &["revoked-1"])?;
        let next = bundle(2, &["a"], &["revoked-1", "revoked-2"])?;
        let keys = Keys(key()?);
        assert!(
            next.verify_for_installation(&OfflineVerificationContext {
                previous: Some(&prior),
                installed_online: false,
                now: time(10),
                maximum_clock_uncertainty: Duration::seconds(1),
                observed_clock_uncertainty: Duration::milliseconds(1),
                keys: &keys,
                signatures: &Signatures,
            })
            .is_ok()
        );
        Ok(())
    }

    #[test]
    fn denies_authority_expansion_and_offline_bootstrap() -> Result<(), Box<dyn std::error::Error>>
    {
        let prior = bundle(1, &["a"], &[])?;
        let expanded = bundle(2, &["a", "b"], &[])?;
        let keys = Keys(key()?);
        assert!(matches!(
            expanded.verify_for_installation(&OfflineVerificationContext {
                previous: Some(&prior),
                installed_online: false,
                now: time(10),
                maximum_clock_uncertainty: Duration::seconds(1),
                observed_clock_uncertainty: Duration::zero(),
                keys: &keys,
                signatures: &Signatures,
            }),
            Err(OfflineBundleError::AuthorityExpansion)
        ));
        assert!(matches!(
            prior.verify_for_installation(&OfflineVerificationContext {
                previous: None,
                installed_online: false,
                now: time(10),
                maximum_clock_uncertainty: Duration::seconds(1),
                observed_clock_uncertainty: Duration::zero(),
                keys: &keys,
                signatures: &Signatures,
            }),
            Err(OfflineBundleError::OfflineBootstrapForbidden)
        ));
        Ok(())
    }

    #[test]
    fn rejects_expired_and_not_yet_valid_bundle() -> Result<(), Box<dyn std::error::Error>> {
        let candidate = bundle(1, &["a"], &[])?;
        let keys = Keys(key()?);
        for now in [time(0), time(90)] {
            let context = OfflineVerificationContext {
                previous: None,
                installed_online: true,
                now,
                maximum_clock_uncertainty: Duration::seconds(1),
                observed_clock_uncertainty: Duration::zero(),
                keys: &keys,
                signatures: &Signatures,
            };
            assert!(matches!(
                candidate.verify_for_installation(&context),
                Err(OfflineBundleError::ExpiredOrNotYetValid)
            ));
        }
        Ok(())
    }
}
