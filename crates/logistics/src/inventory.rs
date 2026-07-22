//! Exact inventory, reservation, and source-to-use custody semantics.
#![allow(missing_docs)]

use crate::LogisticsError;
use chrono::{DateTime, Utc};
use sha2::{Digest as _, Sha256};
use shared_kernel::EntityId;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ExactQuantity {
    amount: u64,
    unit: String,
}
impl ExactQuantity {
    pub fn new(amount: u64, unit: impl Into<String>) -> Result<Self, LogisticsError> {
        let unit = unit.into();
        if amount == 0 || unit.trim().is_empty() {
            return Err(LogisticsError::InvalidQuantity);
        }
        Ok(Self { amount, unit })
    }
    #[must_use]
    pub const fn amount(&self) -> u64 {
        self.amount
    }
    #[must_use]
    pub fn unit(&self) -> &str {
        &self.unit
    }
    pub fn checked_sub(&self, other: &Self) -> Result<Self, LogisticsError> {
        if self.unit != other.unit || other.amount > self.amount {
            return Err(LogisticsError::InvalidQuantity);
        }
        Self::new(self.amount - other.amount, self.unit.clone())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceCondition {
    Serviceable,
    Quarantined,
    Contaminated,
    Unserviceable,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceState {
    Stocked,
    Reserved,
    Issued,
    Consumed,
    Returned,
    Disposed,
}

#[derive(Clone, Debug)]
pub struct ResourceItem {
    id: EntityId,
    batch: Option<String>,
    serial: Option<String>,
    quantity: ExactQuantity,
    remaining: u64,
    condition: ResourceCondition,
    state: ResourceState,
    compatibility: BTreeSet<String>,
    location: String,
    expires_at: Option<DateTime<Utc>>,
    source: String,
}
impl ResourceItem {
    #[allow(clippy::too_many_arguments)]
    pub fn stock(
        id: EntityId,
        batch: Option<String>,
        serial: Option<String>,
        quantity: ExactQuantity,
        condition: ResourceCondition,
        compatibility: impl IntoIterator<Item = String>,
        location: impl Into<String>,
        expires_at: Option<DateTime<Utc>>,
        source: impl Into<String>,
    ) -> Result<Self, LogisticsError> {
        let compatibility = compatibility.into_iter().collect::<BTreeSet<_>>();
        let location = location.into();
        let source = source.into();
        if (batch.as_deref().is_none_or(str::is_empty)
            && serial.as_deref().is_none_or(str::is_empty))
            || compatibility.is_empty()
            || location.trim().is_empty()
            || source.trim().is_empty()
        {
            return Err(LogisticsError::InvalidResource);
        }
        Ok(Self {
            id,
            batch,
            serial,
            remaining: quantity.amount,
            quantity,
            condition,
            state: ResourceState::Stocked,
            compatibility,
            location,
            expires_at,
            source,
        })
    }
    #[must_use]
    pub fn available_to_promise(
        &self,
        capability: &str,
        location: &str,
        now: DateTime<Utc>,
    ) -> u64 {
        let current = self.expires_at.is_none_or(|expiry| now < expiry);
        if self.state == ResourceState::Stocked
            && self.condition == ResourceCondition::Serviceable
            && current
            && self.compatibility.contains(capability)
            && self.location == location
        {
            self.remaining
        } else {
            0
        }
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub fn batch(&self) -> Option<&str> {
        self.batch.as_deref()
    }
    #[must_use]
    pub fn serial(&self) -> Option<&str> {
        self.serial.as_deref()
    }
    #[must_use]
    pub fn unit(&self) -> &str {
        self.quantity.unit()
    }
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }
    pub fn set_condition(&mut self, condition: ResourceCondition) {
        self.condition = condition;
    }
    fn reserve(&mut self, amount: u64) -> Result<(), LogisticsError> {
        if amount == 0 || amount > self.remaining {
            return Err(LogisticsError::OverReserved);
        }
        self.remaining -= amount;
        self.state = ResourceState::Reserved;
        Ok(())
    }
    fn release(&mut self, amount: u64) -> Result<(), LogisticsError> {
        self.remaining = self
            .remaining
            .checked_add(amount)
            .ok_or(LogisticsError::InvalidQuantity)?;
        if self.remaining > self.quantity.amount {
            return Err(LogisticsError::InvalidQuantity);
        }
        self.state = ResourceState::Stocked;
        Ok(())
    }
    fn issue(&mut self) {
        self.state = ResourceState::Issued;
    }
    fn issuable(&self, now: DateTime<Utc>) -> bool {
        self.state == ResourceState::Reserved
            && self.condition == ResourceCondition::Serviceable
            && self.expires_at.is_none_or(|expiry| now < expiry)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryReservation {
    pub id: EntityId,
    pub item_id: EntityId,
    pub amount: u64,
    pub unit: String,
    pub purpose: String,
    pub expires_at: DateTime<Utc>,
    consumed: bool,
    disposition: Option<ResourceState>,
}

#[derive(Clone, Debug, Default)]
pub struct InventoryLedger {
    reservations: BTreeMap<String, InventoryReservation>,
}
impl InventoryLedger {
    #[allow(clippy::too_many_arguments)]
    pub fn reserve(
        &mut self,
        item: &mut ResourceItem,
        id: EntityId,
        amount: u64,
        unit: &str,
        capability: &str,
        location: &str,
        purpose: impl Into<String>,
        now: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<InventoryReservation, LogisticsError> {
        let purpose = purpose.into();
        if expires_at <= now
            || unit != item.unit()
            || purpose.trim().is_empty()
            || item.available_to_promise(capability, location, now) < amount
            || self.reservations.contains_key(&id.to_string())
        {
            return Err(LogisticsError::Unavailable);
        }
        item.reserve(amount)?;
        let reservation = InventoryReservation {
            id,
            item_id: item.id.clone(),
            amount,
            unit: unit.into(),
            purpose,
            expires_at,
            consumed: false,
            disposition: None,
        };
        self.reservations
            .insert(reservation.id.to_string(), reservation.clone());
        Ok(reservation)
    }
    pub fn issue(
        &mut self,
        item: &mut ResourceItem,
        reservation_id: &EntityId,
        now: DateTime<Utc>,
    ) -> Result<(), LogisticsError> {
        let reservation = self
            .reservations
            .get_mut(&reservation_id.to_string())
            .ok_or(LogisticsError::StaleReservation)?;
        if reservation.consumed
            || reservation.item_id != item.id
            || now >= reservation.expires_at
            || !item.issuable(now)
        {
            return Err(LogisticsError::StaleReservation);
        }
        reservation.consumed = true;
        item.issue();
        Ok(())
    }
    pub fn expire(
        &mut self,
        item: &mut ResourceItem,
        reservation_id: &EntityId,
        now: DateTime<Utc>,
    ) -> Result<bool, LogisticsError> {
        let key = reservation_id.to_string();
        let reservation = self
            .reservations
            .get(&key)
            .ok_or(LogisticsError::StaleReservation)?;
        if reservation.consumed || now < reservation.expires_at {
            return Ok(false);
        }
        let amount = reservation.amount;
        self.reservations.remove(&key);
        item.release(amount)?;
        Ok(true)
    }

    pub fn consume(
        &mut self,
        item: &mut ResourceItem,
        reservation_id: &EntityId,
    ) -> Result<bool, LogisticsError> {
        self.finish(item, reservation_id, ResourceState::Consumed)
    }

    pub fn dispose(
        &mut self,
        item: &mut ResourceItem,
        reservation_id: &EntityId,
    ) -> Result<bool, LogisticsError> {
        self.finish(item, reservation_id, ResourceState::Disposed)
    }

    pub fn return_item(
        &mut self,
        item: &mut ResourceItem,
        reservation_id: &EntityId,
    ) -> Result<bool, LogisticsError> {
        let reservation = self
            .reservations
            .get_mut(&reservation_id.to_string())
            .ok_or(LogisticsError::StaleReservation)?;
        if !reservation.consumed {
            return Err(LogisticsError::StaleReservation);
        }
        if reservation.disposition == Some(ResourceState::Returned) {
            return Ok(false);
        }
        if reservation.disposition.is_some() {
            return Err(LogisticsError::StaleReservation);
        }
        reservation.disposition = Some(ResourceState::Returned);
        item.release(reservation.amount)?;
        item.state = ResourceState::Returned;
        Ok(true)
    }

    fn finish(
        &mut self,
        item: &mut ResourceItem,
        reservation_id: &EntityId,
        outcome: ResourceState,
    ) -> Result<bool, LogisticsError> {
        let reservation = self
            .reservations
            .get_mut(&reservation_id.to_string())
            .ok_or(LogisticsError::StaleReservation)?;
        if !reservation.consumed {
            return Err(LogisticsError::StaleReservation);
        }
        if reservation.disposition == Some(outcome) {
            return Ok(false);
        }
        if reservation.disposition.is_some() {
            return Err(LogisticsError::StaleReservation);
        }
        reservation.disposition = Some(outcome);
        item.state = outcome;
        Ok(true)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustodyTransfer {
    pub id: EntityId,
    pub item_id: EntityId,
    pub giver: String,
    pub receiver: String,
    pub amount: u64,
    pub unit: String,
    pub condition: String,
    pub place: String,
    pub occurred_at: DateTime<Utc>,
    pub evidence_digest: [u8; 32],
    pub discrepancy: Option<String>,
    pub previous_digest: [u8; 32],
    pub digest: [u8; 32],
}
impl CustodyTransfer {
    #[must_use]
    pub fn compute_digest(&self) -> [u8; 32] {
        let mut h = Sha256::new();
        for value in [
            self.id.to_string(),
            self.item_id.to_string(),
            self.giver.clone(),
            self.receiver.clone(),
            self.amount.to_string(),
            self.unit.clone(),
            self.condition.clone(),
            self.place.clone(),
            self.occurred_at.timestamp_millis().to_string(),
            self.discrepancy.clone().unwrap_or_default(),
        ] {
            h.update((value.len() as u64).to_be_bytes());
            h.update(value.as_bytes());
        }
        h.update(self.evidence_digest);
        h.update(self.previous_digest);
        h.finalize().into()
    }
}

#[derive(Clone, Debug, Default)]
pub struct CustodyChain {
    transfers: Vec<CustodyTransfer>,
    seen: BTreeMap<String, [u8; 32]>,
}
impl CustodyChain {
    pub fn append(&mut self, transfer: CustodyTransfer) -> Result<bool, LogisticsError> {
        if transfer.digest != transfer.compute_digest() {
            return Err(LogisticsError::InvalidCustody);
        }
        if let Some(digest) = self.seen.get(&transfer.id.to_string()) {
            return if digest == &transfer.digest {
                Ok(false)
            } else {
                Err(LogisticsError::InvalidCustody)
            };
        }
        let expected = self.transfers.last().map_or([0; 32], |v| v.digest);
        let prior_receiver = self
            .transfers
            .iter()
            .rev()
            .find(|prior| prior.item_id == transfer.item_id)
            .map(|prior| prior.receiver.as_str());
        if transfer.item_id.to_string().is_empty()
            || transfer.giver.trim().is_empty()
            || transfer.receiver.trim().is_empty()
            || transfer.giver == transfer.receiver
            || transfer.amount == 0
            || transfer.unit.trim().is_empty()
            || transfer.condition.trim().is_empty()
            || transfer.place.trim().is_empty()
            || transfer.evidence_digest == [0; 32]
            || transfer.digest != transfer.compute_digest()
            || transfer.previous_digest != expected
            || prior_receiver.is_some_and(|receiver| receiver != transfer.giver)
        {
            return Err(LogisticsError::InvalidCustody);
        }
        self.seen.insert(transfer.id.to_string(), transfer.digest);
        self.transfers.push(transfer);
        Ok(true)
    }
    #[must_use]
    pub fn lineage(&self, item: &EntityId) -> Vec<&CustodyTransfer> {
        self.transfers
            .iter()
            .filter(|t| &t.item_id == item)
            .collect()
    }
}
