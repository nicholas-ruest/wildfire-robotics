//! Bounded hierarchical cell membership, fencing, split/merge, and deterministic scale data.
#![allow(missing_docs)]
use crate::{Digest, FleetError};
use shared_kernel::EntityId;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CellState {
    Forming,
    Active,
    Splitting,
    Merging,
    Degraded,
    Closed,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellScope {
    pub tenant: String,
    pub region: String,
    pub purpose: String,
    pub capability_bucket: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapacitySummary {
    pub eligible: u32,
    pub grounded: u32,
    pub energy_wh_lower_bound: u64,
    pub membership_digest: Digest,
    pub source_epoch: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Membership {
    pub asset_id: EntityId,
    pub cell_id: EntityId,
    pub purpose: String,
    pub epoch: u64,
    pub version: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellFence {
    pub cell_id: EntityId,
    pub purpose: String,
    pub epoch: u64,
}

#[derive(Clone, Debug)]
pub struct FleetCell {
    id: EntityId,
    parent: Option<EntityId>,
    scope: CellScope,
    epoch: u64,
    placement_generation: u64,
    capacity: u32,
    state: CellState,
    members: BTreeMap<String, Membership>,
    summary: CapacitySummary,
    version: u64,
}
impl FleetCell {
    pub fn form(
        id: EntityId,
        parent: Option<EntityId>,
        scope: CellScope,
        epoch: u64,
        placement_generation: u64,
        capacity: u32,
    ) -> Result<Self, FleetError> {
        if [
            scope.tenant.as_str(),
            scope.region.as_str(),
            scope.purpose.as_str(),
            scope.capability_bucket.as_str(),
        ]
        .contains(&"")
            || epoch == 0
            || placement_generation == 0
            || capacity == 0
        {
            return Err(FleetError::InvalidField);
        }
        Ok(Self {
            id,
            parent,
            scope,
            epoch,
            placement_generation,
            capacity,
            state: CellState::Forming,
            members: BTreeMap::new(),
            summary: CapacitySummary {
                eligible: 0,
                grounded: 0,
                energy_wh_lower_bound: 0,
                membership_digest: [0; 32],
                source_epoch: epoch,
            },
            version: 1,
        })
    }
    pub fn activate(&mut self) -> Result<(), FleetError> {
        if self.state != CellState::Forming {
            return Err(FleetError::InvalidTransition);
        }
        self.state = CellState::Active;
        self.bump()
    }
    pub fn assign_member(
        &mut self,
        asset_id: EntityId,
        purpose: &str,
        expected_epoch: u64,
    ) -> Result<Membership, FleetError> {
        self.validate_fence(purpose, expected_epoch)?;
        if self.state != CellState::Active
            || self.members.len()
                >= usize::try_from(self.capacity).map_err(|_| FleetError::CellCapacity)?
        {
            return Err(FleetError::CellCapacity);
        }
        let key = asset_id.to_string();
        if self.members.contains_key(&key) {
            return Err(FleetError::AmbiguousMembership);
        }
        let membership = Membership {
            asset_id,
            cell_id: self.id.clone(),
            purpose: purpose.into(),
            epoch: self.epoch,
            version: 1,
        };
        self.members.insert(key, membership.clone());
        self.bump()?;
        Ok(membership)
    }
    pub fn validate_fence(&self, purpose: &str, epoch: u64) -> Result<(), FleetError> {
        if self.state != CellState::Active || purpose != self.scope.purpose || epoch != self.epoch {
            Err(FleetError::StaleEpoch)
        } else {
            Ok(())
        }
    }
    pub fn begin_split(&mut self, expected_epoch: u64) -> Result<(), FleetError> {
        self.validate_fence(&self.scope.purpose, expected_epoch)?;
        self.state = CellState::Splitting;
        self.epoch = self
            .epoch
            .checked_add(1)
            .ok_or(FleetError::VersionExhausted)?;
        self.bump()
    }
    pub fn split(self, left_id: EntityId, right_id: EntityId) -> Result<(Self, Self), FleetError> {
        if self.state != CellState::Splitting || self.capacity < 2 {
            return Err(FleetError::InvalidTransition);
        }
        let child_epoch = self
            .epoch
            .checked_add(1)
            .ok_or(FleetError::VersionExhausted)?;
        let generation = self
            .placement_generation
            .checked_add(1)
            .ok_or(FleetError::VersionExhausted)?;
        let left_capacity = self.capacity.div_ceil(2);
        let mut left = Self::form(
            left_id,
            Some(self.id.clone()),
            self.scope.clone(),
            child_epoch,
            generation,
            left_capacity,
        )?;
        let mut right = Self::form(
            right_id,
            Some(self.id),
            self.scope.clone(),
            child_epoch,
            generation,
            self.capacity - left_capacity,
        )?;
        for (_, member) in self.members {
            let prefer_left = stable_bucket(&member.asset_id).is_multiple_of(2);
            let left_has_room = left.members.len()
                < usize::try_from(left.capacity).map_err(|_| FleetError::CellCapacity)?;
            let right_has_room = right.members.len()
                < usize::try_from(right.capacity).map_err(|_| FleetError::CellCapacity)?;
            if (prefer_left && left_has_room) || !right_has_room {
                left.insert_repartitioned(member)?;
            } else {
                right.insert_repartitioned(member)?;
            }
        }
        left.activate()?;
        right.activate()?;
        Ok((left, right))
    }
    fn insert_repartitioned(&mut self, mut member: Membership) -> Result<(), FleetError> {
        member.cell_id = self.id.clone();
        member.epoch = self.epoch;
        member.version = member
            .version
            .checked_add(1)
            .ok_or(FleetError::VersionExhausted)?;
        self.members.insert(member.asset_id.to_string(), member);
        Ok(())
    }
    pub fn merge(left: Self, right: Self, merged_id: EntityId) -> Result<Self, FleetError> {
        if left.state != CellState::Active
            || right.state != CellState::Active
            || left.scope != right.scope
            || left.epoch != right.epoch
        {
            return Err(FleetError::AmbiguousMembership);
        }
        let capacity = left
            .capacity
            .checked_add(right.capacity)
            .ok_or(FleetError::CellCapacity)?;
        let epoch = left
            .epoch
            .checked_add(1)
            .ok_or(FleetError::VersionExhausted)?;
        let generation = left
            .placement_generation
            .max(right.placement_generation)
            .checked_add(1)
            .ok_or(FleetError::VersionExhausted)?;
        let mut merged = Self::form(
            merged_id,
            left.parent.clone(),
            left.scope,
            epoch,
            generation,
            capacity,
        )?;
        for member in left
            .members
            .into_values()
            .chain(right.members.into_values())
        {
            if merged.members.contains_key(&member.asset_id.to_string()) {
                return Err(FleetError::AmbiguousMembership);
            }
            merged.insert_repartitioned(member)?;
        }
        merged.activate()?;
        Ok(merged)
    }
    pub fn update_summary(&mut self, summary: CapacitySummary) -> Result<(), FleetError> {
        if summary.source_epoch != self.epoch
            || summary.eligible.saturating_add(summary.grounded) > self.capacity
            || summary.membership_digest == [0; 32]
        {
            return Err(FleetError::StaleEpoch);
        }
        self.summary = summary;
        self.bump()
    }
    fn bump(&mut self) -> Result<(), FleetError> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or(FleetError::VersionExhausted)?;
        Ok(())
    }
    #[must_use]
    pub fn fence(&self) -> CellFence {
        CellFence {
            cell_id: self.id.clone(),
            purpose: self.scope.purpose.clone(),
            epoch: self.epoch,
        }
    }
    #[must_use]
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    #[must_use]
    pub fn member_count(&self) -> usize {
        self.members.len()
    }
    pub fn memberships(&self) -> impl Iterator<Item = &Membership> {
        self.members.values()
    }
}
fn stable_bucket(id: &EntityId) -> u64 {
    id.to_string()
        .bytes()
        .fold(14_695_981_039_346_656_037u64, |hash, byte| {
            hash.wrapping_mul(1_099_511_628_211)
                .wrapping_add(u64::from(byte))
        })
}

/// Allocation guard validates exact membership and fence without global coordination.
pub fn authorize_local_operation(
    membership: &Membership,
    fence: &CellFence,
) -> Result<(), FleetError> {
    if membership.cell_id == fence.cell_id
        && membership.purpose == fence.purpose
        && membership.epoch == fence.epoch
    {
        Ok(())
    } else {
        Err(FleetError::StaleEpoch)
    }
}

/// Streaming deterministic million-identity generator; it retains no generated fleet.
#[derive(Clone, Debug)]
pub struct SyntheticIdentityGenerator {
    seed: u64,
    next: u64,
    total: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyntheticIdentity {
    pub ordinal: u64,
    pub stable_id: [u8; 16],
    pub region: u16,
    pub cell: u32,
    pub active: bool,
}
impl SyntheticIdentityGenerator {
    #[must_use]
    pub const fn new(seed: u64, total: u64) -> Self {
        Self {
            seed,
            next: 0,
            total,
        }
    }
}
impl Iterator for SyntheticIdentityGenerator {
    type Item = SyntheticIdentity;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.total {
            return None;
        }
        let ordinal = self.next;
        self.next += 1;
        let a = mix(self.seed ^ ordinal);
        let b = mix(a ^ 0x9e37_79b9_7f4a_7c15);
        let mut stable_id = [0; 16];
        stable_id[..8].copy_from_slice(&a.to_be_bytes());
        stable_id[8..].copy_from_slice(&b.to_be_bytes());
        Some(SyntheticIdentity {
            ordinal,
            stable_id,
            region: u16::try_from(a % 256).ok()?,
            cell: u32::try_from(b % 65_536).ok()?,
            active: a.is_multiple_of(10),
        })
    }
}
const fn mix(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

/// Bounded local index. No method permits an unbounded fleet-wide scan.
#[derive(Clone, Debug, Default)]
pub struct PlacementIndex {
    by_asset: BTreeMap<[u8; 16], (u32, u64)>,
}
impl PlacementIndex {
    pub fn put(&mut self, id: [u8; 16], cell: u32, epoch: u64) -> Result<(), FleetError> {
        if epoch == 0 {
            return Err(FleetError::StaleEpoch);
        }
        if self.by_asset.insert(id, (cell, epoch)).is_some() {
            return Err(FleetError::AmbiguousMembership);
        }
        Ok(())
    }
    #[must_use]
    pub fn lookup(&self, id: &[u8; 16]) -> Option<(u32, u64)> {
        self.by_asset.get(id).copied()
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_asset.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_asset.is_empty()
    }
}
