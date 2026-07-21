//! Canonical signed-command verification boundary (ADR-023).

use crate::{
    crypto::{CryptoError, DetachedSignatureVerifier, KeyPurpose, VerificationKeyResolver},
    offline::OfflineBundle,
};
use chrono::{DateTime, Duration, Utc};
use shared_kernel::{AggregateVersion, ContentDigest};
use thiserror::Error;

/// Exact tenant/incident/mission/vehicle scope bound into a command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandScope {
    /// Tenant identifier.
    pub tenant_id: String,
    /// Optional incident identifier.
    pub incident_id: Option<String>,
    /// Optional mission identifier.
    pub mission_id: Option<String>,
    /// Optional vehicle identifier.
    pub vehicle_id: Option<String>,
}

/// Auditable emergency override metadata; it grants no authority by itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreakGlass {
    /// Independent approving principal.
    pub approver_principal_id: String,
    /// Non-empty operational reason.
    pub reason: String,
    /// Exclusive override expiry.
    pub expires_at: DateTime<Utc>,
}

/// Immutable signed command envelope independent of transport-generated types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedCommandEnvelope {
    command_id: String,
    idempotency_key: String,
    issuer_principal_id: String,
    scope: CommandScope,
    capability: String,
    authority_fingerprint: String,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    expected_version: AggregateVersion,
    payload_digest: ContentDigest,
    policy_bundle_sequence: u64,
    signing_key_id: String,
    canonical_bytes: Vec<u8>,
    signature: Vec<u8>,
    break_glass: Option<BreakGlass>,
}

/// Validated construction input for a signed command.
pub struct SignedCommandInput {
    /// Globally unique command identifier.
    pub command_id: String,
    /// Business idempotency key.
    pub idempotency_key: String,
    /// Issuing principal.
    pub issuer_principal_id: String,
    /// Exact authority scope.
    pub scope: CommandScope,
    /// Requested capability.
    pub capability: String,
    /// Fingerprint of the exact approved grant/obligations.
    pub authority_fingerprint: String,
    /// Signed issuance time.
    pub issued_at: DateTime<Utc>,
    /// Exclusive command expiry.
    pub expires_at: DateTime<Utc>,
    /// Optimistic aggregate version.
    pub expected_version: AggregateVersion,
    /// Digest of payload bytes.
    pub payload_digest: ContentDigest,
    /// Required offline policy bundle sequence.
    pub policy_bundle_sequence: u64,
    /// Command signing key.
    pub signing_key_id: String,
    /// Canonical serialization excluding signature.
    pub canonical_bytes: Vec<u8>,
    /// Detached signature.
    pub signature: Vec<u8>,
    /// Optional audited emergency override.
    pub break_glass: Option<BreakGlass>,
}

impl SignedCommandInput {
    /// Deterministic unsigned representation binding every envelope field.
    pub fn canonical_unsigned_bytes(&self) -> Result<Vec<u8>, VerificationError> {
        let mut bytes = b"wildfire-command-envelope-v1\0".to_vec();
        for value in [
            &self.command_id,
            &self.idempotency_key,
            &self.issuer_principal_id,
            &self.scope.tenant_id,
            &self.capability,
            &self.authority_fingerprint,
            &self.signing_key_id,
        ] {
            push_text(&mut bytes, value)?;
        }
        for value in [
            &self.scope.incident_id,
            &self.scope.mission_id,
            &self.scope.vehicle_id,
        ] {
            match value {
                Some(value) => {
                    bytes.push(1);
                    push_text(&mut bytes, value)?;
                }
                None => bytes.push(0),
            }
        }
        bytes.extend_from_slice(&self.issued_at.timestamp_micros().to_be_bytes());
        bytes.extend_from_slice(&self.expires_at.timestamp_micros().to_be_bytes());
        bytes.extend_from_slice(&self.expected_version.get().to_be_bytes());
        bytes.extend_from_slice(self.payload_digest.as_bytes());
        bytes.extend_from_slice(&self.policy_bundle_sequence.to_be_bytes());
        match &self.break_glass {
            Some(value) => {
                bytes.push(1);
                push_text(&mut bytes, &value.approver_principal_id)?;
                push_text(&mut bytes, &value.reason)?;
                bytes.extend_from_slice(&value.expires_at.timestamp_micros().to_be_bytes());
            }
            None => bytes.push(0),
        }
        Ok(bytes)
    }
}

