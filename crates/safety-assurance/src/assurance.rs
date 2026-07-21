//! Hazard, constraint, ODD, and occurrence aggregates.
#![allow(missing_docs)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_kernel::{EntityId, TimeWindow};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// Immutable content digest used for signed evidence and configuration binding.
pub type Digest = [u8; 32];

fn text(value: impl Into<String>) -> Result<String, SafetyError> {
    let value = value.into();
    if value.trim().is_empty() || value.len() > 512 {
        Err(SafetyError::InvalidField)
    } else {
        Ok(value)
    }
}

/// Lifecycle of an explicitly governed hazard.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HazardState {
    Identified,
    Analyzed,
    Controlled,
    Accepted,
    Monitoring,
    Closed,
    Reopened,
}

/// Human residual-risk decision. Deployment can never manufacture this value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResidualRiskAcceptance {
    pub authority: EntityId,
    pub competency: String,
    pub rationale: String,
    pub scope: String,
    pub accepted_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl ResidualRiskAcceptance {
    /// Constructs a bounded, attributable human decision.
    pub fn new(
        authority: EntityId,
        competency: impl Into<String>,
        rationale: impl Into<String>,
        scope: impl Into<String>,
        accepted_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<Self, SafetyError> {
        if accepted_at >= expires_at {
            return Err(SafetyError::InvalidValidity);
        }
        Ok(Self {
            authority,
            competency: text(competency)?,
            rationale: text(rationale)?,
            scope: text(scope)?,
            accepted_at,
            expires_at,
        })
    }

    #[must_use]
    pub fn is_current_at(&self, now: DateTime<Utc>) -> bool {
        now >= self.accepted_at && now < self.expires_at
    }
}

/// Identity-owned authority/competency check; authentication alone is insufficient.
pub trait CompetencyPort {
    type Error;
    fn is_competent(
        &self,
        authority: &EntityId,
        competency: &str,
        scope: &str,
        at: DateTime<Utc>,
    ) -> Result<bool, Self::Error>;
}

/// Hazard aggregate enforcing SA-INV-001 and SA-INV-005.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hazard {
    id: EntityId,
    owner: EntityId,
    scope: String,
    state: HazardState,
    analysis_method: Option<String>,
    controls: BTreeMap<String, bool>,
    residual: Option<ResidualRiskAcceptance>,
    review_at: DateTime<Utc>,
}

