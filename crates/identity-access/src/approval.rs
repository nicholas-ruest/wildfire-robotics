//! Purpose- and payload-bound approvals (`IA-INV-004`).

use crate::grant::GrantScope;
use shared_kernel::{ContentDigest, PrincipalId, TimeWindow, UtcInstant};
use std::collections::BTreeSet;
use thiserror::Error;

const MAX_TEXT_BYTES: usize = 256;

/// Stable approval identifier.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ApprovalId(String);

impl ApprovalId {
    /// Validates an approval identifier.
    pub fn parse(value: impl Into<String>) -> Result<Self, ApprovalError> {
        let value = value.into();
        validate_token(&value).map_err(|()| ApprovalError::InvalidIdentifier)?;
        Ok(Self(value))
    }

    /// Returns the identifier text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable action purpose to which an approval is cryptographically bound.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ApprovalPurpose(String);

impl ApprovalPurpose {
    /// Validates a purpose such as `payload.release`.
    pub fn parse(value: impl Into<String>) -> Result<Self, ApprovalError> {
        let value = value.into();
        validate_token(&value).map_err(|()| ApprovalError::InvalidPurpose)?;
        Ok(Self(value))
    }

    /// Returns the stable purpose.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Whether an approved authorization may be consumed once or repeatedly until expiry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApprovalUse {
    /// First successful consumption is terminal.
    SingleUse,
    /// May be checked repeatedly inside its original immutable bounds.
    Reusable,
}

/// Signed decision bytes supplied to a cryptographic verification port.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedDecision {
    /// Principal claiming to make the decision.
    pub approver: PrincipalId,
    /// Identifier of the verification key.
    pub key_id: String,
    /// Detached signature; its algorithm and length are policy controlled by the verifier.
    pub signature: Vec<u8>,
}

/// Cryptographic boundary for unforgeable approval decisions.
pub trait DecisionVerifier {
    /// Adapter-specific verification failure.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Verifies a decision over the supplied canonical binding digest.
    fn verify(&self, decision: &SignedDecision, binding: ContentDigest) -> Result<(), Self::Error>;
}

/// Closed Approval lifecycle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApprovalState {
    /// Created but not yet accepting decisions.
    Requested,
    /// Accepting independently verified decisions.
    Pending,
    /// Required independent approvals were collected.
    Approved {
        /// Distinct independently verified approving principals.
        approvers: BTreeSet<PrincipalId>,
    },
    /// At least one authorized approver rejected the request.
    Rejected {
        /// Authorized principal that rejected the request.
        approver: PrincipalId,
        /// Attributable rejection reason.
        reason: String,
    },
    /// A single-use approval has been consumed.
    Consumed {
        /// Exact time of the successful single use.
        consumed_at: UtcInstant,
    },
    /// Approval validity ended before consumption.
    Expired,
}

/// Approval aggregate enforcing purpose, scope, payload, expiry, and replay bounds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Approval {
    id: ApprovalId,
    requested_by: PrincipalId,
    purpose: ApprovalPurpose,
    payload_digest: ContentDigest,
    scope: GrantScope,
    validity: TimeWindow,
    policy_version: String,
    required_approvers: u8,
    use_policy: ApprovalUse,
    state: ApprovalState,
    decisions: BTreeSet<PrincipalId>,
}

impl Approval {
    /// Creates a purpose- and payload-bound approval request.
    #[allow(clippy::too_many_arguments)]
    pub fn request(
        id: ApprovalId,
        requested_by: PrincipalId,
        purpose: ApprovalPurpose,
        payload_digest: ContentDigest,
        scope: GrantScope,
        validity: TimeWindow,
        policy_version: impl Into<String>,
        required_approvers: u8,
        use_policy: ApprovalUse,
    ) -> Result<Self, ApprovalError> {
        if required_approvers == 0 || required_approvers > 8 {
            return Err(ApprovalError::InvalidApproverCount);
        }
        let policy_version = policy_version.into();
        validate_token(&policy_version).map_err(|()| ApprovalError::InvalidPolicyVersion)?;
        Ok(Self {
            id,
            requested_by,
            purpose,
            payload_digest,
            scope,
            validity,
            policy_version,
            required_approvers,
            use_policy,
            state: ApprovalState::Requested,
            decisions: BTreeSet::new(),
        })
    }

    /// Opens the request for decisions.
    pub fn start(&mut self) -> Result<(), ApprovalError> {
        if self.state != ApprovalState::Requested {
            return Err(ApprovalError::InvalidTransition);
        }
        self.state = ApprovalState::Pending;
        Ok(())
    }

