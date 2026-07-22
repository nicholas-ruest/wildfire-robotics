//! Deterministic medic-pod and maintenance-robot simulator adapters.
#![allow(missing_docs)]
use crate::{CareError, RecoveryAssessment, WorkOrder};
use shared_kernel::EntityId;
use std::collections::BTreeSet;
pub trait MedicPodPort {
    fn assess(&self, asset: &EntityId) -> RecoveryAssessment;
    fn stabilize(&mut self, asset: &EntityId) -> Result<(), CareError>;
    fn recover(&mut self, asset: &EntityId) -> Result<(), CareError>;
    fn safe_retreat(&mut self) -> Result<(), CareError>;
}
pub trait MaintenanceRobotPort {
    fn execute(&mut self, order: &WorkOrder) -> Result<(), CareError>;
    fn stop_and_isolate(&mut self, asset: &EntityId) -> Result<(), CareError>;
}
#[derive(Clone, Debug)]
pub struct DeterministicMedicSimulator {
    pub assessment: RecoveryAssessment,
    pub fail_recovery: bool,
    stabilized: BTreeSet<String>,
    recovered: BTreeSet<String>,
    pub retreated: bool,
}
impl DeterministicMedicSimulator {
    #[must_use]
    pub fn new(assessment: RecoveryAssessment) -> Self {
        Self {
            assessment,
            fail_recovery: false,
            stabilized: BTreeSet::new(),
            recovered: BTreeSet::new(),
            retreated: false,
        }
    }
    #[must_use]
    pub fn recovered(&self, id: &EntityId) -> bool {
        self.recovered.contains(&id.to_string())
    }
}
impl MedicPodPort for DeterministicMedicSimulator {
    fn assess(&self, _: &EntityId) -> RecoveryAssessment {
        self.assessment.clone()
    }
    fn stabilize(&mut self, id: &EntityId) -> Result<(), CareError> {
        if self.assessment.energy_isolated != crate::SafetyFact::Passed
            || self.assessment.tools_stabilized != crate::SafetyFact::Passed
        {
            return Err(CareError::UnsafeRecovery);
        }
        self.stabilized.insert(id.to_string());
        Ok(())
    }
    fn recover(&mut self, id: &EntityId) -> Result<(), CareError> {
        if self.fail_recovery || !self.stabilized.contains(&id.to_string()) {
            return Err(CareError::UnsafeRecovery);
        }
        self.recovered.insert(id.to_string());
        Ok(())
    }
    fn safe_retreat(&mut self) -> Result<(), CareError> {
        self.retreated = true;
        Ok(())
    }
}
#[derive(Clone, Debug, Default)]
pub struct DeterministicMaintenanceSimulator {
    effects: BTreeSet<String>,
    isolated: BTreeSet<String>,
}
impl MaintenanceRobotPort for DeterministicMaintenanceSimulator {
    fn execute(&mut self, order: &WorkOrder) -> Result<(), CareError> {
        if !order.isolated {
            return Err(CareError::ProcedureDenied);
        }
        self.effects.insert(order.id.to_string());
        Ok(())
    }
    fn stop_and_isolate(&mut self, id: &EntityId) -> Result<(), CareError> {
        self.isolated.insert(id.to_string());
        Ok(())
    }
}
