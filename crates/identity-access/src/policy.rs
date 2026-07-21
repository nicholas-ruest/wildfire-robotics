//! Default-deny authorization policy core (ADR-035).

use crate::{
    approval::{Approval, ApprovalPurpose},
    grant::{GrantScope, RoleGrant},
};
use shared_kernel::{ContentDigest, PrincipalId, UtcInstant};

/// Authentication evidence. Possessing this evidence never grants authorization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthenticationContext {
    /// Uniquely authenticated principal.
    pub principal: PrincipalId,
    /// Time at which authentication completed.
    pub authenticated_at: UtcInstant,
    /// Exclusive credential expiry.
    pub expires_at: UtcInstant,
    /// Authenticator assurance level determined by the identity adapter.
    pub assurance: AuthenticatorAssurance,
}

/// Strength of the completed authentication ceremony.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AuthenticatorAssurance {
    /// Hardware or workload identity alone.
    Basic,
    /// Multi-factor or equivalent step-up authentication.
    StepUp,
    /// Hardware-backed step-up under an approved trust domain.
    HardwareBacked,
}

/// Approval required for a particular authorization request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequiredApproval {
    /// Exact purpose.
    pub purpose: ApprovalPurpose,
    /// Exact canonical payload digest.
    pub payload_digest: ContentDigest,
}

/// Explicit emergency-access context. It narrows and audits authority; it never creates it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreakGlassContext {
    /// Stable incident/ticket reference.
    pub ticket: String,
    /// Human-readable attributable reason.
    pub reason: String,
    /// Exclusive emergency-access expiry.
    pub expires_at: UtcInstant,
}

/// Complete inputs to one local policy decision.
pub struct AuthorizationRequest<'a> {
    /// Verified authentication, still separate from grants.
    pub authentication: &'a AuthenticationContext,
    /// Requested permission.
    pub permission: &'a str,
    /// Exact requested scope.
    pub scope: &'a GrantScope,
    /// Trusted current UTC supplied by the application clock port.
    pub now: UtcInstant,
    /// Required signed policy bundle version.
    pub policy_version: &'a str,
    /// Minimum authentication strength for this action.
    pub minimum_assurance: AuthenticatorAssurance,
    /// Optional purpose/payload-bound approval requirement.
    pub required_approval: Option<RequiredApproval>,
    /// Optional explicitly enabled emergency-access request.
    pub break_glass: Option<BreakGlassContext>,
}

/// Obligations that must be durably fulfilled with an allow decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationObligation {
    /// Persist complete decision inputs/outcome on the independent audit path.
    RecordAudit,
    /// Notify the configured security/operations channel immediately.
    NotifyBreakGlass,
    /// Create a bounded retrospective human review task.
    RetrospectiveReview,
    /// Independent physical safety controls remain mandatory.
    PreserveIndependentSafetyControls,
}

/// Stable reason for an allow or deny decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionReason {
    /// A current grant authorized the exact request.
    Granted,
    /// Authentication evidence was absent, expired, or below required assurance.
    AuthenticationInsufficient,
    /// No current least-privilege grant matched every request dimension.
    NoMatchingGrant,
    /// Required approval was missing, invalid, expired, or already consumed.
    ApprovalRequired,
    /// Break-glass input was malformed, expired, or disabled by policy.
    BreakGlassDenied,
}

/// Explainable local policy result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationDecision {
    /// Default-deny outcome.
    pub allowed: bool,
    /// Policy version evaluated.
    pub policy_version: String,
    /// Stable reason.
    pub reason: DecisionReason,
    /// Mandatory post-decision actions.
    pub obligations: Vec<AuthorizationObligation>,
}

/// Versioned default-deny policy evaluator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationPolicy {
    version: String,
    break_glass_enabled: bool,
}

impl AuthorizationPolicy {
    /// Creates a policy evaluator for one already verified signed bundle.
    #[must_use]
    pub fn new(version: impl Into<String>, break_glass_enabled: bool) -> Self {
        Self {
            version: version.into(),
            break_glass_enabled,
        }
    }

