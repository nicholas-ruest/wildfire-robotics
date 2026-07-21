//! Local microgrid dispatch, energy estimation, and charging safety.
#![allow(missing_docs)]
use crate::StationError;
use chrono::{DateTime, Utc};
use shared_kernel::EntityId;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LoadPriority {
    OptionalCompute,
    OptionalCharge,
    MissionReadiness,
    Communications,
    Audit,
    Command,
    Identity,
    ThermalSafety,
    LifeSafety,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SiteLoad {
    pub id: EntityId,
    pub watts: u64,
    pub priority: LoadPriority,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GridMode {
    Black,
    Starting,
    Islanded,
    GridConnected,
    Constrained,
    Emergency,
    Shutdown,
}
#[derive(Clone, Debug)]
pub struct Microgrid {
    pub id: EntityId,
    pub mode: GridMode,
    pub emergency_reserve_wh: u64,
    pub available_wh: u64,
}
impl Microgrid {
    pub fn new(id: EntityId, reserve: u64, available: u64) -> Result<Self, StationError> {
        if reserve == 0 || available < reserve {
            return Err(StationError::UnsafeCapacity);
        }
        Ok(Self {
            id,
            mode: GridMode::Black,
            emergency_reserve_wh: reserve,
            available_wh: available,
        })
    }
    pub fn black_start(&mut self, protection_ready: bool) -> Result<(), StationError> {
        if self.mode != GridMode::Black || !protection_ready {
            return Err(StationError::InvalidTransition);
        }
        self.mode = GridMode::Islanded;
        Ok(())
    }
    pub fn dispatch(&mut self, loads: &[SiteLoad], available_watts: u64) -> BTreeSet<String> {
        let mut ordered = loads.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|l| (l.priority, l.id.to_string()));
        let mut used = 0_u64;
        let mut shed = BTreeSet::new();
        for load in ordered.into_iter().rev() {
            if used.saturating_add(load.watts) <= available_watts {
                used += load.watts;
            } else {
                shed.insert(load.id.to_string());
            }
        }
        if !shed.is_empty() {
            self.mode = GridMode::Constrained;
        }
        shed
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnergyEstimate {
    pub usable_wh: u64,
    pub soc_bps: u16,
    pub soh_bps: u16,
    pub power_w: u64,
    pub uncertainty_bps: u16,
    pub source: String,
    pub method: String,
    pub measured_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
}
impl EnergyEstimate {
    pub fn validate(&self, now: DateTime<Utc>) -> Result<(), StationError> {
        if self.soc_bps > 10_000
            || self.soh_bps > 10_000
            || self.uncertainty_bps > 10_000
            || self.usable_wh == 0
            || self.power_w == 0
            || self.source.trim().is_empty()
            || self.method.trim().is_empty()
            || self.measured_at > now
            || now >= self.valid_until
        {
            return Err(StationError::UnsafeCapacity);
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct EnergyStore {
    pub id: EntityId,
    pub estimate: EnergyEstimate,
    pub quarantined: bool,
    reserved_emergency_wh: u64,
    reserved_routine_wh: u64,
}
impl EnergyStore {
    pub fn record(
        id: EntityId,
        estimate: EnergyEstimate,
        now: DateTime<Utc>,
    ) -> Result<Self, StationError> {
        estimate.validate(now)?;
        Ok(Self {
            id,
            estimate,
            quarantined: false,
            reserved_emergency_wh: 0,
            reserved_routine_wh: 0,
        })
    }
    pub fn quarantine(&mut self) {
        self.quarantined = true;
    }
    pub fn reserve_emergency(&mut self, wh: u64) -> Result<(), StationError> {
        self.reserve(wh, true)
    }
    pub fn reserve_routine(&mut self, wh: u64) -> Result<(), StationError> {
        self.reserve(wh, false)
    }
    fn reserve(&mut self, wh: u64, emergency: bool) -> Result<(), StationError> {
        let conservative = self.estimate.usable_wh.saturating_mul(u64::from(
            self.estimate
                .soc_bps
                .saturating_sub(self.estimate.uncertainty_bps),
        )) / 10_000;
        let committed = self
            .reserved_emergency_wh
            .saturating_add(self.reserved_routine_wh);
        if self.quarantined || wh == 0 || committed.saturating_add(wh) > conservative {
            return Err(StationError::EmergencyReserve);
        }
        if emergency {
            self.reserved_emergency_wh += wh;
        } else {
            self.reserved_routine_wh += wh;
        }
        Ok(())
    }
    #[must_use]
    pub const fn routine_reserved_wh(&self) -> u64 {
        self.reserved_routine_wh
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChargeState {
    Requested,
    Admitted,
    Prechecking,
    Charging,
    Complete,
    Aborted,
    Quarantined,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SafetyCheck {
    Passed,
    Failed,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChargePrecheck {
    pub battery: EntityId,
    pub charger: EntityId,
    pub vehicle: EntityId,
    pub compatibility: SafetyCheck,
    pub isolation: SafetyCheck,
    pub temperature: SafetyCheck,
    pub bms_authority: SafetyCheck,
    pub zone_capacity: SafetyCheck,
    pub schedule_fence: u64,
    pub protection: SafetyCheck,
}
#[derive(Clone, Debug)]
pub struct ChargeSession {
    pub id: EntityId,
    pub state: ChargeState,
    pub fence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChargeCandidate {
    pub session_id: EntityId,
    pub readiness_class: u8,
    pub deadline_epoch_seconds: i64,
    pub requested_wh: u64,
    pub max_power_w: u64,
    pub degradation_cost_micros: u64,
    pub compatible: bool,
    pub bms_current: bool,
    pub zone: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduledCharge {
    pub session_id: EntityId,
    pub power_w: u64,
    pub energy_wh: u64,
    pub fence: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChargeSchedule {
    pub scheduled: Vec<ScheduledCharge>,
    pub deferred: Vec<EntityId>,
    pub protected_reserve_wh: u64,
    pub assumptions: Vec<String>,
}

pub trait ChargingOptimizer {
    fn schedule(
        &self,
        candidates: &[ChargeCandidate],
        site_power_w: u64,
        routine_energy_wh: u64,
        emergency_reserve_wh: u64,
        fence: u64,
    ) -> Result<ChargeSchedule, StationError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PartitionedReferenceOptimizer;
impl ChargingOptimizer for PartitionedReferenceOptimizer {
    fn schedule(
        &self,
        candidates: &[ChargeCandidate],
        site_power_w: u64,
        routine_energy_wh: u64,
        emergency_reserve_wh: u64,
        fence: u64,
    ) -> Result<ChargeSchedule, StationError> {
        if fence == 0 || site_power_w == 0 {
            return Err(StationError::UnsafeCapacity);
        }
        let mut ordered = candidates.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|c| {
            (
                u8::MAX - c.readiness_class,
                c.deadline_epoch_seconds,
                c.degradation_cost_micros,
                c.session_id.to_string(),
            )
        });
        let mut available_power = site_power_w;
        let mut available_energy = routine_energy_wh;
        let mut scheduled = Vec::new();
        let mut deferred = Vec::new();
        for candidate in ordered {
            if !candidate.compatible
                || !candidate.bms_current
                || candidate.requested_wh == 0
                || candidate.max_power_w == 0
            {
                deferred.push(candidate.session_id.clone());
                continue;
            }
            let power = candidate.max_power_w.min(available_power);
            let energy = candidate.requested_wh.min(available_energy);
            if power == 0 || energy == 0 {
                deferred.push(candidate.session_id.clone());
                continue;
            }
            scheduled.push(ScheduledCharge {
                session_id: candidate.session_id.clone(),
                power_w: power,
                energy_wh: energy,
                fence,
            });
            available_power -= power;
            available_energy -= energy;
        }
        Ok(ChargeSchedule {
            scheduled,
            deferred,
            protected_reserve_wh: emergency_reserve_wh,
            assumptions: vec![
                "BMS and protection remain authoritative".into(),
                "local conservative energy budget is binding".into(),
            ],
        })
    }
}

pub trait ChargerPort {
    type Error;
    fn energize(&mut self, session: &EntityId, power_w: u64, fence: u64)
    -> Result<(), Self::Error>;
    fn inhibit(&mut self, session: &EntityId) -> Result<(), Self::Error>;
}
pub trait BmsPort {
    fn authoritative_limit_w(&self, battery: &EntityId, now: DateTime<Utc>) -> Option<u64>;
    fn tripped(&self, battery: &EntityId) -> bool;
}

#[derive(Clone, Debug, Default)]
pub struct DeterministicChargerSimulator {
    effects: BTreeMap<String, u64>,
    inhibited: BTreeSet<String>,
}
impl ChargerPort for DeterministicChargerSimulator {
    type Error = ();
    fn energize(
        &mut self,
        session: &EntityId,
        power_w: u64,
        _fence: u64,
    ) -> Result<(), Self::Error> {
        if power_w == 0 || self.inhibited.contains(&session.to_string()) {
            return Err(());
        }
        self.effects.entry(session.to_string()).or_insert(power_w);
        Ok(())
    }
    fn inhibit(&mut self, session: &EntityId) -> Result<(), Self::Error> {
        self.effects.remove(&session.to_string());
        self.inhibited.insert(session.to_string());
        Ok(())
    }
}
impl DeterministicChargerSimulator {
    #[must_use]
    pub fn effect_count(&self) -> usize {
        self.effects.len()
    }
}

pub fn apply_charge_command<P: ChargerPort>(
    session: &mut ChargeSession,
    battery: &EntityId,
    power_w: u64,
    fence: u64,
    now: DateTime<Utc>,
    bms: &impl BmsPort,
    charger: &mut P,
) -> Result<(), StationError> {
    if session.state != ChargeState::Charging || session.fence != fence || bms.tripped(battery) {
        let _ = charger.inhibit(&session.id);
        session.protection_trip();
        return Err(StationError::UnsafeCapacity);
    }
    let limit = bms
        .authoritative_limit_w(battery, now)
        .ok_or(StationError::UnsafeCapacity)?;
    if power_w == 0 || power_w > limit {
        return Err(StationError::UnsafeCapacity);
    }
    charger
        .energize(&session.id, power_w, fence)
        .map_err(|_| StationError::UnsafeCapacity)
}
impl ChargeSession {
    pub fn request(id: EntityId, fence: u64) -> Result<Self, StationError> {
        if fence == 0 {
            return Err(StationError::InvalidTransition);
        }
        Ok(Self {
            id,
            state: ChargeState::Requested,
            fence,
        })
    }
    pub fn start(&mut self, p: &ChargePrecheck) -> Result<(), StationError> {
        if self.state != ChargeState::Requested
            || p.schedule_fence != self.fence
            || p.compatibility != SafetyCheck::Passed
            || p.isolation != SafetyCheck::Passed
            || p.temperature != SafetyCheck::Passed
            || p.bms_authority != SafetyCheck::Passed
            || p.zone_capacity != SafetyCheck::Passed
            || p.protection != SafetyCheck::Passed
        {
            return Err(StationError::UnsafeCapacity);
        }
        self.state = ChargeState::Charging;
        Ok(())
    }
    pub fn protection_trip(&mut self) {
        self.state = ChargeState::Quarantined;
    }
}
