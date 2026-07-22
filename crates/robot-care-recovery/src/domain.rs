//! Robot Care aggregates and safety invariants.
#![allow(missing_docs)]
use chrono::{DateTime, Utc};
use shared_kernel::EntityId;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum CareError {
    #[error("invalid lifecycle transition")]
    InvalidTransition,
    #[error("policy or procedure is not approved and compatible")]
    ProcedureDenied,
    #[error("recovery cannot be safely authorized")]
    UnsafeRecovery,
    #[error("asset requires fire-separated quarantine")]
    QuarantineRequired,
    #[error("hospital or quarantine capacity is unavailable")]
    CapacityUnavailable,
    #[error("repair evidence or serialized provenance is incomplete")]
    IncompleteRepair,
    #[error("return-to-service evidence is incomplete")]
    RecertificationDenied,
    #[error("retirement evidence is incomplete")]
    RetirementDenied,
    #[error("aggregate version exhausted")]
    VersionExhausted,
}
fn bump(version: &mut u64) -> Result<(), CareError> {
    *version = version.checked_add(1).ok_or(CareError::VersionExhausted)?;
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyState {
    Draft,
    Validated,
    Approved,
    Effective,
    Superseded,
}
#[derive(Clone, Debug)]
pub struct ServicePolicy {
    pub id: EntityId,
    pub state: PolicyState,
    pub configuration_digest: [u8; 32],
    pub critical_interval_hours: u64,
    pub procedures: BTreeMap<String, BTreeSet<String>>,
    pub version: u64,
}
impl ServicePolicy {
    pub fn define(
        id: EntityId,
        digest: [u8; 32],
        interval: u64,
        procedures: BTreeMap<String, BTreeSet<String>>,
    ) -> Result<Self, CareError> {
        if digest == [0; 32] || interval == 0 || procedures.is_empty() {
            return Err(CareError::ProcedureDenied);
        }
        Ok(Self {
            id,
            state: PolicyState::Draft,
            configuration_digest: digest,
            critical_interval_hours: interval,
            procedures,
            version: 1,
        })
    }
    pub fn transition(&mut self, from: PolicyState, to: PolicyState) -> Result<(), CareError> {
        if self.state != from {
            return Err(CareError::InvalidTransition);
        }
        self.state = to;
        bump(&mut self.version)
    }
    #[must_use]
    pub fn permits(&self, module: &str, procedure: &str, tool: &str) -> bool {
        self.state == PolicyState::Effective
            && self
                .procedures
                .get(module)
                .is_some_and(|values| values.contains(&format!("{procedure}:{tool}")))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanState {
    Proposed,
    Scheduled,
    Active,
    Complete,
    Overdue,
    Suspended,
}
#[derive(Clone, Debug)]
pub struct MaintenancePlan {
    pub id: EntityId,
    pub asset_id: EntityId,
    pub state: PlanState,
    pub due_at: DateTime<Utc>,
    pub policy_digest: [u8; 32],
    pub duty_hours: u64,
    pub exposure_score: u32,
}
impl MaintenancePlan {
    pub fn propose(
        id: EntityId,
        asset: EntityId,
        due: DateTime<Utc>,
        policy: [u8; 32],
        duty: u64,
        exposure: u32,
    ) -> Result<Self, CareError> {
        if policy == [0; 32] || exposure > 10_000 {
            return Err(CareError::ProcedureDenied);
        }
        Ok(Self {
            id,
            asset_id: asset,
            state: PlanState::Proposed,
            due_at: due,
            policy_digest: policy,
            duty_hours: duty,
            exposure_score: exposure,
        })
    }
    pub fn evaluate_due(&mut self, now: DateTime<Utc>) -> bool {
        if now >= self.due_at && !matches!(self.state, PlanState::Complete | PlanState::Suspended) {
            self.state = PlanState::Overdue;
            true
        } else {
            false
        }
    }
    #[must_use]
    pub fn eligible(&self) -> bool {
        self.state != PlanState::Overdue
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkState {
    Reported,
    Triaged,
    Assigned,
    Servicing,
    Testing,
    Closed,
    Rework,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledPart {
    pub serial: String,
    pub provenance_digest: [u8; 32],
    pub module: String,
    pub cannibalized: bool,
}
#[derive(Clone, Debug)]
pub struct WorkOrder {
    pub id: EntityId,
    pub asset_id: EntityId,
    pub state: WorkState,
    pub procedure: String,
    pub module: String,
    pub tool: String,
    pub isolated: bool,
    pub parts: Vec<InstalledPart>,
    pub measurements: Vec<String>,
}
impl WorkOrder {
    pub fn open(
        id: EntityId,
        asset: EntityId,
        module: impl Into<String>,
        procedure: impl Into<String>,
        tool: impl Into<String>,
    ) -> Self {
        Self {
            id,
            asset_id: asset,
            state: WorkState::Reported,
            procedure: procedure.into(),
            module: module.into(),
            tool: tool.into(),
            isolated: false,
            parts: Vec::new(),
            measurements: Vec::new(),
        }
    }
    pub fn start(&mut self, policy: &ServicePolicy, isolation: bool) -> Result<(), CareError> {
        if !matches!(self.state, WorkState::Reported | WorkState::Assigned)
            || !isolation
            || !policy.permits(&self.module, &self.procedure, &self.tool)
        {
            return Err(CareError::ProcedureDenied);
        }
        self.isolated = true;
        self.state = WorkState::Servicing;
        Ok(())
    }
    pub fn install(&mut self, part: InstalledPart) -> Result<(), CareError> {
        if self.state != WorkState::Servicing
            || part.serial.trim().is_empty()
            || part.provenance_digest == [0; 32]
            || part.module != self.module
            || self.parts.iter().any(|v| v.serial == part.serial)
        {
            return Err(CareError::IncompleteRepair);
        }
        self.parts.push(part);
        Ok(())
    }
    pub fn test(&mut self, measurement: impl Into<String>) -> Result<(), CareError> {
        let value = measurement.into();
        if self.state != WorkState::Servicing || value.trim().is_empty() {
            return Err(CareError::IncompleteRepair);
        }
        self.measurements.push(value);
        self.state = WorkState::Testing;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SafetyFact {
    Passed,
    Failed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HazardState {
    KnownSafe,
    HeatExposed,
    SwollenOrLeaking,
    ElectricalUnsafe,
    Contaminated,
    StructurallyUnstable,
    Unknown,
}
impl HazardState {
    #[must_use]
    pub fn requires_quarantine(self) -> bool {
        self != Self::KnownSafe
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryAssessment {
    pub scene_odd: SafetyFact,
    pub route_odd: SafetyFact,
    pub communications: SafetyFact,
    pub medic_capable: SafetyFact,
    pub mass_compatible: SafetyFact,
    pub lift_tow_cradle_compatible: SafetyFact,
    pub energy_isolated: SafetyFact,
    pub tools_stabilized: SafetyFact,
    pub destination_reserved: SafetyFact,
    pub fallback_ready: SafetyFact,
    pub human_rescue_clear: SafetyFact,
    pub exclusion_clear: SafetyFact,
    pub hazard: HazardState,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryState {
    Requested,
    Assessed,
    Authorized,
    EnRoute,
    Stabilizing,
    Recovering,
    Transferred,
    Aborted,
}
#[derive(Clone, Debug)]
pub struct RecoveryMission {
    pub id: EntityId,
    pub asset_id: EntityId,
    pub state: RecoveryState,
    pub quarantine_transport: bool,
    pub custody_destination: String,
}
impl RecoveryMission {
    pub fn request(
        id: EntityId,
        asset: EntityId,
        destination: impl Into<String>,
    ) -> Result<Self, CareError> {
        let destination = destination.into();
        if destination.trim().is_empty() {
            return Err(CareError::UnsafeRecovery);
        }
        Ok(Self {
            id,
            asset_id: asset,
            state: RecoveryState::Requested,
            quarantine_transport: false,
            custody_destination: destination,
        })
    }
    pub fn authorize(&mut self, a: &RecoveryAssessment) -> Result<(), CareError> {
        let safe = [
            a.scene_odd,
            a.route_odd,
            a.communications,
            a.medic_capable,
            a.mass_compatible,
            a.lift_tow_cradle_compatible,
            a.energy_isolated,
            a.tools_stabilized,
            a.destination_reserved,
            a.fallback_ready,
            a.human_rescue_clear,
            a.exclusion_clear,
        ]
        .into_iter()
        .all(|v| v == SafetyFact::Passed);
        if !safe {
            return Err(CareError::UnsafeRecovery);
        }
        self.quarantine_transport = a.hazard.requires_quarantine();
        self.state = RecoveryState::Authorized;
        Ok(())
    }
    pub fn abort(&mut self) {
        self.state = RecoveryState::Aborted;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssessmentState {
    Collecting,
    Classified,
    Reviewed,
    Superseded,
}
#[derive(Clone, Debug)]
pub struct DamageAssessment {
    pub id: EntityId,
    pub asset_id: EntityId,
    pub state: AssessmentState,
    pub hazard: Option<HazardState>,
    pub evidence: BTreeSet<[u8; 32]>,
}
impl DamageAssessment {
    #[must_use]
    pub fn collect(id: EntityId, asset: EntityId) -> Self {
        Self {
            id,
            asset_id: asset,
            state: AssessmentState::Collecting,
            hazard: None,
            evidence: BTreeSet::new(),
        }
    }
    pub fn classify(&mut self, hazard: HazardState, evidence: [u8; 32]) -> Result<(), CareError> {
        if self.state != AssessmentState::Collecting || evidence == [0; 32] {
            return Err(CareError::InvalidTransition);
        }
        self.evidence.insert(evidence);
        self.hazard = Some(hazard);
        self.state = AssessmentState::Classified;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuarantineState {
    Open,
    Isolated,
    Monitoring,
    Cleared,
    Escalated,
    Disposed,
}
#[derive(Clone, Debug)]
pub struct QuarantineCase {
    pub id: EntityId,
    pub asset_id: EntityId,
    pub state: QuarantineState,
    pub zone: String,
    pub fire_separated: bool,
    pub clearance_evidence: BTreeSet<[u8; 32]>,
}
impl QuarantineCase {
    pub fn open(
        id: EntityId,
        asset: EntityId,
        zone: impl Into<String>,
        fire_separated: bool,
    ) -> Result<Self, CareError> {
        let zone = zone.into();
        if zone.trim().is_empty() || !fire_separated {
            return Err(CareError::CapacityUnavailable);
        }
        Ok(Self {
            id,
            asset_id: asset,
            state: QuarantineState::Open,
            zone,
            fire_separated,
            clearance_evidence: BTreeSet::new(),
        })
    }
    pub fn isolate(&mut self) -> Result<(), CareError> {
        if self.state != QuarantineState::Open {
            return Err(CareError::InvalidTransition);
        }
        self.state = QuarantineState::Isolated;
        Ok(())
    }
    pub fn clear(
        &mut self,
        evidence: impl IntoIterator<Item = [u8; 32]>,
        independent_review: bool,
    ) -> Result<(), CareError> {
        let evidence = evidence.into_iter().collect::<BTreeSet<_>>();
        if !matches!(
            self.state,
            QuarantineState::Isolated | QuarantineState::Monitoring
        ) || !independent_review
            || evidence.len() < 2
            || evidence.contains(&[0; 32])
        {
            return Err(CareError::QuarantineRequired);
        }
        self.clearance_evidence = evidence;
        self.state = QuarantineState::Cleared;
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct HospitalCapacity {
    zones: BTreeMap<String, u32>,
    occupied: BTreeMap<String, u32>,
}
impl HospitalCapacity {
    pub fn configure(&mut self, zone: impl Into<String>, capacity: u32) {
        self.zones.insert(zone.into(), capacity);
    }
    pub fn admit(&mut self, zone: &str) -> Result<(), CareError> {
        let cap = *self.zones.get(zone).unwrap_or(&0);
        let used = *self.occupied.get(zone).unwrap_or(&0);
        if used >= cap {
            return Err(CareError::CapacityUnavailable);
        }
        self.occupied.insert(zone.into(), used + 1);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepairState {
    Admitted,
    Diagnosing,
    Repairing,
    Calibrating,
    BurnIn,
    Recertified,
    Failed,
}
#[derive(Clone, Debug)]
pub struct RepairCase {
    pub id: EntityId,
    pub asset_id: EntityId,
    pub state: RepairState,
    pub parts: Vec<InstalledPart>,
    pub calibration_digest: Option<[u8; 32]>,
    pub burn_in_digest: Option<[u8; 32]>,
    pub fresh_health_digest: Option<[u8; 32]>,
    pub independent_reviewer: Option<EntityId>,
}
impl RepairCase {
    #[must_use]
    pub fn admit(id: EntityId, asset: EntityId) -> Self {
        Self {
            id,
            asset_id: asset,
            state: RepairState::Admitted,
            parts: Vec::new(),
            calibration_digest: None,
            burn_in_digest: None,
            fresh_health_digest: None,
            independent_reviewer: None,
        }
    }
    pub fn begin_repair(&mut self) -> Result<(), CareError> {
        if self.state != RepairState::Admitted {
            return Err(CareError::InvalidTransition);
        }
        self.state = RepairState::Repairing;
        Ok(())
    }
    pub fn install(&mut self, part: InstalledPart) -> Result<(), CareError> {
        if self.state != RepairState::Repairing
            || part.serial.trim().is_empty()
            || part.provenance_digest == [0; 32]
            || self.parts.iter().any(|v| v.serial == part.serial)
        {
            return Err(CareError::IncompleteRepair);
        }
        self.parts.push(part);
        Ok(())
    }
    pub fn calibrate(&mut self, digest: [u8; 32]) -> Result<(), CareError> {
        if self.state != RepairState::Repairing || digest == [0; 32] {
            return Err(CareError::IncompleteRepair);
        }
        self.calibration_digest = Some(digest);
        self.state = RepairState::Calibrating;
        Ok(())
    }
    pub fn burn_in(&mut self, digest: [u8; 32]) -> Result<(), CareError> {
        if self.state != RepairState::Calibrating || digest == [0; 32] {
            return Err(CareError::IncompleteRepair);
        }
        self.burn_in_digest = Some(digest);
        self.state = RepairState::BurnIn;
        Ok(())
    }
    pub fn recertify(
        &mut self,
        health: [u8; 32],
        reviewer: EntityId,
        configuration_promoted: bool,
        quarantine_cleared: bool,
    ) -> Result<(), CareError> {
        if self.state != RepairState::BurnIn
            || self.calibration_digest.is_none()
            || self.burn_in_digest.is_none()
            || health == [0; 32]
            || !configuration_promoted
            || !quarantine_cleared
        {
            return Err(CareError::RecertificationDenied);
        }
        self.fresh_health_digest = Some(health);
        self.independent_reviewer = Some(reviewer);
        self.state = RepairState::Recertified;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetirementState {
    Proposed,
    Approved,
    Depowered,
    Sanitized,
    Salvaged,
    Recycled,
    Closed,
}
#[derive(Clone, Debug)]
pub struct RetirementCase {
    pub id: EntityId,
    pub asset_id: EntityId,
    pub state: RetirementState,
    pub identity_revoked: bool,
    pub data_sanitized: bool,
    pub hazard_disposition: Option<String>,
    pub salvage_custody_digest: Option<[u8; 32]>,
}
impl RetirementCase {
    #[must_use]
    pub fn propose(id: EntityId, asset: EntityId) -> Self {
        Self {
            id,
            asset_id: asset,
            state: RetirementState::Proposed,
            identity_revoked: false,
            data_sanitized: false,
            hazard_disposition: None,
            salvage_custody_digest: None,
        }
    }
    pub fn approve(&mut self) -> Result<(), CareError> {
        if self.state != RetirementState::Proposed {
            return Err(CareError::InvalidTransition);
        }
        self.state = RetirementState::Approved;
        Ok(())
    }
    pub fn depower(&mut self, neutralized: bool) -> Result<(), CareError> {
        if self.state != RetirementState::Approved || !neutralized {
            return Err(CareError::RetirementDenied);
        }
        self.state = RetirementState::Depowered;
        Ok(())
    }
    pub fn sanitize(
        &mut self,
        identity_revoked: bool,
        data_sanitized: bool,
    ) -> Result<(), CareError> {
        if self.state != RetirementState::Depowered || !identity_revoked || !data_sanitized {
            return Err(CareError::RetirementDenied);
        }
        self.identity_revoked = true;
        self.data_sanitized = true;
        self.state = RetirementState::Sanitized;
        Ok(())
    }
    pub fn salvage(
        &mut self,
        disposition: impl Into<String>,
        custody: [u8; 32],
    ) -> Result<(), CareError> {
        let disposition = disposition.into();
        if self.state != RetirementState::Sanitized
            || disposition.trim().is_empty()
            || custody == [0; 32]
        {
            return Err(CareError::RetirementDenied);
        }
        self.hazard_disposition = Some(disposition);
        self.salvage_custody_digest = Some(custody);
        self.state = RetirementState::Salvaged;
        Ok(())
    }
    pub fn close(&mut self) -> Result<(), CareError> {
        if self.state != RetirementState::Salvaged
            || !self.identity_revoked
            || !self.data_sanitized
            || self.hazard_disposition.is_none()
            || self.salvage_custody_digest.is_none()
        {
            return Err(CareError::RetirementDenied);
        }
        self.state = RetirementState::Closed;
        Ok(())
    }
}
