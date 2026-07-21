//! Hardware-neutral energy ports and deterministic site simulator.
#![allow(missing_docs)]
use crate::SafetyCheck;
use chrono::{DateTime, Utc};
use shared_kernel::EntityId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForecastValue {
    pub lower: u64,
    pub expected: u64,
    pub upper: u64,
    pub source: String,
    pub valid_until: DateTime<Utc>,
}
impl ForecastValue {
    #[must_use]
    pub fn valid_at(&self, now: DateTime<Utc>) -> bool {
        self.lower <= self.expected
            && self.expected <= self.upper
            && !self.source.trim().is_empty()
            && now < self.valid_until
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GridSnapshot {
    pub available: SafetyCheck,
    pub import_limit_w: u64,
    pub island_permission: SafetyCheck,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratorSnapshot {
    pub available: SafetyCheck,
    pub fuel_wh: u64,
    pub start_energy_wh: u64,
    pub max_power_w: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageSnapshot {
    pub usable_wh: ForecastValue,
    pub charge_limit_w: u64,
    pub discharge_limit_w: u64,
    pub protection: SafetyCheck,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZoneSnapshot {
    pub zone: String,
    pub capacity: u32,
    pub temperature_millicelsius: i32,
    pub fire_and_isolation: SafetyCheck,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DockSnapshot {
    pub dock: EntityId,
    pub occupied: bool,
    pub compatible: SafetyCheck,
    pub isolation: SafetyCheck,
}

pub trait PvWeatherPort {
    fn pv_forecast(&self, now: DateTime<Utc>) -> ForecastValue;
}
pub trait GridPort {
    fn grid(&self) -> GridSnapshot;
}
pub trait StoragePort {
    fn storage(&self) -> StorageSnapshot;
}
pub trait GeneratorFuelPort {
    fn generator(&self) -> GeneratorSnapshot;
}
pub trait ThermalFireZonePort {
    fn zone(&self, id: &str) -> Option<ZoneSnapshot>;
}
pub trait DockPort {
    fn dock(&self, id: &EntityId) -> Option<DockSnapshot>;
}

#[derive(Clone, Debug)]
pub struct SiteScenario {
    pub pv: ForecastValue,
    pub grid: GridSnapshot,
    pub storage: StorageSnapshot,
    pub generator: GeneratorSnapshot,
    pub zones: Vec<ZoneSnapshot>,
    pub docks: Vec<DockSnapshot>,
}
#[derive(Clone, Debug)]
pub struct DeterministicSiteSimulator {
    scenario: SiteScenario,
}
impl DeterministicSiteSimulator {
    #[must_use]
    pub fn new(scenario: SiteScenario) -> Self {
        Self { scenario }
    }
}
impl PvWeatherPort for DeterministicSiteSimulator {
    fn pv_forecast(&self, _: DateTime<Utc>) -> ForecastValue {
        self.scenario.pv.clone()
    }
}
impl GridPort for DeterministicSiteSimulator {
    fn grid(&self) -> GridSnapshot {
        self.scenario.grid.clone()
    }
}
impl StoragePort for DeterministicSiteSimulator {
    fn storage(&self) -> StorageSnapshot {
        self.scenario.storage.clone()
    }
}
impl GeneratorFuelPort for DeterministicSiteSimulator {
    fn generator(&self) -> GeneratorSnapshot {
        self.scenario.generator.clone()
    }
}
impl ThermalFireZonePort for DeterministicSiteSimulator {
    fn zone(&self, id: &str) -> Option<ZoneSnapshot> {
        self.scenario.zones.iter().find(|z| z.zone == id).cloned()
    }
}
impl DockPort for DeterministicSiteSimulator {
    fn dock(&self, id: &EntityId) -> Option<DockSnapshot> {
        self.scenario.docks.iter().find(|d| &d.dock == id).cloned()
    }
}
