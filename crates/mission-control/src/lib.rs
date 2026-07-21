#![forbid(unsafe_code)]
//! Mission lifecycle with fail-closed authorization and exclusive expiring leases.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_kernel::{EntityId, TimeWindow};
use thiserror::Error;

/// Safety-relevant mission lifecycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MissionState {
    /// Constructed but not authorized.
    Planned,
    /// Human authority and safety policy approved execution.
    Authorized,
    /// A controller with a valid lease dispatched the mission.
    Dispatched,
    /// Objective completed normally.
    Completed,
    /// Execution was permanently halted.
    Aborted,
}

/// Stable denial reasons returned by a cross-context authorization adapter.
///
/// The adapter maps signed Safety Assurance and Incident Command contracts to
/// this local language; their domain types never enter this crate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationDenial {
    /// The applicable signed constraint is absent, invalid, or expired.
    ConstraintNotActive,
    /// The mission conditions exceed the promoted operational design domain.
    OutsideVerifiedOdd,
}

/// Port used to validate external authority and safety facts at authorization.
pub trait AuthorizationPolicy {
    /// Evaluates the mission against immutable contract snapshots at `now`.
    fn authorize(
        &self,
        mission_id: &EntityId,
        assignment_validity: TimeWindow,
        now: DateTime<Utc>,
    ) -> Result<(), AuthorizationDenial>;
}

/// Exclusive time-bounded controller ownership.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissionLease {
    holder: EntityId,
    validity: TimeWindow,
    mission_version: u64,
}

impl MissionLease {
    /// Returns whether this holder still owns the expected mission version.
    #[must_use]
    pub fn permits(&self, holder: &EntityId, version: u64, now: DateTime<Utc>) -> bool {
        &self.holder == holder && self.mission_version == version && self.validity.contains(now)
    }
}

/// Aggregate protecting mission state and authority.
#[derive(Clone, Debug)]
pub struct Mission {
    /// Mission identity.
    pub id: EntityId,
    /// Vehicle allocated to this mission.
    pub vehicle_id: EntityId,
    /// Current lifecycle state.
    pub state: MissionState,
    /// Optimistic concurrency version.
    pub version: u64,
    assignment_validity: TimeWindow,
    lease: Option<MissionLease>,
    minimum_risk_reason: Option<String>,
}

impl Mission {
    /// Plans a mission without granting execution authority.
    #[must_use]
    pub fn plan(id: EntityId, vehicle_id: EntityId, assignment_validity: TimeWindow) -> Self {
        Self {
            id,
            vehicle_id,
            state: MissionState::Planned,
            version: 1,
            assignment_validity,
            lease: None,
            minimum_risk_reason: None,
        }
    }

    /// Authorizes only inside assignment time and after the injected policy port approves.
    pub fn authorize(
        &mut self,
        policy: &impl AuthorizationPolicy,
        now: DateTime<Utc>,
    ) -> Result<(), MissionError> {
        if self.state != MissionState::Planned {
            return Err(MissionError::InvalidTransition);
        }
        if !self.assignment_validity.contains(now) {
            return Err(MissionError::AssignmentNotActive);
        }
        policy
            .authorize(&self.id, self.assignment_validity, now)
            .map_err(|denial| match denial {
                AuthorizationDenial::ConstraintNotActive => MissionError::SafetyConstraintNotActive,
                AuthorizationDenial::OutsideVerifiedOdd => MissionError::OutsideVerifiedOdd,
            })?;
        self.state = MissionState::Authorized;
        self.version += 1;
        Ok(())
    }

    /// Acquires exclusive control. Existing unexpired leases cannot be replaced.
    pub fn acquire_lease(
        &mut self,
        holder: EntityId,
        validity: TimeWindow,
        now: DateTime<Utc>,
        expected_version: u64,
    ) -> Result<(), MissionError> {
        if self.version != expected_version {
            return Err(MissionError::ConcurrencyConflict);
        }
        if self.state != MissionState::Authorized {
            return Err(MissionError::InvalidTransition);
        }
        if !validity.contains(now) || validity.ends_at > self.assignment_validity.ends_at {
            return Err(MissionError::InvalidLeaseWindow);
        }
        if self
            .lease
            .as_ref()
            .is_some_and(|lease| lease.validity.contains(now))
        {
            return Err(MissionError::LeaseAlreadyHeld);
        }
        self.lease = Some(MissionLease {
            holder,
            validity,
            mission_version: self.version,
        });
        Ok(())
    }

