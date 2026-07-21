//! Prompt 12 simulator, gateway, acknowledgement, telemetry, and fault campaigns.
#![allow(clippy::expect_used)]
use chrono::{DateTime, Duration, Utc};
use shared_kernel::EntityId;
use std::collections::{BTreeMap, BTreeSet};
use vehicle_integration::*;

fn now() -> DateTime<Utc> {
    DateTime::<Utc>::UNIX_EPOCH + Duration::days(50)
}
fn gateway_session(vehicle: &EntityId) -> GatewaySession {
    let mut s = GatewaySession::open(
        EntityId::new(),
        vehicle.clone(),
        "spiffe://vehicle/device",
        1,
    )
    .expect("session");
    s.authenticate_and_negotiate(
        true,
        [Capability::Drive, Capability::Flight, Capability::Tool],
    )
    .expect("authenticate");
    s
}
fn intent(id: EntityId, vehicle: EntityId, fence: u64) -> VehicleIntent {
    VehicleIntent {
        intent_id: id,
        vehicle_id: vehicle,
        capability: Capability::Drive,
        operation: "advance".into(),
        parameters: BTreeMap::from([("distance_mm".into(), 100)]),
        payload_digest: [7; 32],
        issued_at: now() - Duration::seconds(1),
        expires_at: now() + Duration::minutes(1),
        fence,
        safety_version: 2,
        signature: "valid-command-signature".into(),
    }
}
struct Permit;
impl IntentVerifier for Permit {
    type Error = ();
    fn verify(&self, _: &VehicleIntent) -> Result<bool, Self::Error> {
        Ok(true)
    }
}
impl LocalConstraintPort for Permit {
    type Error = ();
    fn permits(&self, _: &VehicleIntent) -> Result<bool, Self::Error> {
        Ok(true)
    }
}
fn validated(intent: VehicleIntent, session: &mut GatewaySession) -> CommandDelivery {
    let mut delivery = CommandDelivery::receive(intent, 3).expect("receive");
    delivery
        .validate(session, &Permit, &Permit, now(), 1, 10)
        .expect("validate");
    delivery.queue().expect("queue");
    delivery
}

#[test]
fn duplicate_and_reordered_commands_create_one_physical_effect() {
    let vehicle = EntityId::new();
    let id = EntityId::new();
    let mut session = gateway_session(&vehicle);
    let first = validated(intent(id.clone(), vehicle.clone(), 1), &mut session);
    let mut second = CommandDelivery::receive(intent(id, vehicle, 1), 3).expect("duplicate");
    second
        .validate(&mut session, &Permit, &Permit, now(), 1, 10)
        .expect("validate same fence");
    second.queue().expect("queue");
    let mut simulator = DeterministicSimulator::default();
    let mut one = first;
    assert_eq!(
        dispatch(&mut one, &mut simulator),
        Ok(ControllerResult::Accepted)
    );
    assert!(matches!(
        dispatch(&mut second, &mut simulator),
        Ok(ControllerResult::Duplicate { .. })
    ));
    assert_eq!(simulator.effect_count(), 1);
    assert_eq!(simulator.state().drive_distance_mm, 100);
    assert_eq!(
        one.record(AckClass::ExecutionStarted, None),
        Err(VehicleError::InvalidDeliveryTransition),
        "reordered execution evidence cannot skip ack and acceptance"
    );
}

#[test]
fn acknowledgement_stages_are_distinct_and_physical_outcome_needs_evidence() {
    let vehicle = EntityId::new();
    let mut session = gateway_session(&vehicle);
    let mut delivery = validated(intent(EntityId::new(), vehicle, 1), &mut session);
    let mut simulator = DeterministicSimulator::default();
    dispatch(&mut delivery, &mut simulator).expect("dispatch");
    delivery
        .record(AckClass::Transport, None)
        .expect("transport");
    assert_eq!(delivery.state(), DeliveryState::TransportAcknowledged);
    delivery
        .record(AckClass::VehicleAccepted, None)
        .expect("acceptance");
    delivery
        .record(AckClass::ExecutionStarted, None)
        .expect("execution");
    assert_eq!(
        delivery.record(AckClass::PhysicalOutcome, None),
        Err(VehicleError::InvalidOutcome)
    );
    delivery
        .record(AckClass::PhysicalOutcome, Some([8; 32]))
        .expect("physical evidence");
    assert_eq!(delivery.state(), DeliveryState::Completed);
}

