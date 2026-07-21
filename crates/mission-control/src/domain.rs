//! `Mission`, `Allocation`, `MissionLease`, and `ConflictSet` aggregates.
#![allow(missing_docs)]
use chrono::{DateTime, Utc};
use shared_kernel::{EntityId, TimeWindow};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
pub type Digest = [u8; 32];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionedSnapshot {
    pub name: String,
    pub digest: Digest,
    pub version: u64,
    pub valid_until: DateTime<Utc>,
}
impl VersionedSnapshot {
    #[must_use]
    pub fn current_at(&self, now: DateTime<Utc>) -> bool {
        !self.name.trim().is_empty()
            && self.digest != [0; 32]
            && self.version > 0
            && now < self.valid_until
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationSnapshot {
    pub assignment: VersionedSnapshot,
    pub policy: VersionedSnapshot,
    pub restriction: VersionedSnapshot,
    pub constraint: VersionedSnapshot,
    pub odd: VersionedSnapshot,
    pub hazard: VersionedSnapshot,
    pub fleet: VersionedSnapshot,
    pub plan: VersionedSnapshot,
}
impl AuthorizationSnapshot {
    #[must_use]
    pub fn current_at(&self, now: DateTime<Utc>) -> bool {
        [
            &self.assignment,
            &self.policy,
            &self.restriction,
            &self.constraint,
            &self.odd,
            &self.hazard,
            &self.fleet,
            &self.plan,
        ]
        .into_iter()
        .all(|v| v.current_at(now))
    }
    #[must_use]
    pub fn digest_set(&self) -> BTreeSet<Digest> {
        [
            self.assignment.digest,
            self.policy.digest,
            self.restriction.digest,
            self.constraint.digest,
            self.odd.digest,
            self.hazard.digest,
            self.fleet.digest,
            self.plan.digest,
        ]
        .into_iter()
        .collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConflictKind {
    Collision,
    Airspace,
    Resource,
    IncompatibleObjective,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConflictState {
    Open,
    Mitigated,
    Accepted,
    Closed,
}
#[derive(Clone, Debug)]
pub struct Conflict {
    id: EntityId,
    kind: ConflictKind,
    description: String,
    mitigation: Option<String>,
    state: ConflictState,
}
#[derive(Clone, Debug)]
pub struct ConflictSet {
    id: EntityId,
    mission_id: EntityId,
    conflicts: BTreeMap<String, Conflict>,
    version: u64,
}
impl ConflictSet {
    #[must_use]
    pub fn open(id: EntityId, mission_id: EntityId) -> Self {
        Self {
            id,
            mission_id,
            conflicts: BTreeMap::new(),
            version: 1,
        }
    }
    pub fn detect(
        &mut self,
        id: EntityId,
        kind: ConflictKind,
        description: impl Into<String>,
    ) -> Result<(), MissionError> {
        let description = description.into();
        if description.trim().is_empty() || self.conflicts.contains_key(&id.to_string()) {
            return Err(MissionError::InvalidConflict);
        }
        self.conflicts.insert(
            id.to_string(),
            Conflict {
                id,
                kind,
                description,
                mitigation: None,
                state: ConflictState::Open,
            },
        );
        self.bump()
    }
    pub fn mitigate(
        &mut self,
        id: &EntityId,
        mitigation: impl Into<String>,
    ) -> Result<(), MissionError> {
        let mitigation = mitigation.into();
        let conflict = self
            .conflicts
            .get_mut(&id.to_string())
            .ok_or(MissionError::InvalidConflict)?;
        if conflict.state != ConflictState::Open || mitigation.trim().is_empty() {
            return Err(MissionError::InvalidConflict);
        }
        conflict.mitigation = Some(mitigation);
        conflict.state = ConflictState::Mitigated;
        self.bump()
    }
    pub fn accept_residual(
        &mut self,
        id: &EntityId,
        approver: &EntityId,
        detector: &EntityId,
        permitted: bool,
    ) -> Result<(), MissionError> {
        let conflict = self
            .conflicts
            .get_mut(&id.to_string())
            .ok_or(MissionError::InvalidConflict)?;
        if conflict.state != ConflictState::Mitigated
            || matches!(
                conflict.kind,
                ConflictKind::Collision | ConflictKind::Airspace
            )
            || approver == detector
            || !permitted
        {
            return Err(MissionError::ResidualConflictNotApproved);
        }
        conflict.state = ConflictState::Accepted;
        self.bump()
    }
    pub fn close(&mut self, id: &EntityId) -> Result<(), MissionError> {
        let conflict = self
            .conflicts
            .get_mut(&id.to_string())
            .ok_or(MissionError::InvalidConflict)?;
        if !matches!(
            conflict.state,
            ConflictState::Mitigated | ConflictState::Accepted
        ) {
            return Err(MissionError::InvalidConflict);
        }
        conflict.state = ConflictState::Closed;
        self.bump()
    }
    #[must_use]
    pub fn dispatch_clear(&self) -> bool {
        self.conflicts
            .values()
            .all(|c| matches!(c.state, ConflictState::Accepted | ConflictState::Closed))
    }
    fn bump(&mut self) -> Result<(), MissionError> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or(MissionError::VersionExhausted)?;
        Ok(())
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub fn mission_id(&self) -> &EntityId {
        &self.mission_id
    }
    pub fn timeline(&self) -> impl Iterator<Item = (&EntityId, ConflictKind, &str, ConflictState)> {
        self.conflicts
            .values()
            .map(|c| (&c.id, c.kind, c.description.as_str(), c.state))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AllocationState {
    Proposed,
    Reserved,
    Committed,
    Released,
    Expired,
}
#[derive(Clone, Debug)]
pub struct Allocation {
    id: EntityId,
    mission_id: EntityId,
    resources: BTreeSet<String>,
    capability_digest: Digest,
    cell_id: EntityId,
    cell_epoch: u64,
    state: AllocationState,
    validity: TimeWindow,
    version: u64,
}
impl Allocation {
    pub fn propose(
        id: EntityId,
        mission_id: EntityId,
        resources: impl IntoIterator<Item = String>,
        capability_digest: Digest,
        cell_id: EntityId,
        cell_epoch: u64,
        validity: TimeWindow,
    ) -> Result<Self, MissionError> {
        let resources = resources.into_iter().collect::<BTreeSet<_>>();
        if resources.is_empty()
            || resources.iter().any(|v| v.trim().is_empty())
            || capability_digest == [0; 32]
            || cell_epoch == 0
        {
            return Err(MissionError::InvalidAllocation);
        }
        Ok(Self {
            id,
            mission_id,
            resources,
            capability_digest,
            cell_id,
            cell_epoch,
            state: AllocationState::Proposed,
            validity,
            version: 1,
        })
    }
    pub fn reserve(
        &mut self,
        book: &mut ReservationBook,
        now: DateTime<Utc>,
    ) -> Result<(), MissionError> {
        if self.state != AllocationState::Proposed || !self.validity.contains(now) {
            return Err(MissionError::InvalidAllocation);
        }
        book.reserve(self)?;
        self.state = AllocationState::Reserved;
        self.bump()
    }
    pub fn commit(
        &mut self,
        book: &ReservationBook,
        current_cell_epoch: u64,
        current_capability: Digest,
        now: DateTime<Utc>,
    ) -> Result<(), MissionError> {
        if self.state != AllocationState::Reserved
            || !self.validity.contains(now)
            || current_cell_epoch != self.cell_epoch
            || current_capability != self.capability_digest
            || !book.owned_by(self)
        {
            return Err(MissionError::StaleAllocation);
        }
        self.state = AllocationState::Committed;
        self.bump()
    }
    pub fn release(&mut self, book: &mut ReservationBook) -> Result<(), MissionError> {
        if !matches!(
            self.state,
            AllocationState::Reserved | AllocationState::Committed
        ) {
            return Err(MissionError::InvalidAllocation);
        }
        book.release(self);
        self.state = AllocationState::Released;
        self.bump()
    }
    fn bump(&mut self) -> Result<(), MissionError> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or(MissionError::VersionExhausted)?;
        Ok(())
    }
    #[must_use]
    pub fn committed(&self) -> bool {
        self.state == AllocationState::Committed
    }
    #[must_use]
    pub fn mission_id(&self) -> &EntityId {
        &self.mission_id
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub fn cell_id(&self) -> &EntityId {
        &self.cell_id
    }
}

#[derive(Clone, Debug, Default)]
pub struct ReservationBook {
    owners: BTreeMap<String, String>,
}
impl ReservationBook {
    fn reserve(&mut self, a: &Allocation) -> Result<(), MissionError> {
        if a.resources.iter().any(|r| self.owners.contains_key(r)) {
            return Err(MissionError::DoubleAllocation);
        }
        for resource in &a.resources {
            self.owners.insert(resource.clone(), a.id.to_string());
        }
        Ok(())
    }
    fn release(&mut self, a: &Allocation) {
        self.owners.retain(|_, owner| owner != &a.id.to_string());
    }
    fn owned_by(&self, a: &Allocation) -> bool {
        a.resources
            .iter()
            .all(|r| self.owners.get(r) == Some(&a.id.to_string()))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LeaseState {
    Offered,
    Active,
    Renewed,
    Revoked,
    Expired,
}
#[derive(Clone, Debug)]
pub struct MissionLease {
    id: EntityId,
    mission_id: EntityId,
    holder: EntityId,
    fence: u64,
    validity: TimeWindow,
    state: LeaseState,
    version: u64,
}
impl MissionLease {
    pub fn offer(
        id: EntityId,
        mission_id: EntityId,
        holder: EntityId,
        fence: u64,
        validity: TimeWindow,
    ) -> Result<Self, MissionError> {
        if fence == 0 {
            return Err(MissionError::InvalidLease);
        }
        Ok(Self {
            id,
            mission_id,
            holder,
            fence,
            validity,
            state: LeaseState::Offered,
            version: 1,
        })
    }
    pub fn acquire(&mut self, now: DateTime<Utc>) -> Result<(), MissionError> {
        if self.state != LeaseState::Offered || !self.validity.contains(now) {
            return Err(MissionError::InvalidLease);
        }
        self.state = LeaseState::Active;
        self.bump()
    }
    pub fn renew(
        &mut self,
        current_fence: u64,
        new_fence: u64,
        new_validity: TimeWindow,
        now: DateTime<Utc>,
    ) -> Result<(), MissionError> {
        if !matches!(self.state, LeaseState::Active | LeaseState::Renewed)
            || current_fence != self.fence
            || new_fence <= self.fence
            || !new_validity.contains(now)
            || new_validity.starts_at() < self.validity.starts_at()
        {
            return Err(MissionError::StaleLease);
        }
        self.fence = new_fence;
        self.validity = new_validity;
        self.state = LeaseState::Renewed;
        self.bump()
    }
    pub fn revoke(&mut self) -> Result<(), MissionError> {
        if matches!(self.state, LeaseState::Revoked | LeaseState::Expired) {
            return Ok(());
        }
        self.state = LeaseState::Revoked;
        self.bump()
    }
    pub fn expire_if_due(&mut self, now: DateTime<Utc>) -> Result<bool, MissionError> {
        if matches!(self.state, LeaseState::Active | LeaseState::Renewed)
            && !self.validity.contains(now)
        {
            self.state = LeaseState::Expired;
            self.bump()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    #[must_use]
    pub fn permits(
        &self,
        mission: &EntityId,
        holder: &EntityId,
        fence: u64,
        now: DateTime<Utc>,
    ) -> bool {
        self.mission_id == *mission
            && self.holder == *holder
            && self.fence == fence
            && matches!(self.state, LeaseState::Active | LeaseState::Renewed)
            && self.validity.contains(now)
    }
    fn bump(&mut self) -> Result<(), MissionError> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or(MissionError::VersionExhausted)?;
        Ok(())
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub fn fence(&self) -> u64 {
        self.fence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelayRequirement {
    pub service_class: String,
    pub coverage_digest: Digest,
    pub duration_seconds: u32,
    pub spectrum_profile: String,
    pub energy_wh: u64,
    pub return_reserve_wh: u64,
    pub airspace_digest: Digest,
    pub handoff: String,
    pub fallback: String,
}
impl RelayRequirement {
    pub fn validate(&self) -> Result<(), MissionError> {
        if self.service_class.trim().is_empty()
            || self.coverage_digest == [0; 32]
            || self.duration_seconds == 0
            || self.spectrum_profile.trim().is_empty()
            || self.energy_wh == 0
            || self.return_reserve_wh == 0
            || self.airspace_digest == [0; 32]
            || self.handoff.trim().is_empty()
            || self.fallback.trim().is_empty()
        {
            Err(MissionError::InvalidRelayPlan)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MissionState {
    Draft,
    Validating,
    Authorized,
    Dispatched,
    Executing,
    Completed,
    Aborted,
    Failed,
    Expired,
}
#[derive(Clone, Debug)]
pub struct Mission {
    id: EntityId,
    state: MissionState,
    assignment_validity: TimeWindow,
    plan_digest: Digest,
    snapshots: Option<AuthorizationSnapshot>,
    allocation_id: Option<EntityId>,
    conflict_set_id: Option<EntityId>,
    relay: Option<RelayRequirement>,
    abort_reason: Option<String>,
    version: u64,
}
impl Mission {
    pub fn plan(
        id: EntityId,
        assignment_validity: TimeWindow,
        plan_digest: Digest,
        relay: Option<RelayRequirement>,
    ) -> Result<Self, MissionError> {
        if plan_digest == [0; 32] {
            return Err(MissionError::InvalidPlan);
        }
        if let Some(r) = &relay {
            r.validate()?;
        }
        Ok(Self {
            id,
            state: MissionState::Draft,
            assignment_validity,
            plan_digest,
            snapshots: None,
            allocation_id: None,
            conflict_set_id: None,
            relay,
            abort_reason: None,
            version: 1,
        })
    }
    pub fn begin_validation(&mut self) -> Result<(), MissionError> {
        self.transition(MissionState::Draft, MissionState::Validating)
    }
    pub fn authorize(
        &mut self,
        snapshots: AuthorizationSnapshot,
        allocation: &Allocation,
        conflicts: &ConflictSet,
        now: DateTime<Utc>,
    ) -> Result<(), MissionError> {
        if self.state != MissionState::Validating
            || !self.assignment_validity.contains(now)
            || !snapshots.current_at(now)
            || snapshots.plan.digest != self.plan_digest
            || allocation.mission_id() != &self.id
            || !allocation.committed()
            || conflicts.mission_id() != &self.id
            || !conflicts.dispatch_clear()
        {
            return Err(MissionError::AuthorizationDenied);
        }
        self.snapshots = Some(snapshots);
        self.allocation_id = Some(allocation.id().clone());
        self.conflict_set_id = Some(conflicts.id().clone());
        self.state = MissionState::Authorized;
        self.bump()
    }
    pub fn dispatch(
        &mut self,
        lease: &MissionLease,
        holder: &EntityId,
        fence: u64,
        current: &AuthorizationSnapshot,
        now: DateTime<Utc>,
    ) -> Result<(), MissionError> {
        if self.state != MissionState::Authorized
            || !self.assignment_validity.contains(now)
            || !lease.permits(&self.id, holder, fence, now)
            || !current.current_at(now)
            || self.snapshots.as_ref() != Some(current)
        {
            return Err(MissionError::DispatchDenied);
        }
        self.state = MissionState::Dispatched;
        self.bump()
    }
    pub fn advance(
        &mut self,
        lease: &MissionLease,
        holder: &EntityId,
        fence: u64,
        now: DateTime<Utc>,
    ) -> Result<(), MissionError> {
        if !matches!(
            self.state,
            MissionState::Dispatched | MissionState::Executing
        ) || !lease.permits(&self.id, holder, fence, now)
            || !self.assignment_validity.contains(now)
            || self.abort_reason.is_some()
        {
            return Err(MissionError::StaleLease);
        }
        self.state = MissionState::Executing;
        self.bump()
    }
    pub fn abort(
        &mut self,
        reason: impl Into<String>,
        lease: &mut MissionLease,
    ) -> Result<(), MissionError> {
        if self.state == MissionState::Completed {
            return Err(MissionError::InvalidTransition);
        }
        let reason = reason.into();
        if reason.trim().is_empty() {
            return Err(MissionError::MissingAbortReason);
        }
        lease.revoke()?;
        self.abort_reason = Some(reason);
        self.state = MissionState::Aborted;
        self.bump()
    }
    pub fn complete(
        &mut self,
        lease: &MissionLease,
        holder: &EntityId,
        fence: u64,
        now: DateTime<Utc>,
    ) -> Result<(), MissionError> {
        if self.state != MissionState::Executing
            || !lease.permits(&self.id, holder, fence, now)
            || self.abort_reason.is_some()
        {
            return Err(MissionError::InvalidTransition);
        }
        self.state = MissionState::Completed;
        self.bump()
    }
    fn transition(&mut self, f: MissionState, t: MissionState) -> Result<(), MissionError> {
        if self.state != f {
            return Err(MissionError::InvalidTransition);
        }
        self.state = t;
        self.bump()
    }
    fn bump(&mut self) -> Result<(), MissionError> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or(MissionError::VersionExhausted)?;
        Ok(())
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub fn state(&self) -> MissionState {
        self.state
    }
    #[must_use]
    pub fn version(&self) -> u64 {
        self.version
    }
    #[must_use]
    pub fn relay(&self) -> Option<&RelayRequirement> {
        self.relay.as_ref()
    }
}

pub trait CommandGateway {
    type Error;
    fn dispatch(
        &mut self,
        mission: &EntityId,
        plan: Digest,
        fence: u64,
    ) -> Result<GatewayOutcome, Self::Error>;
    fn minimum_risk(&mut self, mission: &EntityId, reason: &str) -> Result<(), Self::Error>;
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GatewayOutcome {
    Accepted,
    Rejected,
    Unknown,
}
pub fn dispatch_command<G: CommandGateway>(
    mission: &mut Mission,
    lease: &MissionLease,
    holder: &EntityId,
    fence: u64,
    snapshot: &AuthorizationSnapshot,
    now: DateTime<Utc>,
    gateway: &mut G,
) -> Result<GatewayOutcome, MissionError> {
    mission.dispatch(lease, holder, fence, snapshot, now)?;
    match gateway.dispatch(mission.id(), mission.plan_digest, fence) {
        Ok(GatewayOutcome::Accepted) => Ok(GatewayOutcome::Accepted),
        Ok(GatewayOutcome::Rejected) => {
            gateway
                .minimum_risk(mission.id(), "gateway rejected dispatch")
                .map_err(|_| MissionError::CompensationFailed)?;
            Ok(GatewayOutcome::Rejected)
        }
        Ok(GatewayOutcome::Unknown) | Err(_) => {
            gateway
                .minimum_risk(mission.id(), "dispatch outcome unknown")
                .map_err(|_| MissionError::CompensationFailed)?;
            Ok(GatewayOutcome::Unknown)
        }
    }
}

pub fn abort_with_compensation<G: CommandGateway>(
    mission: &mut Mission,
    lease: &mut MissionLease,
    reason: impl Into<String>,
    gateway: &mut G,
) -> Result<(), MissionError> {
    let reason = reason.into();
    mission.abort(reason.clone(), lease)?;
    gateway
        .minimum_risk(mission.id(), &reason)
        .map_err(|_| MissionError::CompensationFailed)
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum MissionError {
    #[error("mission transition is invalid")]
    InvalidTransition,
    #[error("plan is invalid")]
    InvalidPlan,
    #[error("allocation is invalid")]
    InvalidAllocation,
    #[error("resource is already exclusively allocated")]
    DoubleAllocation,
    #[error("allocation capability or cell epoch is stale")]
    StaleAllocation,
    #[error("conflict record or transition is invalid")]
    InvalidConflict,
    #[error("residual conflict lacks independent permitted approval")]
    ResidualConflictNotApproved,
    #[error("mission authorization snapshots or dependencies are incomplete/stale")]
    AuthorizationDenied,
    #[error("lease is invalid")]
    InvalidLease,
    #[error("lease holder, fence, or validity is stale")]
    StaleLease,
    #[error("dispatch is denied by lease, snapshots, conflicts, or authority")]
    DispatchDenied,
    #[error("relay connectivity plan is incomplete")]
    InvalidRelayPlan,
    #[error("abort reason is required")]
    MissingAbortReason,
    #[error("minimum-risk compensation failed")]
    CompensationFailed,
    #[error("aggregate version exhausted")]
    VersionExhausted,
}
