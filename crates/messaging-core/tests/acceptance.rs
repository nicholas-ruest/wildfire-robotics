//! Outside-in messaging substrate acceptance tests.

use chrono::{DateTime, Utc};
use messaging_core::subject::{Action, Subject, SubjectAuthorizer, SubjectGrant};
use messaging_core::{
    delivery::{DeliveryDisposition, FailureClass, RetryPolicy},
    envelope::{EnvelopeError, EnvelopeInput, MessageEnvelope, SafetyReferences},
    store_forward::{EnqueueOutcome, StoreForwardQueue, TelemetryTier},
};
use operations_core::telemetry::PropagationContext;
use shared_kernel::{DataClassification, SemanticVersion};
use uuid::Uuid;

#[test]
fn should_deny_cross_tenant_and_wildcard_subjects_before_broker_io()
-> Result<(), Box<dyn std::error::Error>> {
    let grant = SubjectGrant::new(
        Action::Publish,
        "prod",
        "ca-bc",
        "tenant-a",
        "fleet-operations",
        ["VehicleGrounded"],
    )?;
    let authorizer = SubjectAuthorizer::new([grant]);
    let allowed =
        Subject::parse("wr.prod.ca-bc.tenant-a.fleet-operations.vehicle.VehicleGrounded.v1")?;
    let crossed =
        Subject::parse("wr.prod.ca-bc.tenant-b.fleet-operations.vehicle.VehicleGrounded.v1")?;

    assert!(authorizer.authorize(Action::Publish, &allowed).is_ok());
    assert!(authorizer.authorize(Action::Publish, &crossed).is_err());
    assert!(Subject::parse("wr.prod.*.tenant-a.fleet.vehicle.Event.v1").is_err());
    assert!(Subject::parse("wr.prod.ca.tenant-a.fleet.vehicle.Event.v0").is_err());
    Ok(())
}

#[test]
fn should_bound_retries_then_quarantine_poison_without_hot_looping()
-> Result<(), Box<dyn std::error::Error>> {
    let policy = RetryPolicy::new(3, 10, 1_000, 20)?;
    assert!(matches!(
        policy.disposition("message-1", 1, FailureClass::Retryable),
        DeliveryDisposition::Nak { .. }
    ));
    assert!(matches!(
        policy.disposition("message-1", 3, FailureClass::Retryable),
        DeliveryDisposition::Quarantine { .. }
    ));
    assert!(matches!(
        policy.disposition("message-1", 1, FailureClass::Permanent),
        DeliveryDisposition::Quarantine { .. }
    ));
    Ok(())
}

#[test]
fn should_reserve_store_forward_capacity_for_safety_and_account_lower_tier_drops()
-> Result<(), Box<dyn std::error::Error>> {
    let mut queue = StoreForwardQueue::new(3, 30, 1)?;
    assert_eq!(
        queue.enqueue(TelemetryTier::Bulk, vec![1; 10])?,
        EnqueueOutcome::Accepted
    );
    assert_eq!(
        queue.enqueue(TelemetryTier::Diagnostic, vec![2; 10])?,
        EnqueueOutcome::Accepted
    );
    assert_eq!(
        queue.enqueue(TelemetryTier::Operational, vec![3; 10])?,
        EnqueueOutcome::DroppedLowerTier
    );
    assert_eq!(
        queue.enqueue(TelemetryTier::SafetyCritical, vec![4; 10])?,
        EnqueueOutcome::Accepted
    );
    assert_eq!(queue.dropped(TelemetryTier::Operational), 1);
    assert_eq!(
        queue.pop().map(|item| item.tier),
        Some(TelemetryTier::SafetyCritical)
    );
    Ok(())
}

fn envelope_input() -> Result<EnvelopeInput, Box<dyn std::error::Error>> {
    Ok(EnvelopeInput {
        message_id: Uuid::new_v4(),
        event_type: "VehicleGrounded".into(),
        schema_version: SemanticVersion::new(1, 0, 0),
        occurred_at: DateTime::<Utc>::UNIX_EPOCH,
        recorded_at: DateTime::<Utc>::UNIX_EPOCH,
        producer: "fleet-operations".into(),
        producer_version: SemanticVersion::new(1, 2, 3),
        aggregate_type: "vehicle".into(),
        aggregate_id: Uuid::new_v4(),
        aggregate_version: 4,
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
            "wr.prod.ca-bc.tenant-a.fleet-operations.vehicle.VehicleGrounded.v1",
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
    })
}

#[test]
fn should_bind_payload_scope_subject_trace_and_safety_references()
-> Result<(), Box<dyn std::error::Error>> {
    let envelope = MessageEnvelope::new(envelope_input()?, true)?;
    envelope.verify_payload(&[1, 2, 3])?;
    assert_eq!(
        envelope.verify_payload(&[1, 2, 4]),
        Err(EnvelopeError::DigestMismatch)
    );
    assert!(
        envelope
            .verify_transport_subject(
                "wr.prod.ca-bc.tenant-b.fleet-operations.vehicle.VehicleGrounded.v1"
            )
            .is_err()
    );
    let mut crossed = envelope_input()?;
    crossed.tenant_id = "tenant-b".into();
    assert_eq!(
        MessageEnvelope::new(crossed, true),
        Err(EnvelopeError::ScopeOrSubjectMismatch)
    );
    Ok(())
}
