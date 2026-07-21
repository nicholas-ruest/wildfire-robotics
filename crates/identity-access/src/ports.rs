//! Replaceable identity, attestation, and credential trust boundaries.

use crate::domain::{AttestationEvidence, DeviceId, IssuedCredential, VerifiedProvenance};
use chrono::{DateTime, Duration, Utc};
use shared_kernel::PrincipalId;
use std::{future::Future, pin::Pin};
use thiserror::Error;

/// Allocation-free-at-call-site asynchronous port result.
pub type PortFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, TrustPortError>> + Send + 'a>>;

/// Human identity-provider and step-up MFA boundary.
pub trait HumanIdentityVerifier: Send + Sync {
    /// Verifies an external assertion and returns attributable provenance.
    fn verify<'a>(
        &'a self,
        assertion: &'a [u8],
        minimum_assurance: &'a str,
    ) -> PortFuture<'a, VerifiedProvenance>;
}

/// Workload identity and workload-attestation boundary.
pub trait WorkloadIdentityVerifier: Send + Sync {
    /// Verifies workload selector evidence without accepting network location as identity.
    fn verify<'a>(
        &'a self,
        attestation: &'a [u8],
        expected_trust_domain: &'a str,
    ) -> PortFuture<'a, VerifiedProvenance>;
}

/// Hardware-attestation verifier boundary.
pub trait HardwareAttestationVerifier: Send + Sync {
    /// Verifies manufacturer/onboarding evidence, measurements, revocation, and trust root.
    fn verify<'a>(
        &'a self,
        device_evidence: &'a [u8],
        evaluated_at: DateTime<Utc>,
    ) -> PortFuture<'a, AttestationEvidence>;
}

/// Managed PKI/HSM credential issuer. Returned metadata contains no private key.
pub trait CredentialIssuer: Send + Sync {
    /// Issues a purpose-bound credential for an authenticated principal.
    fn issue<'a>(
        &'a self,
        principal_id: PrincipalId,
        purpose: &'a str,
        requested_lifetime: Duration,
    ) -> PortFuture<'a, IssuedCredential>;
    /// Revokes a credential and durably records the reason.
    fn revoke<'a>(&'a self, credential_id: &'a str, reason: &'a str) -> PortFuture<'a, ()>;
}

/// Trust-root inventory used during automated rollover and compromise response.
pub trait TrustRootProvider: Send + Sync {
    /// Returns whether a root version is accepted at the evaluated time and not revoked.
    fn accepts<'a>(
        &'a self,
        trust_root_version: &'a str,
        evaluated_at: DateTime<Utc>,
    ) -> PortFuture<'a, bool>;
}

/// Atomic identity namespace reservation required by `IA-INV-001`.
pub trait IdentityUniqueness: Send + Sync {
    /// Reserves a provider subject for exactly one principal, rejecting shared reuse.
    fn reserve_principal_subject<'a>(
        &'a self,
        provider: &'a str,
        subject: &'a str,
        principal_id: PrincipalId,
    ) -> PortFuture<'a, ()>;

    /// Reserves manufacturer/device provenance for exactly one device identity.
    fn reserve_device_provenance<'a>(
        &'a self,
        provenance_id: &'a str,
        device_id: DeviceId,
    ) -> PortFuture<'a, ()>;
}

/// Fail-closed trust adapter failures.
#[derive(Debug, Error)]
pub enum TrustPortError {
    /// Evidence is malformed, mismatched, or cryptographically invalid.
    #[error("trust evidence is invalid")]
    InvalidEvidence,
    /// Identity, key, software, configuration, or trust root is revoked.
    #[error("trust evidence is revoked")]
    Revoked,
    /// Trust service is unavailable; callers must deny trust expansion.
    #[error("trust service is unavailable")]
    Unavailable,
    /// Adapter rejected the requested credential lifetime or purpose.
    #[error("credential request violates issuance policy")]
    IssuancePolicy,
    /// Adapter failure safe for audit but not containing secret material.
    #[error("trust provider failed: {0}")]
    Provider(String),
}