    /// Records one cryptographically verified, independently authorized approval.
    pub fn approve<V: DecisionVerifier>(
        &mut self,
        decision: &SignedDecision,
        approver_authorized: bool,
        binding_digest: ContentDigest,
        verifier: &V,
        now: UtcInstant,
    ) -> Result<(), ApprovalError> {
        if self.state != ApprovalState::Pending {
            return Err(ApprovalError::InvalidTransition);
        }
        if !self.validity.contains(now) {
            return Err(ApprovalError::Expired);
        }
        if !approver_authorized {
            return Err(ApprovalError::ApproverUnauthorized);
        }
        if decision.approver == self.requested_by {
            return Err(ApprovalError::SeparationOfDuties);
        }
        if validate_token(&decision.key_id).is_err() || decision.signature.is_empty() {
            return Err(ApprovalError::InvalidSignature);
        }
        if binding_digest != self.binding_digest() {
            return Err(ApprovalError::BindingMismatch);
        }
        verifier
            .verify(decision, binding_digest)
            .map_err(|_| ApprovalError::InvalidSignature)?;
        if !self.decisions.insert(decision.approver) {
            return Err(ApprovalError::DuplicateDecision);
        }
        if self.decisions.len() >= usize::from(self.required_approvers) {
            self.state = ApprovalState::Approved {
                approvers: self.decisions.clone(),
            };
        }
        Ok(())
    }

    /// Records an authorized rejection, making the request terminal.
    pub fn reject(
        &mut self,
        approver: PrincipalId,
        approver_authorized: bool,
        reason: impl Into<String>,
    ) -> Result<(), ApprovalError> {
        if self.state != ApprovalState::Pending || !approver_authorized {
            return Err(ApprovalError::InvalidTransition);
        }
        let reason = reason.into();
        if reason.trim().is_empty() || reason.len() > MAX_TEXT_BYTES {
            return Err(ApprovalError::InvalidRejectionReason);
        }
        self.state = ApprovalState::Rejected { approver, reason };
        Ok(())
    }

    /// Checks all immutable bindings and consumes a single-use approval atomically in memory.
    pub fn consume(
        &mut self,
        purpose: &ApprovalPurpose,
        payload_digest: ContentDigest,
        scope: &GrantScope,
        policy_version: &str,
        now: UtcInstant,
    ) -> Result<(), ApprovalError> {
        if !matches!(self.state, ApprovalState::Approved { .. }) {
            return Err(ApprovalError::NotApproved);
        }
        if !self.validity.contains(now) {
            self.state = ApprovalState::Expired;
            return Err(ApprovalError::Expired);
        }
        if purpose != &self.purpose
            || payload_digest != self.payload_digest
            || scope != &self.scope
            || policy_version != self.policy_version
        {
            return Err(ApprovalError::BindingMismatch);
        }
        if self.use_policy == ApprovalUse::SingleUse {
            self.state = ApprovalState::Consumed { consumed_at: now };
        }
        Ok(())
    }

    /// Expires a non-terminal approval after its validity interval.
    pub fn expire(&mut self, now: UtcInstant) -> Result<(), ApprovalError> {
        if now < self.validity.ends_at()
            || matches!(
                self.state,
                ApprovalState::Consumed { .. }
                    | ApprovalState::Rejected { .. }
                    | ApprovalState::Expired
            )
        {
            return Err(ApprovalError::InvalidTransition);
        }
        self.state = ApprovalState::Expired;
        Ok(())
    }

    /// Canonical binding digest used by the verifier. This is deterministic and domain-local.
    #[must_use]
    pub fn binding_digest(&self) -> ContentDigest {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        {
            let mut bind = |bytes: &[u8]| {
                hasher.update(u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_be_bytes());
                hasher.update(bytes);
            };
            bind(self.id.as_str().as_bytes());
            bind(self.requested_by.to_string().as_bytes());
            bind(self.purpose.as_str().as_bytes());
            bind(self.payload_digest.as_bytes());
            bind(self.scope.tenant_id().to_string().as_bytes());
            if let Some(incident_id) = self.scope.incident_id() {
                bind(incident_id.to_string().as_bytes());
            }
            if let Some(resource) = self.scope.resource() {
                bind(resource.as_bytes());
            }
            if let Some(geography) = self.scope.geography_digest() {
                bind(geography.as_bytes());
            }
            let starts_at = self.validity.starts_at().get();
            bind(starts_at.timestamp().to_be_bytes().as_slice());
            bind(starts_at.timestamp_subsec_nanos().to_be_bytes().as_slice());
            let ends_at = self.validity.ends_at().get();
            bind(ends_at.timestamp().to_be_bytes().as_slice());
            bind(ends_at.timestamp_subsec_nanos().to_be_bytes().as_slice());
            bind(self.policy_version.as_bytes());
            bind(&[self.required_approvers]);
            bind(&[match self.use_policy {
                ApprovalUse::SingleUse => 1,
                ApprovalUse::Reusable => 2,
            }]);
        }
        ContentDigest::from_sha256_bytes(hasher.finalize().into())
    }

    /// Current aggregate state.
    #[must_use]
    pub const fn state(&self) -> &ApprovalState {
        &self.state
    }
    /// Immutable purpose.
    #[must_use]
    pub const fn purpose(&self) -> &ApprovalPurpose {
        &self.purpose
    }
    /// Immutable payload digest.
    #[must_use]
    pub const fn payload_digest(&self) -> ContentDigest {
        self.payload_digest
    }
    /// Immutable scope.
    #[must_use]
    pub const fn scope(&self) -> &GrantScope {
        &self.scope
    }
}