fn push_text(bytes: &mut Vec<u8>, value: &str) -> Result<(), VerificationError> {
    let length = u32::try_from(value.len()).map_err(|_| VerificationError::MalformedEnvelope)?;
    bytes.extend_from_slice(&length.to_be_bytes());
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

impl SignedCommandEnvelope {
    /// Rejects incomplete or ambiguous envelope construction.
    pub fn new(input: SignedCommandInput) -> Result<Self, VerificationError> {
        let required = [
            input.command_id.as_str(),
            input.idempotency_key.as_str(),
            input.issuer_principal_id.as_str(),
            input.capability.as_str(),
            input.authority_fingerprint.as_str(),
            input.signing_key_id.as_str(),
        ];
        let expected_canonical = input.canonical_unsigned_bytes()?;
        if required.iter().any(|value| value.trim().is_empty())
            || input.scope.tenant_id.trim().is_empty()
            || [
                input.scope.incident_id.as_deref(),
                input.scope.mission_id.as_deref(),
                input.scope.vehicle_id.as_deref(),
            ]
            .into_iter()
            .flatten()
            .any(|value| value.trim().is_empty())
            || input.issued_at >= input.expires_at
            || input.policy_bundle_sequence == 0
            || input.canonical_bytes.is_empty()
            || input.signature.is_empty()
            || input.canonical_bytes != expected_canonical
        {
            return Err(VerificationError::MalformedEnvelope);
        }
        Ok(Self {
            command_id: input.command_id,
            idempotency_key: input.idempotency_key,
            issuer_principal_id: input.issuer_principal_id,
            scope: input.scope,
            capability: input.capability,
            authority_fingerprint: input.authority_fingerprint,
            issued_at: input.issued_at,
            expires_at: input.expires_at,
            expected_version: input.expected_version,
            payload_digest: input.payload_digest,
            policy_bundle_sequence: input.policy_bundle_sequence,
            signing_key_id: input.signing_key_id,
            canonical_bytes: input.canonical_bytes,
            signature: input.signature,
            break_glass: input.break_glass,
        })
    }
}

/// Atomic replay protection supplied by the persistence adapter.
pub trait ReplayGuard {
    /// Claims both identifiers exactly once after all other checks pass.
    fn claim(&self, command_id: &str, idempotency_key: &str) -> Result<(), VerificationError>;
}

/// Independent durable audit path for privileged command verification.
pub trait VerificationAudit {
    /// Records every break-glass attempt before its acceptance or rejection.
    fn record_break_glass_attempt(
        &self,
        command_id: &str,
        issuer: &str,
        approver: &str,
        reason: &str,
    ) -> Result<(), VerificationError>;
}

/// Trusted state against which an envelope is verified.
pub struct VerificationContext<'a> {
    /// Adapter-provided UTC now.
    pub now: DateTime<Utc>,
    /// Current clock uncertainty.
    pub clock_uncertainty: Duration,
    /// Maximum uncertainty allowed for this capability.
    pub maximum_clock_uncertainty: Duration,
    /// Exact target scope.
    pub expected_scope: &'a CommandScope,
    /// Current aggregate version.
    pub current_version: AggregateVersion,
    /// Digest computed independently from received payload bytes.
    pub computed_payload_digest: ContentDigest,
    /// Lowest signing-key generation accepted after rotation.
    pub minimum_signing_key_generation: u64,
    /// Installed verified offline bundle.
    pub offline_bundle: &'a OfflineBundle,
}

/// Successful authentication result. Authorization remains separately enforced.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedCommand {
    /// Command identifier claimed by replay protection.
    pub command_id: String,
    /// Issuing authenticated principal.
    pub issuer_principal_id: String,
    /// Requested capability, not an authorization decision.
    pub capability: String,
    /// Whether break-glass metadata accompanied the command.
    pub break_glass: bool,
}

/// Canonical fail-closed verifier with injected trust and persistence ports.
pub struct CommandVerifier<'a> {
    /// Key-resolution adapter.
    pub keys: &'a dyn VerificationKeyResolver,
    /// Detached-signature adapter.
    pub signatures: &'a dyn DetachedSignatureVerifier,
    /// Atomic replay adapter.
    pub replay: &'a dyn ReplayGuard,
    /// Independent privileged audit adapter.
    pub audit: &'a dyn VerificationAudit,
}

