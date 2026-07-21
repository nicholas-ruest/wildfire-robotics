//! Incident activation, acknowledgement gaps, and safety-biased restriction reconciliation.
#![allow(missing_docs)]

use crate::{Assignment, AssignmentState, AuthorityEnvelope, IncidentError, Restriction};
use chrono::{DateTime, Utc};
use shared_kernel::EntityId;
use std::collections::{BTreeMap, BTreeSet};

pub trait QualificationPort {
    type Error;
    fn qualified(
        &self,
        principal: &EntityId,
        role: &str,
        incident: &EntityId,
        at: DateTime<Utc>,
    ) -> Result<bool, Self::Error>;
}
pub trait RestrictionDistributionPort {
    type Error;
    fn publish(&self, incident: &EntityId, sequence: u64) -> Result<BTreeSet<String>, Self::Error>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivationState {
    Opened,
    PeriodEstablished,
    AuthorityValidated,
    RestrictionsPublished,
    DistributionConfirmed,
    AssignmentsPermitted,
    Blocked,
    Expired,
}
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ActivationBlocker {
    AmbiguousAuthority,
    Qualification,
    ExpiredPeriod,
    RestrictionConflict,
    DistributionGap,
}

/// Restart-safe process manager; monotonically records each completed prerequisite.
#[derive(Clone, Debug)]
pub struct IncidentActivation {
    pub correlation_id: EntityId,
    pub incident_id: EntityId,
    pub period_id: EntityId,
    state: ActivationState,
    restriction_sequence: u64,
    acknowledged_nodes: BTreeSet<String>,
    required_nodes: BTreeSet<String>,
    processed: BTreeSet<String>,
    blockers: BTreeSet<ActivationBlocker>,
    version: u64,
}
impl IncidentActivation {
    #[must_use]
    pub fn start(
        correlation_id: EntityId,
        incident_id: EntityId,
        period_id: EntityId,
        required_nodes: impl IntoIterator<Item = String>,
    ) -> Self {
        Self {
            correlation_id,
            incident_id,
            period_id,
            state: ActivationState::Opened,
            restriction_sequence: 0,
            acknowledged_nodes: BTreeSet::new(),
            required_nodes: required_nodes.into_iter().collect(),
            processed: BTreeSet::new(),
            blockers: BTreeSet::new(),
            version: 1,
        }
    }
    pub fn period_established(&mut self, event_id: impl Into<String>) -> Result<(), IncidentError> {
        self.once(
            event_id,
            ActivationState::Opened,
            ActivationState::PeriodEstablished,
        )
    }
    pub fn authority_validated(
        &mut self,
        event_id: impl Into<String>,
    ) -> Result<(), IncidentError> {
        self.once(
            event_id,
            ActivationState::PeriodEstablished,
            ActivationState::AuthorityValidated,
        )
    }
    pub fn restrictions_published(
        &mut self,
        event_id: impl Into<String>,
        sequence: u64,
    ) -> Result<(), IncidentError> {
        if sequence == 0 {
            return Err(IncidentError::InvalidField);
        }
        self.once(
            event_id,
            ActivationState::AuthorityValidated,
            ActivationState::RestrictionsPublished,
        )?;
        self.restriction_sequence = sequence;
        Ok(())
    }
    pub fn record_ack(&mut self, node: impl Into<String>) {
        self.acknowledged_nodes.insert(node.into());
        if self.state == ActivationState::RestrictionsPublished
            && self.required_nodes.is_subset(&self.acknowledged_nodes)
        {
            self.state = ActivationState::DistributionConfirmed;
            self.version = self.version.saturating_add(1);
        }
    }
    pub fn permit_assignments(&mut self) -> Result<(), IncidentError> {
        if self.state != ActivationState::DistributionConfirmed || !self.blockers.is_empty() {
            return Err(IncidentError::PolicyDistributionGap);
        }
        self.state = ActivationState::AssignmentsPermitted;
        self.version = self
            .version
            .checked_add(1)
            .ok_or(IncidentError::VersionExhausted)?;
        Ok(())
    }
    pub fn block(&mut self, reason: ActivationBlocker) {
        self.blockers.insert(reason);
        self.state = ActivationState::Blocked;
        self.version = self.version.saturating_add(1);
    }
    fn once(
        &mut self,
        event_id: impl Into<String>,
        expected: ActivationState,
        next: ActivationState,
    ) -> Result<(), IncidentError> {
        let event_id = event_id.into();
        if self.processed.contains(&event_id) {
            return Ok(());
        }
        if self.state != expected {
            return Err(IncidentError::InvalidTransition);
        }
        self.processed.insert(event_id);
        self.state = next;
        self.version = self
            .version
            .checked_add(1)
            .ok_or(IncidentError::VersionExhausted)?;
        Ok(())
    }
    #[must_use]
    pub fn can_issue(&self) -> bool {
        self.state == ActivationState::AssignmentsPermitted && self.blockers.is_empty()
    }
    #[must_use]
    pub fn acknowledgement_gaps(&self) -> BTreeSet<String> {
        self.required_nodes
            .difference(&self.acknowledged_nodes)
            .cloned()
            .collect()
    }
    #[must_use]
    pub fn version(&self) -> u64 {
        self.version
    }
}

/// Deterministic merge of active restrictions: every rule must contain the effective meet.
pub fn effective_restriction<'a>(
    restrictions: impl IntoIterator<Item = &'a Restriction>,
    now: DateTime<Utc>,
) -> Result<Option<&'a AuthorityEnvelope>, IncidentError> {
    let active = restrictions
        .into_iter()
        .filter(|r| r.is_effective_at(now))
        .collect::<Vec<_>>();
    if active.is_empty() {
        return Ok(None);
    }
    let mut strict = active[0].envelope();
    for item in active.iter().skip(1) {
        let candidate = item.envelope();
        if strict.contains(candidate) {
            strict = candidate;
        } else if !candidate.contains(strict) {
            return Err(IncidentError::AmbiguousAuthority);
        }
    }
    Ok(Some(strict))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssignmentBoardRow {
    pub assignment_id: EntityId,
    pub state: AssignmentState,
    pub acknowledged: bool,
    pub source_version: u64,
    pub projected_at: DateTime<Utc>,
}
#[derive(Clone, Debug, Default)]
pub struct AssignmentBoard {
    rows: BTreeMap<String, AssignmentBoardRow>,
}
impl AssignmentBoard {
    pub fn project(&mut self, assignment: &Assignment, at: DateTime<Utc>) {
        let key = assignment.id().to_string();
        let row = AssignmentBoardRow {
            assignment_id: assignment.id().clone(),
            state: assignment.state(),
            acknowledged: assignment.is_acknowledged(),
            source_version: assignment.version(),
            projected_at: at,
        };
        match self.rows.get(&key) {
            Some(current) if current.source_version >= row.source_version => {}
            _ => {
                self.rows.insert(key, row);
            }
        }
    }
    #[must_use]
    pub fn acknowledgement_gaps(&self) -> Vec<&AssignmentBoardRow> {
        self.rows
            .values()
            .filter(|row| row.state == AssignmentState::Issued && !row.acknowledged)
            .collect()
    }
}

/// Offline station cache. Strictly newer restrictions apply before permissive backlog.
#[derive(Clone, Debug, Default)]
pub struct OfflineRestrictionCache {
    highest_sequence: u64,
    digests: BTreeMap<u64, [u8; 32]>,
}
impl OfflineRestrictionCache {
    pub fn apply(&mut self, sequence: u64, digest: [u8; 32]) -> Result<bool, IncidentError> {
        match self.digests.get(&sequence) {
            Some(existing) if existing == &digest => return Ok(false),
            Some(_) => return Err(IncidentError::AmbiguousAuthority),
            None => {}
        }
        if sequence < self.highest_sequence {
            return Ok(false);
        }
        self.digests.insert(sequence, digest);
        self.highest_sequence = sequence;
        Ok(true)
    }
    #[must_use]
    pub fn highest_sequence(&self) -> u64 {
        self.highest_sequence
    }
}