    /// Dispatches only the current exclusive holder before all authority expires.
    pub fn dispatch(&mut self, holder: &EntityId, now: DateTime<Utc>) -> Result<(), MissionError> {
        if self.state != MissionState::Authorized || !self.assignment_validity.contains(now) {
            return Err(MissionError::AssignmentNotActive);
        }
        let lease = self.lease.as_ref().ok_or(MissionError::LeaseRequired)?;
        if !lease.permits(holder, self.version, now) {
            return Err(MissionError::LeaseNotValid);
        }
        self.state = MissionState::Dispatched;
        self.version += 1;
        self.lease = None;
        Ok(())
    }

    /// Idempotently aborts and records the minimum-risk reason.
    pub fn abort(&mut self, reason: impl Into<String>) -> Result<(), MissionError> {
        if self.state == MissionState::Completed {
            return Err(MissionError::InvalidTransition);
        }
        if self.state == MissionState::Aborted {
            return Ok(());
        }
        let reason = reason.into();
        if reason.trim().is_empty() {
            return Err(MissionError::MissingAbortReason);
        }
        self.state = MissionState::Aborted;
        self.minimum_risk_reason = Some(reason);
        self.lease = None;
        self.version += 1;
        Ok(())
    }

    /// Completes only a dispatched mission.
    pub fn complete(&mut self) -> Result<(), MissionError> {
        if self.state != MissionState::Dispatched {
            return Err(MissionError::InvalidTransition);
        }
        self.state = MissionState::Completed;
        self.version += 1;
        Ok(())
    }
}

/// Fail-closed mission errors.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum MissionError {
    /// Lifecycle operation is not permitted in the current state.
    #[error("invalid mission state transition")]
    InvalidTransition,
    /// Assignment is not active at the evaluation time.
    #[error("assignment is not active")]
    AssignmentNotActive,
    /// Runtime safety constraint is absent or expired.
    #[error("safety constraint is not active")]
    SafetyConstraintNotActive,
    /// Requested conditions exceed verified vehicle capability.
    #[error("mission is outside the vehicle operational design domain")]
    OutsideVerifiedOdd,
    /// Expected aggregate version is stale.
    #[error("mission version conflict")]
    ConcurrencyConflict,
    /// Lease begins outside authority or outlives the assignment.
    #[error("invalid mission lease window")]
    InvalidLeaseWindow,
    /// Another controller has an active lease.
    #[error("mission lease is already held")]
    LeaseAlreadyHeld,
    /// Dispatch requires a lease.
    #[error("mission lease is required")]
    LeaseRequired,
    /// Lease holder/version/time does not match.
    #[error("mission lease is not valid")]
    LeaseNotValid,
    /// Abort audit records require a reason.
    #[error("abort reason is required")]
    MissingAbortReason,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    struct Policy(Result<(), AuthorizationDenial>);

    impl AuthorizationPolicy for Policy {
        fn authorize(
            &self,
            _mission_id: &EntityId,
            _assignment_validity: TimeWindow,
            _now: DateTime<Utc>,
        ) -> Result<(), AuthorizationDenial> {
            self.0
        }
    }

    fn fixture() -> Result<(Mission, DateTime<Utc>), Box<dyn std::error::Error>> {
        let now = Utc::now();
        let validity = TimeWindow::new(now - Duration::minutes(1), now + Duration::minutes(30))?;
        let mission = Mission::plan(EntityId::new(), EntityId::new(), validity);
        Ok((mission, now))
    }

    #[test]
    fn valid_authority_and_lease_dispatch() -> Result<(), Box<dyn std::error::Error>> {
        let (mut mission, now) = fixture()?;
        mission.authorize(&Policy(Ok(())), now)?;
        let holder = EntityId::new();
        let lease = TimeWindow::new(now, now + Duration::minutes(5))?;
        mission.acquire_lease(holder.clone(), lease, now, mission.version)?;
        mission.dispatch(&holder, now)?;
        assert_eq!(mission.state, MissionState::Dispatched);
        Ok(())
    }

    #[test]
    fn unverified_operational_domain_is_denied() -> Result<(), Box<dyn std::error::Error>> {
        let (mut mission, now) = fixture()?;
        assert_eq!(
            mission.authorize(&Policy(Err(AuthorizationDenial::OutsideVerifiedOdd)), now),
            Err(MissionError::OutsideVerifiedOdd)
        );
        Ok(())
    }

    #[test]
    fn stale_version_cannot_take_lease() -> Result<(), Box<dyn std::error::Error>> {
        let (mut mission, now) = fixture()?;
        mission.authorize(&Policy(Ok(())), now)?;
        let lease = TimeWindow::new(now, now + Duration::minutes(5))?;
        assert_eq!(
            mission.acquire_lease(EntityId::new(), lease, now, 1),
            Err(MissionError::ConcurrencyConflict)
        );
        Ok(())
    }
}