impl CommandVerifier<'_> {
    /// Verifies authentication and technical authority preconditions in safe order.
    pub fn verify(
        &self,
        envelope: &SignedCommandEnvelope,
        context: &VerificationContext<'_>,
    ) -> Result<VerifiedCommand, VerificationError> {
        if let Some(override_request) = &envelope.break_glass {
            self.audit.record_break_glass_attempt(
                &envelope.command_id,
                &envelope.issuer_principal_id,
                &override_request.approver_principal_id,
                &override_request.reason,
            )?;
            if override_request.reason.trim().is_empty()
                || override_request.approver_principal_id == envelope.issuer_principal_id
                || context.now >= override_request.expires_at
            {
                return Err(VerificationError::InvalidBreakGlass);
            }
        }
        if context.clock_uncertainty < Duration::zero()
            || context.clock_uncertainty > context.maximum_clock_uncertainty
        {
            return Err(VerificationError::ClockUncertain);
        }
        if context.now < envelope.issued_at || context.now >= envelope.expires_at {
            return Err(VerificationError::StaleCommand);
        }
        if &envelope.scope != context.expected_scope {
            return Err(VerificationError::ScopeMismatch);
        }
        if envelope.expected_version != context.current_version {
            return Err(VerificationError::VersionConflict);
        }
        if envelope.payload_digest != context.computed_payload_digest {
            return Err(VerificationError::PayloadDigestMismatch);
        }
        if envelope.policy_bundle_sequence != context.offline_bundle.sequence()
            || !context
                .offline_bundle
                .retains_authority(&envelope.authority_fingerprint)
        {
            return Err(VerificationError::StaleOrMissingAuthority);
        }
        if context
            .offline_bundle
            .revokes_principal(&envelope.issuer_principal_id)
            || context.offline_bundle.revokes_key(&envelope.signing_key_id)
        {
            return Err(VerificationError::Revoked);
        }
        let key = self.keys.resolve(&envelope.signing_key_id)?;
        if key.generation() < context.minimum_signing_key_generation {
            return Err(VerificationError::KeyGenerationRollback);
        }
        key.permits(KeyPurpose::CommandSigning, envelope.issued_at)?;
        self.signatures
            .verify(&key, &envelope.canonical_bytes, &envelope.signature)?;
        self.replay
            .claim(&envelope.command_id, &envelope.idempotency_key)?;
        Ok(VerifiedCommand {
            command_id: envelope.command_id.clone(),
            issuer_principal_id: envelope.issuer_principal_id.clone(),
            capability: envelope.capability.clone(),
            break_glass: envelope.break_glass.is_some(),
        })
    }
}

/// Stable signed-command rejection reasons.
#[derive(Debug, Error)]
pub enum VerificationError {
    /// Required envelope data was absent or internally inconsistent.
    #[error("signed command envelope is malformed")]
    MalformedEnvelope,
    /// Clock quality cannot safely establish freshness.
    #[error("clock uncertainty exceeds command policy")]
    ClockUncertain,
    /// Command is expired or not yet valid.
    #[error("command is outside its validity interval")]
    StaleCommand,
    /// Command scope differs from the target resource.
    #[error("command scope does not match target")]
    ScopeMismatch,
    /// Expected aggregate version is stale or from the future.
    #[error("expected aggregate version conflicts")]
    VersionConflict,
    /// Independently calculated payload digest differs.
    #[error("command payload digest does not match")]
    PayloadDigestMismatch,
    /// Installed policy does not retain the exact requested authority.
    #[error("command authority is missing or stale")]
    StaleOrMissingAuthority,
    /// Principal or key is revoked by the installed bundle.
    #[error("command principal or key is revoked")]
    Revoked,
    /// A cryptographically valid but superseded rotation generation was presented.
    #[error("command signing key generation has been superseded")]
    KeyGenerationRollback,
    /// Command or idempotency key was previously claimed.
    #[error("command replay detected")]
    Replay,
    /// Break-glass metadata violates expiry, reason, or separation policy.
    #[error("break-glass request is invalid")]
    InvalidBreakGlass,
    /// Independent audit storage was unavailable; privileged action fails closed.
    #[error("verification audit is unavailable")]
    AuditUnavailable,
    /// Cryptographic verification failed closed.
    #[error(transparent)]
    Crypto(#[from] CryptoError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        crypto::{KeyPurpose, VerificationKey},
        offline::{OfflineBundle, OfflineBundleInput},
    };
    use std::{cell::RefCell, collections::BTreeSet};

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

    #[derive(Default)]
    struct Replay(RefCell<BTreeSet<String>>);
    impl ReplayGuard for Replay {
        fn claim(&self, command_id: &str, idempotency_key: &str) -> Result<(), VerificationError> {
            let mut claimed = self.0.borrow_mut();
            if !claimed.insert(format!("{command_id}:{idempotency_key}")) {
                return Err(VerificationError::Replay);
            }
            Ok(())
        }
    }

