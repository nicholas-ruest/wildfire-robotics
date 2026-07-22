use crate::{Digest, OperationalDomain, PlanningError};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone)]
pub struct SimulatorValidity {
    pub artifact_digest: Digest,
    pub valid_odd: OperationalDomain,
    pub calibrated_against: Vec<Digest>,
    pub simulation_to_reality_gap: f64,
}
#[derive(Debug, Clone)]
pub struct ExpectedInvariant {
    pub name: String,
    pub expected: f64,
    pub tolerance: f64,
}
#[derive(Debug, Clone)]
pub struct ScenarioDefinition {
    pub id: String,
    pub seed: u64,
    pub simulator: SimulatorValidity,
    pub requirements: BTreeSet<String>,
    pub hazards: BTreeSet<String>,
    pub expected: Vec<ExpectedInvariant>,
}
#[derive(Debug, Clone)]
pub struct SpreadScenario {
    definition: ScenarioDefinition,
}
impl SpreadScenario {
    pub fn define(d: ScenarioDefinition) -> Result<Self, PlanningError> {
        if d.id.is_empty()
            || d.requirements.is_empty()
            || d.hazards.is_empty()
            || d.expected.is_empty()
            || d.simulator.calibrated_against.is_empty()
            || !d.simulator.simulation_to_reality_gap.is_finite()
            || d.simulator.simulation_to_reality_gap < 0.0
            || d.expected.iter().any(|e| {
                e.name.is_empty()
                    || !e.expected.is_finite()
                    || !e.tolerance.is_finite()
                    || e.tolerance < 0.0
            })
        {
            return Err(PlanningError::InvalidScenario(
                "evidence and tolerances required",
            ));
        }
        Ok(Self { definition: d })
    }
    pub fn seed(&self) -> u64 {
        self.definition.seed
    }
    pub fn definition(&self) -> &ScenarioDefinition {
        &self.definition
    }
}
#[derive(Default)]
pub struct ScenarioRegistry {
    scenarios: BTreeMap<String, SpreadScenario>,
}
impl ScenarioRegistry {
    pub fn register(&mut self, s: SpreadScenario) -> Result<(), PlanningError> {
        if self.scenarios.contains_key(&s.definition.id) {
            return Err(PlanningError::DuplicateIdentifier);
        }
        self.scenarios.insert(s.definition.id.clone(), s);
        Ok(())
    }
    pub fn get(&self, id: &str) -> Option<&SpreadScenario> {
        self.scenarios.get(id)
    }
}
