//! Gateway, delivery, controller ports, and telemetry domain.
#![allow(missing_docs)]
use chrono::{DateTime, Utc};
use shared_kernel::EntityId;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
pub type Digest = [u8; 32];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Capability {
    Flight,
    Drive,
    Tool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionState {
    Negotiating,
    Active,
    Degraded,
    Closed,
    Compromised,
}
#[derive(Clone, Debug)]
pub struct GatewaySession {
    id: EntityId,
    vehicle_id: EntityId,
    peer_identity: String,
    adapter_version: u32,
    capabilities: BTreeSet<Capability>,
    state: SessionState,
    last_fence: u64,
    version: u64,
}
impl GatewaySession {
    pub fn open(
        id: EntityId,
        vehicle_id: EntityId,
        peer_identity: impl Into<String>,
        adapter_version: u32,
    ) -> Result<Self, VehicleError> {
        let peer_identity = peer_identity.into();
        if peer_identity.trim().is_empty() || adapter_version == 0 {
            return Err(VehicleError::InvalidSession);
        }
        Ok(Self {
            id,
            vehicle_id,
            peer_identity,
            adapter_version,
            capabilities: BTreeSet::new(),
            state: SessionState::Negotiating,
            last_fence: 0,
            version: 1,
        })
    }
    pub fn authenticate_and_negotiate(
        &mut self,
        authenticated: bool,
        capabilities: impl IntoIterator<Item = Capability>,
    ) -> Result<(), VehicleError> {
        let capabilities = capabilities.into_iter().collect::<BTreeSet<_>>();
        if self.state != SessionState::Negotiating || !authenticated || capabilities.is_empty() {
            self.state = SessionState::Compromised;
            return Err(VehicleError::Unauthenticated);
        }
        self.capabilities = capabilities;
        self.state = SessionState::Active;
        self.bump()
    }
    pub fn accept_fence(&mut self, fence: u64) -> Result<(), VehicleError> {
        if self.state != SessionState::Active || fence == 0 || fence < self.last_fence {
            return Err(VehicleError::StaleFence);
        }
        self.last_fence = fence;
        Ok(())
    }
    pub fn degrade(&mut self) -> Result<(), VehicleError> {
        if self.state != SessionState::Active {
            return Err(VehicleError::InvalidSession);
        }
        self.state = SessionState::Degraded;
        self.bump()
    }
    pub fn close(&mut self) -> Result<(), VehicleError> {
        if matches!(self.state, SessionState::Closed | SessionState::Compromised) {
            return Err(VehicleError::InvalidSession);
        }
        self.state = SessionState::Closed;
        self.bump()
    }
    fn bump(&mut self) -> Result<(), VehicleError> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or(VehicleError::VersionExhausted)?;
        Ok(())
    }
    #[must_use]
    pub fn supports(&self, c: Capability) -> bool {
        self.state == SessionState::Active && self.capabilities.contains(&c)
    }
    #[must_use]
    pub fn vehicle_id(&self) -> &EntityId {
        &self.vehicle_id
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub fn peer_identity(&self) -> &str {
        &self.peer_identity
    }
    #[must_use]
    pub const fn adapter_version(&self) -> u32 {
        self.adapter_version
    }
}

/// Protocol-neutral objective intent, never a vendor command packet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VehicleIntent {
    pub intent_id: EntityId,
    pub vehicle_id: EntityId,
    pub capability: Capability,
    pub operation: String,
    pub parameters: BTreeMap<String, i64>,
    pub payload_digest: Digest,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub fence: u64,
    pub safety_version: u64,
    pub signature: String,
}
impl VehicleIntent {
    pub fn validate_shape(&self, now: DateTime<Utc>) -> Result<(), VehicleError> {
        if self.operation.trim().is_empty()
            || self.operation.len() > 128
            || self.parameters.len() > 32
            || self.payload_digest == [0; 32]
            || self.issued_at > now
            || now >= self.expires_at
            || self.fence == 0
            || self.safety_version == 0
            || self.signature.len() < 16
        {
            return Err(VehicleError::InvalidIntent);
        }
        Ok(())
    }
}
pub trait IntentVerifier {
    type Error;
    fn verify(&self, intent: &VehicleIntent) -> Result<bool, Self::Error>;
}
pub trait LocalConstraintPort {
    type Error;
    fn permits(&self, intent: &VehicleIntent) -> Result<bool, Self::Error>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryState {
    Received,
    Validated,
    Queued,
    Sent,
    TransportAcknowledged,
    Accepted,
    Executing,
    Completed,
    Rejected,
    Expired,
    Unknown,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AckClass {
    Transport,
    VehicleAccepted,
    ExecutionStarted,
    PhysicalOutcome,
}
#[derive(Clone, Debug)]
pub struct CommandDelivery {
    intent: VehicleIntent,
    state: DeliveryState,
    attempts: u8,
    maximum_attempts: u8,
    outcome_digest: Option<Digest>,
    version: u64,
}
impl CommandDelivery {
    pub fn receive(intent: VehicleIntent, maximum_attempts: u8) -> Result<Self, VehicleError> {
        if maximum_attempts == 0 || maximum_attempts > 10 {
            return Err(VehicleError::RetryExhausted);
        }
        Ok(Self {
            intent,
            state: DeliveryState::Received,
            attempts: 0,
            maximum_attempts,
            outcome_digest: None,
            version: 1,
        })
    }
    pub fn validate<V: IntentVerifier, C: LocalConstraintPort>(
        &mut self,
        session: &mut GatewaySession,
        verifier: &V,
        constraints: &C,
        now: DateTime<Utc>,
        clock_uncertainty_ms: u64,
        maximum_clock_uncertainty_ms: u64,
    ) -> Result<(), VehicleError> {
        if self.state != DeliveryState::Received {
            return Err(VehicleError::InvalidDeliveryTransition);
        }
        self.intent.validate_shape(now)?;
        if clock_uncertainty_ms > maximum_clock_uncertainty_ms {
            return self.reject(VehicleError::ClockUncertain);
        }
        if self.intent.vehicle_id != *session.vehicle_id()
            || !session.supports(self.intent.capability)
        {
            return self.reject(VehicleError::ScopeMismatch);
        }
        if let Err(error) = session.accept_fence(self.intent.fence) {
            return self.reject(error);
        }
        if !matches!(verifier.verify(&self.intent), Ok(true)) {
            return self.reject(VehicleError::InvalidSignature);
        }
        if !matches!(constraints.permits(&self.intent), Ok(true)) {
            return self.reject(VehicleError::LocalConstraint);
        }
        self.state = DeliveryState::Validated;
        self.bump()
    }
    pub fn queue(&mut self) -> Result<(), VehicleError> {
        self.transition(DeliveryState::Validated, DeliveryState::Queued)
    }
    pub fn mark_sent(&mut self) -> Result<(), VehicleError> {
        if self.state != DeliveryState::Queued || self.attempts >= self.maximum_attempts {
            return Err(VehicleError::RetryExhausted);
        }
        self.attempts = self.attempts.saturating_add(1);
        self.state = DeliveryState::Sent;
        self.bump()
    }
    pub fn record(&mut self, ack: AckClass, outcome: Option<Digest>) -> Result<(), VehicleError> {
        let (expected, next) = match ack {
            AckClass::Transport => (DeliveryState::Sent, DeliveryState::TransportAcknowledged),
            AckClass::VehicleAccepted => (
                DeliveryState::TransportAcknowledged,
                DeliveryState::Accepted,
            ),
            AckClass::ExecutionStarted => (DeliveryState::Accepted, DeliveryState::Executing),
            AckClass::PhysicalOutcome => (DeliveryState::Executing, DeliveryState::Completed),
        };
        if self.state != expected {
            return Err(VehicleError::InvalidDeliveryTransition);
        }
        if ack == AckClass::PhysicalOutcome {
            let digest = outcome.ok_or(VehicleError::InvalidOutcome)?;
            if digest == [0; 32] {
                return Err(VehicleError::InvalidOutcome);
            }
            self.outcome_digest = Some(digest);
        }
        self.state = next;
        self.bump()
    }
    pub fn mark_unknown(&mut self) -> Result<(), VehicleError> {
        if !matches!(
            self.state,
            DeliveryState::Sent
                | DeliveryState::TransportAcknowledged
                | DeliveryState::Accepted
                | DeliveryState::Executing
        ) {
            return Err(VehicleError::InvalidDeliveryTransition);
        }
        self.state = DeliveryState::Unknown;
        self.bump()
    }
    pub fn expire(&mut self, now: DateTime<Utc>) -> Result<(), VehicleError> {
        if now < self.intent.expires_at {
            return Err(VehicleError::InvalidDeliveryTransition);
        }
        self.state = DeliveryState::Expired;
        self.bump()
    }
    fn reject(&mut self, error: VehicleError) -> Result<(), VehicleError> {
        self.state = DeliveryState::Rejected;
        self.bump()?;
        Err(error)
    }
    fn transition(&mut self, f: DeliveryState, t: DeliveryState) -> Result<(), VehicleError> {
        if self.state != f {
            return Err(VehicleError::InvalidDeliveryTransition);
        }
        self.state = t;
        self.bump()
    }
    fn bump(&mut self) -> Result<(), VehicleError> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or(VehicleError::VersionExhausted)?;
        Ok(())
    }
    #[must_use]
    pub fn intent(&self) -> &VehicleIntent {
        &self.intent
    }
    #[must_use]
    pub fn state(&self) -> DeliveryState {
        self.state
    }
    #[must_use]
    pub fn attempts(&self) -> u8 {
        self.attempts
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ControllerResult {
    Accepted,
    Duplicate { prior_outcome: Option<Digest> },
    Rejected,
    Unknown,
}
pub trait CapabilityController {
    type Error;
    fn apply(&mut self, intent: &VehicleIntent) -> Result<ControllerResult, Self::Error>;
    fn minimum_risk(
        &mut self,
        capability: Capability,
        reason: SafeStateReason,
    ) -> Result<(), Self::Error>;
}

/// Crosses the adapter boundary once; ambiguous failures become Unknown plus minimum-risk.
pub fn dispatch<C: CapabilityController>(
    delivery: &mut CommandDelivery,
    controller: &mut C,
) -> Result<ControllerResult, VehicleError> {
    delivery.mark_sent()?;
    match controller.apply(delivery.intent()) {
        Ok(ControllerResult::Unknown) | Err(_) => {
            delivery.mark_unknown()?;
            controller
                .minimum_risk(
                    delivery.intent().capability,
                    SafeStateReason::UnknownOutcome,
                )
                .map_err(|_| VehicleError::InvalidOutcome)?;
            Ok(ControllerResult::Unknown)
        }
        Ok(result) => Ok(result),
    }
}

/// Makes link loss explicit without fabricating rejection or physical completion.
pub fn handle_link_loss<C: CapabilityController>(
    session: &mut GatewaySession,
    delivery: &mut CommandDelivery,
    controller: &mut C,
) -> Result<(), VehicleError> {
    session.degrade()?;
    if matches!(
        delivery.state(),
        DeliveryState::Sent
            | DeliveryState::TransportAcknowledged
            | DeliveryState::Accepted
            | DeliveryState::Executing
    ) {
        delivery.mark_unknown()?;
    }
    controller
        .minimum_risk(delivery.intent().capability, SafeStateReason::LinkLoss)
        .map_err(|_| VehicleError::InvalidOutcome)
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SafeStateReason {
    LinkLoss,
    AdapterCrash,
    ClockFault,
    UnknownOutcome,
    LocalConstraint,
    StaleFence,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TelemetryTier {
    Bulk,
    Diagnostic,
    Operational,
    SafetyCritical,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClockQuality {
    pub synchronized: bool,
    pub uncertainty_ms: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedSample {
    pub sequence: u64,
    pub observed_at: DateTime<Utc>,
    pub received_at: DateTime<Utc>,
    pub source: String,
    pub values: BTreeMap<String, i64>,
    pub quality_flags: BTreeSet<String>,
    pub clock: ClockQuality,
    pub tier: TelemetryTier,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemetryState {
    Opening,
    Active,
    Degraded,
    Closed,
}
#[derive(Clone, Debug)]
pub struct TelemetryStream {
    state: TelemetryState,
    last_sequence: u64,
    tier: TelemetryTier,
    gaps: u64,
    drops: BTreeMap<TelemetryTier, u64>,
}
impl TelemetryStream {
    #[must_use]
    pub fn open(tier: TelemetryTier) -> Self {
        Self {
            state: TelemetryState::Opening,
            last_sequence: 0,
            tier,
            gaps: 0,
            drops: BTreeMap::new(),
        }
    }
    pub fn activate(&mut self) -> Result<(), VehicleError> {
        if self.state != TelemetryState::Opening {
            return Err(VehicleError::InvalidTelemetry);
        }
        self.state = TelemetryState::Active;
        Ok(())
    }
    pub fn accept(
        &mut self,
        sample: &NormalizedSample,
        maximum_clock_uncertainty_ms: u64,
    ) -> Result<(), VehicleError> {
        if self.state != TelemetryState::Active
            || sample.sequence != self.last_sequence.saturating_add(1)
            || sample.source.trim().is_empty()
            || sample.values.is_empty()
            || !sample.clock.synchronized
            || sample.clock.uncertainty_ms > maximum_clock_uncertainty_ms
        {
            return self.gap();
        }
        self.last_sequence = sample.sequence;
        self.tier = sample.tier;
        Ok(())
    }
    pub fn account_drop(&mut self, tier: TelemetryTier, count: u64) {
        let current = self.drops.get(&tier).copied().unwrap_or(0);
        self.drops.insert(tier, current.saturating_add(count));
    }
    fn gap(&mut self) -> Result<(), VehicleError> {
        self.gaps = self.gaps.saturating_add(1);
        self.state = TelemetryState::Degraded;
        Err(VehicleError::InvalidTelemetry)
    }
    #[must_use]
    pub fn gaps(&self) -> u64 {
        self.gaps
    }
    #[must_use]
    pub fn dropped(&self, tier: TelemetryTier) -> u64 {
        self.drops.get(&tier).copied().unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum VehicleError {
    #[error("gateway session is invalid")]
    InvalidSession,
    #[error("peer is unauthenticated or capability negotiation failed")]
    Unauthenticated,
    #[error("intent shape, freshness, or safety version is invalid")]
    InvalidIntent,
    #[error("intent signature is invalid")]
    InvalidSignature,
    #[error("intent scope or capability does not match session")]
    ScopeMismatch,
    #[error("fencing token is stale")]
    StaleFence,
    #[error("local safety constraint denied intent")]
    LocalConstraint,
    #[error("vehicle clock uncertainty is unsafe")]
    ClockUncertain,
    #[error("delivery transition is invalid")]
    InvalidDeliveryTransition,
    #[error("bounded retry budget exhausted")]
    RetryExhausted,
    #[error("physical outcome evidence is invalid")]
    InvalidOutcome,
    #[error("telemetry sample is invalid, stale, or gapped")]
    InvalidTelemetry,
    #[error("aggregate version exhausted")]
    VersionExhausted,
}
