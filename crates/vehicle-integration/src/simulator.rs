//! Deterministic simulator adapter and protocol idempotency guard.
#![allow(missing_docs)]
use crate::{
    Capability, CapabilityController, ControllerResult, Digest, SafeStateReason, VehicleIntent,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SimulatedVehicleState {
    pub flight_position_mm: [i64; 3],
    pub drive_distance_mm: i64,
    pub tool_units: i64,
    pub minimum_risk_entries: u64,
    pub last_safe_reason: Option<SafeStateReason>,
}
#[derive(Clone, Debug)]
pub struct DeterministicSimulator {
    state: SimulatedVehicleState,
    effects: BTreeMap<String, Digest>,
    crash_next: bool,
}
impl Default for DeterministicSimulator {
    fn default() -> Self {
        Self {
            state: SimulatedVehicleState {
                flight_position_mm: [0; 3],
                drive_distance_mm: 0,
                tool_units: 0,
                minimum_risk_entries: 0,
                last_safe_reason: None,
            },
            effects: BTreeMap::new(),
            crash_next: false,
        }
    }
}
impl DeterministicSimulator {
    pub fn inject_crash(&mut self) {
        self.crash_next = true;
    }
    #[must_use]
    pub fn state(&self) -> &SimulatedVehicleState {
        &self.state
    }
    #[must_use]
    pub fn effect_count(&self) -> usize {
        self.effects.len()
    }
    fn effect_digest(intent: &VehicleIntent) -> Digest {
        intent.payload_digest
    }
}
impl CapabilityController for DeterministicSimulator {
    type Error = ();
    fn apply(&mut self, intent: &VehicleIntent) -> Result<ControllerResult, Self::Error> {
        if self.crash_next {
            self.crash_next = false;
            return Err(());
        }
        let key = intent.intent_id.to_string();
        if let Some(outcome) = self.effects.get(&key) {
            return Ok(ControllerResult::Duplicate {
                prior_outcome: Some(*outcome),
            });
        }
        match intent.capability {
            Capability::Flight => {
                self.state.flight_position_mm[0] = self.state.flight_position_mm[0]
                    .saturating_add(intent.parameters.get("north_mm").copied().unwrap_or(0));
                self.state.flight_position_mm[1] = self.state.flight_position_mm[1]
                    .saturating_add(intent.parameters.get("east_mm").copied().unwrap_or(0));
                self.state.flight_position_mm[2] = self.state.flight_position_mm[2]
                    .saturating_add(intent.parameters.get("up_mm").copied().unwrap_or(0));
            }
            Capability::Drive => {
                self.state.drive_distance_mm = self
                    .state
                    .drive_distance_mm
                    .saturating_add(intent.parameters.get("distance_mm").copied().unwrap_or(0));
            }
            Capability::Tool => {
                self.state.tool_units = self
                    .state
                    .tool_units
                    .saturating_add(intent.parameters.get("units").copied().unwrap_or(0));
            }
        }
        self.effects.insert(key, Self::effect_digest(intent));
        Ok(ControllerResult::Accepted)
    }
    fn minimum_risk(&mut self, _: Capability, reason: SafeStateReason) -> Result<(), Self::Error> {
        self.state.minimum_risk_entries = self.state.minimum_risk_entries.saturating_add(1);
        self.state.last_safe_reason = Some(reason);
        Ok(())
    }
}
