//! Signed runtime supervisor, portable adapter ports, local caches, and recovery.
#![allow(missing_docs)]
use crate::{DeploymentManifest, Digest, EdgeDeployment, ResourceBudget, StationError};
use chrono::{DateTime, Utc};
use sha2::{Digest as _, Sha256};
use std::collections::{BTreeMap, VecDeque};

pub trait ManifestVerifier {
    type Error;
    fn verify(&self, manifest: &DeploymentManifest) -> Result<bool, Self::Error>;
}
pub trait DeploymentRuntime {
    type Error;
    fn stage(&mut self, manifest: &DeploymentManifest) -> Result<Digest, Self::Error>;
    fn activate(&mut self, deployment: Digest) -> Result<(), Self::Error>;
    fn rollback(&mut self, release: Digest, checkpoint: Digest) -> Result<(), Self::Error>;
}

pub struct EdgeRuntimeSupervisor<V, R> {
    verifier: V,
    runtime: R,
    runtime_version: u32,
    capacity: ResourceBudget,
    maximum_clock_uncertainty_ms: u64,
}
impl<V: ManifestVerifier, R: DeploymentRuntime> EdgeRuntimeSupervisor<V, R> {
    #[must_use]
    pub const fn new(
        verifier: V,
        runtime: R,
        runtime_version: u32,
        capacity: ResourceBudget,
        maximum_clock_uncertainty_ms: u64,
    ) -> Self {
        Self {
            verifier,
            runtime,
            runtime_version,
            capacity,
            maximum_clock_uncertainty_ms,
        }
    }
    pub fn verify_and_activate(
        &mut self,
        deployment: &mut EdgeDeployment,
        now: DateTime<Utc>,
        clock_uncertainty_ms: u64,
    ) -> Result<(), StationError> {
        deployment.manifest().validate_shape(now)?;
        if clock_uncertainty_ms > self.maximum_clock_uncertainty_ms {
            return Err(StationError::ClockUncertain);
        }
        if deployment.manifest().minimum_runtime_version > self.runtime_version
            || !self.capacity.contains(&deployment.manifest().required)
        {
            return Err(StationError::IncompatibleDeployment);
        }
        if !matches!(self.verifier.verify(deployment.manifest()), Ok(true)) {
            return Err(StationError::InvalidSignature);
        }
        let checkpoint = self
            .runtime
            .stage(deployment.manifest())
            .map_err(|_| StationError::IncompatibleDeployment)?;
        deployment.mark_verified(now, checkpoint)?;
        self.runtime
            .activate(deployment.manifest().release_digest)
            .map_err(|_| StationError::IncompatibleDeployment)?;
        deployment.activate()
    }
    pub fn rollback(&mut self, deployment: &mut EdgeDeployment) -> Result<(), StationError> {
        let release = deployment.begin_rollback()?;
        let checkpoint = deployment
            .recovery_checkpoint()
            .ok_or(StationError::RollbackUnavailable)?;
        self.runtime
            .rollback(release, checkpoint)
            .map_err(|_| StationError::RollbackUnavailable)?;
        deployment.complete_rollback()
    }
}