    #[derive(Default)]
    struct Audit(RefCell<Vec<String>>);
    impl VerificationAudit for Audit {
        fn record_break_glass_attempt(
            &self,
            command_id: &str,
            _: &str,
            _: &str,
            _: &str,
        ) -> Result<(), VerificationError> {
            self.0.borrow_mut().push(command_id.to_owned());
            Ok(())
        }
    }

    fn time(seconds: i64) -> DateTime<Utc> {
        DateTime::<Utc>::UNIX_EPOCH + Duration::seconds(seconds)
    }

    fn key() -> Result<VerificationKey, CryptoError> {
        VerificationKey::new(
            "command-key-2",
            "TEST-ONLY",
            vec![2],
            2,
            KeyPurpose::CommandSigning,
            time(0),
            time(100),
            false,
        )
    }

    fn bundle() -> Result<OfflineBundle, crate::offline::OfflineBundleError> {
        OfflineBundle::new(OfflineBundleInput {
            sequence: 7,
            issued_at: time(0),
            expires_at: time(100),
            policy_version: "1.0.0".into(),
            authority_fingerprints: BTreeSet::from(["grant-digest".into()]),
            revoked_principals: BTreeSet::new(),
            revoked_keys: BTreeSet::new(),
            signer_key_id: "policy-key".into(),
            canonical_bytes: vec![9],
            signature: vec![9],
        })
    }

    fn command_input() -> Result<SignedCommandInput, VerificationError> {
        let mut input = SignedCommandInput {
            command_id: "command-1".into(),
            idempotency_key: "effect-1".into(),
            issuer_principal_id: "issuer".into(),
            scope: CommandScope {
                tenant_id: "tenant".into(),
                incident_id: Some("incident".into()),
                mission_id: None,
                vehicle_id: Some("vehicle".into()),
            },
            capability: "stop".into(),
            authority_fingerprint: "grant-digest".into(),
            issued_at: time(10),
            expires_at: time(30),
            expected_version: AggregateVersion::from_u64(4),
            payload_digest: ContentDigest::from_sha256_bytes([3; 32]),
            policy_bundle_sequence: 7,
            signing_key_id: "command-key-2".into(),
            canonical_bytes: Vec::new(),
            signature: Vec::new(),
            break_glass: None,
        };
        input.canonical_bytes = input.canonical_unsigned_bytes()?;
        input.signature.clone_from(&input.canonical_bytes);
        Ok(input)
    }

    fn command() -> Result<SignedCommandEnvelope, VerificationError> {
        SignedCommandEnvelope::new(command_input()?)
    }

    #[test]
    fn canonical_bytes_bind_every_command_field() -> Result<(), VerificationError> {
        let mut input = command_input()?;
        input.capability = "different-capability".into();
        assert!(matches!(
            SignedCommandEnvelope::new(input),
            Err(VerificationError::MalformedEnvelope)
        ));
        Ok(())
    }

    #[test]
    fn accepts_exact_scope_digest_version_freshness_and_rotation_generation()
    -> Result<(), Box<dyn std::error::Error>> {
        let command = command()?;
        let bundle = bundle()?;
        let keys = Keys(key()?);
        let replay = Replay::default();
        let audit = Audit::default();
        let verifier = CommandVerifier {
            keys: &keys,
            signatures: &Signatures,
            replay: &replay,
            audit: &audit,
        };
        let context = VerificationContext {
            now: time(20),
            clock_uncertainty: Duration::milliseconds(5),
            maximum_clock_uncertainty: Duration::milliseconds(10),
            expected_scope: &command.scope,
            current_version: AggregateVersion::from_u64(4),
            computed_payload_digest: ContentDigest::from_sha256_bytes([3; 32]),
            minimum_signing_key_generation: 2,
            offline_bundle: &bundle,
        };
        assert!(verifier.verify(&command, &context).is_ok());
        assert!(matches!(
            verifier.verify(&command, &context),
            Err(VerificationError::Replay)
        ));
        Ok(())
    }

    #[test]
    fn rejects_uncertain_clock_before_signature_or_replay_claim()
    -> Result<(), Box<dyn std::error::Error>> {
        let command = command()?;
        let bundle = bundle()?;
        let keys = Keys(key()?);
        let replay = Replay::default();
        let audit = Audit::default();
        let verifier = CommandVerifier {
            keys: &keys,
            signatures: &Signatures,
            replay: &replay,
            audit: &audit,
        };
        let context = VerificationContext {
            now: time(20),
            clock_uncertainty: Duration::seconds(2),
            maximum_clock_uncertainty: Duration::seconds(1),
            expected_scope: &command.scope,
            current_version: AggregateVersion::from_u64(4),
            computed_payload_digest: ContentDigest::from_sha256_bytes([3; 32]),
            minimum_signing_key_generation: 2,
            offline_bundle: &bundle,
        };
        assert!(matches!(
            verifier.verify(&command, &context),
            Err(VerificationError::ClockUncertain)
        ));
        assert!(replay.0.borrow().is_empty());
        Ok(())
    }

