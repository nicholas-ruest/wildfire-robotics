//! Least-privilege role grants (`IA-INV-002`).

use shared_kernel::{ContentDigest, IncidentId, PrincipalId, TenantId, TimeWindow, UtcInstant};
use std::collections::BTreeSet;
use thiserror::Error;

const MAX_TOKEN_BYTES: usize = 128;
const MAX_PERMISSIONS: usize = 64;

/// Stable identifier for a role grant.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RoleGrantId(String);

impl RoleGrantId {
    /// Validates a stable identifier supplied by the application boundary.
    pub fn parse(value: impl Into<String>) -> Result<Self, GrantError> {
        let value = value.into();
        validate_token(&value).map_err(|()| GrantError::InvalidIdentifier)?;
        Ok(Self(value))
    }

    /// Returns the identifier text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Explicit authorization boundary. Omitted dimensions are not fabricated.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrantScope {
    tenant_id: TenantId,
    incident_id: Option<IncidentId>,
    resource: Option<String>,
    geography_digest: Option<ContentDigest>,
}

impl GrantScope {
    /// Creates a tenant-wide scope.
    #[must_use]
    pub const fn tenant(tenant_id: TenantId) -> Self {
        Self {
            tenant_id,
            incident_id: None,
            resource: None,
            geography_digest: None,
        }
    }

    /// Narrows this scope to one incident.
    #[must_use]
    pub const fn within_incident(mut self, incident_id: IncidentId) -> Self {
        self.incident_id = Some(incident_id);
        self
    }

    /// Narrows this scope to one stable resource name.
    pub fn for_resource(mut self, resource: impl Into<String>) -> Result<Self, GrantError> {
        let resource = resource.into();
        validate_token(&resource).map_err(|()| GrantError::InvalidScope)?;
        self.resource = Some(resource);
        Ok(self)
    }

    /// Narrows this scope to content-addressed geography.
    #[must_use]
    pub const fn within_geography(mut self, digest: ContentDigest) -> Self {
        self.geography_digest = Some(digest);
        self
    }

    /// Returns the tenant boundary.
    #[must_use]
    pub const fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }

    /// Returns the optional incident boundary.
    #[must_use]
    pub const fn incident_id(&self) -> Option<IncidentId> {
        self.incident_id
    }

    /// Returns the optional resource boundary.
    #[must_use]
    pub fn resource(&self) -> Option<&str> {
        self.resource.as_deref()
    }

    /// Returns the optional geography boundary.
    #[must_use]
    pub const fn geography_digest(&self) -> Option<ContentDigest> {
        self.geography_digest
    }

    /// Whether this grant scope contains the requested scope without widening it.
    #[must_use]
    pub fn contains(&self, requested: &Self) -> bool {
        self.tenant_id == requested.tenant_id
            && optional_dimension_contains(self.incident_id, requested.incident_id)
            && optional_text_contains(self.resource.as_deref(), requested.resource.as_deref())
            && optional_dimension_contains(self.geography_digest, requested.geography_digest)
    }
}

fn optional_dimension_contains<T: Eq + Copy>(granted: Option<T>, requested: Option<T>) -> bool {
    granted.is_none_or(|value| requested == Some(value))
}

fn optional_text_contains(granted: Option<&str>, requested: Option<&str>) -> bool {
    granted.is_none_or(|value| requested == Some(value))
}

/// Closed lifecycle for a role grant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RoleGrantState {
    /// Awaiting an authorized approval.
    Requested,
    /// Approved but not yet activated.
    Approved {
        /// Independently authorized approving principal.
        approver: PrincipalId,
    },
    /// Effective subject to its time, scope, permission, and policy bounds.
    Active {
        /// Principal that activated the approved grant.
        activated_by: PrincipalId,
    },
    /// Validity ended.
    Expired,
    /// Explicitly revoked.
    Revoked {
        /// Attributable revocation reason.
        reason: String,
    },
}

/// `RoleGrant` aggregate enforcing `IA-INV-002`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleGrant {
    id: RoleGrantId,
    subject: PrincipalId,
    requested_by: PrincipalId,
    scope: GrantScope,
    permissions: BTreeSet<String>,
    validity: TimeWindow,
    policy_version: String,
    separation_required: bool,
    state: RoleGrantState,
}

