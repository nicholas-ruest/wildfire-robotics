//! Zone-local ground transition, anchoring, sealing and temporary protection.
//!
//! The aggregate coordinates physical installation, but never owns authority for
//! suppressant, vegetation, scope expansion, or robot capability decisions.
use crate::{DomainError, GroundInstallationId, GroundZoneId, InstallationPhase};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const ABSOLUTE_ZONE_LIMIT: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrerequisiteKind {
    Landed,
    RobotSafe,
    ToolSafe,
    TerrainRights,
    UtilitiesClear,
    PeopleClear,
    WildlifeClear,
    SlopeCompatible,
    WindUpliftCompatible,
    FuelFireCompatible,
    AnchorCompatible,
}

impl PrerequisiteKind {
    pub const ALL: [Self; 11] = [
        Self::Landed,
        Self::RobotSafe,
        Self::ToolSafe,
        Self::TerrainRights,
        Self::UtilitiesClear,
        Self::PeopleClear,
        Self::WildlifeClear,
        Self::SlopeCompatible,
        Self::WindUpliftCompatible,
        Self::FuelFireCompatible,
        Self::AnchorCompatible,
    ];
}

/// A truth asserted by its owning source, with an explicit confidence bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrerequisiteAssessment {
    pub kind: PrerequisiteKind,
    pub satisfied: bool,
    /// Worst-case uncertainty in basis points. Policy requires it not exceed its bound.
    pub uncertainty_bps: u16,
    pub observed_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
}

