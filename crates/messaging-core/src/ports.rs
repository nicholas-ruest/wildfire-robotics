//! Broker-neutral asynchronous messaging ports.

use crate::{
    envelope::MessageEnvelope,
    request::{RequestBudget, RequestOutcome},
    subject::Subject,
};
use shared_kernel::MonotonicInstant;
use std::{collections::BTreeMap, future::Future, pin::Pin};

/// Transport-independent metadata carrier.
pub type MessageHeaders = BTreeMap<String, String>;

/// Broker confirmation that a stable message was durably accepted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishConfirmation {
    /// Logical persistence container chosen by the broker adapter.
    pub stream: String,
    /// Monotonic position assigned within that container.
    pub sequence: u64,
    /// Whether the broker suppressed a duplicate stable message identifier.
    pub duplicate: bool,
}

/// Boxed future used by object-safe messaging ports without an async runtime dependency.
pub type PortFuture<'a, T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'a>>;

/// Durable publication boundary implemented by broker adapters.
pub trait DurablePublisher: Send + Sync {
    /// Adapter-specific stable error type.
    type Error;

    /// Publishes an immutable envelope and waits for durable confirmation.
    fn publish<'a>(
        &'a self,
        envelope: &'a MessageEnvelope,
    ) -> PortFuture<'a, PublishConfirmation, Self::Error>;
}

/// Deadline-bounded request/reply boundary implemented by broker adapters.
pub trait RequestReply: Send + Sync {
    /// Adapter-specific stable error type.
    type Error;

    /// Sends a request without leaking broker-native header representations.
    fn request<'a>(
        &'a self,
        subject: &'a Subject,
        headers: MessageHeaders,
        payload: Vec<u8>,
        budget: &'a RequestBudget,
        now: MonotonicInstant,
    ) -> PortFuture<'a, RequestOutcome<Vec<u8>>, Self::Error>;
}
