#![forbid(unsafe_code)]
//! NATS `JetStream` adapter isolated behind broker-neutral messaging contracts.
//!
//! Authentication, TLS roots, credential rotation, and reconnect callbacks are
//! configured while constructing [`async_nats::Client`]; this adapter accepts an
//! already authenticated client and never handles or logs secret material.

use async_nats::jetstream::{
    AckKind,
    message::PublishMessage,
    stream::{DiscardPolicy, StorageType},
};
use async_nats::{Client, HeaderMap, jetstream};
use messaging_core::{
    delivery::DeliveryDisposition,
    envelope::MessageEnvelope,
    ports::{DurablePublisher, MessageHeaders, PortFuture, PublishConfirmation, RequestReply},
    request::{RequestBudget, RequestError, RequestOutcome},
    subject::{Action, SubjectAuthorizer},
    topology::ContextStreamSpec,
};
use std::time::Duration;
use thiserror::Error;

/// Authenticated `JetStream` adapter with application-level subject authorization.
#[derive(Clone)]
pub struct JetStreamAdapter {
    client: Client,
    context: jetstream::Context,
    authorizer: SubjectAuthorizer,
}

impl JetStreamAdapter {
    /// Wraps an already authenticated/TLS-configured NATS client.
    #[must_use]
    pub fn new(client: Client, authorizer: SubjectAuthorizer) -> Self {
        Self {
            context: jetstream::new(client.clone()),
            client,
            authorizer,
        }
    }

    /// Reports the client's current connection state without broker I/O.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.client.connection_state() == async_nats::connection::State::Connected
    }

    /// Creates a reviewed context-owned stream; unsafe production replication fails first.
    pub async fn create_context_stream(
        &self,
        spec: &ContextStreamSpec,
        production: bool,
    ) -> Result<(), NatsAdapterError> {
        spec.validate(production)
            .map_err(|_| NatsAdapterError::UnsafeConfiguration)?;
        let max_bytes =
            i64::try_from(spec.maximum_bytes).map_err(|_| NatsAdapterError::UnsafeConfiguration)?;
        let max_messages = i64::try_from(spec.maximum_messages)
            .map_err(|_| NatsAdapterError::UnsafeConfiguration)?;
        let max_consumers = i32::try_from(spec.maximum_consumers)
            .map_err(|_| NatsAdapterError::UnsafeConfiguration)?;
        self.context
            .create_stream(jetstream::stream::Config {
                name: spec.name.clone(),
                subjects: vec![spec.owned_subject_pattern()],
                max_bytes,
                max_messages,
                max_age: spec.maximum_age,
                max_consumers,
                max_message_size: 1_048_576,
                storage: StorageType::File,
                num_replicas: usize::from(spec.replicas),
                duplicate_window: spec.duplicate_window,
                discard: DiscardPolicy::New,
                no_ack: false,
                deny_delete: true,
                deny_purge: true,
                description: Some(format!("owned by {} bounded context", spec.context)),
                ..Default::default()
            })
            .await
            .map_err(|error| NatsAdapterError::Broker(error.to_string()))?;
        Ok(())
    }

    /// Fails closed if existing broker configuration drifted from reviewed ownership/bounds.
    pub async fn verify_context_stream(
        &self,
        spec: &ContextStreamSpec,
        production: bool,
    ) -> Result<(), NatsAdapterError> {
        spec.validate(production)
            .map_err(|_| NatsAdapterError::UnsafeConfiguration)?;
        let mut stream = self
            .context
            .get_stream(&spec.name)
            .await
            .map_err(|error| NatsAdapterError::Broker(error.to_string()))?;
        let actual = &stream
            .info()
            .await
            .map_err(|error| NatsAdapterError::Broker(error.to_string()))?
            .config;
        let matches = actual.subjects == [spec.owned_subject_pattern()]
            && actual.max_bytes
                == i64::try_from(spec.maximum_bytes)
                    .map_err(|_| NatsAdapterError::UnsafeConfiguration)?
            && actual.max_messages
                == i64::try_from(spec.maximum_messages)
                    .map_err(|_| NatsAdapterError::UnsafeConfiguration)?
            && actual.max_age == spec.maximum_age
            && actual.duplicate_window == spec.duplicate_window
            && actual.num_replicas == usize::from(spec.replicas)
            && actual.storage == StorageType::File
            && actual.discard == DiscardPolicy::New
            && !actual.no_ack
            && actual.deny_delete
            && actual.deny_purge;
        if matches {
            Ok(())
        } else {
            Err(NatsAdapterError::ConfigurationDrift)
        }
    }

    /// Publishes with a stable `JetStream` deduplication ID and waits for durable acknowledgement.
    pub async fn publish(
        &self,
        envelope: &MessageEnvelope,
    ) -> Result<PublishConfirmation, NatsAdapterError> {
        let subject = &envelope.input().subject;
        self.authorizer
            .authorize(Action::Publish, subject)
            .map_err(|_| NatsAdapterError::Unauthorized)?;
        envelope
            .verify_payload(&envelope.input().payload)
            .map_err(|_| NatsAdapterError::IntegrityFailure)?;
        let mut headers = propagation_headers(envelope)?;
        headers.insert("Nats-Msg-Id", envelope.input().message_id.to_string());
        let future = self
            .context
            .send_publish(
                subject.as_str().to_owned(),
                PublishMessage::build()
                    .payload(envelope.input().payload.clone().into())
                    .headers(headers)
                    .message_id(envelope.input().message_id.to_string()),
            )
            .await
            .map_err(|error| classify_publish_start(&error))?;
        let acknowledgement = future.await.map_err(|error| {
            if error.kind() == jetstream::context::PublishErrorKind::TimedOut {
                NatsAdapterError::UnknownPublishOutcome
            } else {
                NatsAdapterError::Broker(error.to_string())
            }
        })?;
        Ok(PublishConfirmation {
            stream: acknowledgement.stream,
            sequence: acknowledgement.sequence,
            duplicate: acknowledgement.duplicate,
        })
    }

    /// Executes request/reply within a prevalidated monotonic budget.
    pub async fn request(
        &self,
        subject: &messaging_core::subject::Subject,
        headers: HeaderMap,
        payload: Vec<u8>,
        budget: &RequestBudget,
        now: shared_kernel::MonotonicInstant,
    ) -> Result<RequestOutcome<Vec<u8>>, NatsAdapterError> {
        self.authorizer
            .authorize(Action::Publish, subject)
            .map_err(|_| NatsAdapterError::Unauthorized)?;
        let remaining = budget
            .remaining(now, payload.len())
            .map_err(map_request_error)?;
        match tokio::time::timeout(
            remaining,
            self.client
                .request_with_headers(subject.as_str().to_owned(), headers, payload.into()),
        )
        .await
        {
            Ok(Ok(reply)) => Ok(RequestOutcome::Reply(reply.payload.to_vec())),
            Ok(Err(error)) => Err(NatsAdapterError::Broker(error.to_string())),
            Err(_) => Ok(RequestOutcome::UnknownAfterDeadline),
        }
    }
}

