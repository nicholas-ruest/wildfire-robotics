//! Private-state authority aggregates implementing IC-INV-001 through IC-INV-005.
#![allow(missing_docs)]

use chrono::{DateTime, Utc};
use shared_kernel::{EntityId, TimeWindow};
use std::collections::BTreeSet;
use thiserror::Error;

fn bounded(value: impl Into<String>) -> Result<String, IncidentError> {
    let value = value.into();
    if value.trim().is_empty() || value.len() > 256 {
        Err(IncidentError::InvalidField)
    } else {
        Ok(value)
    }
}

/// Spatial, temporal, and capability ceiling. Subset comparison is the authority lattice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityEnvelope {
    zones: BTreeSet<String>,
    capabilities: BTreeSet<String>,
    validity: TimeWindow,
    maximum_resources: u32,
}
impl AuthorityEnvelope {
    pub fn new(
        zones: impl IntoIterator<Item = String>,
        capabilities: impl IntoIterator<Item = String>,
        validity: TimeWindow,
        maximum_resources: u32,
    ) -> Result<Self, IncidentError> {
        let zones = zones
            .into_iter()
            .map(bounded)
            .collect::<Result<BTreeSet<_>, _>>()?;
        let capabilities = capabilities
            .into_iter()
            .map(bounded)
            .collect::<Result<BTreeSet<_>, _>>()?;
        if zones.is_empty() || capabilities.is_empty() || maximum_resources == 0 {
            return Err(IncidentError::InvalidAuthority);
        }
        Ok(Self {
            zones,
            capabilities,
            validity,
            maximum_resources,
        })
    }
    #[must_use]
    pub fn contains(&self, other: &Self) -> bool {
        other.zones.is_subset(&self.zones)
            && other.capabilities.is_subset(&self.capabilities)
            && other.validity.starts_at() >= self.validity.starts_at()
            && other.validity.ends_at() <= self.validity.ends_at()
            && other.maximum_resources <= self.maximum_resources
    }
    #[must_use]
    pub fn active_at(&self, now: DateTime<Utc>) -> bool {
        self.validity.contains(now)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IncidentState {
    Draft,
    Active,
    Contained,
    Closed,
    Archived,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandAuthority {
    pub commander: EntityId,
    pub predecessor: Option<EntityId>,
    pub sequence: u64,
    pub effective_at: DateTime<Utc>,
    pub accepted_at: DateTime<Utc>,
    pub reason: String,
}

/// Incident aggregate with a single lineage head for command authority.
#[derive(Clone, Debug)]
pub struct Incident {
    id: EntityId,
    state: IncidentState,
    envelope: AuthorityEnvelope,
    command: CommandAuthority,
    version: u64,
}
impl Incident {
    pub fn open(
        id: EntityId,
        envelope: AuthorityEnvelope,
        commander: EntityId,
        at: DateTime<Utc>,
    ) -> Result<Self, IncidentError> {
        if !envelope.active_at(at) {
            return Err(IncidentError::ExpiredAuthority);
        }
        Ok(Self {
            id,
            state: IncidentState::Active,
            envelope,
            command: CommandAuthority {
                commander,
                predecessor: None,
                sequence: 1,
                effective_at: at,
                accepted_at: at,
                reason: "incident opened".into(),
            },
            version: 1,
        })
    }
    pub fn transfer_command(
        &mut self,
        expected: u64,
        from: &EntityId,
        to: EntityId,
        effective_at: DateTime<Utc>,
        accepted_at: DateTime<Utc>,
        reason: impl Into<String>,
    ) -> Result<(), IncidentError> {
        self.expect(expected)?;
        if &self.command.commander != from
            || effective_at > accepted_at
            || !self.envelope.active_at(effective_at)
        {
            return Err(IncidentError::AmbiguousAuthority);
        }
        let prior = self.command.commander.clone();
        self.command = CommandAuthority {
            commander: to,
            predecessor: Some(prior),
            sequence: self
                .command
                .sequence
                .checked_add(1)
                .ok_or(IncidentError::VersionExhausted)?,
            effective_at,
            accepted_at,
            reason: bounded(reason)?,
        };
        self.bump()
    }
    pub fn narrow_authority(
        &mut self,
        expected: u64,
        narrower: AuthorityEnvelope,
    ) -> Result<(), IncidentError> {
        self.expect(expected)?;
        if !self.envelope.contains(&narrower) {
            return Err(IncidentError::AuthorityExpansion);
        }
        self.envelope = narrower;
        self.bump()
    }
    pub fn contain(&mut self, expected: u64) -> Result<(), IncidentError> {
        self.transition(expected, IncidentState::Active, IncidentState::Contained)
    }
    pub fn close(&mut self, expected: u64) -> Result<(), IncidentError> {
        self.transition(expected, IncidentState::Contained, IncidentState::Closed)
    }
    pub fn archive(&mut self, expected: u64) -> Result<(), IncidentError> {
        self.transition(expected, IncidentState::Closed, IncidentState::Archived)
    }
    fn transition(
        &mut self,
        e: u64,
        from: IncidentState,
        to: IncidentState,
    ) -> Result<(), IncidentError> {
        self.expect(e)?;
        if self.state != from {
            return Err(IncidentError::InvalidTransition);
        }
        self.state = to;
        self.bump()
    }
    fn expect(&self, e: u64) -> Result<(), IncidentError> {
        if e == self.version {
            Ok(())
        } else {
            Err(IncidentError::ConcurrencyConflict)
        }
    }
    fn bump(&mut self) -> Result<(), IncidentError> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or(IncidentError::VersionExhausted)?;
        Ok(())
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub fn version(&self) -> u64 {
        self.version
    }
    #[must_use]
    pub fn envelope(&self) -> &AuthorityEnvelope {
        &self.envelope
    }
    #[must_use]
    pub fn commander(&self) -> &EntityId {
        &self.command.commander
    }
    #[must_use]
    pub fn is_active_at(&self, now: DateTime<Utc>) -> bool {
        self.state == IncidentState::Active && self.envelope.active_at(now)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PeriodState {
    Draft,
    Approved,
    Active,
    Expired,
    Closed,
}
#[derive(Clone, Debug)]
pub struct OperationalPeriod {
    id: EntityId,
    incident_id: EntityId,
    envelope: AuthorityEnvelope,
    commander: EntityId,
    state: PeriodState,
    objectives: BTreeSet<String>,
    restriction_set_version: u64,
    approved_by: Option<EntityId>,
    version: u64,
}
impl OperationalPeriod {
    #[must_use]
    pub fn define(
        id: EntityId,
        incident_id: EntityId,
        envelope: AuthorityEnvelope,
        commander: EntityId,
    ) -> Self {
        Self {
            id,
            incident_id,
            envelope,
            commander,
            state: PeriodState::Draft,
            objectives: BTreeSet::new(),
            restriction_set_version: 0,
            approved_by: None,
            version: 1,
        }
    }
    pub fn add_objective(&mut self, expected: u64, id: &EntityId) -> Result<(), IncidentError> {
        self.expect(expected)?;
        if self.state != PeriodState::Draft {
            return Err(IncidentError::InvalidTransition);
        }
        self.objectives.insert(id.to_string());
        self.bump()
    }
    pub fn approve(
        &mut self,
        expected: u64,
        approver: EntityId,
        restriction_set_version: u64,
    ) -> Result<(), IncidentError> {
        self.expect(expected)?;
        if self.state != PeriodState::Draft
            || approver == self.commander
            || self.objectives.is_empty()
            || restriction_set_version == 0
        {
            return Err(IncidentError::MissingApproval);
        }
        self.approved_by = Some(approver);
        self.restriction_set_version = restriction_set_version;
        self.state = PeriodState::Approved;
        self.bump()
    }
    pub fn activate(
        &mut self,
        expected: u64,
        incident: &Incident,
        now: DateTime<Utc>,
    ) -> Result<(), IncidentError> {
        self.expect(expected)?;
        if self.state != PeriodState::Approved
            || self.incident_id != *incident.id()
            || !incident.is_active_at(now)
            || !incident.envelope().contains(&self.envelope)
            || !self.envelope.active_at(now)
        {
            return Err(IncidentError::InvalidAuthority);
        }
        self.state = PeriodState::Active;
        self.bump()
    }
    pub fn expire_if_due(&mut self, now: DateTime<Utc>) -> Result<bool, IncidentError> {
        if self.state == PeriodState::Active && !self.envelope.active_at(now) {
            self.state = PeriodState::Expired;
            self.bump()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    fn expect(&self, e: u64) -> Result<(), IncidentError> {
        if e == self.version {
            Ok(())
        } else {
            Err(IncidentError::ConcurrencyConflict)
        }
    }
    fn bump(&mut self) -> Result<(), IncidentError> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or(IncidentError::VersionExhausted)?;
        Ok(())
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub fn version(&self) -> u64 {
        self.version
    }
    #[must_use]
    pub fn is_active_at(&self, now: DateTime<Utc>) -> bool {
        self.state == PeriodState::Active && self.envelope.active_at(now)
    }
    #[must_use]
    pub fn envelope(&self) -> &AuthorityEnvelope {
        &self.envelope
    }
    #[must_use]
    pub fn restriction_version(&self) -> u64 {
        self.restriction_set_version
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectiveState {
    Proposed,
    Approved,
    Active,
    Completed,
    Cancelled,
}
#[derive(Clone, Debug)]
pub struct Objective {
    id: EntityId,
    period_id: EntityId,
    description: String,
    priority: u8,
    state: ObjectiveState,
    approved_by: Option<EntityId>,
    version: u64,
}
impl Objective {
    pub fn propose(
        id: EntityId,
        period_id: EntityId,
        description: impl Into<String>,
        priority: u8,
    ) -> Result<Self, IncidentError> {
        if priority == 0 {
            return Err(IncidentError::InvalidField);
        }
        Ok(Self {
            id,
            period_id,
            description: bounded(description)?,
            priority,
            state: ObjectiveState::Proposed,
            approved_by: None,
            version: 1,
        })
    }
    pub fn approve(&mut self, expected: u64, approver: EntityId) -> Result<(), IncidentError> {
        if expected != self.version {
            return Err(IncidentError::ConcurrencyConflict);
        }
        if self.state != ObjectiveState::Proposed {
            return Err(IncidentError::InvalidTransition);
        }
        self.approved_by = Some(approver);
        self.state = ObjectiveState::Approved;
        self.version += 1;
        Ok(())
    }
    pub fn activate(&mut self, expected: u64) -> Result<(), IncidentError> {
        self.transition(expected, ObjectiveState::Approved, ObjectiveState::Active)
    }
    pub fn complete(&mut self, expected: u64) -> Result<(), IncidentError> {
        self.transition(expected, ObjectiveState::Active, ObjectiveState::Completed)
    }
    pub fn cancel(&mut self, expected: u64) -> Result<(), IncidentError> {
        if expected != self.version
            || matches!(
                self.state,
                ObjectiveState::Completed | ObjectiveState::Cancelled
            )
        {
            return Err(IncidentError::InvalidTransition);
        }
        self.state = ObjectiveState::Cancelled;
        self.version += 1;
        Ok(())
    }
    fn transition(
        &mut self,
        e: u64,
        f: ObjectiveState,
        t: ObjectiveState,
    ) -> Result<(), IncidentError> {
        if e != self.version {
            return Err(IncidentError::ConcurrencyConflict);
        }
        if self.state != f {
            return Err(IncidentError::InvalidTransition);
        }
        self.state = t;
        self.version += 1;
        Ok(())
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }
    #[must_use]
    pub const fn priority(&self) -> u8 {
        self.priority
    }
    #[must_use]
    pub fn period_id(&self) -> &EntityId {
        &self.period_id
    }
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.state == ObjectiveState::Active
    }
}

/// Resource/capability request only; deliberately has no actuator payload or vehicle command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceRequest {
    pub capability: String,
    pub quantity: u32,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssignmentState {
    Draft,
    Approved,
    Issued,
    Accepted,
    Completed,
    Revoked,
    Expired,
}
#[derive(Clone, Debug)]
pub struct Assignment {
    id: EntityId,
    incident_id: EntityId,
    period_id: EntityId,
    objective_id: EntityId,
    envelope: AuthorityEnvelope,
    request: ResourceRequest,
    constraints: BTreeSet<String>,
    state: AssignmentState,
    approver: Option<EntityId>,
    acknowledged_by: Option<EntityId>,
    restriction_version: u64,
    version: u64,
}
impl Assignment {
    pub fn draft(
        id: EntityId,
        incident_id: EntityId,
        period_id: EntityId,
        objective_id: EntityId,
        envelope: AuthorityEnvelope,
        request: ResourceRequest,
        constraints: impl IntoIterator<Item = String>,
    ) -> Result<Self, IncidentError> {
        if request.quantity == 0 || request.capability.trim().is_empty() {
            return Err(IncidentError::InvalidField);
        }
        let constraints = constraints
            .into_iter()
            .map(bounded)
            .collect::<Result<BTreeSet<_>, _>>()?;
        Ok(Self {
            id,
            incident_id,
            period_id,
            objective_id,
            envelope,
            request,
            constraints,
            state: AssignmentState::Draft,
            approver: None,
            acknowledged_by: None,
            restriction_version: 0,
            version: 1,
        })
    }
    pub fn approve(
        &mut self,
        expected: u64,
        issuer: &EntityId,
        approver: EntityId,
        incident: &Incident,
        period: &OperationalPeriod,
        objective: &Objective,
    ) -> Result<(), IncidentError> {
        self.expect(expected)?;
        if self.state != AssignmentState::Draft
            || &approver == issuer
            || self.incident_id != *incident.id()
            || self.period_id != *period.id()
            || self.objective_id != *objective.id()
            || !objective.is_active()
            || !incident.envelope().contains(&self.envelope)
            || !period.envelope().contains(&self.envelope)
            || self.request.quantity > self.envelope.maximum_resources
            || !self
                .envelope
                .capabilities
                .contains(&self.request.capability)
        {
            return Err(IncidentError::InvalidAuthority);
        }
        self.approver = Some(approver);
        self.restriction_version = period.restriction_version();
        self.state = AssignmentState::Approved;
        self.bump()
    }
    pub fn issue(
        &mut self,
        expected: u64,
        now: DateTime<Utc>,
        current_restriction_version: u64,
    ) -> Result<(), IncidentError> {
        self.expect(expected)?;
        if self.state != AssignmentState::Approved
            || !self.envelope.active_at(now)
            || current_restriction_version != self.restriction_version
        {
            return Err(IncidentError::PolicyDistributionGap);
        }
        self.state = AssignmentState::Issued;
        self.bump()
    }
    pub fn acknowledge(&mut self, expected: u64, principal: EntityId) -> Result<(), IncidentError> {
        self.expect(expected)?;
        if self.state != AssignmentState::Issued {
            return Err(IncidentError::InvalidTransition);
        }
        self.acknowledged_by = Some(principal);
        self.state = AssignmentState::Accepted;
        self.bump()
    }
    pub fn revoke(&mut self, expected: u64) -> Result<(), IncidentError> {
        self.expect(expected)?;
        if matches!(
            self.state,
            AssignmentState::Completed | AssignmentState::Revoked | AssignmentState::Expired
        ) {
            return Err(IncidentError::InvalidTransition);
        }
        self.state = AssignmentState::Revoked;
        self.bump()
    }
    pub fn expire_if_due(&mut self, now: DateTime<Utc>) -> Result<bool, IncidentError> {
        if !matches!(
            self.state,
            AssignmentState::Completed | AssignmentState::Revoked | AssignmentState::Expired
        ) && !self.envelope.active_at(now)
        {
            self.state = AssignmentState::Expired;
            self.bump()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    fn expect(&self, e: u64) -> Result<(), IncidentError> {
        if e == self.version {
            Ok(())
        } else {
            Err(IncidentError::ConcurrencyConflict)
        }
    }
    fn bump(&mut self) -> Result<(), IncidentError> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or(IncidentError::VersionExhausted)?;
        Ok(())
    }
    #[must_use]
    pub fn version(&self) -> u64 {
        self.version
    }
    #[must_use]
    pub fn state(&self) -> AssignmentState {
        self.state
    }
    #[must_use]
    pub fn is_acknowledged(&self) -> bool {
        self.acknowledged_by.is_some()
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub fn constraints(&self) -> &BTreeSet<String> {
        &self.constraints
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestrictionState {
    Proposed,
    Effective,
    Superseded,
    Expired,
}
#[derive(Clone, Debug)]
pub struct Restriction {
    id: EntityId,
    incident_id: EntityId,
    envelope: AuthorityEnvelope,
    reason: String,
    sequence: u64,
    state: RestrictionState,
    source_authority: EntityId,
    version: u64,
}
impl Restriction {
    pub fn propose(
        id: EntityId,
        incident_id: EntityId,
        envelope: AuthorityEnvelope,
        reason: impl Into<String>,
        source_authority: EntityId,
        sequence: u64,
    ) -> Result<Self, IncidentError> {
        if sequence == 0 {
            return Err(IncidentError::InvalidField);
        }
        Ok(Self {
            id,
            incident_id,
            envelope,
            reason: bounded(reason)?,
            sequence,
            state: RestrictionState::Proposed,
            source_authority,
            version: 1,
        })
    }
    pub fn make_effective(
        &mut self,
        expected: u64,
        incident: &Incident,
        now: DateTime<Utc>,
    ) -> Result<(), IncidentError> {
        if expected != self.version {
            return Err(IncidentError::ConcurrencyConflict);
        }
        if self.state != RestrictionState::Proposed
            || self.incident_id != *incident.id()
            || !incident.envelope().contains(&self.envelope)
            || !self.envelope.active_at(now)
        {
            return Err(IncidentError::AuthorityExpansion);
        }
        self.state = RestrictionState::Effective;
        self.version += 1;
        Ok(())
    }
    pub fn supersede(
        mut self,
        next: Self,
        expansion_approved: bool,
    ) -> Result<(Self, Self), IncidentError> {
        if self.state != RestrictionState::Effective
            || next.incident_id != self.incident_id
            || next.sequence <= self.sequence
            || next.state != RestrictionState::Proposed
            || (!self.envelope.contains(&next.envelope) && !expansion_approved)
        {
            return Err(IncidentError::AuthorityExpansion);
        }
        self.state = RestrictionState::Superseded;
        Ok((self, next))
    }
    pub fn expire_if_due(&mut self, now: DateTime<Utc>) -> Result<bool, IncidentError> {
        if self.state == RestrictionState::Effective && !self.envelope.active_at(now) {
            self.state = RestrictionState::Expired;
            self.version = self
                .version
                .checked_add(1)
                .ok_or(IncidentError::VersionExhausted)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    #[must_use]
    pub fn envelope(&self) -> &AuthorityEnvelope {
        &self.envelope
    }
    #[must_use]
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
    #[must_use]
    pub fn is_effective_at(&self, now: DateTime<Utc>) -> bool {
        self.state == RestrictionState::Effective && self.envelope.active_at(now)
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
    #[must_use]
    pub fn source_authority(&self) -> &EntityId {
        &self.source_authority
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IncidentEvent {
    IncidentOpened {
        incident_id: EntityId,
    },
    IncidentCommandChanged {
        incident_id: EntityId,
        sequence: u64,
    },
    OperationalPeriodActivated {
        period_id: EntityId,
    },
    AssignmentIssued {
        assignment_id: EntityId,
    },
    AssignmentRevoked {
        assignment_id: EntityId,
    },
    RestrictionChanged {
        restriction_id: EntityId,
        sequence: u64,
    },
}

pub trait Repository<A> {
    type Error;
    fn load(&self, id: &EntityId) -> Result<Option<A>, Self::Error>;
    fn save(
        &self,
        aggregate: &A,
        expected_version: u64,
        events: &[IncidentEvent],
    ) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum IncidentError {
    #[error("invalid governed field")]
    InvalidField,
    #[error("authority envelope is invalid")]
    InvalidAuthority,
    #[error("authority expired")]
    ExpiredAuthority,
    #[error("authority is ambiguous or transfer custody is broken")]
    AmbiguousAuthority,
    #[error("operation would expand authority")]
    AuthorityExpansion,
    #[error("invalid aggregate transition")]
    InvalidTransition,
    #[error("approval or separation of duties is missing")]
    MissingApproval,
    #[error("authority does not cover operation")]
    InvalidAuthorityScope,
    #[error("restriction acknowledgement/distribution is incomplete")]
    PolicyDistributionGap,
    #[error("optimistic concurrency conflict")]
    ConcurrencyConflict,
    #[error("aggregate version exhausted")]
    VersionExhausted,
}