    #[test]
    fn break_glass_is_audited_and_cannot_bypass_separation()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut command = command()?;
        command.break_glass = Some(BreakGlass {
            approver_principal_id: "issuer".into(),
            reason: "emergency stop".into(),
            expires_at: time(25),
        });
        let bundle = bundle()?;
        let keys = Keys(key()?);
        let replay = Replay::default();
        let audit = Audit::default();
        let verifier = CommandVerifier {
            keys: &keys,
            signatures: &Signatures,
            replay: &replay,
            audit: &audit,
        };
        let context = VerificationContext {
            now: time(20),
            clock_uncertainty: Duration::zero(),
            maximum_clock_uncertainty: Duration::seconds(1),
            expected_scope: &command.scope,
            current_version: AggregateVersion::from_u64(4),
            computed_payload_digest: ContentDigest::from_sha256_bytes([3; 32]),
            minimum_signing_key_generation: 2,
            offline_bundle: &bundle,
        };
        assert!(matches!(
            verifier.verify(&command, &context),
            Err(VerificationError::InvalidBreakGlass)
        ));
        assert_eq!(audit.0.borrow().as_slice(), ["command-1"]);
        Ok(())
    }

    #[test]
    fn rejects_freshness_scope_version_and_digest_adversarial_paths()
    -> Result<(), Box<dyn std::error::Error>> {
        let base = command()?;
        let bundle = bundle()?;
        let keys = Keys(key()?);
        let replay = Replay::default();
        let audit = Audit::default();
        let verifier = CommandVerifier {
            keys: &keys,
            signatures: &Signatures,
            replay: &replay,
            audit: &audit,
        };
        let other_scope = CommandScope {
            tenant_id: "other".into(),
            incident_id: None,
            mission_id: None,
            vehicle_id: None,
        };
        let cases = [
            (
                time(30),
                &base.scope,
                AggregateVersion::from_u64(4),
                [3; 32],
                VerificationError::StaleCommand,
            ),
            (
                time(9),
                &base.scope,
                AggregateVersion::from_u64(4),
                [3; 32],
                VerificationError::StaleCommand,
            ),
            (
                time(20),
                &other_scope,
                AggregateVersion::from_u64(4),
                [3; 32],
                VerificationError::ScopeMismatch,
            ),
            (
                time(20),
                &base.scope,
                AggregateVersion::from_u64(5),
                [3; 32],
                VerificationError::VersionConflict,
            ),
            (
                time(20),
                &base.scope,
                AggregateVersion::from_u64(4),
                [4; 32],
                VerificationError::PayloadDigestMismatch,
            ),
        ];
        for (now, scope, version, digest, expected) in cases {
            let context = VerificationContext {
                now,
                clock_uncertainty: Duration::zero(),
                maximum_clock_uncertainty: Duration::seconds(1),
                expected_scope: scope,
                current_version: version,
                computed_payload_digest: ContentDigest::from_sha256_bytes(digest),
                minimum_signing_key_generation: 2,
                offline_bundle: &bundle,
            };
            assert_eq!(
                verifier
                    .verify(&base, &context)
                    .err()
                    .map(|error| error.to_string()),
                Some(expected.to_string())
            );
        }
        assert!(replay.0.borrow().is_empty());
        Ok(())
    }

    #[test]
    fn rejects_superseded_rotation_generation() -> Result<(), Box<dyn std::error::Error>> {
        let command = command()?;
        let bundle = bundle()?;
        let keys = Keys(key()?);
        let replay = Replay::default();
        let audit = Audit::default();
        let verifier = CommandVerifier {
            keys: &keys,
            signatures: &Signatures,
            replay: &replay,
            audit: &audit,
        };
        let context = VerificationContext {
            now: time(20),
            clock_uncertainty: Duration::zero(),
            maximum_clock_uncertainty: Duration::seconds(1),
            expected_scope: &command.scope,
            current_version: AggregateVersion::from_u64(4),
            computed_payload_digest: ContentDigest::from_sha256_bytes([3; 32]),
            minimum_signing_key_generation: 3,
            offline_bundle: &bundle,
        };
        assert!(matches!(
            verifier.verify(&command, &context),
            Err(VerificationError::KeyGenerationRollback)
        ));
        Ok(())
    }
}