impl PrerequisiteAssessment {
    fn permits(self, now: DateTime<Utc>, max_uncertainty_bps: u16) -> bool {
        self.satisfied
            && self.uncertainty_bps <= max_uncertainty_bps
            && now >= self.observed_at
            && now < self.valid_until
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroundPrerequisites {
    assessments: Vec<PrerequisiteAssessment>,
}

impl GroundPrerequisites {
    #[must_use]
    pub fn new(assessments: Vec<PrerequisiteAssessment>) -> Self {
        Self { assessments }
    }

    fn permit(&self, now: DateTime<Utc>, max_uncertainty_bps: u16) -> bool {
        PrerequisiteKind::ALL.iter().all(|kind| {
            self.assessments
                .iter()
                .filter(|item| item.kind == *kind)
                .count()
                == 1
                && self
                    .assessments
                    .iter()
                    .any(|item| item.kind == *kind && item.permits(now, max_uncertainty_bps))
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SensorKind {
    PanelContact,
    TopTemperature,
    BottomTemperature,
    EmberExposure,
    Tension,
    Uplift,
    Tear,
    Vent,
    Gap,
}

impl SensorKind {
    pub const ALL: [Self; 9] = [
        Self::PanelContact,
        Self::TopTemperature,
        Self::BottomTemperature,
        Self::EmberExposure,
        Self::Tension,
        Self::Uplift,
        Self::Tear,
        Self::Vent,
        Self::Gap,
    ];
}

/// Signed fixed-point observation in the unit bound by the promoted configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SensorReading {
    pub kind: SensorKind,
    pub value: i64,
    pub uncertainty: u64,
    pub observed_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SensorSuite {
    readings: Vec<SensorReading>,
}

impl SensorSuite {
    #[must_use]
    pub fn new(readings: Vec<SensorReading>) -> Self {
        Self { readings }
    }
    #[must_use]
    pub fn is_current_and_bounded(&self, now: DateTime<Utc>, max_uncertainty: u64) -> bool {
        SensorKind::ALL.iter().all(|kind| {
            self.readings.iter().filter(|r| r.kind == *kind).count() == 1
                && self.readings.iter().any(|r| {
                    r.kind == *kind
                        && r.uncertainty <= max_uncertainty
                        && now >= r.observed_at
                        && now < r.valid_until
                })
        })
    }
    #[must_use]
    pub fn reading(&self, kind: SensorKind) -> Option<&SensorReading> {
        self.readings.iter().find(|reading| reading.kind == kind)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroundFault {
    TerrainIntrusion,
    UtilityIntrusion,
    StaleSensing,
    AnchorPullout,
    Uplift,
    Tear,
    HeatTransfer,
    Gap,
    RobotTool,
    LostCommunications,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroundAction {
    Pause,
    Vent,
    Isolate,
    Reposition,
    RequestSuppression,
    Escalate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectivenessAssessment {
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalWorkKind {
    SuppressantApplication,
    VegetationWork,
    MissionExpansion,
    RobotCapability,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalWorkRequest {
    pub zone: GroundZoneId,
    pub kind: ExternalWorkKind,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroundZone {
    pub id: GroundZoneId,
    pub phase: InstallationPhase,
    pub inhibited: bool,
    pub isolated: bool,
    local_actions: Vec<GroundAction>,
    pub effectiveness: EffectivenessAssessment,
    pub last_sensors: Option<SensorSuite>,
    pub external_requests: Vec<ExternalWorkRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroundInstallation {
    pub id: GroundInstallationId,
    zones: Vec<GroundZone>,
    max_prerequisite_uncertainty_bps: u16,
    max_sensor_uncertainty: u64,
}

impl GroundInstallation {
    pub fn new(
        id: GroundInstallationId,
        zones: Vec<GroundZoneId>,
        max_prerequisite_uncertainty_bps: u16,
        max_sensor_uncertainty: u64,
    ) -> Result<Self, DomainError> {
        let mut unique = zones.clone();
        unique.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        unique.dedup();
        if zones.is_empty()
            || zones.len() > ABSOLUTE_ZONE_LIMIT
            || unique.len() != zones.len()
            || max_prerequisite_uncertainty_bps > 10_000
        {
            return Err(DomainError::InvalidGroundZone);
        }
        Ok(Self {
            id,
            zones: zones
                .into_iter()
                .map(|id| GroundZone {
                    id,
                    phase: InstallationPhase::Landed,
                    inhibited: false,
                    isolated: false,
                    local_actions: Vec::new(),
                    effectiveness: EffectivenessAssessment::Unknown,
                    last_sensors: None,
                    external_requests: Vec::new(),
                })
                .collect(),
            max_prerequisite_uncertainty_bps,
            max_sensor_uncertainty,
        })
    }

    #[must_use]
    pub fn zone(&self, id: &GroundZoneId) -> Option<&GroundZone> {
        self.zones.iter().find(|z| &z.id == id)
    }
    fn zone_mut(&mut self, id: &GroundZoneId) -> Result<&mut GroundZone, DomainError> {
        self.zones
            .iter_mut()
            .find(|z| &z.id == id)
            .ok_or(DomainError::InvalidGroundZone)
    }

    pub fn advance(
        &mut self,
        zone: &GroundZoneId,
        target: InstallationPhase,
        prerequisites: &GroundPrerequisites,
        sensors: Option<SensorSuite>,
        now: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        if !prerequisites.permit(now, self.max_prerequisite_uncertainty_bps) {
            self.zone_mut(zone)?.inhibited = true;
            return Err(DomainError::GroundWorkInhibited);
        }
        let current = self.zone(zone).ok_or(DomainError::InvalidGroundZone)?.phase;
        let legal = matches!(
            (current, target),
            (InstallationPhase::Landed, InstallationPhase::Transitioning)
                | (
                    InstallationPhase::Transitioning,
                    InstallationPhase::Anchoring
                )
                | (InstallationPhase::Anchoring, InstallationPhase::Sealing)
                | (InstallationPhase::Sealing, InstallationPhase::Active)
                | (InstallationPhase::Degraded, InstallationPhase::Recovering)
                | (
                    InstallationPhase::Recovering,
                    InstallationPhase::Removed | InstallationPhase::TemporarilyLeft
                )
        );
        if !legal {
            return Err(DomainError::InvalidTransition);
        }
        if matches!(target, InstallationPhase::Active) {
            let Some(ref suite) = sensors else {
                return Err(DomainError::GroundSensingInhibited);
            };
            if !suite.is_current_and_bounded(now, self.max_sensor_uncertainty) {
                self.zone_mut(zone)?.inhibited = true;
                return Err(DomainError::GroundSensingInhibited);
            }
        }
        let state = self.zone_mut(zone)?;
        state.phase = target;
        state.inhibited = false;
        if sensors.is_some() {
            state.last_sensors = sensors;
        }
        Ok(())
    }

    pub fn apply_fault(
        &mut self,
        zone: &GroundZoneId,
        fault: GroundFault,
    ) -> Result<GroundAction, DomainError> {
        let state = self.zone_mut(zone)?;
        state.inhibited = true;
        if !matches!(
            state.phase,
            InstallationPhase::Removed | InstallationPhase::TemporarilyLeft
        ) {
            state.phase = InstallationPhase::Degraded;
        }
        let action = match fault {
            GroundFault::HeatTransfer | GroundFault::Uplift => GroundAction::Vent,
            GroundFault::AnchorPullout | GroundFault::Tear | GroundFault::Gap => {
                GroundAction::Isolate
            }
            GroundFault::TerrainIntrusion
            | GroundFault::UtilityIntrusion
            | GroundFault::RobotTool
            | GroundFault::StaleSensing
            | GroundFault::LostCommunications => GroundAction::Pause,
        };
        if action == GroundAction::Isolate {
            state.isolated = true;
        }
        if !state.local_actions.contains(&action) {
            state.local_actions.push(action);
        }
        Ok(action)
    }

    /// Applies only containment actions owned by this aggregate and only to one zone.
    pub fn apply_local_policy(
        &mut self,
        zone: &GroundZoneId,
        action: GroundAction,
    ) -> Result<(), DomainError> {
        if action == GroundAction::RequestSuppression {
            return Err(DomainError::GroundPolicyInhibited);
        }
        let state = self.zone_mut(zone)?;
        match action {
            GroundAction::Pause
            | GroundAction::Vent
            | GroundAction::Reposition
            | GroundAction::Escalate => {}
            GroundAction::Isolate => {
                state.isolated = true;
                state.inhibited = true;
            }
            GroundAction::RequestSuppression => unreachable!("rejected above"),
        }
        if !state.local_actions.contains(&action) {
            state.local_actions.push(action);
        }
        Ok(())
    }

    #[must_use]
    pub fn has_applied_action(&self, zone: &GroundZoneId, action: GroundAction) -> bool {
        self.zone(zone)
            .is_some_and(|state| state.local_actions.contains(&action))
    }

    /// Re-evaluates the continuously required suite; loss of freshness is a local fault.
    pub fn reevaluate_sensing(
        &mut self,
        zone: &GroundZoneId,
        now: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        let current = self.zone(zone).ok_or(DomainError::InvalidGroundZone)?;
        if current.phase != InstallationPhase::Active {
            return Ok(());
        }
        let current = current
            .last_sensors
            .as_ref()
            .is_some_and(|suite| suite.is_current_and_bounded(now, self.max_sensor_uncertainty));
        if current {
            return Ok(());
        }
        self.apply_fault(zone, GroundFault::StaleSensing)?;
        Err(DomainError::GroundSensingInhibited)
    }

    pub fn request_external_work(
        &mut self,
        zone: &GroundZoneId,
        kind: ExternalWorkKind,
        rationale: &str,
    ) -> Result<ExternalWorkRequest, DomainError> {
        if rationale.trim().is_empty() {
            return Err(DomainError::GroundPolicyInhibited);
        }
        let request = ExternalWorkRequest {
            zone: zone.clone(),
            kind,
            rationale: rationale.trim().to_owned(),
        };
        self.zone_mut(zone)?.external_requests.push(request.clone());
        Ok(request)
    }
}