#[test]
fn stale_fence_clock_fault_link_loss_and_adapter_crash_are_explicit_safe_states() {
    let vehicle = EntityId::new();
    let mut session = gateway_session(&vehicle);
    let mut current = validated(intent(EntityId::new(), vehicle.clone(), 5), &mut session);
    let stale =
        CommandDelivery::receive(intent(EntityId::new(), vehicle.clone(), 4), 2).expect("receive");
    let mut stale = stale;
    assert_eq!(
        stale.validate(&mut session, &Permit, &Permit, now(), 1, 10),
        Err(VehicleError::StaleFence)
    );
    let mut clock =
        CommandDelivery::receive(intent(EntityId::new(), vehicle.clone(), 6), 2).expect("receive");
    assert_eq!(
        clock.validate(&mut session, &Permit, &Permit, now(), 11, 10),
        Err(VehicleError::ClockUncertain)
    );
    let mut simulator = DeterministicSimulator::default();
    dispatch(&mut current, &mut simulator).expect("dispatch");
    handle_link_loss(&mut session, &mut current, &mut simulator).expect("safe link loss");
    assert_eq!(current.state(), DeliveryState::Unknown);
    assert_eq!(
        simulator.state().last_safe_reason,
        Some(SafeStateReason::LinkLoss)
    );
    let mut new_session = gateway_session(&vehicle);
    let mut crashing = validated(intent(EntityId::new(), vehicle, 7), &mut new_session);
    let mut crash_sim = DeterministicSimulator::default();
    crash_sim.inject_crash();
    assert_eq!(
        dispatch(&mut crashing, &mut crash_sim),
        Ok(ControllerResult::Unknown)
    );
    assert_eq!(crashing.state(), DeliveryState::Unknown);
    assert_eq!(crash_sim.state().minimum_risk_entries, 1);
}

#[test]
fn telemetry_gaps_clock_faults_and_drop_tiers_are_explicit() {
    let mut stream = TelemetryStream::open(TelemetryTier::Operational);
    stream.activate().expect("active");
    let sample = NormalizedSample {
        sequence: 1,
        observed_at: now(),
        received_at: now(),
        source: "sim".into(),
        values: BTreeMap::from([("speed_mm_s".into(), 10)]),
        quality_flags: BTreeSet::new(),
        clock: ClockQuality {
            synchronized: true,
            uncertainty_ms: 2,
        },
        tier: TelemetryTier::Operational,
    };
    stream.accept(&sample, 10).expect("sample");
    let mut gap = sample.clone();
    gap.sequence = 3;
    assert_eq!(stream.accept(&gap, 10), Err(VehicleError::InvalidTelemetry));
    assert_eq!(stream.gaps(), 1);
    stream.account_drop(TelemetryTier::Bulk, u64::MAX);
    stream.account_drop(TelemetryTier::Bulk, 1);
    assert_eq!(stream.dropped(TelemetryTier::Bulk), u64::MAX);
}

#[cfg(all(feature = "mavlink-adapter", feature = "ros2-adapter"))]
#[test]
fn feature_facades_translate_only_supported_capabilities_without_type_leak() {
    let vehicle = EntityId::new();
    let drive = intent(EntityId::new(), vehicle.clone(), 1);
    assert!(vehicle_integration::ros2::translate(&drive).is_some());
    assert!(vehicle_integration::mavlink::translate(&drive).is_none());
    let mut flight = intent(EntityId::new(), vehicle, 2);
    flight.capability = Capability::Flight;
    assert!(vehicle_integration::mavlink::translate(&flight).is_some());
    assert!(vehicle_integration::ros2::translate(&flight).is_none());
}