impl DurablePublisher for JetStreamAdapter {
    type Error = NatsAdapterError;

    fn publish<'a>(
        &'a self,
        envelope: &'a MessageEnvelope,
    ) -> PortFuture<'a, PublishConfirmation, Self::Error> {
        Box::pin(JetStreamAdapter::publish(self, envelope))
    }
}

impl RequestReply for JetStreamAdapter {
    type Error = NatsAdapterError;

    fn request<'a>(
        &'a self,
        subject: &'a messaging_core::subject::Subject,
        headers: MessageHeaders,
        payload: Vec<u8>,
        budget: &'a RequestBudget,
        now: shared_kernel::MonotonicInstant,
    ) -> PortFuture<'a, RequestOutcome<Vec<u8>>, Self::Error> {
        let mut native_headers = HeaderMap::new();
        for (name, value) in headers {
            native_headers.insert(name, value);
        }
        Box::pin(JetStreamAdapter::request(
            self,
            subject,
            native_headers,
            payload,
            budget,
            now,
        ))
    }
}

fn propagation_headers(envelope: &MessageEnvelope) -> Result<HeaderMap, NatsAdapterError> {
    let mut carrier = std::collections::BTreeMap::new();
    envelope
        .input()
        .propagation
        .inject(&mut carrier)
        .map_err(|_| NatsAdapterError::IntegrityFailure)?;
    let mut headers = HeaderMap::new();
    for (key, value) in carrier {
        headers.insert(key, value);
    }
    headers.insert(
        "X-Wildfire-Payload-SHA256",
        envelope.payload_digest().to_string(),
    );
    Ok(headers)
}

fn classify_publish_start(error: &jetstream::context::PublishError) -> NatsAdapterError {
    NatsAdapterError::DefinitePublishFailure(error.to_string())
}

fn map_request_error(error: RequestError) -> NatsAdapterError {
    match error {
        RequestError::DeadlineExceeded => NatsAdapterError::RequestDeadlineExceeded,
        RequestError::ClockEpochMismatch => NatsAdapterError::ClockEpochMismatch,
        RequestError::InvalidRequest => NatsAdapterError::UnsafeConfiguration,
    }
}

/// Applies an explicit core disposition to a delivered `JetStream` message.
pub async fn acknowledge(
    message: &jetstream::Message,
    disposition: &DeliveryDisposition,
) -> Result<(), NatsAdapterError> {
    let kind = match disposition {
        DeliveryDisposition::Ack => AckKind::Ack,
        DeliveryDisposition::Nak { delay_millis } => {
            AckKind::Nak(Some(Duration::from_millis(*delay_millis)))
        }
        DeliveryDisposition::Quarantine { .. } => AckKind::Term,
    };
    message
        .ack_with(kind)
        .await
        .map_err(|error| NatsAdapterError::Broker(error.to_string()))
}

/// Stable adapter failures preserving unknown publish outcomes.
#[derive(Debug, Error)]
pub enum NatsAdapterError {
    /// Application-level subject authorization denied before broker I/O.
    #[error("subject operation is unauthorized")]
    Unauthorized,
    /// Reviewed stream/leaf/request bounds are unsafe.
    #[error("messaging configuration is unsafe")]
    UnsafeConfiguration,
    /// Existing stream differs from reviewed context ownership or limits.
    #[error("JetStream configuration drift detected")]
    ConfigurationDrift,
    /// Envelope/payload/trace integrity failed.
    #[error("message integrity validation failed")]
    IntegrityFailure,
    /// Publish was rejected before it could have reached the broker.
    #[error("publish definitely failed: {0}")]
    DefinitePublishFailure(String),
    /// Broker may have committed the stable message ID but acknowledgement was lost.
    #[error("publish acknowledgement timed out; outcome is unknown")]
    UnknownPublishOutcome,
    /// Request deadline elapsed before broker I/O.
    #[error("request deadline exceeded")]
    RequestDeadlineExceeded,
    /// Monotonic clock epoch changed.
    #[error("monotonic clock epoch changed")]
    ClockEpochMismatch,
    /// Broker operation failed.
    #[error("NATS operation failed: {0}")]
    Broker(String),
}