impl RoleGrant {
    /// Requests a scoped, time-bounded, policy-versioned least-privilege grant.
    #[allow(clippy::too_many_arguments)]
    pub fn request(
        id: RoleGrantId,
        subject: PrincipalId,
        requested_by: PrincipalId,
        scope: GrantScope,
        permissions: impl IntoIterator<Item = String>,
        validity: TimeWindow,
        policy_version: impl Into<String>,
        separation_required: bool,
    ) -> Result<Self, GrantError> {
        let permissions: BTreeSet<_> = permissions.into_iter().collect();
        if permissions.is_empty()
            || permissions.len() > MAX_PERMISSIONS
            || permissions
                .iter()
                .any(|permission| validate_token(permission).is_err())
        {
            return Err(GrantError::InvalidPermissions);
        }
        let policy_version = policy_version.into();
        validate_token(&policy_version).map_err(|()| GrantError::InvalidPolicyVersion)?;
        Ok(Self {
            id,
            subject,
            requested_by,
            scope,
            permissions,
            validity,
            policy_version,
            separation_required,
            state: RoleGrantState::Requested,
        })
    }

    /// Records approval by an independently authorized principal.
    pub fn approve(
        &mut self,
        approver: PrincipalId,
        approver_authorized: bool,
    ) -> Result<(), GrantError> {
        if self.state != RoleGrantState::Requested {
            return Err(GrantError::InvalidTransition);
        }
        if !approver_authorized {
            return Err(GrantError::ApproverUnauthorized);
        }
        if self.separation_required && (approver == self.subject || approver == self.requested_by) {
            return Err(GrantError::SeparationOfDuties);
        }
        self.state = RoleGrantState::Approved { approver };
        Ok(())
    }

    /// Activates an approved grant inside its validity interval.
    pub fn activate(&mut self, actor: PrincipalId, now: UtcInstant) -> Result<(), GrantError> {
        if !matches!(self.state, RoleGrantState::Approved { .. }) {
            return Err(GrantError::InvalidTransition);
        }
        if !self.validity.contains(now) {
            return Err(GrantError::OutsideValidity);
        }
        self.state = RoleGrantState::Active {
            activated_by: actor,
        };
        Ok(())
    }

    /// Expires the grant at or after the exclusive validity end.
    pub fn expire(&mut self, now: UtcInstant) -> Result<(), GrantError> {
        if matches!(
            self.state,
            RoleGrantState::Expired | RoleGrantState::Revoked { .. }
        ) {
            return Err(GrantError::InvalidTransition);
        }
        if now < self.validity.ends_at() {
            return Err(GrantError::OutsideValidity);
        }
        self.state = RoleGrantState::Expired;
        Ok(())
    }

    /// Revokes any non-terminal grant for an attributable reason.
    pub fn revoke(&mut self, reason: impl Into<String>) -> Result<(), GrantError> {
        if matches!(
            self.state,
            RoleGrantState::Expired | RoleGrantState::Revoked { .. }
        ) {
            return Err(GrantError::InvalidTransition);
        }
        let reason = reason.into();
        if reason.trim().is_empty() || reason.len() > 512 {
            return Err(GrantError::InvalidRevocationReason);
        }
        self.state = RoleGrantState::Revoked { reason };
        Ok(())
    }

    /// Whether this active grant authorizes the exact request under the current policy.
    #[must_use]
    pub fn authorizes(
        &self,
        subject: PrincipalId,
        permission: &str,
        requested_scope: &GrantScope,
        now: UtcInstant,
        required_policy_version: &str,
    ) -> bool {
        matches!(self.state, RoleGrantState::Active { .. })
            && self.subject == subject
            && self.permissions.contains(permission)
            && self.scope.contains(requested_scope)
            && self.validity.contains(now)
            && self.policy_version == required_policy_version
    }

