#![forbid(unsafe_code)]
//! Generic version, cursor, and monotonic safety-state reconciliation.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

/// Per-replica causal counter used without assuming wall-clock order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReplicaVersion {
    replica: String,
    counter: u64,
}

impl ReplicaVersion {
    /// Creates a positive causal version for a bounded replica identifier.
    pub fn new(replica: impl Into<String>, counter: u64) -> Result<Self, ReconciliationError> {
        let replica = replica.into();
        if replica.trim().is_empty() || replica.len() > 128 || counter == 0 {
            return Err(ReconciliationError::InvalidVersion);
        }
        Ok(Self { replica, counter })
    }
}

/// Generic authority-impacting safety state ordered from permissive to strict.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AuthorityState {
    /// No additional substrate-level restriction.
    Permitted,
    /// Context-defined restriction severity; larger values are stricter.
    Restricted(u16),
    /// Asset or capability is grounded.
    Grounded,
    /// Authority was revoked.
    Revoked,
    /// Active operation was aborted.
    Aborted,
}

impl AuthorityState {
    fn strictness(&self) -> (u8, u16) {
        match self {
            Self::Permitted => (0, 0),
            Self::Restricted(level) => (1, *level),
            Self::Grounded => (2, 0),
            Self::Revoked => (3, 0),
            Self::Aborted => (4, 0),
        }
    }
}

/// Immutable authority fact received from a replica.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuthorityFact {
    /// Causal origin and counter.
    pub version: ReplicaVersion,
    /// Safety-biased state.
    pub state: AuthorityState,
}

impl AuthorityFact {
    /// Creates an immutable authority fact.
    #[must_use]
    pub const fn new(version: ReplicaVersion, state: AuthorityState) -> Self {
        Self { version, state }
    }

    /// Reconciles without last-write-wins or wall-clock ordering.
    #[must_use]
    pub fn reconcile(&self, remote: &Self) -> ReconcileDecision {
        let local_rank = self.state.strictness();
        let remote_rank = remote.state.strictness();
        if remote_rank > local_rank {
            return ReconcileDecision::ApplyRemote(remote.clone());
        }
        if remote_rank < local_rank {
            return ReconcileDecision::KeepLocal;
        }
        if self == remote {
            return ReconcileDecision::Duplicate;
        }
        if self.version.replica == remote.version.replica {
            return match remote.version.counter.cmp(&self.version.counter) {
                std::cmp::Ordering::Greater => ReconcileDecision::ApplyRemote(remote.clone()),
                std::cmp::Ordering::Less => ReconcileDecision::KeepLocal,
                std::cmp::Ordering::Equal => ReconcileDecision::SuspendAmbiguous {
                    local: self.clone(),
                    remote: remote.clone(),
                },
            };
        }
        ReconcileDecision::SuspendAmbiguous {
            local: self.clone(),
            remote: remote.clone(),
        }
    }
}

/// Deterministic generic reconciliation disposition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReconcileDecision {
    /// Durably apply a stricter or causally newer remote fact.
    ApplyRemote(AuthorityFact),
    /// Preserve the local stricter or causally newer fact.
    KeepLocal,
    /// Fact was already applied.
    Duplicate,
    /// Incomparable authority-bearing facts require human resolution.
    SuspendAmbiguous {
        /// Preserved local alternative.
        local: AuthorityFact,
        /// Preserved remote alternative.
        remote: AuthorityFact,
    },
}

/// Stable validation failures.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ReconciliationError {
    /// Replica ID is invalid or causal counter is zero.
    #[error("replica version is invalid")]
    InvalidVersion,
    /// A cursor attempted to move backward or skip unapplied data.
    #[error("synchronization cursor cannot regress or skip")]
    InvalidCursorAdvance,
}

/// Durable per-peer cursor advanced only after application commits.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncCursor {
    peer: String,
    applied_sequence: u64,
}

