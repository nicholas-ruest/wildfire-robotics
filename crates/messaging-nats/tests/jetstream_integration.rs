//! Real pinned NATS/JetStream adapter integration tests.

use chrono::{DateTime, Utc};
use futures::StreamExt;
use messaging_core::delivery::DeliveryDisposition;
use messaging_core::{
    envelope::{EnvelopeInput, MessageEnvelope, SafetyReferences},
    subject::{Action, Subject, SubjectAuthorizer, SubjectGrant},
    topology::ContextStreamSpec,
};
use messaging_nats::JetStreamAdapter;
use messaging_nats::acknowledge;
use operations_core::telemetry::PropagationContext;
use shared_kernel::{DataClassification, SemanticVersion};
use std::time::Duration;
use testcontainers::{ImageExt, runners::AsyncRunner};
use testcontainers_modules::nats::{Nats, NatsServerCmd};
use uuid::Uuid;

fn stream_spec() -> ContextStreamSpec {
    ContextStreamSpec {
        name: "FLEET_TENANT_A".into(),
        environment: "test".into(),
        region: "ca-bc".into(),
        tenant: "tenant-a".into(),
        context: "fleet-operations".into(),
        maximum_bytes: 16_000_000,
        maximum_messages: 10_000,
        maximum_age: Duration::from_hours(1),
        duplicate_window: Duration::from_mins(5),
        replicas: 1,
        maximum_consumers: 16,
    }
}

#[tokio::test]
async fn should_redeliver_unacked_and_term_poison_messages()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let command = NatsServerCmd::default().with_jetstream();
    let container = Nats::default().with_cmd(&command).start().await?;
    let host = container.get_host().await?;
    let port = container.get_host_port_ipv4(4222).await?;
    let client = async_nats::ConnectOptions::new()
        .reconnect_delay_callback(|_| Duration::from_millis(50))
        .connect(format!("{host}:{port}"))
        .await?;
    let grant = SubjectGrant::new(
        Action::Publish,
        "test",
        "ca-bc",
        "tenant-a",
        "fleet-operations",
        ["VehicleGrounded"],
    )?;
    let adapter = JetStreamAdapter::new(client.clone(), SubjectAuthorizer::new([grant]));
    adapter.create_context_stream(&stream_spec(), false).await?;
    adapter.publish(&envelope()?).await?;
    let context = async_nats::jetstream::new(client);
    let stream = context.get_stream("FLEET_TENANT_A").await?;
    let consumer: async_nats::jetstream::consumer::PullConsumer = stream
        .create_consumer(async_nats::jetstream::consumer::pull::Config {
            durable_name: Some("fault-consumer".into()),
            ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
            ack_wait: Duration::from_millis(100),
            max_deliver: 3,
            ..Default::default()
        })
        .await?;
    let mut first_batch = consumer.fetch().max_messages(1).messages().await?;
    let first = first_batch.next().await.ok_or("missing first delivery")??;
    let first_info = first.info()?;
    assert_eq!(first_info.delivered, 1);
    drop(first);
    tokio::time::sleep(Duration::from_millis(150)).await;
    let mut second_batch = consumer.fetch().max_messages(1).messages().await?;
    let second = second_batch.next().await.ok_or("missing redelivery")??;
    assert!(second.info()?.delivered >= 2);
    acknowledge(
        &second,
        &DeliveryDisposition::Quarantine {
            reason: "invalid schema",
        },
    )
    .await?;
    tokio::time::sleep(Duration::from_millis(150)).await;
    let mut after_term = consumer
        .fetch()
        .max_messages(1)
        .expires(Duration::from_millis(100))
        .messages()
        .await?;
    assert!(after_term.next().await.is_none());
    Ok(())
}