impl Hazard {
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub fn owner(&self) -> &EntityId {
        &self.owner
    }
    #[must_use]
    pub const fn state(&self) -> HazardState {
        self.state
    }
    #[must_use]
    pub fn scope(&self) -> &str {
        &self.scope
    }
    pub fn register(
        id: EntityId,
        owner: EntityId,
        scope: impl Into<String>,
        review_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<Self, SafetyError> {
        if review_at <= now {
            return Err(SafetyError::InvalidValidity);
        }
        Ok(Self {
            id,
            owner,
            scope: text(scope)?,
            state: HazardState::Identified,
            analysis_method: None,
            controls: BTreeMap::new(),
            residual: None,
            review_at,
        })
    }

    pub fn analyze(&mut self, method: impl Into<String>) -> Result<(), SafetyError> {
        if !matches!(self.state, HazardState::Identified | HazardState::Reopened) {
            return Err(SafetyError::InvalidTransition);
        }
        self.analysis_method = Some(text(method)?);
        self.state = HazardState::Analyzed;
        Ok(())
    }

    pub fn attach_control(&mut self, control_id: impl Into<String>) -> Result<(), SafetyError> {
        if !matches!(self.state, HazardState::Analyzed | HazardState::Controlled) {
            return Err(SafetyError::InvalidTransition);
        }
        self.controls.entry(text(control_id)?).or_insert(false);
        self.state = HazardState::Controlled;
        Ok(())
    }

    pub fn verify_control(&mut self, control_id: &str) -> Result<(), SafetyError> {
        let verified = self
            .controls
            .get_mut(control_id)
            .ok_or(SafetyError::UnknownControl)?;
        *verified = true;
        Ok(())
    }

    pub fn accept_residual<C: CompetencyPort>(
        &mut self,
        decision: ResidualRiskAcceptance,
        now: DateTime<Utc>,
        competency: &C,
    ) -> Result<(), SafetyError> {
        if self.state != HazardState::Controlled
            || self.controls.is_empty()
            || self.controls.values().any(|v| !v)
            || decision.scope != self.scope
            || !decision.is_current_at(now)
            || !matches!(
                competency.is_competent(
                    &decision.authority,
                    &decision.competency,
                    &decision.scope,
                    now
                ),
                Ok(true)
            )
        {
            return Err(SafetyError::HazardIncomplete);
        }
        self.residual = Some(decision);
        self.state = HazardState::Accepted;
        Ok(())
    }

    pub fn begin_monitoring(&mut self) -> Result<(), SafetyError> {
        if self.state != HazardState::Accepted {
            return Err(SafetyError::InvalidTransition);
        }
        self.state = HazardState::Monitoring;
        Ok(())
    }
    pub fn close(&mut self) -> Result<(), SafetyError> {
        if self.state != HazardState::Monitoring {
            return Err(SafetyError::InvalidTransition);
        }
        self.state = HazardState::Closed;
        Ok(())
    }
    pub fn reopen(&mut self) -> Result<(), SafetyError> {
        if !matches!(
            self.state,
            HazardState::Accepted | HazardState::Monitoring | HazardState::Closed
        ) {
            return Err(SafetyError::InvalidTransition);
        }
        self.residual = None;
        self.state = HazardState::Reopened;
        Ok(())
    }

    #[must_use]
    pub fn promotion_ready_at(&self, now: DateTime<Utc>) -> bool {
        matches!(
            self.state,
            HazardState::Accepted | HazardState::Monitoring | HazardState::Closed
        ) && now < self.review_at
            && self.analysis_method.is_some()
            && !self.controls.is_empty()
            && self.controls.values().all(|v| *v)
            && self.residual.as_ref().is_some_and(|v| v.is_current_at(now))
    }
}

/// Conditions under which a capability has been verified.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OperationalDesignDomain {
    id: EntityId,
    terrains: BTreeSet<String>,
    weather: BTreeSet<String>,
    maximum_wind_kph: u16,
    requires_positioning: bool,
    allows_disconnected_operation: bool,
    approved: bool,
    version: u64,
}

impl OperationalDesignDomain {
    pub fn new(
        terrains: impl IntoIterator<Item = String>,
        weather: impl IntoIterator<Item = String>,
        maximum_wind_kph: u16,
        requires_positioning: bool,
        allows_disconnected_operation: bool,
    ) -> Result<Self, SafetyError> {
        let terrains = normalized(terrains)?;
        let weather = normalized(weather)?;
        if maximum_wind_kph == 0 || maximum_wind_kph > 250 {
            return Err(SafetyError::InvalidWindLimit);
        }
        Ok(Self {
            id: EntityId::new(),
            terrains,
            weather,
            maximum_wind_kph,
            requires_positioning,
            allows_disconnected_operation,
            approved: false,
            version: 1,
        })
    }
    pub fn approve(&mut self) {
        self.approved = true;
    }
    #[must_use]
    pub fn is_approved(&self) -> bool {
        self.approved
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }
    #[must_use]
    pub fn contains(&self, requested: &Self) -> bool {
        requested.terrains.is_subset(&self.terrains)
            && requested.weather.is_subset(&self.weather)
            && requested.maximum_wind_kph <= self.maximum_wind_kph
            && (!self.requires_positioning || requested.requires_positioning)
            && (self.allows_disconnected_operation || !requested.allows_disconnected_operation)
    }
    pub fn narrow_to(&mut self, narrower: &Self) -> Result<(), SafetyError> {
        if !self.approved || !self.contains(narrower) {
            return Err(SafetyError::AuthorityExpansion);
        }
        self.terrains.clone_from(&narrower.terrains);
        self.weather.clone_from(&narrower.weather);
        self.maximum_wind_kph = narrower.maximum_wind_kph;
        self.requires_positioning = narrower.requires_positioning;
        self.allows_disconnected_operation = narrower.allows_disconnected_operation;
        self.version = self
            .version
            .checked_add(1)
            .ok_or(SafetyError::VersionExhausted)?;
        Ok(())
    }
}

