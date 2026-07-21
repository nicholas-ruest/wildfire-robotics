//! Identity aggregates and invariant-preserving transitions.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use shared_kernel::{AggregateVersion, PrincipalId};
use thiserror::Error;

/// Kind of independently attributable principal.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PrincipalKind {
    /// A natural person authenticated through the human identity trust domain.
    Human,
    /// A deployed service authenticated through workload attestation.
    Workload,
}

/// Principal lifecycle from invitation through irreversible disablement.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PrincipalState {
    /// Registered but not yet identity-verified.
    Invited,
    /// Verified and allowed to authenticate.
    Active,
    /// Temporarily denied pending explicit reactivation.
    Suspended,
    /// Permanently disabled identity.
    Disabled,
}

/// Verified provenance supplied by the appropriate trust domain.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VerifiedProvenance {
    provider: String,
    subject: String,
    evidence_id: String,
    verified_at: DateTime<Utc>,
    assurance: AuthenticatorAssurance,
}

impl VerifiedProvenance {
    /// Creates non-ambiguous provenance from a successful trust-port result.
    pub fn new(
        provider: impl Into<String>,
        subject: impl Into<String>,
        evidence_id: impl Into<String>,
        verified_at: DateTime<Utc>,
        assurance: AuthenticatorAssurance,
    ) -> Result<Self, IdentityError> {
        let provider = required(provider.into())?;
        let subject = required(subject.into())?;
        let evidence_id = required(evidence_id.into())?;
        Ok(Self {
            provider,
            subject,
            evidence_id,
            verified_at,
            assurance,
        })
    }
}

/// Authentication strength proven at identity verification.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum AuthenticatorAssurance {
    /// Single-factor or workload baseline.
    Basic,
    /// Multi-factor or equivalently strong workload proof.
    MultiFactor,
    /// Hardware-backed phishing-resistant authentication.
    HardwareBound,
}

/// Independently owned, non-shared person or workload identity (`IA-INV-001`).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Principal {
    id: PrincipalId,
    kind: PrincipalKind,
    lifecycle_owner: PrincipalId,
    state: PrincipalState,
    provenance: Option<VerifiedProvenance>,
    suspension_reason: Option<String>,
    version: AggregateVersion,
}

impl Principal {
    /// Registers an invited identity. Ownership cannot be anonymous or shared.
    #[must_use]
    pub fn register(id: PrincipalId, kind: PrincipalKind, lifecycle_owner: PrincipalId) -> Self {
        Self {
            id,
            kind,
            lifecycle_owner,
            state: PrincipalState::Invited,
            provenance: None,
            suspension_reason: None,
            version: AggregateVersion::INITIAL,
        }
    }

    /// Attaches verified provenance once; subject uniqueness is enforced by the repository port.
    pub fn verify_identity(&mut self, provenance: VerifiedProvenance) -> Result<(), IdentityError> {
        if self.state != PrincipalState::Invited {
            return Err(IdentityError::IllegalPrincipalTransition);
        }
        if self.provenance.is_some() {
            return Err(IdentityError::ProvenanceAlreadyVerified);
        }
        self.provenance = Some(provenance);
        self.advance()
    }

    /// Activates a verified invited or suspended principal.
    pub fn activate(&mut self) -> Result<(), IdentityError> {
        if self.provenance.is_none()
            || !matches!(
                self.state,
                PrincipalState::Invited | PrincipalState::Suspended
            )
        {
            return Err(IdentityError::IllegalPrincipalTransition);
        }
        self.state = PrincipalState::Active;
        self.suspension_reason = None;
        self.advance()
    }

    /// Suspends an active identity with an auditable reason.
    pub fn suspend(&mut self, reason: impl Into<String>) -> Result<(), IdentityError> {
        if self.state != PrincipalState::Active {
            return Err(IdentityError::IllegalPrincipalTransition);
        }
        self.suspension_reason = Some(required(reason.into())?);
        self.state = PrincipalState::Suspended;
        self.advance()
    }

    /// Irreversibly disables a non-disabled identity.
    pub fn disable(&mut self, reason: impl Into<String>) -> Result<(), IdentityError> {
        if self.state == PrincipalState::Disabled {
            return Err(IdentityError::IllegalPrincipalTransition);
        }
        self.suspension_reason = Some(required(reason.into())?);
        self.state = PrincipalState::Disabled;
        self.advance()
    }

    /// Identity.
    #[must_use]
    pub const fn id(&self) -> PrincipalId {
        self.id
    }
    /// Lifecycle owner responsible for recovery and disablement.
    #[must_use]
    pub const fn lifecycle_owner(&self) -> PrincipalId {
        self.lifecycle_owner
    }
    /// Current lifecycle state.
    #[must_use]
    pub const fn state(&self) -> PrincipalState {
        self.state
    }
    /// Principal trust domain.
    #[must_use]
    pub const fn kind(&self) -> PrincipalKind {
        self.kind
    }
    /// Optimistic version.
    #[must_use]
    pub const fn version(&self) -> AggregateVersion {
        self.version
    }

