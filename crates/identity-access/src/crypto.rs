//! Replaceable cryptographic ports for PKI, KMS, and hardware-backed keys.

use chrono::{DateTime, Utc};
use thiserror::Error;

/// Cryptographic purpose bound into a verification-key record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KeyPurpose {
    /// Signing canonical operational command envelopes.
    CommandSigning,
    /// Signing offline revocation and policy bundles.
    OfflinePolicySigning,
}

/// Public verification material returned by a PKI or KMS adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationKey {
    key_id: String,
    algorithm: String,
    public_material: Vec<u8>,
    generation: u64,
    purpose: KeyPurpose,
    valid_from: DateTime<Utc>,
    valid_until: DateTime<Utc>,
    revoked: bool,
}

impl VerificationKey {
    /// Builds a key record received from a trusted key adapter.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        key_id: impl Into<String>,
        algorithm: impl Into<String>,
        public_material: Vec<u8>,
        generation: u64,
        purpose: KeyPurpose,
        valid_from: DateTime<Utc>,
        valid_until: DateTime<Utc>,
        revoked: bool,
    ) -> Result<Self, CryptoError> {
        let key_id = key_id.into();
        let algorithm = algorithm.into();
        if key_id.trim().is_empty()
            || algorithm.trim().is_empty()
            || public_material.is_empty()
            || generation == 0
            || valid_from >= valid_until
        {
            return Err(CryptoError::InvalidKeyMetadata);
        }
        Ok(Self {
            key_id,
            algorithm,
            public_material,
            generation,
            purpose,
            valid_from,
            valid_until,
            revoked,
        })
    }

    /// Stable key identifier.
    #[must_use]
    pub fn key_id(&self) -> &str {
        &self.key_id
    }
    /// Algorithm identifier enforced by the crypto adapter.
    #[must_use]
    pub fn algorithm(&self) -> &str {
        &self.algorithm
    }
    /// Adapter-specific public key bytes.
    #[must_use]
    pub fn public_material(&self) -> &[u8] {
        &self.public_material
    }
    /// Monotonic rotation generation.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    /// Purpose for which this key may be accepted.
    #[must_use]
    pub const fn purpose(&self) -> KeyPurpose {
        self.purpose
    }

    /// Validates purpose, revocation, and the key validity interval.
    pub fn permits(&self, purpose: KeyPurpose, instant: DateTime<Utc>) -> Result<(), CryptoError> {
        if self.revoked {
            return Err(CryptoError::RevokedKey);
        }
        if self.purpose != purpose {
            return Err(CryptoError::WrongKeyPurpose);
        }
        if instant < self.valid_from || instant >= self.valid_until {
            return Err(CryptoError::KeyOutsideValidity);
        }
        Ok(())
    }
}

/// Resolves verification keys across active and overlap rotation generations.
pub trait VerificationKeyResolver {
    /// Resolves a key by stable identifier. Unknown keys fail closed.
    fn resolve(&self, key_id: &str) -> Result<VerificationKey, CryptoError>;
}

/// Performs detached-signature verification using approved cryptographic code.
pub trait DetachedSignatureVerifier {
    /// Verifies `signature` over the exact canonical `message` bytes.
    fn verify(
        &self,
        key: &VerificationKey,
        message: &[u8],
        signature: &[u8],
    ) -> Result<(), CryptoError>;
}

/// Stable failures from replaceable cryptographic adapters.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum CryptoError {
    /// Key metadata was missing, ambiguous, or internally inconsistent.
    #[error("verification key metadata is invalid")]
    InvalidKeyMetadata,
    /// The requested key is unknown to the current trust set.
    #[error("verification key is unknown")]
    UnknownKey,
    /// The key has been revoked.
    #[error("verification key is revoked")]
    RevokedKey,
    /// The key is not valid at the signed instant.
    #[error("verification key is outside its validity interval")]
    KeyOutsideValidity,
    /// The key is being used for a different cryptographic purpose.
    #[error("verification key purpose does not match")]
    WrongKeyPurpose,
    /// Detached signature verification failed.
    #[error("detached signature is invalid")]
    InvalidSignature,
    /// PKI, KMS, HSM, or verifier adapter was unavailable.
    #[error("cryptographic dependency is unavailable")]
    DependencyUnavailable,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn time(seconds: i64) -> DateTime<Utc> {
        DateTime::<Utc>::UNIX_EPOCH + Duration::seconds(seconds)
    }

    #[test]
    fn key_rejects_expiry_revocation_and_wrong_purpose() -> Result<(), CryptoError> {
        let expired = VerificationKey::new(
            "k1",
            "TEST-ONLY",
            vec![1],
            1,
            KeyPurpose::CommandSigning,
            time(0),
            time(10),
            false,
        )?;
        assert_eq!(
            expired.permits(KeyPurpose::CommandSigning, time(10)),
            Err(CryptoError::KeyOutsideValidity)
        );
        let revoked = VerificationKey::new(
            "k2",
            "TEST-ONLY",
            vec![2],
            2,
            KeyPurpose::CommandSigning,
            time(0),
            time(20),
            true,
        )?;
        assert_eq!(
            revoked.permits(KeyPurpose::CommandSigning, time(5)),
            Err(CryptoError::RevokedKey)
        );
        let policy = VerificationKey::new(
            "k3",
            "TEST-ONLY",
            vec![3],
            3,
            KeyPurpose::OfflinePolicySigning,
            time(0),
            time(20),
            false,
        )?;
        assert_eq!(
            policy.permits(KeyPurpose::CommandSigning, time(5)),
            Err(CryptoError::WrongKeyPurpose)
        );
        Ok(())
    }
}