    /// Returns the aggregate identifier.
    #[must_use]
    pub const fn id(&self) -> &RoleGrantId {
        &self.id
    }
    /// Returns the grantee.
    #[must_use]
    pub const fn subject(&self) -> PrincipalId {
        self.subject
    }
    /// Returns the current lifecycle state.
    #[must_use]
    pub const fn state(&self) -> &RoleGrantState {
        &self.state
    }
    /// Returns the immutable scope.
    #[must_use]
    pub const fn scope(&self) -> &GrantScope {
        &self.scope
    }
    /// Returns the policy version bound at request time.
    #[must_use]
    pub fn policy_version(&self) -> &str {
        &self.policy_version
    }
}

/// Stable role-grant failures.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum GrantError {
    /// Aggregate identifier is malformed.
    #[error("invalid role grant identifier")]
    InvalidIdentifier,
    /// Scope contains malformed or ambiguous data.
    #[error("invalid grant scope")]
    InvalidScope,
    /// Permissions are empty, excessive, duplicated down to an empty set, or malformed.
    #[error("invalid least-privilege permission set")]
    InvalidPermissions,
    /// Policy version is missing or malformed.
    #[error("invalid policy version")]
    InvalidPolicyVersion,
    /// Lifecycle transition is illegal.
    #[error("invalid role grant transition")]
    InvalidTransition,
    /// Approver lacks authority to approve grants.
    #[error("grant approver is unauthorized")]
    ApproverUnauthorized,
    /// Required approver independence was violated.
    #[error("grant violates separation of duties")]
    SeparationOfDuties,
    /// Operation occurred outside the grant's validity interval.
    #[error("grant operation is outside its validity interval")]
    OutsideValidity,
    /// Revocation reason is missing or too long.
    #[error("invalid revocation reason")]
    InvalidRevocationReason,
}

fn validate_token(value: &str) -> Result<(), ()> {
    if value.is_empty()
        || value.len() > MAX_TOKEN_BYTES
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

    fn principal(suffix: u8) -> Result<PrincipalId, Box<dyn std::error::Error>> {
        Ok(PrincipalId::parse(&format!(
            "00000000-0000-4000-8000-{suffix:012}"
        ))?)
    }
    fn tenant() -> Result<TenantId, Box<dyn std::error::Error>> {
        Ok(TenantId::parse("00000000-0000-4000-8000-000000000101")?)
    }
    fn instant(seconds: i64) -> UtcInstant {
        UtcInstant::new(
            DateTime::<Utc>::from_timestamp(seconds, 0)
                .unwrap_or_else(|| unreachable!("valid timestamp")),
        )
    }
    fn grant(separation: bool) -> Result<RoleGrant, Box<dyn std::error::Error>> {
        Ok(RoleGrant::request(
            RoleGrantId::parse("grant-1")?,
            principal(1)?,
            principal(2)?,
            GrantScope::tenant(tenant()?),
            ["mission.authorize".to_owned()],
            TimeWindow::new(instant(100), instant(200))?,
            "policy-7",
            separation,
        )?)
    }

    #[test]
    fn self_approval_is_denied_when_separation_is_required()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut grant = grant(true)?;
        assert_eq!(
            grant.approve(principal(1)?, true),
            Err(GrantError::SeparationOfDuties)
        );
        assert_eq!(grant.state(), &RoleGrantState::Requested);
        Ok(())
    }

    #[test]
    fn authorization_requires_exact_subject_permission_scope_time_and_policy()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut grant = grant(true)?;
        grant.approve(principal(3)?, true)?;
        grant.activate(principal(4)?, instant(120))?;
        let scope = GrantScope::tenant(tenant()?);
        assert!(grant.authorizes(
            principal(1)?,
            "mission.authorize",
            &scope,
            instant(150),
            "policy-7"
        ));
        assert!(!grant.authorizes(
            principal(1)?,
            "mission.abort",
            &scope,
            instant(150),
            "policy-7"
        ));
        assert!(!grant.authorizes(
            principal(1)?,
            "mission.authorize",
            &scope,
            instant(200),
            "policy-7"
        ));
        assert!(!grant.authorizes(
            principal(1)?,
            "mission.authorize",
            &scope,
            instant(150),
            "policy-8"
        ));
        Ok(())
    }
}