    fn advance(&mut self) -> Result<(), IdentityError> {
        self.version = self
            .version
            .checked_next()
            .map_err(|_| IdentityError::VersionExhausted)?;
        Ok(())
    }
}

/// Device identity independent of a fleet vehicle identity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DeviceId(PrincipalId);

impl DeviceId {
    /// Creates a device identity from a dedicated UUID namespace at the adapter boundary.
    #[must_use]
    pub const fn from_opaque_id(value: PrincipalId) -> Self {
        Self(value)
    }
}

/// Device lifecycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DeviceState {
    /// Recorded at manufacturing with manufacturer provenance pending verification.
    Manufactured,
    /// Imported from an external inventory with provenance pending verification.
    Imported,
    /// Enrollment/attestation is in progress.
    Enrolling,
    /// Current attestation permits authentication.
    Trusted,
    /// Temporarily isolated pending investigation and re-enrollment.
    Quarantined,
    /// Trust is irreversibly revoked.
    Revoked,
    /// Credential material has been destroyed and the device retired.
    Retired,
}

/// Current hardware and software evidence required by `IA-INV-003`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttestationEvidence {
    /// Immutable evidence identifier.
    pub evidence_id: String,
    /// Hardware-root key identifier; never secret key material.
    pub hardware_key_id: String,
    /// Trust-root version used to verify the evidence.
    pub trust_root_version: String,
    /// Measured software release.
    pub software_version: String,
    /// Measured configuration digest.
    pub configuration_digest: String,
    /// Evidence observation time.
    pub attested_at: DateTime<Utc>,
    /// Evidence expiry.
    pub expires_at: DateTime<Utc>,
    /// Whether the verifier proved a non-exportable hardware key.
    pub hardware_backed: bool,
    /// Whether software/configuration passed the signed compatibility policy.
    pub compatible_and_not_revoked: bool,
}

impl AttestationEvidence {
    fn validate(&self, evaluated_at: DateTime<Utc>) -> Result<(), IdentityError> {
        if !self.hardware_backed || !self.compatible_and_not_revoked {
            return Err(IdentityError::UntrustedAttestation);
        }
        if self.attested_at > evaluated_at
            || self.expires_at <= evaluated_at
            || self.expires_at <= self.attested_at
        {
            return Err(IdentityError::ExpiredOrFutureAttestation);
        }
        required_ref(&self.evidence_id)?;
        required_ref(&self.hardware_key_id)?;
        required_ref(&self.trust_root_version)?;
        required_ref(&self.software_version)?;
        required_ref(&self.configuration_digest)?;
        Ok(())
    }
}

/// Hardware-rooted device identity (`IA-INV-001`, `IA-INV-003`).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeviceIdentity {
    id: DeviceId,
    lifecycle_owner: PrincipalId,
    state: DeviceState,
    attestation: Option<AttestationEvidence>,
    reason: Option<String>,
    version: AggregateVersion,
}