fn normalized(values: impl IntoIterator<Item = String>) -> Result<BTreeSet<String>, SafetyError> {
    let set: BTreeSet<_> = values
        .into_iter()
        .map(|v| v.trim().to_lowercase())
        .filter(|v| !v.is_empty())
        .collect();
    if set.is_empty() {
        Err(SafetyError::EmptyConditionSet)
    } else {
        Ok(set)
    }
}

/// Immutable scope of a runtime safety constraint.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ConstraintScope {
    pub tenant: String,
    pub capability: String,
    pub odd_id: EntityId,
    pub maximum_authority: u32,
}

impl ConstraintScope {
    #[must_use]
    pub fn is_equal_or_stricter_than(&self, previous: &Self) -> bool {
        self.tenant == previous.tenant
            && self.capability == previous.capability
            && self.odd_id == previous.odd_id
            && self.maximum_authority <= previous.maximum_authority
    }
}

/// Immutable signed safety constraint; changes create a new version.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SafetyConstraint {
    id: EntityId,
    version: u64,
    validity: TimeWindow,
    approved_by: EntityId,
    signature: String,
    scope: ConstraintScope,
    content_digest: Digest,
}

impl SafetyConstraint {
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub fn approved_by(&self) -> &EntityId {
        &self.approved_by
    }
    #[must_use]
    pub fn signature(&self) -> &str {
        &self.signature
    }
    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }
    #[must_use]
    pub fn scope(&self) -> &ConstraintScope {
        &self.scope
    }
    #[must_use]
    pub const fn content_digest(&self) -> Digest {
        self.content_digest
    }
    #[allow(clippy::too_many_arguments)]
    pub fn define(
        id: EntityId,
        version: u64,
        validity: TimeWindow,
        approved_by: EntityId,
        signature: impl Into<String>,
        scope: ConstraintScope,
        content_digest: Digest,
    ) -> Result<Self, SafetyError> {
        let signature = signature.into();
        if version == 0 {
            return Err(SafetyError::InvalidConstraintVersion);
        }
        if signature.trim().len() < 16 || content_digest == [0; 32] {
            return Err(SafetyError::MissingSignature);
        }
        for v in [&scope.tenant, &scope.capability] {
            text(v.clone())?;
        }
        Ok(Self {
            id,
            version,
            validity,
            approved_by,
            signature,
            scope,
            content_digest,
        })
    }
    #[must_use]
    pub fn is_active_at(&self, now: DateTime<Utc>) -> bool {
        self.validity.contains(now)
    }
    pub fn supersede(&self, next: Self) -> Result<Self, SafetyError> {
        if next.id != self.id
            || next.version
                != self
                    .version
                    .checked_add(1)
                    .ok_or(SafetyError::VersionExhausted)?
            || !next.scope.is_equal_or_stricter_than(&self.scope)
            || next.validity.starts_at() < self.validity.starts_at()
            || next.validity.ends_at() > self.validity.ends_at()
        {
            return Err(SafetyError::AuthorityExpansion);
        }
        Ok(next)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum OccurrenceSeverity {
    Observation,
    NearMiss,
    Serious,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OccurrenceState {
    Reported,
    Triaged,
    Investigating,
    ActionsOpen,
    Closed,
    Reopened,
}

/// Safety occurrence with immutable facts and accountable corrective actions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SafetyOccurrence {
    id: EntityId,
    scope: String,
    severity: OccurrenceSeverity,
    state: OccurrenceState,
    occurred_at: DateTime<Utc>,
    findings: Vec<String>,
    actions: BTreeMap<String, bool>,
}

impl SafetyOccurrence {
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub const fn state(&self) -> OccurrenceState {
        self.state
    }
    #[must_use]
    pub const fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    pub fn report(
        id: EntityId,
        scope: impl Into<String>,
        severity: OccurrenceSeverity,
        occurred_at: DateTime<Utc>,
    ) -> Result<Self, SafetyError> {
        Ok(Self {
            id,
            scope: text(scope)?,
            severity,
            state: OccurrenceState::Reported,
            occurred_at,
            findings: vec![],
            actions: BTreeMap::new(),
        })
    }
    pub fn triage(&mut self) -> Result<(), SafetyError> {
        self.transition(OccurrenceState::Reported, OccurrenceState::Triaged)
    }
    pub fn investigate(&mut self) -> Result<(), SafetyError> {
        self.transition(OccurrenceState::Triaged, OccurrenceState::Investigating)
    }
    pub fn record_finding(&mut self, finding: impl Into<String>) -> Result<(), SafetyError> {
        if self.state != OccurrenceState::Investigating {
            return Err(SafetyError::InvalidTransition);
        }
        self.findings.push(text(finding)?);
        Ok(())
    }
    pub fn assign_action(&mut self, id: impl Into<String>) -> Result<(), SafetyError> {
        if self.state != OccurrenceState::Investigating
            && self.state != OccurrenceState::ActionsOpen
        {
            return Err(SafetyError::InvalidTransition);
        }
        self.actions.insert(text(id)?, false);
        self.state = OccurrenceState::ActionsOpen;
        Ok(())
    }
    pub fn complete_action(&mut self, id: &str) -> Result<(), SafetyError> {
        *self.actions.get_mut(id).ok_or(SafetyError::UnknownAction)? = true;
        Ok(())
    }
    pub fn close(&mut self) -> Result<(), SafetyError> {
        if self.findings.is_empty() || self.actions.is_empty() || self.actions.values().any(|v| !v)
        {
            return Err(SafetyError::OpenSafetyAction);
        }
        self.state = OccurrenceState::Closed;
        Ok(())
    }
    pub fn reopen(&mut self) -> Result<(), SafetyError> {
        self.transition(OccurrenceState::Closed, OccurrenceState::Reopened)
    }
    fn transition(
        &mut self,
        from: OccurrenceState,
        to: OccurrenceState,
    ) -> Result<(), SafetyError> {
        if self.state == from {
            self.state = to;
            Ok(())
        } else {
            Err(SafetyError::InvalidTransition)
        }
    }
    #[must_use]
    pub fn blocks_scope(&self, scope: &str) -> bool {
        self.scope == scope
            && self.severity >= OccurrenceSeverity::NearMiss
            && self.state != OccurrenceState::Closed
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum SafetyError {
    #[error("invalid or empty governed field")]
    InvalidField,
    #[error("invalid lifecycle transition")]
    InvalidTransition,
    #[error("validity or review interval is invalid")]
    InvalidValidity,
    #[error("hazard assurance is incomplete")]
    HazardIncomplete,
    #[error("unknown hazard control")]
    UnknownControl,
    #[error("unknown corrective action")]
    UnknownAction,
    #[error("corrective safety actions remain open")]
    OpenSafetyAction,
    #[error("operational design domain condition sets cannot be empty")]
    EmptyConditionSet,
    #[error("maximum wind must be between 1 and 250 km/h")]
    InvalidWindLimit,
    #[error("constraint version must be positive")]
    InvalidConstraintVersion,
    #[error("constraint must include signature and digest")]
    MissingSignature,
    #[error("transition would expand authority")]
    AuthorityExpansion,
    #[error("aggregate version exhausted")]
    VersionExhausted,
}
