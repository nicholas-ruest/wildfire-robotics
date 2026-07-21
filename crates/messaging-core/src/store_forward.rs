//! Bounded priority store-forward queue with reserved safety capacity.

use std::collections::VecDeque;
use thiserror::Error;

/// ADR-026 telemetry/delivery priority.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TelemetryTier {
    /// High-rate replaceable data shed first.
    Bulk,
    /// Troubleshooting data shed before operational state.
    Diagnostic,
    /// Operational state retained unless safety capacity requires protection.
    Operational,
    /// Safety fact or command acknowledgement never silently dropped.
    SafetyCritical,
}

impl TelemetryTier {
    const fn index(self) -> usize {
        self as usize
    }
}

/// Queued store-forward item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueuedItem {
    /// Priority governing drain and shedding.
    pub tier: TelemetryTier,
    /// Exact opaque encoded message bytes.
    pub payload: Vec<u8>,
}

/// Enqueue result with explicit accounted shedding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnqueueOutcome {
    /// Item entered the bounded queue.
    Accepted,
    /// Permitted low-priority shedding occurred and was counted.
    DroppedLowerTier,
}

/// In-memory policy model; durable station adapters persist equivalent queues.
#[derive(Debug)]
pub struct StoreForwardQueue {
    queues: [VecDeque<Vec<u8>>; 4],
    max_items: usize,
    max_bytes: usize,
    reserved_safety_items: usize,
    bytes: usize,
    dropped: [u64; 4],
}

impl StoreForwardQueue {
    /// Creates item/byte bounds and a safety reservation.
    pub fn new(
        max_items: usize,
        max_bytes: usize,
        reserved_safety_items: usize,
    ) -> Result<Self, QueueError> {
        if max_items == 0
            || max_bytes == 0
            || reserved_safety_items == 0
            || reserved_safety_items > max_items
        {
            return Err(QueueError::InvalidCapacity);
        }
        Ok(Self {
            queues: std::array::from_fn(|_| VecDeque::new()),
            max_items,
            max_bytes,
            reserved_safety_items,
            bytes: 0,
            dropped: [0; 4],
        })
    }
    /// Enqueues without allowing lower tiers to consume reserved safety slots.
    pub fn enqueue(
        &mut self,
        tier: TelemetryTier,
        payload: Vec<u8>,
    ) -> Result<EnqueueOutcome, QueueError> {
        if payload.is_empty() || payload.len() > self.max_bytes {
            return Err(QueueError::InvalidPayload);
        }
        let items = self.queues.iter().map(VecDeque::len).sum::<usize>();
        let item_limit = if tier == TelemetryTier::SafetyCritical {
            self.max_items
        } else {
            self.max_items - self.reserved_safety_items
        };
        if items >= item_limit || self.bytes.saturating_add(payload.len()) > self.max_bytes {
            if tier == TelemetryTier::SafetyCritical {
                return Err(QueueError::SafetyCapacityExhausted);
            }
            self.dropped[tier.index()] = self.dropped[tier.index()].saturating_add(1);
            return Ok(EnqueueOutcome::DroppedLowerTier);
        }
        self.bytes += payload.len();
        self.queues[tier.index()].push_back(payload);
        Ok(EnqueueOutcome::Accepted)
    }
    /// Drains strict priority: safety, operational, diagnostic, then bulk.
    pub fn pop(&mut self) -> Option<QueuedItem> {
        for tier in [
            TelemetryTier::SafetyCritical,
            TelemetryTier::Operational,
            TelemetryTier::Diagnostic,
            TelemetryTier::Bulk,
        ] {
            if let Some(payload) = self.queues[tier.index()].pop_front() {
                self.bytes = self.bytes.saturating_sub(payload.len());
                return Some(QueuedItem { tier, payload });
            }
        }
        None
    }
    /// Saturating drop count for a tier.
    #[must_use]
    pub const fn dropped(&self, tier: TelemetryTier) -> u64 {
        self.dropped[tier.index()]
    }
}

/// Queue validation and explicit safety-degradation errors.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum QueueError {
    /// Item/byte/reservation bounds are zero or inconsistent.
    #[error("store-forward capacity is invalid")]
    InvalidCapacity,
    /// Payload is empty or larger than the entire queue.
    #[error("store-forward payload is invalid")]
    InvalidPayload,
    /// Critical reservation cannot accept another fact; callers must degrade safely.
    #[error("reserved safety capacity is exhausted; new authority must fail safe")]
    SafetyCapacityExhausted,
}
