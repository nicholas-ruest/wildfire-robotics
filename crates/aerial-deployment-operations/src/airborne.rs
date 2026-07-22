//! Deterministic, cohort-local airborne deployment protocol (AD-INV-005/006/009).
use crate::{AirborneDeploymentId, CommandId, DeploymentPhase, DomainError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl DeploymentPhase {
    /// The only nominal airborne phase ordering. Terminal containment branches are excluded.
    pub const PROTOCOL: [Self; 11] = [
        Self::Retained,
        Self::Extracted,
        Self::Stabilized,
        Self::CohortReleasing,
        Self::ParafoilEstablished,
        Self::FormationAcquired,
        Self::SectionReefedRelease,
        Self::TensionBalancedExpansion,
        Self::TerrainAlignment,
        Self::Landing,
        Self::Landed,
    ];

    const fn nominal_successor(self) -> Option<Self> {
        match self {
            Self::Retained => Some(Self::Extracted),
            Self::Extracted => Some(Self::Stabilized),
            Self::Stabilized => Some(Self::CohortReleasing),
            Self::CohortReleasing => Some(Self::ParafoilEstablished),
            Self::ParafoilEstablished => Some(Self::FormationAcquired),
            Self::FormationAcquired => Some(Self::SectionReefedRelease),
            Self::SectionReefedRelease => Some(Self::TensionBalancedExpansion),
            Self::TensionBalancedExpansion => Some(Self::TerrainAlignment),
            Self::TerrainAlignment => Some(Self::Landing),
            Self::Landing => Some(Self::Landed),
            Self::Landed | Self::Isolated | Self::Jettisoned => None,
        }
    }
}

/// Every independently measured safety margin required before a nominal transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(usize)]
pub enum MarginKind {
    Separation = 0,
    TetherRouting = 1,
    TetherRate = 2,
    TetherTension = 3,
    ReefingState = 4,
    VentState = 5,
    Stability = 6,
    Wind = 7,
    NavigationTime = 8,
    Communications = 9,
    Clearance = 10,
    RemainingContingencies = 11,
}

impl MarginKind {
    pub const COUNT: usize = 12;
}

/// Immutable sensor snapshot used for exactly one transition decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionMargins {
    measured_at: DateTime<Utc>,
    valid_until: DateTime<Utc>,
    /// Signed engineering margins in the units declared by the bound configuration.
    values: [i32; MarginKind::COUNT],
}

impl TransitionMargins {
    pub fn new(
        measured_at: DateTime<Utc>,
        valid_until: DateTime<Utc>,
        values: [i32; MarginKind::COUNT],
    ) -> Result<Self, DomainError> {
        if valid_until <= measured_at || values.iter().any(|value| *value <= 0) {
            return Err(DomainError::UnsafeTransitionMargin);
        }
        Ok(Self {
            measured_at,
            valid_until,
            values,
        })
    }

    #[must_use]
    pub const fn margin(&self, kind: MarginKind) -> i32 {
        self.values[kind as usize]
    }