impl SyncCursor {
    /// Creates a cursor at an already durable sequence.
    pub fn new(
        peer: impl Into<String>,
        applied_sequence: u64,
    ) -> Result<Self, ReconciliationError> {
        let peer = peer.into();
        if peer.trim().is_empty() || peer.len() > 128 {
            return Err(ReconciliationError::InvalidVersion);
        }
        Ok(Self {
            peer,
            applied_sequence,
        })
    }
    /// Advances exactly one sequence after its fact and effect commit together.
    pub fn advance_after_commit(&mut self, sequence: u64) -> Result<(), ReconciliationError> {
        if sequence
            != self
                .applied_sequence
                .checked_add(1)
                .ok_or(ReconciliationError::InvalidCursorAdvance)?
        {
            return Err(ReconciliationError::InvalidCursorAdvance);
        }
        self.applied_sequence = sequence;
        Ok(())
    }
    /// Last durably applied sequence.
    #[must_use]
    pub const fn applied_sequence(&self) -> u64 {
        self.applied_sequence
    }
}

/// Causal version vector used to detect concurrent replica histories.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct VersionVector(BTreeMap<String, u64>);

impl VersionVector {
    /// Records a strictly positive monotonically increasing replica counter.
    pub fn observe(&mut self, version: &ReplicaVersion) -> Result<(), ReconciliationError> {
        let current = self.0.entry(version.replica.clone()).or_default();
        if version.counter < *current {
            return Err(ReconciliationError::InvalidVersion);
        }
        *current = version.counter;
        Ok(())
    }
    /// Computes the causal relationship without timestamps.
    #[must_use]
    pub fn relation(&self, other: &Self) -> CausalRelation {
        let mut less = false;
        let mut greater = false;
        for replica in self.0.keys().chain(other.0.keys()) {
            let local = self.0.get(replica).copied().unwrap_or(0);
            let remote = other.0.get(replica).copied().unwrap_or(0);
            less |= local < remote;
            greater |= local > remote;
        }
        match (less, greater) {
            (false, false) => CausalRelation::Equal,
            (true, false) => CausalRelation::Before,
            (false, true) => CausalRelation::After,
            (true, true) => CausalRelation::Concurrent,
        }
    }
}

/// Causal relationship between two version vectors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalRelation {
    /// Vectors contain identical counters.
    Equal,
    /// Local history causally precedes remote history.
    Before,
    /// Local history causally succeeds remote history.
    After,
    /// Histories contain incomparable concurrent changes.
    Concurrent,
}

/// Result of checking an aggregate fact against a durable contiguous prefix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AggregateDisposition {
    /// Next contiguous version may be applied transactionally.
    Apply,
    /// Exact version and digest was already applied.
    Duplicate,
    /// A version is missing; buffer or suspend rather than skipping.
    Gap {
        /// Next contiguous version required.
        expected: u64,
        /// Out-of-order version encountered.
        actual: u64,
    },
    /// Same version carries different immutable bytes.
    Contradiction {
        /// Reused version carrying a different digest.
        version: u64,
    },
}

/// Per-aggregate contiguous-version tracker with digest contradiction detection.
#[derive(Clone, Debug, Default)]
pub struct AggregateTracker {
    applied: BTreeMap<u64, [u8; 32]>,
    current: u64,
}

impl AggregateTracker {
    /// Checks a fact without mutating state.
    #[must_use]
    pub fn classify(&self, version: u64, digest: [u8; 32]) -> AggregateDisposition {
        if let Some(stored) = self.applied.get(&version) {
            return if *stored == digest {
                AggregateDisposition::Duplicate
            } else {
                AggregateDisposition::Contradiction { version }
            };
        }
        let expected = self.current.saturating_add(1);
        if version == expected {
            AggregateDisposition::Apply
        } else {
            AggregateDisposition::Gap {
                expected,
                actual: version,
            }
        }
    }
    /// Records only the next fact after its owning transaction commits.
    pub fn record_committed(
        &mut self,
        version: u64,
        digest: [u8; 32],
    ) -> Result<(), ReconciliationError> {
        if self.classify(version, digest) != AggregateDisposition::Apply {
            return Err(ReconciliationError::InvalidCursorAdvance);
        }
        self.applied.insert(version, digest);
        self.current = version;
        Ok(())
    }
}
