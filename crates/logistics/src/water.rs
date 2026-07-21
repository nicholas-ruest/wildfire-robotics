//! Water-source freshness, quality, reservation, and relay cycles.
#![allow(missing_docs)]
use crate::LogisticsError;
use chrono::{DateTime, Utc};
use shared_kernel::EntityId;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaterState {
    Candidate,
    Available,
    Restricted,
    Depleted,
    Contaminated,
}
#[derive(Clone, Debug)]
pub struct WaterSource {
    id: EntityId,
    usable_litres: u64,
    reserved_litres: u64,
    quality_digest: [u8; 32],
    quality_valid_until: DateTime<Utc>,
    state: WaterState,
}
impl WaterSource {
    pub fn verify(
        id: EntityId,
        usable_litres: u64,
        quality_digest: [u8; 32],
        quality_valid_until: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<Self, LogisticsError> {
        if usable_litres == 0 || quality_digest == [0; 32] || quality_valid_until <= now {
            return Err(LogisticsError::UnsafeWaterSource);
        }
        Ok(Self {
            id,
            usable_litres,
            reserved_litres: 0,
            quality_digest,
            quality_valid_until,
            state: WaterState::Available,
        })
    }
    pub fn reserve(&mut self, litres: u64, now: DateTime<Utc>) -> Result<(), LogisticsError> {
        let available = self.usable_litres.saturating_sub(self.reserved_litres);
        if self.state != WaterState::Available
            || now >= self.quality_valid_until
            || litres == 0
            || litres > available
        {
            return Err(LogisticsError::UnsafeWaterSource);
        }
        self.reserved_litres += litres;
        Ok(())
    }
    pub fn contaminate(&mut self) {
        self.state = WaterState::Contaminated;
    }
    #[must_use]
    pub fn eligible(&self, now: DateTime<Utc>) -> bool {
        self.state == WaterState::Available && now < self.quality_valid_until
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub const fn quality_digest(&self) -> [u8; 32] {
        self.quality_digest
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelayState {
    Planned,
    Reserved,
    Active,
    Complete,
    Interrupted,
}
#[derive(Clone, Debug)]
pub struct RelayCycle {
    pub id: EntityId,
    pub source_id: EntityId,
    pub litres: u64,
    pub vehicle_slots: Vec<EntityId>,
    pub state: RelayState,
    completed: BTreeMap<String, u64>,
}
impl RelayCycle {
    pub fn plan(
        id: EntityId,
        source_id: EntityId,
        litres: u64,
        slots: Vec<EntityId>,
    ) -> Result<Self, LogisticsError> {
        if litres == 0 || slots.is_empty() {
            return Err(LogisticsError::InvalidRelay);
        }
        Ok(Self {
            id,
            source_id,
            litres,
            vehicle_slots: slots,
            state: RelayState::Planned,
            completed: BTreeMap::new(),
        })
    }
    pub fn reserve(
        &mut self,
        source: &mut WaterSource,
        now: DateTime<Utc>,
    ) -> Result<(), LogisticsError> {
        if self.state != RelayState::Planned || self.source_id != *source.id() {
            return Err(LogisticsError::InvalidRelay);
        }
        source.reserve(self.litres, now)?;
        self.state = RelayState::Reserved;
        Ok(())
    }
    pub fn record_leg(
        &mut self,
        source: &WaterSource,
        idempotency_key: impl Into<String>,
        litres: u64,
        now: DateTime<Utc>,
    ) -> Result<bool, LogisticsError> {
        let key = idempotency_key.into();
        if key.trim().is_empty()
            || litres == 0
            || self.source_id != *source.id()
            || !source.eligible(now)
        {
            return Err(LogisticsError::InvalidRelay);
        }
        if self.completed.contains_key(&key) {
            return Ok(false);
        }
        let total = self
            .completed
            .values()
            .try_fold(litres, |a, b| a.checked_add(*b))
            .ok_or(LogisticsError::InvalidRelay)?;
        if total > self.litres {
            return Err(LogisticsError::InvalidRelay);
        }
        self.completed.insert(key, litres);
        self.state = if total == self.litres {
            RelayState::Complete
        } else {
            RelayState::Active
        };
        Ok(true)
    }
    pub fn interrupt(&mut self) {
        self.state = RelayState::Interrupted;
    }
}
