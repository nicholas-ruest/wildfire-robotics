//! Habitat and maintenance-zone readiness gates.
#![allow(missing_docs)]
use crate::StationError;
use shared_kernel::EntityId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Readiness {
    Ready,
    Unsafe,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HabitatAssessment {
    pub structure: Readiness,
    pub environment: Readiness,
    pub communications: Readiness,
    pub emergency_energy: Readiness,
    pub isolation_and_fire: Readiness,
    pub compatible_docks: u32,
    pub maintenance_capacity: u32,
    pub evacuation_digest: [u8; 32],
    pub deployment_digest: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HabitatState {
    Planned,
    Commissioned,
    Ready,
    Degraded,
    Isolated,
    Decommissioned,
}

#[derive(Clone, Debug)]
pub struct RobotHabitat {
    id: EntityId,
    pub state: HabitatState,
    assessment: Option<HabitatAssessment>,
}
impl RobotHabitat {
    #[must_use]
    pub fn plan(id: EntityId) -> Self {
        Self {
            id,
            state: HabitatState::Planned,
            assessment: None,
        }
    }
    pub fn commission(&mut self) -> Result<(), StationError> {
        if self.state != HabitatState::Planned {
            return Err(StationError::InvalidTransition);
        }
        self.state = HabitatState::Commissioned;
        Ok(())
    }
    pub fn assess(&mut self, assessment: HabitatAssessment) -> Result<(), StationError> {
        if self.state != HabitatState::Commissioned
            || [
                assessment.structure,
                assessment.environment,
                assessment.communications,
                assessment.emergency_energy,
                assessment.isolation_and_fire,
            ]
            .contains(&Readiness::Unsafe)
            || assessment.compatible_docks == 0
            || assessment.maintenance_capacity == 0
            || assessment.evacuation_digest == [0; 32]
            || assessment.deployment_digest == [0; 32]
        {
            self.state = HabitatState::Degraded;
            return Err(StationError::UnsafeCapacity);
        }
        self.assessment = Some(assessment);
        self.state = HabitatState::Ready;
        Ok(())
    }
    pub fn isolate(&mut self) {
        self.state = HabitatState::Isolated;
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BayState {
    Available,
    Reserved,
    Servicing,
    Blocked,
}
#[derive(Clone, Debug)]
pub struct MaintenanceBay {
    pub id: EntityId,
    pub compatible_procedures: Vec<String>,
    pub state: BayState,
}
impl MaintenanceBay {
    pub fn new(id: EntityId, procedures: Vec<String>) -> Result<Self, StationError> {
        if procedures.is_empty() || procedures.iter().any(|v| v.trim().is_empty()) {
            return Err(StationError::UnsafeCapacity);
        }
        Ok(Self {
            id,
            compatible_procedures: procedures,
            state: BayState::Available,
        })
    }
    pub fn reserve(&mut self, procedure: &str) -> Result<(), StationError> {
        if self.state != BayState::Available
            || !self.compatible_procedures.iter().any(|v| v == procedure)
        {
            return Err(StationError::InvalidTransition);
        }
        self.state = BayState::Reserved;
        Ok(())
    }
    pub fn block(&mut self) {
        self.state = BayState::Blocked;
    }
}