#[tokio::test]
async fn should_reconnect_after_partition_without_losing_durable_fact()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let command = NatsServerCmd::default().with_jetstream();
    let container = Nats::default().with_cmd(&command).start().await?;
    let host = container.get_host().await?;
    let port = container.get_host_port_ipv4(4222).await?;
    let client = async_nats::ConnectOptions::new()
        .reconnect_delay_callback(|_| Duration::from_millis(50))
        .connect(format!("{host}:{port}"))
        .await?;
    let grant = SubjectGrant::new(
        Action::Publish,
        "test",
        "ca-bc",
        "tenant-a",
        "fleet-operations",
        ["VehicleGrounded"],
    )?;
    let adapter = JetStreamAdapter::new(client, SubjectAuthorizer::new([grant]));
    let spec = stream_spec();
    adapter.create_context_stream(&spec, false).await?;
    let event = envelope()?;
    let original = adapter.publish(&event).await?;
    container.pause().await?;
    tokio::time::sleep(Duration::from_millis(250)).await;
    container.unpause().await?;
    tokio::time::sleep(Duration::from_millis(250)).await;
    adapter.verify_context_stream(&spec, false).await?;
    let replay = adapter.publish(&event).await?;
    assert!(replay.duplicate);
    assert_eq!(original.sequence, replay.sequence);
    Ok(())
}

fn envelope() -> Result<MessageEnvelope, Box<dyn std::error::Error + Send + Sync>> {
    MessageEnvelope::new(
        EnvelopeInput {
            message_id: Uuid::new_v4(),
            event_type: "VehicleGrounded".into(),
            schema_version: SemanticVersion::new(1, 0, 0),
            occurred_at: DateTime::<Utc>::UNIX_EPOCH,
            recorded_at: DateTime::<Utc>::UNIX_EPOCH,
            producer: "fleet-operations".into(),
            producer_version: SemanticVersion::new(1, 0, 0),
            aggregate_type: "vehicle".into(),
            aggregate_id: Uuid::new_v4(),
            aggregate_version: 1,
            tenant_id: "tenant-a".into(),
            region_id: Some("ca-bc".into()),
            incident_id: None,
            propagation: PropagationContext {
                traceparent: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01".into(),
                tracestate: None,
                tenant_id: "tenant-a".into(),
                incident_id: None,
                correlation_id: "correlation-1".into(),
                causation_id: None,
                release_id: "release-1".into(),
            },
            classification: DataClassification::Restricted,
            subject: Subject::parse(
                "wr.test.ca-bc.tenant-a.fleet-operations.vehicle.VehicleGrounded.v1",
            )?,
            content_type: "application/protobuf".into(),
            payload: vec![1, 2, 3],
            safety: Some(SafetyReferences {
                authority_id: "authority-1".into(),
                odd_id: "ODD-1".into(),
                constraint_id: "constraint-1".into(),
                evidence_id: "EVD-1".into(),
                clock_quality_id: "clock-1".into(),
            }),
        },
        true,
    )
    .map_err(Into::into)
}

#[tokio::test]
async fn should_persist_deduplicate_and_detect_stream_drift()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let command = NatsServerCmd::default().with_jetstream();
    let container = Nats::default().with_cmd(&command).start().await?;
    let host = container.get_host().await?;
    let port = container.get_host_port_ipv4(4222).await?;
    let client = async_nats::connect(format!("{host}:{port}")).await?;
    let grant = SubjectGrant::new(
        Action::Publish,
        "test",
        "ca-bc",
        "tenant-a",
        "fleet-operations",
        ["VehicleGrounded"],
    )?;
    let adapter = JetStreamAdapter::new(client, SubjectAuthorizer::new([grant]));
    let spec = stream_spec();
    adapter.create_context_stream(&spec, false).await?;
    adapter.verify_context_stream(&spec, false).await?;
    let event = envelope()?;
    let first = adapter.publish(&event).await?;
    let duplicate = adapter.publish(&event).await?;
    assert!(!first.duplicate);
    assert!(duplicate.duplicate);
    assert_eq!(first.sequence, duplicate.sequence);
    let mut drifted = spec;
    drifted.maximum_messages = 999;
    assert!(
        adapter
            .verify_context_stream(&drifted, false)
            .await
            .is_err()
    );
    Ok(())
}
