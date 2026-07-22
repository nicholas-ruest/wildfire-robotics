#![forbid(unsafe_code)]
#![allow(
    missing_docs,
    clippy::must_use_candidate,
    clippy::redundant_slicing,
    clippy::needless_pass_by_value,
    clippy::semicolon_if_nothing_returned
)]
use sha2::{Digest as _, Sha256};
use std::marker::PhantomData;
pub const CONTEXT_NAME: &str = "suppression-operations";
fn canonical_put(h: &mut Sha256, b: &[u8]) {
    h.update((b.len() as u64).to_be_bytes());
    h.update(b);
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SuppressionError {
    #[error("invalid target")]
    InvalidTarget,
    #[error("invalid plan")]
    InvalidPlan,
    #[error("authority stale")]
    StaleAuthority,
    #[error("approval invalid")]
    InvalidApproval,
    #[error("mode forbidden")]
    ModeForbidden,
    #[error("envelope breached")]
    EnvelopeBreach,
    #[error("independent stop failed")]
    IndependentStopFailed,
    #[error("invalid transition")]
    InvalidTransition,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x_mm: i32,
    pub y_mm: i32,
}
#[derive(Debug, Clone)]
pub struct Candidate;
#[derive(Debug, Clone)]
pub struct Verified;
#[derive(Debug, Clone)]
pub struct Approved;
#[derive(Debug, Clone)]
pub struct Target<S> {
    id: String,
    point: Point,
    verification: Option<VerificationEvidence>,
    approval: Option<TargetApproval>,
    state: PhantomData<S>,
}
#[derive(Debug, Clone, Copy)]
pub struct VerificationEvidence {
    pub digest: [u8; 32],
    pub uncertainty_mm: u32,
}
#[derive(Debug, Clone)]
pub struct TargetApproval {
    pub authority_id: String,
    pub digest: [u8; 32],
}
impl Target<Candidate> {
    pub fn candidate(id: &str, point: Point) -> Self {
        Self {
            id: id.into(),
            point,
            verification: None,
            approval: None,
            state: PhantomData,
        }
    }
    pub fn verify(self, e: VerificationEvidence) -> Result<Target<Verified>, SuppressionError> {
        if self.id.is_empty() || e.digest == [0; 32] {
            return Err(SuppressionError::InvalidTarget);
        }
        Ok(Target {
            id: self.id,
            point: self.point,
            verification: Some(e),
            approval: None,
            state: PhantomData,
        })
    }
}
impl Target<Verified> {
    pub fn approve(self, a: TargetApproval) -> Result<Target<Approved>, SuppressionError> {
        if a.authority_id.is_empty() || a.digest == [0; 32] {
            return Err(SuppressionError::InvalidTarget);
        }
        Ok(Target {
            id: self.id,
            point: self.point,
            verification: self.verification,
            approval: Some(a),
            state: PhantomData,
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fence {
    pub min_x_mm: i32,
    pub max_x_mm: i32,
    pub min_y_mm: i32,
    pub max_y_mm: i32,
}
impl Fence {
    fn contains(&self, p: Point) -> bool {
        p.x_mm >= self.min_x_mm
            && p.x_mm <= self.max_x_mm
            && p.y_mm >= self.min_y_mm
            && p.y_mm <= self.max_y_mm
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    WaterNozzle,
    FoamNozzle,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActuationEnvelope {
    pub fence: Fence,
    pub max_dose_ml: u64,
    pub max_rate_ml_s: u64,
    pub max_pressure_kpa: u32,
    pub max_environment_bps: u16,
    pub tool: ToolKind,
}
impl ActuationEnvelope {
    pub fn new(
        fence: Fence,
        dose: u64,
        rate: u64,
        pressure: u32,
        environment: u16,
        tool: ToolKind,
    ) -> Result<Self, SuppressionError> {
        if fence.min_x_mm > fence.max_x_mm
            || fence.min_y_mm > fence.max_y_mm
            || dose == 0
            || rate == 0
            || pressure == 0
            || environment > 10_000
        {
            return Err(SuppressionError::InvalidPlan);
        }
        Ok(Self {
            fence,
            max_dose_ml: dose,
            max_rate_ml_s: rate,
            max_pressure_kpa: pressure,
            max_environment_bps: environment,
            tool,
        })
    }
}
#[derive(Debug, Clone)]
pub struct CapabilitySnapshot {
    pub id: String,
    pub promoted: bool,
    pub odd_valid: bool,
    pub digest: [u8; 32],
}
#[derive(Debug, Clone)]
pub struct AgentBatch {
    pub id: String,
    pub approved: bool,
    pub digest: [u8; 32],
}
#[derive(Debug, Clone)]
pub struct SuppressionPlan {
    id: String,
    target: Target<Approved>,
    envelope: ActuationEnvelope,
    capability: CapabilitySnapshot,
    batch: AgentBatch,
}
impl SuppressionPlan {
    pub fn new(
        id: &str,
        target: Target<Approved>,
        envelope: ActuationEnvelope,
        capability: CapabilitySnapshot,
        batch: AgentBatch,
    ) -> Result<Self, SuppressionError> {
        if id.is_empty()
            || !capability.promoted
            || !capability.odd_valid
            || capability.digest == [0; 32]
            || !batch.approved
            || batch.digest == [0; 32]
            || !envelope.fence.contains(target.point)
        {
            return Err(SuppressionError::InvalidPlan);
        }
        Ok(Self {
            id: id.into(),
            target,
            envelope,
            capability,
            batch,
        })
    }
}
#[derive(Debug, Clone)]
pub struct AuthoritySnapshot {
    pub incident_id: String,
    pub mission_id: String,
    pub fence_digest: [u8; 32],
    pub issued_tick: u64,
    pub expires_tick: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationMode {
    Simulation,
    HitlTeleoperation,
    Unsupervised,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationState {
    Prepared,
    Armed,
    Inhibited,
    InhibitUnconfirmed,
}
#[derive(Debug, Clone)]
pub struct ApprovalEvidence {
    pub principal: String,
    pub credential_id: String,
    pub incident_id: String,
    pub mission_id: String,
    pub purpose: String,
    pub issued_tick: u64,
    pub expires_tick: u64,
    pub qualification_expires_tick: u64,
    pub requester: bool,
    pub arming_digest: [u8; 32],
    pub evidence_digest: [u8; 32],
}
impl ApprovalEvidence {
    #[allow(clippy::too_many_arguments)]
    pub fn qualified(
        principal: &str,
        credential: &str,
        incident: &str,
        mission: &str,
        purpose: &str,
        issued: u64,
        expires: u64,
        qualification_expires: u64,
        requester: bool,
        arming: [u8; 32],
        evidence: [u8; 32],
    ) -> Self {
        Self {
            principal: principal.into(),
            credential_id: credential.into(),
            incident_id: incident.into(),
            mission_id: mission.into(),
            purpose: purpose.into(),
            issued_tick: issued,
            expires_tick: expires,
            qualification_expires_tick: qualification_expires,
            requester,
            arming_digest: arming,
            evidence_digest: evidence,
        }
    }
}
#[derive(Debug, Clone)]
pub struct StopAttestation {
    pub channel_id: String,
    pub config_digest: [u8; 32],
    pub proof_tick: u64,
    pub expires_tick: u64,
    pub response_bound_ms: u32,
    pub healthy: bool,
    pub readback: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InhibitReason {
    Intrusion,
    Uncertainty,
    SpatialBreach,
    EnvironmentalBreach,
    DoseBreach,
    RateBreach,
    PressureBreach,
    ToolFault,
    SensorFault,
    ActuatorFault,
    LostSupervision,
    StopRequested,
    IndependentStopFault,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Occurrence {
    pub sequence: u64,
    pub reason: InhibitReason,
    pub tick: u64,
    pub evidence_digest: [u8; 32],
}
#[derive(Debug, Clone, Copy)]
pub struct Command {
    pub target: Point,
    pub dose_ml: u64,
    pub rate_ml_s: u64,
    pub pressure_kpa: u32,
    pub tool: ToolKind,
}
#[derive(Debug, Clone, Copy)]
pub struct Measurements {
    pub position: Point,
    pub environment_bps: u16,
    pub supervision_fresh: bool,
    pub sensor_ok: bool,
    pub uncertainty_mm: u32,
}
pub trait ActuatorPort {
    fn command(&mut self, c: &Command) -> Result<(), SuppressionError>;
    fn stop(&mut self) -> Result<(), SuppressionError>;
}
pub trait IndependentInhibitPort {
    fn inhibit(&mut self) -> Result<(), SuppressionError>;
}
#[derive(Default)]
pub struct SimulatedActuator {
    pub flow_ml: u64,
    pub fail: bool,
}
impl ActuatorPort for SimulatedActuator {
    fn command(&mut self, c: &Command) -> Result<(), SuppressionError> {
        if self.fail {
            return Err(SuppressionError::EnvelopeBreach);
        }
        self.flow_ml = c.dose_ml;
        Ok(())
    }
    fn stop(&mut self) -> Result<(), SuppressionError> {
        self.flow_ml = 0;
        if self.fail {
            Err(SuppressionError::EnvelopeBreach)
        } else {
            Ok(())
        }
    }
}
#[derive(Default)]
pub struct SimulatedIndependentInhibit {
    pub stop_calls: u64,
    pub fail: bool,
}
impl IndependentInhibitPort for SimulatedIndependentInhibit {
    fn inhibit(&mut self) -> Result<(), SuppressionError> {
        self.stop_calls += 1;
        if self.fail {
            Err(SuppressionError::IndependentStopFailed)
        } else {
            Ok(())
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeasuredEffect {
    pub measured_ml: u64,
    pub uncertainty_ml: u64,
    pub residual_flow_ml: u64,
    pub evidence_digest: [u8; 32],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectRecord {
    pub commanded_ml: u64,
    pub measured: MeasuredEffect,
}
#[derive(Debug)]
pub struct Operation {
    id: String,
    plan: SuppressionPlan,
    mode: OperationMode,
    state: OperationState,
    occurrences: Vec<Occurrence>,
    commands: Vec<Command>,
    effects: Vec<EffectRecord>,
    cumulative_commanded_ml: u64,
    cumulative_measured_upper_ml: u64,
}
impl Operation {
    pub fn prepare(
        id: &str,
        plan: SuppressionPlan,
        mode: OperationMode,
    ) -> Result<Self, SuppressionError> {
        if id.is_empty() {
            return Err(SuppressionError::InvalidPlan);
        }
        if mode == OperationMode::Unsupervised {
            return Err(SuppressionError::ModeForbidden);
        }
        Ok(Self {
            id: id.into(),
            plan,
            mode,
            state: OperationState::Prepared,
            occurrences: vec![],
            commands: vec![],
            effects: vec![],
            cumulative_commanded_ml: 0,
            cumulative_measured_upper_ml: 0,
        })
    }
    pub fn arming_digest(
        &self,
        a: &AuthoritySnapshot,
        now: u64,
    ) -> Result<[u8; 32], SuppressionError> {
        if a.incident_id.is_empty()
            || a.mission_id.is_empty()
            || a.fence_digest == [0; 32]
            || a.issued_tick >= a.expires_tick
            || now < a.issued_tick
            || now >= a.expires_tick
        {
            return Err(SuppressionError::StaleAuthority);
        }
        let mut h = Sha256::new();
        for b in [
            self.id.as_bytes(),
            self.plan.id.as_bytes(),
            self.plan.target.id.as_bytes(),
            a.incident_id.as_bytes(),
            a.mission_id.as_bytes(),
        ] {
            canonical_put(&mut h, b)
        }
        for n in [
            i64::from(self.plan.target.point.x_mm),
            i64::from(self.plan.target.point.y_mm),
            i64::from(self.plan.envelope.fence.min_x_mm),
            i64::from(self.plan.envelope.fence.max_x_mm),
            i64::from(self.plan.envelope.fence.min_y_mm),
            i64::from(self.plan.envelope.fence.max_y_mm),
        ] {
            h.update(n.to_be_bytes())
        }
        h.update(self.plan.envelope.max_dose_ml.to_be_bytes());
        h.update(self.plan.envelope.max_rate_ml_s.to_be_bytes());
        h.update(self.plan.envelope.max_pressure_kpa.to_be_bytes());
        h.update(self.plan.envelope.max_environment_bps.to_be_bytes());
        h.update([self.plan.envelope.tool as u8, self.mode as u8]);
        h.update(
            self.plan
                .target
                .verification
                .as_ref()
                .ok_or(SuppressionError::InvalidTarget)?
                .digest,
        );
        h.update(
            self.plan
                .target
                .approval
                .as_ref()
                .ok_or(SuppressionError::InvalidTarget)?
                .digest,
        );
        h.update(self.plan.capability.digest);
        h.update(self.plan.batch.digest);
        h.update(a.fence_digest);
        Ok(h.finalize().into())
    }
    pub fn arm(
        &mut self,
        a: &AuthoritySnapshot,
        now: u64,
        approvals: [ApprovalEvidence; 2],
        stop: &StopAttestation,
    ) -> Result<(), SuppressionError> {
        let digest = self.arming_digest(a, now)?;
        if self.state != OperationState::Prepared
            || stop.channel_id.is_empty()
            || stop.config_digest == [0; 32]
            || !stop.healthy
            || !stop.readback
            || stop.response_bound_ms == 0
            || stop.proof_tick > now
            || now >= stop.expires_tick
            || approvals[0].principal == approvals[1].principal
            || approvals[0].credential_id == approvals[1].credential_id
            || approvals.iter().filter(|x| x.requester).count() != 1
            || approvals.iter().any(|x| {
                x.principal.is_empty()
                    || x.credential_id.is_empty()
                    || x.incident_id != a.incident_id
                    || x.mission_id != a.mission_id
                    || x.purpose != "suppression-arm"
                    || x.issued_tick > now
                    || now >= x.expires_tick
                    || now >= x.qualification_expires_tick
                    || x.arming_digest != digest
                    || x.evidence_digest == [0; 32]
            })
        {
            return Err(SuppressionError::InvalidApproval);
        }
        self.state = OperationState::Armed;
        Ok(())
    }
    pub fn inhibit(&mut self, r: InhibitReason, tick: u64) {
        self.state = OperationState::Inhibited;
        self.occurrences.push(Occurrence {
            sequence: self.occurrences.len() as u64 + 1,
            reason: r,
            tick,
            evidence_digest: [(tick.to_le_bytes()[0]).max(1); 32],
        });
    }
    pub fn apply(
        &mut self,
        c: Command,
        m: Measurements,
        tick: u64,
        act: &mut impl ActuatorPort,
        stop: &mut impl IndependentInhibitPort,
    ) -> Result<(), SuppressionError> {
        if self.state != OperationState::Armed {
            return Err(SuppressionError::InvalidTransition);
        }
        let reason = if !self.plan.envelope.fence.contains(c.target)
            || !self.plan.envelope.fence.contains(m.position)
        {
            Some(InhibitReason::SpatialBreach)
        } else if m.uncertainty_mm
            > self
                .plan
                .target
                .verification
                .as_ref()
                .map_or(0, |v| v.uncertainty_mm)
        {
            Some(InhibitReason::Uncertainty)
        } else if m.environment_bps > self.plan.envelope.max_environment_bps {
            Some(InhibitReason::EnvironmentalBreach)
        } else if self.cumulative_commanded_ml.saturating_add(c.dose_ml)
            > self.plan.envelope.max_dose_ml
        {
            Some(InhibitReason::DoseBreach)
        } else if c.rate_ml_s > self.plan.envelope.max_rate_ml_s {
            Some(InhibitReason::RateBreach)
        } else if c.pressure_kpa > self.plan.envelope.max_pressure_kpa {
            Some(InhibitReason::PressureBreach)
        } else if c.tool != self.plan.envelope.tool {
            Some(InhibitReason::ToolFault)
        } else if !m.sensor_ok {
            Some(InhibitReason::SensorFault)
        } else if !m.supervision_fresh {
            Some(InhibitReason::LostSupervision)
        } else {
            None
        };
        if let Some(r) = reason {
            self.inhibit(r, tick);
            let _ = act.stop();
            stop.inhibit()
                .map_err(|_| SuppressionError::IndependentStopFailed)?;
            return Err(SuppressionError::EnvelopeBreach);
        }
        if act.command(&c).is_err() {
            self.inhibit(InhibitReason::ActuatorFault, tick);
            let _ = stop.inhibit();
            return Err(SuppressionError::EnvelopeBreach);
        }
        self.cumulative_commanded_ml = self.cumulative_commanded_ml.saturating_add(c.dose_ml);
        self.commands.push(c);
        Ok(())
    }
    pub fn emergency_stop(
        &mut self,
        tick: u64,
        act: &mut impl ActuatorPort,
        stop: &mut impl IndependentInhibitPort,
    ) -> Result<(), SuppressionError> {
        self.inhibit(InhibitReason::StopRequested, tick);
        let normal = act.stop();
        let independent = stop.inhibit();
        if independent.is_err() {
            self.inhibit(InhibitReason::IndependentStopFault, tick);
            self.state = OperationState::InhibitUnconfirmed;
            return Err(SuppressionError::IndependentStopFailed);
        }
        normal?;
        Ok(())
    }
    pub fn record_effect(&mut self, e: MeasuredEffect) -> Result<(), SuppressionError> {
        if e.evidence_digest == [0; 32] {
            return Err(SuppressionError::InvalidTransition);
        }
        let commanded = self
            .commands
            .last()
            .ok_or(SuppressionError::InvalidTransition)?
            .dose_ml;
        self.cumulative_measured_upper_ml = self
            .cumulative_measured_upper_ml
            .saturating_add(e.measured_ml)
            .saturating_add(e.uncertainty_ml)
            .saturating_add(e.residual_flow_ml);
        if self.cumulative_measured_upper_ml > self.plan.envelope.max_dose_ml {
            self.inhibit(InhibitReason::DoseBreach, 0);
            return Err(SuppressionError::EnvelopeBreach);
        }
        self.effects.push(EffectRecord {
            commanded_ml: commanded,
            measured: e,
        });
        Ok(())
    }
    pub fn state(&self) -> OperationState {
        self.state
    }
    pub fn occurrences(&self) -> &[Occurrence] {
        &self.occurrences
    }
    pub fn effects(&self) -> &[EffectRecord] {
        &self.effects
    }
    pub fn plan(&self) -> &SuppressionPlan {
        &self.plan
    }
}