    fn is_current_at(&self, now: DateTime<Utc>) -> bool {
        now >= self.measured_at && now < self.valid_until
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalTrust {
    /// Trust is assigned only by an owning authority outside advisory adapters.
    Unassigned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContainmentAction {
    Retain,
    Pause,
    Reef,
    Vent,
    Isolate,
    Breakaway,
    EmergencyLand,
    SafeSectorJettison,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainmentOutcome {
    Retained,
    Paused,
    Reefed,
    Vented,
    Isolated,
    BrokenAway,
    EmergencyLanding,
    Jettisoned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FaultKind {
    Robot,
    Panel,
    Tether,
    Parafoil,
    Controller,
    Network,
    LearnedAdapter,
}

impl FaultKind {
    pub const ALL: [Self; 7] = [
        Self::Robot,
        Self::Panel,
        Self::Tether,
        Self::Parafoil,
        Self::Controller,
        Self::Network,
        Self::LearnedAdapter,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalCohort {
    pub index: u8,
    pub phase: DeploymentPhase,
    pub trust: LocalTrust,
    paused: bool,
    reefed: bool,
    vented: bool,
    faults: Vec<FaultKind>,
    last_margins: Option<TransitionMargins>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HierarchicalSummary {
    pub total: u8,
    pub active: u8,
    pub isolated: u8,
    pub jettisoned: u8,
}

/// Adapter input contains observations only and deliberately exposes no command handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvisorySnapshot {
    pub cohort: u8,
    pub phase: DeploymentPhase,
}

/// Opaque advisory data. It has no authorization, trust, transition, or override field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvisoryOutput {
    source: String,
    declared_score_bps: u16,
    opaque_payload: Vec<u8>,
}

impl AdvisoryOutput {
    pub fn new(
        source: &str,
        declared_score_bps: u16,
        opaque_payload: Vec<u8>,
    ) -> Result<Self, DomainError> {
        let source = source.trim();
        if source.is_empty() || source.len() > 128 || declared_score_bps > 10_000 {
            return Err(DomainError::InvalidAdvisory);
        }
        Ok(Self {
            source: source.to_owned(),
            declared_score_bps,
            opaque_payload,
        })
    }

    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }
}

/// Optional ruv-drone/RVM integrations implement this authority-free port.
pub trait AdvisoryAdapter {
    fn advise(&self, snapshot: &AdvisorySnapshot) -> AdvisoryOutput;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AirborneDeployment {
    pub id: AirborneDeploymentId,
    cohort_limit: u8,
    cohorts: Vec<LocalCohort>,
    applied_commands: HashMap<CommandId, (u8, ContainmentAction, bool, ContainmentOutcome)>,
}

impl AirborneDeployment {
    pub fn new(
        id: AirborneDeploymentId,
        cohort_count: u8,
        cohort_limit: u8,
    ) -> Result<Self, DomainError> {
        const ABSOLUTE_COHORT_LIMIT: u8 = 8;
        if cohort_count == 0
            || cohort_limit == 0
            || cohort_limit > ABSOLUTE_COHORT_LIMIT
            || cohort_count > cohort_limit
        {
            return Err(DomainError::InvalidCohort);
        }
        Ok(Self {
            id,
            cohort_limit,
            cohorts: (0..cohort_count)
                .map(|index| LocalCohort {
                    index,
                    phase: DeploymentPhase::Retained,
                    trust: LocalTrust::Unassigned,
                    paused: false,
                    reefed: false,
                    vented: false,
                    faults: Vec::new(),
                    last_margins: None,
                })
                .collect(),
            applied_commands: HashMap::new(),
        })
    }

    #[must_use]
    pub fn cohort(&self, index: u8) -> Option<&LocalCohort> {
        self.cohorts.get(usize::from(index))
    }

    #[must_use]
    pub const fn cohort_limit(&self) -> u8 {
        self.cohort_limit
    }

    pub fn transition(
        &mut self,
        cohort: u8,
        target: DeploymentPhase,
        now: DateTime<Utc>,
        margins: TransitionMargins,
    ) -> Result<(), DomainError> {
        let local = self
            .cohorts
            .get_mut(usize::from(cohort))
            .ok_or(DomainError::InvalidCohort)?;
        if local.paused
            || local.phase.nominal_successor() != Some(target)
            || !margins.is_current_at(now)
        {
            return Err(if margins.is_current_at(now) {
                DomainError::InvalidTransition
            } else {
                DomainError::UnsafeTransitionMargin
            });
        }
        local.phase = target;
        local.last_margins = Some(margins);
        Ok(())
    }

    pub fn contain(
        &mut self,
        command: CommandId,
        cohort: u8,
        action: ContainmentAction,
        safe_sector_confirmed: bool,
    ) -> Result<ContainmentOutcome, DomainError> {
        if let Some((seen_cohort, seen_action, seen_sector, outcome)) =
            self.applied_commands.get(&command)
        {
            return if (*seen_cohort, *seen_action, *seen_sector)
                == (cohort, action, safe_sector_confirmed)
            {
                Ok(*outcome)
            } else {
                Err(DomainError::ReplayConflict)
            };
        }
        let local = self
            .cohorts
            .get_mut(usize::from(cohort))
            .ok_or(DomainError::InvalidCohort)?;
        if matches!(
            local.phase,
            DeploymentPhase::Isolated | DeploymentPhase::Jettisoned | DeploymentPhase::Landed
        ) || (action == ContainmentAction::Retain && local.phase != DeploymentPhase::Retained)
        {
            return Err(DomainError::InvalidTransition);
        }
        if action == ContainmentAction::SafeSectorJettison && !safe_sector_confirmed {
            return Err(DomainError::SafeSectorNotConfirmed);
        }
        let outcome = match action {
            ContainmentAction::Retain => ContainmentOutcome::Retained,
            ContainmentAction::Pause => {
                local.paused = true;
                ContainmentOutcome::Paused
            }
            ContainmentAction::Reef => {
                local.reefed = true;
                ContainmentOutcome::Reefed
            }
            ContainmentAction::Vent => {
                local.vented = true;
                ContainmentOutcome::Vented
            }
            ContainmentAction::Isolate => {
                local.phase = DeploymentPhase::Isolated;
                ContainmentOutcome::Isolated
            }
            ContainmentAction::Breakaway => {
                local.phase = DeploymentPhase::Isolated;
                ContainmentOutcome::BrokenAway
            }
            ContainmentAction::EmergencyLand => {
                local.phase = DeploymentPhase::Landing;
                ContainmentOutcome::EmergencyLanding
            }
            ContainmentAction::SafeSectorJettison => {
                local.phase = DeploymentPhase::Jettisoned;
                ContainmentOutcome::Jettisoned
            }
        };
        self.applied_commands
            .insert(command, (cohort, action, safe_sector_confirmed, outcome));
        Ok(outcome)
    }

    pub fn report_fault(&mut self, cohort: u8, fault: FaultKind) -> Result<(), DomainError> {
        let local = self
            .cohorts
            .get_mut(usize::from(cohort))
            .ok_or(DomainError::InvalidCohort)?;
        if !local.faults.contains(&fault) {
            local.faults.push(fault);
        }
        if !matches!(
            local.phase,
            DeploymentPhase::Landed | DeploymentPhase::Jettisoned
        ) {
            local.phase = DeploymentPhase::Isolated;
        }
        Ok(())
    }

    #[must_use]
    pub fn summary(&self) -> HierarchicalSummary {
        let isolated = self
            .cohorts
            .iter()
            .filter(|cohort| cohort.phase == DeploymentPhase::Isolated)
            .fold(0_u8, |count, _| count.saturating_add(1));
        let jettisoned = self
            .cohorts
            .iter()
            .filter(|cohort| cohort.phase == DeploymentPhase::Jettisoned)
            .fold(0_u8, |count, _| count.saturating_add(1));
        let total = self
            .cohorts
            .iter()
            .fold(0_u8, |count, _| count.saturating_add(1));
        HierarchicalSummary {
            total,
            active: total - isolated - jettisoned,
            isolated,
            jettisoned,
        }
    }

    pub fn collect_advice(
        &self,
        cohort: u8,
        adapter: &dyn AdvisoryAdapter,
    ) -> Result<AdvisoryOutput, DomainError> {
        let local = self.cohort(cohort).ok_or(DomainError::InvalidCohort)?;
        Ok(adapter.advise(&AdvisorySnapshot {
            cohort,
            phase: local.phase,
        }))
    }
}
