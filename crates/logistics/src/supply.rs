//! Explainable deterministic supply planning behind a replaceable optimizer port.
#![allow(missing_docs)]
use crate::LogisticsError;
use shared_kernel::EntityId;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Demand {
    pub capability: String,
    pub quantity: u64,
    pub unit: String,
    pub priority: u8,
    pub destination: String,
    pub deadline_epoch_seconds: i64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StockOption {
    pub item_id: EntityId,
    pub capability: String,
    pub available: u64,
    pub unit: String,
    pub location: String,
    pub lead_time_seconds: u64,
    pub lead_time_uncertainty_seconds: u64,
    pub energy_ready: bool,
    pub maintenance_ready: bool,
    pub transport_ready: bool,
    pub substitution_for: BTreeSet<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplyAllocation {
    pub demand_index: usize,
    pub item_id: EntityId,
    pub quantity: u64,
    pub unit: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplyRecommendation {
    pub allocations: Vec<SupplyAllocation>,
    pub assumptions: Vec<String>,
    pub bottlenecks: Vec<String>,
    pub alternatives: Vec<String>,
}
pub trait SupplyOptimizer {
    fn optimize(
        &self,
        demands: &[Demand],
        stock: &[StockOption],
    ) -> Result<SupplyRecommendation, LogisticsError>;
}
#[derive(Clone, Copy, Debug, Default)]
pub struct DeterministicBaselineOptimizer;
impl SupplyOptimizer for DeterministicBaselineOptimizer {
    fn optimize(
        &self,
        demands: &[Demand],
        stock: &[StockOption],
    ) -> Result<SupplyRecommendation, LogisticsError> {
        if demands.is_empty()
            || demands.iter().any(|d| {
                d.capability.trim().is_empty() || d.quantity == 0 || d.unit.trim().is_empty()
            })
        {
            return Err(LogisticsError::InvalidSupplyPlan);
        }
        let mut remaining = stock
            .iter()
            .map(|s| (s.item_id.to_string(), s.available))
            .collect::<BTreeMap<_, _>>();
        let mut order = (0..demands.len()).collect::<Vec<_>>();
        order.sort_by_key(|i| {
            (
                u8::MAX - demands[*i].priority,
                demands[*i].deadline_epoch_seconds,
                *i,
            )
        });
        let mut allocations = Vec::new();
        let mut bottlenecks = Vec::new();
        let mut alternatives = BTreeSet::new();
        for index in order {
            let demand = &demands[index];
            let mut needed = demand.quantity;
            let mut candidates = stock
                .iter()
                .filter(|s| {
                    s.unit == demand.unit
                        && s.energy_ready
                        && s.maintenance_ready
                        && s.transport_ready
                        && (s.capability == demand.capability
                            || s.substitution_for.contains(&demand.capability))
                })
                .collect::<Vec<_>>();
            candidates.sort_by_key(|s| {
                (
                    s.lead_time_seconds
                        .saturating_add(s.lead_time_uncertainty_seconds),
                    s.item_id.to_string(),
                )
            });
            for option in candidates {
                let available = *remaining.get(&option.item_id.to_string()).unwrap_or(&0);
                let take = available.min(needed);
                if take == 0 {
                    continue;
                }
                if option.capability != demand.capability {
                    alternatives.insert(format!(
                        "substitute {} for {}",
                        option.capability, demand.capability
                    ));
                }
                allocations.push(SupplyAllocation {
                    demand_index: index,
                    item_id: option.item_id.clone(),
                    quantity: take,
                    unit: demand.unit.clone(),
                });
                remaining.insert(option.item_id.to_string(), available - take);
                needed -= take;
                if needed == 0 {
                    break;
                }
            }
            if needed > 0 {
                bottlenecks.push(format!(
                    "{} {} {} short at {}",
                    needed, demand.unit, demand.capability, demand.destination
                ));
            }
        }
        Ok(SupplyRecommendation {
            allocations,
            assumptions: vec![
                "stock snapshot is authoritative".into(),
                "lead-time upper bound includes stated uncertainty".into(),
                "energy, maintenance, and transport readiness are hard constraints".into(),
            ],
            bottlenecks,
            alternatives: alternatives.into_iter().collect(),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SupplyPlanState {
    Draft,
    Optimized,
    Approved,
    Executing,
    Replanning,
    Closed,
}
#[derive(Clone, Debug)]
pub struct SupplyPlan {
    pub id: EntityId,
    pub state: SupplyPlanState,
    pub recommendation: Option<SupplyRecommendation>,
    pub input_digest: [u8; 32],
    pub version: u64,
}
impl SupplyPlan {
    pub fn draft(id: EntityId, input_digest: [u8; 32]) -> Result<Self, LogisticsError> {
        if input_digest == [0; 32] {
            return Err(LogisticsError::InvalidSupplyPlan);
        }
        Ok(Self {
            id,
            state: SupplyPlanState::Draft,
            recommendation: None,
            input_digest,
            version: 1,
        })
    }
    pub fn optimize(
        &mut self,
        optimizer: &impl SupplyOptimizer,
        demands: &[Demand],
        stock: &[StockOption],
    ) -> Result<(), LogisticsError> {
        if !matches!(
            self.state,
            SupplyPlanState::Draft | SupplyPlanState::Replanning
        ) {
            return Err(LogisticsError::InvalidSupplyPlan);
        }
        self.recommendation = Some(optimizer.optimize(demands, stock)?);
        self.state = SupplyPlanState::Optimized;
        self.bump()
    }
    pub fn approve(&mut self, allow_shortage: bool) -> Result<(), LogisticsError> {
        let recommendation = self
            .recommendation
            .as_ref()
            .ok_or(LogisticsError::InvalidSupplyPlan)?;
        if self.state != SupplyPlanState::Optimized
            || (!allow_shortage && !recommendation.bottlenecks.is_empty())
        {
            return Err(LogisticsError::InfeasibleSupply);
        }
        self.state = SupplyPlanState::Approved;
        self.bump()
    }
    pub fn replan(&mut self) -> Result<(), LogisticsError> {
        if !matches!(
            self.state,
            SupplyPlanState::Approved | SupplyPlanState::Executing
        ) {
            return Err(LogisticsError::InvalidSupplyPlan);
        }
        self.state = SupplyPlanState::Replanning;
        self.bump()
    }
    fn bump(&mut self) -> Result<(), LogisticsError> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or(LogisticsError::VersionExhausted)?;
        Ok(())
    }
}
