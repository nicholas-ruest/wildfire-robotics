//! Intermodal pods, hybrid carriers, bounded waves, and time-expanded flow.
#![allow(missing_docs)]
use crate::LogisticsError;
use shared_kernel::EntityId;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PodLoad {
    pub asset_id: EntityId,
    pub mass_grams: u64,
    pub volume_cm3: u64,
    pub longitudinal_mm: i64,
    pub securement: bool,
    pub energy_isolated: bool,
    pub compatible_interface: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PodLimits {
    pub max_mass_grams: u64,
    pub max_volume_cm3: u64,
    pub max_abs_cog_mm: u64,
    pub max_axle_grams: u64,
    pub required_interface: String,
    pub max_assets: u32,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PodState {
    Available,
    Loading,
    Sealed,
    InTransit,
    Staged,
    Unloading,
    Returned,
    Servicing,
}
#[derive(Clone, Debug)]
pub struct TransportPod {
    pub id: EntityId,
    pub state: PodState,
    pub limits: PodLimits,
    manifest: Vec<PodLoad>,
    manifest_digest: Option<[u8; 32]>,
}
impl TransportPod {
    pub fn new(id: EntityId, limits: PodLimits) -> Result<Self, LogisticsError> {
        if limits.max_mass_grams == 0
            || limits.max_volume_cm3 == 0
            || limits.max_axle_grams == 0
            || limits.required_interface.trim().is_empty()
            || limits.max_assets == 0
        {
            return Err(LogisticsError::UnsafeManifest);
        }
        Ok(Self {
            id,
            state: PodState::Available,
            limits,
            manifest: Vec::new(),
            manifest_digest: None,
        })
    }
    pub fn load(&mut self, entry: PodLoad) -> Result<(), LogisticsError> {
        if !matches!(self.state, PodState::Available | PodState::Loading)
            || self.manifest.iter().any(|v| v.asset_id == entry.asset_id)
            || !entry.securement
            || !entry.energy_isolated
            || entry.compatible_interface != self.limits.required_interface
            || self.manifest.len() >= self.limits.max_assets as usize
        {
            return Err(LogisticsError::UnsafeManifest);
        }
        self.manifest.push(entry);
        if let Err(error) = self.validate_load() {
            self.manifest.pop();
            return Err(error);
        }
        self.state = PodState::Loading;
        Ok(())
    }
    pub fn seal(&mut self, digest: [u8; 32]) -> Result<(), LogisticsError> {
        self.validate_load()?;
        if self.state != PodState::Loading || digest == [0; 32] || self.manifest.is_empty() {
            return Err(LogisticsError::UnsafeManifest);
        }
        self.manifest_digest = Some(digest);
        self.state = PodState::Sealed;
        Ok(())
    }
    fn validate_load(&self) -> Result<(), LogisticsError> {
        let mass = self
            .manifest
            .iter()
            .try_fold(0u64, |a, v| a.checked_add(v.mass_grams))
            .ok_or(LogisticsError::UnsafeManifest)?;
        let volume = self
            .manifest
            .iter()
            .try_fold(0u64, |a, v| a.checked_add(v.volume_cm3))
            .ok_or(LogisticsError::UnsafeManifest)?;
        let moment = self
            .manifest
            .iter()
            .try_fold(0i128, |a, v| {
                a.checked_add(i128::from(v.longitudinal_mm) * i128::from(v.mass_grams))
            })
            .ok_or(LogisticsError::UnsafeManifest)?;
        let cog = if mass == 0 {
            0
        } else {
            moment / i128::from(mass)
        };
        if mass > self.limits.max_mass_grams
            || volume > self.limits.max_volume_cm3
            || cog.unsigned_abs() > u128::from(self.limits.max_abs_cog_mm)
            || mass.saturating_add(1) / 2 > self.limits.max_axle_grams
        {
            return Err(LogisticsError::UnsafeManifest);
        }
        Ok(())
    }
    #[must_use]
    pub fn asset_count(&self) -> usize {
        self.manifest.len()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CarrierState {
    Available,
    Reserved,
    Loading,
    Ready,
    InTransit,
    Recovering,
    Servicing,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationalCheck {
    Passed,
    Failed,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CarrierReadiness {
    pub traction_energy_wh: u64,
    pub reserve_wh: u64,
    pub fuel_energy_wh: u64,
    pub braking: OperationalCheck,
    pub steering: OperationalCheck,
    pub tires: OperationalCheck,
    pub odd: OperationalCheck,
    pub recovery: OperationalCheck,
}
#[derive(Clone, Debug)]
pub struct Carrier {
    pub id: EntityId,
    pub state: CarrierState,
    pub max_pod_mass_grams: u64,
    pub readiness: CarrierReadiness,
    pub platoon_epoch: u64,
}
impl Carrier {
    pub fn new(
        id: EntityId,
        max_mass: u64,
        readiness: CarrierReadiness,
    ) -> Result<Self, LogisticsError> {
        if max_mass == 0 {
            return Err(LogisticsError::UnsafeCarrier);
        }
        Ok(Self {
            id,
            state: CarrierState::Available,
            max_pod_mass_grams: max_mass,
            readiness,
            platoon_epoch: 1,
        })
    }
    pub fn dispatch(&mut self, required_wh: u64, current_epoch: u64) -> Result<(), LogisticsError> {
        let energy = self
            .readiness
            .traction_energy_wh
            .saturating_add(self.readiness.fuel_energy_wh);
        if self.state != CarrierState::Available
            || current_epoch != self.platoon_epoch
            || energy.saturating_sub(required_wh) < self.readiness.reserve_wh
            || self.readiness.braking != OperationalCheck::Passed
            || self.readiness.steering != OperationalCheck::Passed
            || self.readiness.tires != OperationalCheck::Passed
            || self.readiness.odd != OperationalCheck::Passed
            || self.readiness.recovery != OperationalCheck::Passed
        {
            return Err(LogisticsError::UnsafeCarrier);
        }
        self.state = CarrierState::InTransit;
        Ok(())
    }
    pub fn v2x_lost(&mut self) {
        self.state = CarrierState::Recovering;
    }
}

pub trait CarrierController {
    type Error;
    fn drive(&mut self, carrier: &EntityId, epoch: u64) -> Result<(), Self::Error>;
    fn safe_stop(&mut self, carrier: &EntityId) -> Result<(), Self::Error>;
    fn request_manual_recovery(&mut self, carrier: &EntityId) -> Result<(), Self::Error>;
}
#[derive(Clone, Debug, Default)]
pub struct HybridCarrierSimulator {
    moving: BTreeSet<String>,
    stopped: BTreeSet<String>,
    manual: BTreeSet<String>,
}
impl CarrierController for HybridCarrierSimulator {
    type Error = ();
    fn drive(&mut self, id: &EntityId, epoch: u64) -> Result<(), Self::Error> {
        if epoch == 0 {
            return Err(());
        }
        self.moving.insert(id.to_string());
        Ok(())
    }
    fn safe_stop(&mut self, id: &EntityId) -> Result<(), Self::Error> {
        self.moving.remove(&id.to_string());
        self.stopped.insert(id.to_string());
        Ok(())
    }
    fn request_manual_recovery(&mut self, id: &EntityId) -> Result<(), Self::Error> {
        self.manual.insert(id.to_string());
        Ok(())
    }
}
impl HybridCarrierSimulator {
    #[must_use]
    pub fn moving(&self, id: &EntityId) -> bool {
        self.moving.contains(&id.to_string())
    }
    #[must_use]
    pub fn stopped(&self, id: &EntityId) -> bool {
        self.stopped.contains(&id.to_string())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatoonPlan {
    pub cell_id: EntityId,
    pub epoch: u64,
    pub members: Vec<EntityId>,
    pub route_digest: [u8; 32],
    pub fallback: String,
}
impl PlatoonPlan {
    pub fn validate(&self) -> Result<(), LogisticsError> {
        let unique = self
            .members
            .iter()
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
        if self.epoch == 0
            || self.members.is_empty()
            || self.members.len() > 32
            || unique.len() != self.members.len()
            || self.route_digest == [0; 32]
            || self.fallback.trim().is_empty()
        {
            return Err(LogisticsError::UnsafeCarrier);
        }
        Ok(())
    }
}
pub trait PlatoonController {
    type Error;
    fn activate(&mut self, plan: &PlatoonPlan) -> Result<(), Self::Error>;
    fn v2x_lost(&mut self, plan: &PlatoonPlan) -> Result<(), Self::Error>;
}
impl PlatoonController for HybridCarrierSimulator {
    type Error = ();
    fn activate(&mut self, plan: &PlatoonPlan) -> Result<(), Self::Error> {
        plan.validate().map_err(|_| ())?;
        for member in &plan.members {
            self.drive(member, plan.epoch)?;
        }
        Ok(())
    }
    fn v2x_lost(&mut self, plan: &PlatoonPlan) -> Result<(), Self::Error> {
        for member in &plan.members {
            self.safe_stop(member)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CapacityKind {
    Route,
    Bridge,
    Ferry,
    Rail,
    Barge,
    Charge,
    Refuel,
    Stage,
    Load,
    Unload,
    Admission,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapacitySlot {
    pub id: String,
    pub kind: CapacityKind,
    pub from_epoch: u64,
    pub to_epoch: u64,
    pub capacity: u64,
}
#[derive(Clone, Debug, Default)]
pub struct CapacityLedger {
    used: BTreeMap<String, u64>,
    assets: BTreeSet<String>,
}
impl CapacityLedger {
    pub fn reserve(
        &mut self,
        slot: &CapacitySlot,
        asset: &EntityId,
        amount: u64,
    ) -> Result<(), LogisticsError> {
        let asset_key = format!("{}:{}", slot.id, asset);
        if amount == 0
            || slot.capacity == 0
            || slot.from_epoch >= slot.to_epoch
            || self.assets.contains(&asset_key)
        {
            return Err(LogisticsError::MobilizationCapacity);
        }
        let used = *self.used.get(&slot.id).unwrap_or(&0);
        if used.saturating_add(amount) > slot.capacity {
            return Err(LogisticsError::MobilizationCapacity);
        }
        self.used.insert(slot.id.clone(), used + amount);
        self.assets.insert(asset_key);
        Ok(())
    }
    #[must_use]
    pub fn used(&self, id: &str) -> u64 {
        *self.used.get(id).unwrap_or(&0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowDemand {
    pub cohort_id: EntityId,
    pub robots: u64,
    pub required_arrival_epoch: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowPath {
    pub id: String,
    pub slots: Vec<CapacitySlot>,
    pub travel_epochs: u64,
    pub useful_arrival_capacity: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowAssignment {
    pub cohort_id: EntityId,
    pub path_id: String,
    pub robots: u64,
    pub arrival_epoch: u64,
}
pub trait MobilizationPlanner {
    fn plan(
        &self,
        demands: &[FlowDemand],
        paths: &[FlowPath],
    ) -> Result<Vec<FlowAssignment>, LogisticsError>;
}
#[derive(Clone, Copy, Debug, Default)]
pub struct TimeExpandedFlowPlanner;
impl MobilizationPlanner for TimeExpandedFlowPlanner {
    fn plan(
        &self,
        demands: &[FlowDemand],
        paths: &[FlowPath],
    ) -> Result<Vec<FlowAssignment>, LogisticsError> {
        let mut path_remaining = paths
            .iter()
            .map(|p| (p.id.clone(), p.useful_arrival_capacity))
            .collect::<BTreeMap<_, _>>();
        let mut slot_remaining = BTreeMap::<String, u64>::new();
        for path in paths {
            for slot in &path.slots {
                slot_remaining
                    .entry(slot.id.clone())
                    .and_modify(|v| *v = (*v).min(slot.capacity))
                    .or_insert(slot.capacity);
            }
        }
        let mut demands = demands.to_vec();
        demands.sort_by_key(|d| (d.required_arrival_epoch, d.cohort_id.to_string()));
        let mut paths = paths.iter().collect::<Vec<_>>();
        paths.sort_by_key(|p| (p.travel_epochs, p.id.clone()));
        let mut output = Vec::new();
        for demand in demands {
            let mut needed = demand.robots;
            for path in &paths {
                let available = path
                    .slots
                    .iter()
                    .map(|slot| *slot_remaining.get(&slot.id).unwrap_or(&0))
                    .min()
                    .unwrap_or(0)
                    .min(*path_remaining.get(&path.id).unwrap_or(&0));
                let take = needed.min(available);
                if take == 0 {
                    continue;
                }
                let arrival = path
                    .slots
                    .first()
                    .map_or(0, |s| s.from_epoch)
                    .saturating_add(path.travel_epochs);
                if arrival > demand.required_arrival_epoch {
                    continue;
                }
                output.push(FlowAssignment {
                    cohort_id: demand.cohort_id.clone(),
                    path_id: path.id.clone(),
                    robots: take,
                    arrival_epoch: arrival,
                });
                path_remaining.insert(
                    path.id.clone(),
                    path_remaining.get(&path.id).copied().unwrap_or(0) - take,
                );
                for slot in &path.slots {
                    if let Some(value) = slot_remaining.get_mut(&slot.id) {
                        *value -= take;
                    }
                }
                needed -= take;
                if needed == 0 {
                    break;
                }
            }
            if needed > 0 {
                return Err(LogisticsError::MobilizationCapacity);
            }
        }
        Ok(output)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaveState {
    Draft,
    CapacityChecked,
    Authorized,
    Releasing,
    InTransit,
    Arriving,
    Complete,
    Aborted,
}
#[derive(Clone, Debug)]
pub struct MobilizationWave {
    pub id: EntityId,
    pub target_robots: u64,
    pub assignments: Vec<FlowAssignment>,
    pub state: WaveState,
    pub useful_arrivals: u64,
}
impl MobilizationWave {
    pub fn draft(id: EntityId, target: u64) -> Result<Self, LogisticsError> {
        if target == 0 {
            return Err(LogisticsError::MobilizationCapacity);
        }
        Ok(Self {
            id,
            target_robots: target,
            assignments: Vec::new(),
            state: WaveState::Draft,
            useful_arrivals: 0,
        })
    }
    pub fn check_capacity(
        &mut self,
        planner: &impl MobilizationPlanner,
        demands: &[FlowDemand],
        paths: &[FlowPath],
    ) -> Result<(), LogisticsError> {
        if self.state != WaveState::Draft {
            return Err(LogisticsError::MobilizationCapacity);
        }
        let assignments = planner.plan(demands, paths)?;
        if assignments.iter().map(|a| a.robots).sum::<u64>() < self.target_robots {
            return Err(LogisticsError::MobilizationCapacity);
        }
        self.assignments = assignments;
        self.state = WaveState::CapacityChecked;
        Ok(())
    }
    pub fn release(&mut self, downstream_capacity: u64) -> Result<u64, LogisticsError> {
        if !matches!(
            self.state,
            WaveState::CapacityChecked | WaveState::Releasing
        ) || downstream_capacity == 0
        {
            return Err(LogisticsError::MobilizationCapacity);
        }
        let released = self.target_robots.min(downstream_capacity);
        self.state = WaveState::Releasing;
        Ok(released)
    }
    pub fn record_useful_arrival(
        &mut self,
        robots: u64,
        inspected: u64,
        energized: u64,
        connected: u64,
        admitted: u64,
    ) -> Result<(), LogisticsError> {
        let useful = robots
            .min(inspected)
            .min(energized)
            .min(connected)
            .min(admitted);
        self.useful_arrivals = self
            .useful_arrivals
            .checked_add(useful)
            .ok_or(LogisticsError::MobilizationCapacity)?;
        if self.useful_arrivals > self.target_robots {
            return Err(LogisticsError::MobilizationCapacity);
        }
        if self.useful_arrivals == self.target_robots {
            self.state = WaveState::Complete;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobilizationScenario {
    pub total_robots: u64,
    pub cohort_size: u64,
    pub demands: Vec<FlowDemand>,
}
pub fn generate_mobilization_scenario(
    total: u64,
    cohort_size: u64,
) -> Result<MobilizationScenario, LogisticsError> {
    if total == 0 || cohort_size == 0 || cohort_size > 10_000 {
        return Err(LogisticsError::MobilizationCapacity);
    }
    let mut demands = Vec::new();
    let mut remaining = total;
    let mut epoch = 100;
    while remaining > 0 {
        let robots = remaining.min(cohort_size);
        demands.push(FlowDemand {
            cohort_id: EntityId::new(),
            robots,
            required_arrival_epoch: epoch,
        });
        remaining -= robots;
        epoch += 1;
    }
    Ok(MobilizationScenario {
        total_robots: total,
        cohort_size,
        demands,
    })
}