#[derive(Clone, Debug, Default)]
pub struct LightweightRuntimeAdapter {
    staged: BTreeMap<Digest, Digest>,
    active: Option<Digest>,
    fail_activation: bool,
}
impl LightweightRuntimeAdapter {
    pub fn inject_activation_failure(&mut self) {
        self.fail_activation = true;
    }
    #[must_use]
    pub const fn active(&self) -> Option<Digest> {
        self.active
    }
}
impl DeploymentRuntime for LightweightRuntimeAdapter {
    type Error = ();
    fn stage(&mut self, m: &DeploymentManifest) -> Result<Digest, Self::Error> {
        let mut h = Sha256::new();
        h.update(m.release_digest);
        h.update(m.configuration_digest);
        h.update(m.policy_digest);
        let checkpoint: Digest = h.finalize().into();
        self.staged.insert(m.release_digest, checkpoint);
        Ok(checkpoint)
    }
    fn activate(&mut self, deployment: Digest) -> Result<(), Self::Error> {
        if self.fail_activation {
            self.fail_activation = false;
            return Err(());
        }
        if !self.staged.contains_key(&deployment) {
            return Err(());
        }
        self.active = Some(deployment);
        Ok(())
    }
    fn rollback(&mut self, release: Digest, _checkpoint: Digest) -> Result<(), Self::Error> {
        self.active = Some(release);
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CacheKind {
    Policy,
    Identity,
    Map,
    Mission,
    Audit,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheEntry {
    pub key: String,
    pub digest: Digest,
    pub version: u64,
    pub expires_at: Option<DateTime<Utc>>,
    pub tombstone: bool,
    pub authority_rank: u8,
}
#[derive(Clone, Debug, Default)]
pub struct OfflineCache {
    entries: BTreeMap<(CacheKind, String), CacheEntry>,
    quarantine: Vec<CacheEntry>,
}
impl OfflineCache {
    pub fn reconcile(
        &mut self,
        kind: CacheKind,
        incoming: CacheEntry,
        now: DateTime<Utc>,
    ) -> Result<bool, StationError> {
        if incoming.digest == [0; 32] || incoming.version == 0 {
            return Err(StationError::ReconciliationConflict);
        }
        let key = (kind, incoming.key.clone());
        if let Some(current) = self.entries.get(&key) {
            if incoming.version < current.version {
                return Ok(false);
            }
            if incoming.version == current.version && incoming.digest != current.digest {
                self.quarantine.push(incoming);
                return Err(StationError::ReconciliationConflict);
            }
            let authority = matches!(
                kind,
                CacheKind::Policy | CacheKind::Identity | CacheKind::Mission
            );
            if authority
                && (incoming.authority_rank < current.authority_rank
                    || current.tombstone && !incoming.tombstone
                    || current.expires_at.is_some_and(|expiry| now >= expiry)
                        && incoming.expires_at.is_some_and(|expiry| expiry > now))
            {
                self.quarantine.push(incoming);
                return Err(StationError::ReconciliationConflict);
            }
        }
        self.entries.insert(key, incoming);
        Ok(true)
    }
    #[must_use]
    pub fn usable(&self, kind: CacheKind, key: &str, now: DateTime<Utc>) -> bool {
        self.entries.get(&(kind, key.into())).is_some_and(|entry| {
            !entry.tombstone && entry.expires_at.is_none_or(|expiry| now < expiry)
        })
    }
    #[must_use]
    pub fn quarantine_len(&self) -> usize {
        self.quarantine.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalRecord {
    pub sequence: u64,
    pub kind: CacheKind,
    pub payload_digest: Digest,
    pub previous_digest: Digest,
    pub record_digest: Digest,
}
#[derive(Clone, Debug)]
pub struct DurableLocalBuffer {
    records: VecDeque<LocalRecord>,
    capacity: usize,
    reserved_audit: usize,
    next_sequence: u64,
}
impl DurableLocalBuffer {
    pub fn new(capacity: usize, reserved_audit: usize) -> Result<Self, StationError> {
        if capacity == 0 || reserved_audit == 0 || reserved_audit > capacity {
            return Err(StationError::CorruptLog);
        }
        Ok(Self {
            records: VecDeque::new(),
            capacity,
            reserved_audit,
            next_sequence: 1,
        })
    }
    pub fn restore(
        records: VecDeque<LocalRecord>,
        capacity: usize,
        reserved_audit: usize,
    ) -> Result<Self, StationError> {
        let next_sequence = records
            .back()
            .map_or(1, |record| record.sequence.saturating_add(1));
        let value = Self {
            records,
            capacity,
            reserved_audit,
            next_sequence,
        };
        if value.records.len() > capacity {
            return Err(StationError::CorruptLog);
        }
        value.verify()?;
        Ok(value)
    }
    #[must_use]
    pub fn snapshot(&self) -> VecDeque<LocalRecord> {
        self.records.clone()
    }
    pub fn append(&mut self, kind: CacheKind, payload_digest: Digest) -> Result<u64, StationError> {
        if payload_digest == [0; 32] {
            return Err(StationError::CorruptLog);
        }
        let non_audit = self
            .records
            .iter()
            .filter(|r| r.kind != CacheKind::Audit)
            .count();
        if self.records.len() >= self.capacity
            || (kind != CacheKind::Audit && non_audit >= self.capacity - self.reserved_audit)
        {
            return Err(StationError::CorruptLog);
        }
        let previous_digest = self.records.back().map_or([0; 32], |r| r.record_digest);
        let sequence = self.next_sequence;
        let mut h = Sha256::new();
        h.update(sequence.to_be_bytes());
        h.update([kind as u8]);
        h.update(payload_digest);
        h.update(previous_digest);
        let record_digest = h.finalize().into();
        self.records.push_back(LocalRecord {
            sequence,
            kind,
            payload_digest,
            previous_digest,
            record_digest,
        });
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(StationError::VersionExhausted)?;
        Ok(sequence)
    }
    pub fn verify(&self) -> Result<(), StationError> {
        let mut previous = [0; 32];
        for (index, record) in self.records.iter().enumerate() {
            let expected_sequence = u64::try_from(index)
                .map_err(|_| StationError::CorruptLog)?
                .checked_add(1)
                .ok_or(StationError::CorruptLog)?;
            let mut h = Sha256::new();
            h.update(record.sequence.to_be_bytes());
            h.update([record.kind as u8]);
            h.update(record.payload_digest);
            h.update(record.previous_digest);
            let digest: Digest = h.finalize().into();
            if record.sequence != expected_sequence
                || record.previous_digest != previous
                || record.record_digest != digest
            {
                return Err(StationError::CorruptLog);
            }
            previous = record.record_digest;
        }
        Ok(())
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ReconciliationCursors {
    values: BTreeMap<String, u64>,
}
impl ReconciliationCursors {
    pub fn advance(
        &mut self,
        stream: impl Into<String>,
        expected: u64,
        next: u64,
    ) -> Result<(), StationError> {
        let stream = stream.into();
        let current = self.values.get(&stream).copied().unwrap_or(0);
        if current != expected
            || next
                != expected
                    .checked_add(1)
                    .ok_or(StationError::VersionExhausted)?
        {
            return Err(StationError::ReconciliationConflict);
        }
        self.values.insert(stream, next);
        Ok(())
    }
    #[must_use]
    pub fn get(&self, stream: &str) -> u64 {
        self.values.get(stream).copied().unwrap_or(0)
    }
}