    /// Evaluates grants and atomically consumes a required single-use approval.
    #[must_use]
    pub fn authorize(
        &self,
        request: AuthorizationRequest<'_>,
        grants: &[RoleGrant],
        approval: Option<&mut Approval>,
    ) -> AuthorizationDecision {
        let deny = |reason| AuthorizationDecision {
            allowed: false,
            policy_version: self.version.clone(),
            reason,
            obligations: vec![AuthorizationObligation::RecordAudit],
        };
        if request.policy_version != self.version
            || request.now < request.authentication.authenticated_at
            || request.now >= request.authentication.expires_at
            || request.authentication.assurance < request.minimum_assurance
        {
            return deny(DecisionReason::AuthenticationInsufficient);
        }
        let granted = grants.iter().any(|grant| {
            grant.authorizes(
                request.authentication.principal,
                request.permission,
                request.scope,
                request.now,
                &self.version,
            )
        });
        if !granted {
            return deny(DecisionReason::NoMatchingGrant);
        }
        if let Some(required) = request.required_approval {
            let consumed = approval.is_some_and(|candidate| {
                candidate
                    .consume(
                        &required.purpose,
                        required.payload_digest,
                        request.scope,
                        &self.version,
                        request.now,
                    )
                    .is_ok()
            });
            if !consumed {
                return deny(DecisionReason::ApprovalRequired);
            }
        }
        let mut obligations = vec![AuthorizationObligation::RecordAudit];
        if let Some(emergency) = request.break_glass {
            let valid = self.break_glass_enabled
                && request.authentication.assurance >= AuthenticatorAssurance::StepUp
                && request.now < emergency.expires_at
                && !emergency.ticket.trim().is_empty()
                && !emergency.reason.trim().is_empty()
                && emergency.ticket.len() <= 128
                && emergency.reason.len() <= 512;
            if !valid {
                return deny(DecisionReason::BreakGlassDenied);
            }
            obligations.extend([
                AuthorizationObligation::NotifyBreakGlass,
                AuthorizationObligation::RetrospectiveReview,
                AuthorizationObligation::PreserveIndependentSafetyControls,
            ]);
        }
        AuthorizationDecision {
            allowed: true,
            policy_version: self.version.clone(),
            reason: DecisionReason::Granted,
            obligations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grant::{RoleGrant, RoleGrantId};
    use chrono::{DateTime, Utc};
    use shared_kernel::{TenantId, TimeWindow};

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
    fn active_grant() -> Result<(RoleGrant, GrantScope), Box<dyn std::error::Error>> {
        let scope = GrantScope::tenant(TenantId::parse("00000000-0000-4000-8000-000000000101")?);
        let mut grant = RoleGrant::request(
            RoleGrantId::parse("grant-policy")?,
            principal(1)?,
            principal(2)?,
            scope.clone(),
            ["mission.authorize".into()],
            TimeWindow::new(instant(100), instant(300))?,
            "policy-7",
            true,
        )?;
        grant.approve(principal(3)?, true)?;
        grant.activate(principal(3)?, instant(110))?;
        Ok((grant, scope))
    }

    #[test]
    fn authentication_never_implies_authorization() -> Result<(), Box<dyn std::error::Error>> {
        let (_, scope) = active_grant()?;
        let authentication = AuthenticationContext {
            principal: principal(1)?,
            authenticated_at: instant(100),
            expires_at: instant(200),
            assurance: AuthenticatorAssurance::HardwareBacked,
        };
        let decision = AuthorizationPolicy::new("policy-7", false).authorize(
            AuthorizationRequest {
                authentication: &authentication,
                permission: "mission.authorize",
                scope: &scope,
                now: instant(150),
                policy_version: "policy-7",
                minimum_assurance: AuthenticatorAssurance::Basic,
                required_approval: None,
                break_glass: None,
            },
            &[],
            None,
        );
        assert_eq!(decision.reason, DecisionReason::NoMatchingGrant);
        assert!(!decision.allowed);
        Ok(())
    }

    #[test]
    fn break_glass_keeps_grant_requirement_and_adds_all_obligations()
    -> Result<(), Box<dyn std::error::Error>> {
        let (grant, scope) = active_grant()?;
        let authentication = AuthenticationContext {
            principal: principal(1)?,
            authenticated_at: instant(100),
            expires_at: instant(200),
            assurance: AuthenticatorAssurance::HardwareBacked,
        };
        let decision = AuthorizationPolicy::new("policy-7", true).authorize(
            AuthorizationRequest {
                authentication: &authentication,
                permission: "mission.authorize",
                scope: &scope,
                now: instant(150),
                policy_version: "policy-7",
                minimum_assurance: AuthenticatorAssurance::StepUp,
                required_approval: None,
                break_glass: Some(BreakGlassContext {
                    ticket: "SEC-42".into(),
                    reason: "restore expiring safety control".into(),
                    expires_at: instant(160),
                }),
            },
            &[grant],
            None,
        );
        assert!(decision.allowed);
        assert_eq!(decision.obligations.len(), 4);
        Ok(())
    }
}