impl DeviceIdentity {
    /// Records a newly manufactured device.
    #[must_use]
    pub fn manufactured(id: DeviceId, lifecycle_owner: PrincipalId) -> Self {
        Self::new(id, lifecycle_owner, DeviceState::Manufactured)
    }
    /// Records an imported device whose provenance must still be enrolled.
    #[must_use]
    pub fn imported(id: DeviceId, lifecycle_owner: PrincipalId) -> Self {
        Self::new(id, lifecycle_owner, DeviceState::Imported)
    }
    fn new(id: DeviceId, lifecycle_owner: PrincipalId, state: DeviceState) -> Self {
        Self {
            id,
            lifecycle_owner,
            state,
            attestation: None,
            reason: None,
            version: AggregateVersion::INITIAL,
        }
    }
    /// Begins enrollment from either provenance entry state.
    pub fn begin_enrollment(&mut self) -> Result<(), IdentityError> {
        if !matches!(
            self.state,
            DeviceState::Manufactured | DeviceState::Imported | DeviceState::Quarantined
        ) {
            return Err(IdentityError::IllegalDeviceTransition);
        }
        self.state = DeviceState::Enrolling;
        self.advance()
    }
    /// Trusts a device only with current, hardware-backed, compatible evidence.
    pub fn attest(
        &mut self,
        evidence: AttestationEvidence,
        evaluated_at: DateTime<Utc>,
    ) -> Result<(), IdentityError> {
        if self.state != DeviceState::Enrolling {
            return Err(IdentityError::IllegalDeviceTransition);
        }
        evidence.validate(evaluated_at)?;
        self.attestation = Some(evidence);
        self.state = DeviceState::Trusted;
        self.reason = None;
        self.advance()
    }
    /// Rotates the device key through fresh attestation; stale root evidence fails validation at the port.
    pub fn rotate_key(
        &mut self,
        evidence: AttestationEvidence,
        evaluated_at: DateTime<Utc>,
    ) -> Result<(), IdentityError> {
        if self.state != DeviceState::Trusted {
            return Err(IdentityError::IllegalDeviceTransition);
        }
        evidence.validate(evaluated_at)?;
        if self
            .attestation
            .as_ref()
            .is_some_and(|old| old.hardware_key_id == evidence.hardware_key_id)
        {
            return Err(IdentityError::KeyWasNotRotated);
        }
        self.attestation = Some(evidence);
        self.advance()
    }
    /// Quarantines a currently enrolling or trusted device.
    pub fn quarantine(&mut self, reason: impl Into<String>) -> Result<(), IdentityError> {
        if !matches!(self.state, DeviceState::Enrolling | DeviceState::Trusted) {
            return Err(IdentityError::IllegalDeviceTransition);
        }
        self.reason = Some(required(reason.into())?);
        self.state = DeviceState::Quarantined;
        self.advance()
    }
    /// Irreversibly revokes device trust.
    pub fn revoke(&mut self, reason: impl Into<String>) -> Result<(), IdentityError> {
        if matches!(self.state, DeviceState::Revoked | DeviceState::Retired) {
            return Err(IdentityError::IllegalDeviceTransition);
        }
        self.reason = Some(required(reason.into())?);
        self.state = DeviceState::Revoked;
        self.advance()
    }
    /// Retires a revoked device after credential destruction evidence is recorded by the caller.
    pub fn retire(&mut self) -> Result<(), IdentityError> {
        if self.state != DeviceState::Revoked {
            return Err(IdentityError::IllegalDeviceTransition);
        }
        self.state = DeviceState::Retired;
        self.advance()
    }
    /// Whether current evidence remains usable at the supplied trusted time.
    #[must_use]
    pub fn is_trusted_at(&self, now: DateTime<Utc>) -> bool {
        self.state == DeviceState::Trusted
            && self
                .attestation
                .as_ref()
                .is_some_and(|value| value.validate(now).is_ok())
    }
    /// Lifecycle state.
    #[must_use]
    pub const fn state(&self) -> DeviceState {
        self.state
    }
    /// Lifecycle owner.
    #[must_use]
    pub const fn lifecycle_owner(&self) -> PrincipalId {
        self.lifecycle_owner
    }
    /// Device ID.
    #[must_use]
    pub const fn id(&self) -> DeviceId {
        self.id
    }
    fn advance(&mut self) -> Result<(), IdentityError> {
        self.version = self
            .version
            .checked_next()
            .map_err(|_| IdentityError::VersionExhausted)?;
        Ok(())
    }
}

/// Short-lived credential metadata; secret material remains in the issuing adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuedCredential {
    /// Opaque credential identifier.
    pub credential_id: String,
    /// Trust-domain issuer.
    pub issuer: String,
    /// Issue time.
    pub issued_at: DateTime<Utc>,
    /// Hard expiry.
    pub expires_at: DateTime<Utc>,
    /// Key/root version for rollover and revocation checks.
    pub trust_root_version: String,
}

impl IssuedCredential {
    /// Validates bounded lifetime and required attribution.
    pub fn validate(&self, maximum_lifetime: Duration) -> Result<(), IdentityError> {
        required_ref(&self.credential_id)?;
        required_ref(&self.issuer)?;
        required_ref(&self.trust_root_version)?;
        if maximum_lifetime <= Duration::zero()
            || self.expires_at <= self.issued_at
            || self.expires_at - self.issued_at > maximum_lifetime
        {
            return Err(IdentityError::CredentialLifetimeInvalid);
        }
        Ok(())
    }
}

/// Domain validation failures.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum IdentityError {
    /// A required string is blank or exceeds its boundary.
    #[error("required identity value is blank or too long")]
    InvalidValue,
    /// Requested principal transition is not in its lifecycle table.
    #[error("principal lifecycle transition is illegal")]
    IllegalPrincipalTransition,
    /// Immutable verification provenance cannot be replaced.
    #[error("principal provenance was already verified")]
    ProvenanceAlreadyVerified,
    /// Requested device transition is not in its lifecycle table.
    #[error("device lifecycle transition is illegal")]
    IllegalDeviceTransition,
    /// Evidence did not prove hardware backing or compatibility.
    #[error("attestation is not hardware-backed and compatible")]
    UntrustedAttestation,
    /// Evidence time interval is not current and coherent.
    #[error("attestation is expired, future-dated, or has an invalid interval")]
    ExpiredOrFutureAttestation,
    /// Rotation attempted to retain the old key identifier.
    #[error("device key identifier did not change during rotation")]
    KeyWasNotRotated,
    /// Credential is not bounded by the configured maximum lifetime.
    #[error("credential lifetime is absent or exceeds policy")]
    CredentialLifetimeInvalid,
    /// Aggregate version cannot advance without wrapping.
    #[error("aggregate version exhausted")]
    VersionExhausted,
}

fn required(value: String) -> Result<String, IdentityError> {
    required_ref(&value)?;
    Ok(value)
}
fn required_ref(value: &str) -> Result<(), IdentityError> {
    if value.trim().is_empty() || value.len() > 512 {
        Err(IdentityError::InvalidValue)
    } else {
        Ok(())
    }
}