/// Stable approval failures.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ApprovalError {
    /// Approval identifier is malformed.
    #[error("invalid approval identifier")]
    InvalidIdentifier,
    /// Purpose is missing or malformed.
    #[error("invalid approval purpose")]
    InvalidPurpose,
    /// Required approver count is outside the supported bound.
    #[error("approval requires 1 to 8 approvers")]
    InvalidApproverCount,
    /// Policy version is missing or malformed.
    #[error("invalid approval policy version")]
    InvalidPolicyVersion,
    /// Lifecycle transition is illegal.
    #[error("invalid approval transition")]
    InvalidTransition,
    /// Approver lacks authority for this decision.
    #[error("approval decision is unauthorized")]
    ApproverUnauthorized,
    /// Requester attempted to approve their own action.
    #[error("approval violates separation of duties")]
    SeparationOfDuties,
    /// Signature verification failed.
    #[error("approval decision signature is invalid")]
    InvalidSignature,
    /// Same principal attempted a duplicate decision.
    #[error("duplicate approval decision")]
    DuplicateDecision,
    /// Purpose, payload, scope, time, or policy binding differs.
    #[error("approval binding mismatch")]
    BindingMismatch,
    /// Approval expired.
    #[error("approval expired")]
    Expired,
    /// Approval has not reached approved state or was already consumed.
    #[error("approval is not consumable")]
    NotApproved,
    /// Rejection reason is invalid.
    #[error("invalid approval rejection reason")]
    InvalidRejectionReason,
}

fn validate_token(value: &str) -> Result<(), ()> {
    if value.is_empty()
        || value.len() > MAX_TEXT_BYTES
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b':' | b'_' | b'-' | b'/')
        })
    {
        return Err(());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use shared_kernel::{TenantId, TimeWindow};
    use std::io;

    struct AcceptSignature;
    impl DecisionVerifier for AcceptSignature {
        type Error = io::Error;
        fn verify(
            &self,
            decision: &SignedDecision,
            _binding: ContentDigest,
        ) -> Result<(), Self::Error> {
            if decision.signature == b"valid" {
                Ok(())
            } else {
                Err(io::Error::other("invalid signature"))
            }
        }
    }
    fn principal(suffix: u8) -> Result<PrincipalId, Box<dyn std::error::Error>> {
        Ok(PrincipalId::parse(&format!(
            "00000000-0000-4000-8000-{suffix:012}"
        ))?)
    }
    fn instant(seconds: i64) -> UtcInstant {
        UtcInstant::new(
            DateTime::<Utc>::from_timestamp(seconds, 0)
                .unwrap_or_else(|| unreachable!("valid timestamp")),
        )
    }
    fn approval() -> Result<Approval, Box<dyn std::error::Error>> {
        Ok(Approval::request(
            ApprovalId::parse("approval-1")?,
            principal(1)?,
            ApprovalPurpose::parse("payload.release")?,
            ContentDigest::from_sha256_bytes([7; 32]),
            GrantScope::tenant(TenantId::parse("00000000-0000-4000-8000-000000000101")?),
            TimeWindow::new(instant(100), instant(200))?,
            "policy-7",
            1,
            ApprovalUse::SingleUse,
        )?)
    }

    #[test]
    fn purpose_payload_and_scope_cannot_be_replayed() -> Result<(), Box<dyn std::error::Error>> {
        let mut approval = approval()?;
        approval.start()?;
        let decision = SignedDecision {
            approver: principal(2)?,
            key_id: "key-1".into(),
            signature: b"valid".to_vec(),
        };
        approval.approve(
            &decision,
            true,
            approval.binding_digest(),
            &AcceptSignature,
            instant(120),
        )?;
        let original_scope = approval.scope().clone();
        let original_digest = approval.payload_digest();
        assert_eq!(
            approval.consume(
                &ApprovalPurpose::parse("mission.abort")?,
                original_digest,
                &original_scope,
                "policy-7",
                instant(130)
            ),
            Err(ApprovalError::BindingMismatch)
        );
        let purpose = approval.purpose().clone();
        let digest = approval.payload_digest();
        let scope = approval.scope().clone();
        approval.consume(&purpose, digest, &scope, "policy-7", instant(130))?;
        assert_eq!(
            approval.consume(&purpose, digest, &scope, "policy-7", instant(131)),
            Err(ApprovalError::NotApproved)
        );
        Ok(())
    }

    #[test]
    fn requester_cannot_supply_separated_approval() -> Result<(), Box<dyn std::error::Error>> {
        let mut approval = approval()?;
        approval.start()?;
        let decision = SignedDecision {
            approver: principal(1)?,
            key_id: "key-1".into(),
            signature: b"valid".to_vec(),
        };
        assert_eq!(
            approval.approve(
                &decision,
                true,
                approval.binding_digest(),
                &AcceptSignature,
                instant(120)
            ),
            Err(ApprovalError::SeparationOfDuties)
        );
        Ok(())
    }
}
