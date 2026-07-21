//! Deterministic hierarchical reference planning over bounded cell summaries.
#![allow(missing_docs)]
use crate::{Digest, MissionError, RelayRequirement};
use shared_kernel::EntityId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellSummary {
    pub cell_id: EntityId,
    pub epoch: u64,
    pub capability: String,
    pub eligible_count: u32,
    pub energy_wh_lower_bound: u64,
    pub valid_until_epoch_seconds: i64,
    pub digest: Digest,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanningRequest {
    pub capability: String,
    pub quantity: u32,
    pub energy_wh_per_asset: u64,
    pub relay: Option<RelayRequirement>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanCandidate {
    pub selected_cell: EntityId,
    pub cell_epoch: u64,
    pub quantity: u32,
    pub summary_digest: Digest,
}
pub trait HierarchicalPlanner {
    fn plan(
        &self,
        request: &PlanningRequest,
        summaries: &[CellSummary],
    ) -> Result<PlanCandidate, MissionError>;
}
#[derive(Clone, Copy, Debug, Default)]
pub struct DeterministicReferencePlanner;
impl HierarchicalPlanner for DeterministicReferencePlanner {
    fn plan(
        &self,
        request: &PlanningRequest,
        summaries: &[CellSummary],
    ) -> Result<PlanCandidate, MissionError> {
        if request.capability.trim().is_empty()
            || request.quantity == 0
            || request.energy_wh_per_asset == 0
            || summaries.len() > 10_000
        {
            return Err(MissionError::InvalidPlan);
        }
        if let Some(relay) = &request.relay {
            relay.validate()?;
        }
        let required_energy = request
            .energy_wh_per_asset
            .checked_mul(u64::from(request.quantity))
            .ok_or(MissionError::InvalidPlan)?;
        let mut candidates = summaries
            .iter()
            .filter(|s| {
                s.epoch > 0
                    && s.digest != [0; 32]
                    && s.capability == request.capability
                    && s.eligible_count >= request.quantity
                    && s.energy_wh_lower_bound >= required_energy
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|a, b| {
            b.eligible_count
                .cmp(&a.eligible_count)
                .then_with(|| a.cell_id.to_string().cmp(&b.cell_id.to_string()))
        });
        let selected = candidates.first().ok_or(MissionError::InvalidPlan)?;
        Ok(PlanCandidate {
            selected_cell: selected.cell_id.clone(),
            cell_epoch: selected.epoch,
            quantity: request.quantity,
            summary_digest: selected.digest,
        })
    }
}
