#![forbid(unsafe_code)]
#![allow(missing_docs, clippy::must_use_candidate)]
use std::collections::BTreeSet;
pub const CONTEXT_NAME: &str = "vegetation-management";
fn resource_digest(value: u64, domain: u8) -> [u8; 32] {
    let mut out = [domain; 32];
    out[..8].copy_from_slice(&value.to_be_bytes());
    out
}
fn polygon_area(p: &Polygon) -> u64 {
    p.vertices
        .iter()
        .zip(p.vertices.iter().cycle().skip(1))
        .take(p.vertices.len())
        .map(|(a, b)| i64::from(a.x_mm) * i64::from(b.y_mm) - i64::from(b.x_mm) * i64::from(a.y_mm))
        .sum::<i64>()
        .unsigned_abs()
        / 2
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum VegetationError {
    #[error("land authority missing")]
    MissingAuthority,
    #[error("stale revision")]
    StaleRevision,
    #[error("invalid geometry or prescription")]
    InvalidPlan,
    #[error("invalid transition")]
    InvalidTransition,
    #[error("independent release required")]
    IndependentReleaseRequired,
    #[error("biomass capacity insufficient")]
    InsufficientBiomassCapacity,
    #[error("robot unserviceable")]
    RobotUnserviceable,
    #[error("simulation unavailable")]
    SimulationUnavailable,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x_mm: i32,
    pub y_mm: i32,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Polygon {
    pub vertices: Vec<Point>,
}
impl Polygon {
    pub fn new(vertices: Vec<Point>) -> Result<Self, VegetationError> {
        if vertices.len() < 3 {
            return Err(VegetationError::InvalidPlan);
        }
        let area = vertices
            .iter()
            .zip(vertices.iter().cycle().skip(1))
            .take(vertices.len())
            .map(|(a, b)| {
                i64::from(a.x_mm) * i64::from(b.y_mm) - i64::from(b.x_mm) * i64::from(a.y_mm)
            })
            .sum::<i64>()
            .abs();
        if area == 0 {
            return Err(VegetationError::InvalidPlan);
        }
        Ok(Self { vertices })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExclusionKind {
    Ecological,
    Cultural,
    Utility,
    Wildlife,
}
#[derive(Debug, Clone)]
pub struct Exclusion {
    pub kind: ExclusionKind,
    pub area: Polygon,
}
#[derive(Debug, Clone)]
pub struct TreatmentProgram {
    pub id: String,
    pub land_authority_id: String,
    pub land_revision: u64,
}
impl TreatmentProgram {
    pub fn authorize(id: &str, authority: &str, revision: u64) -> Result<Self, VegetationError> {
        if id.is_empty() || authority.is_empty() {
            return Err(VegetationError::MissingAuthority);
        }
        if revision == 0 {
            return Err(VegetationError::StaleRevision);
        }
        Ok(Self {
            id: id.into(),
            land_authority_id: authority.into(),
            land_revision: revision,
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Mechanical,
    Grazing,
    Manual,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    Mulcher,
    Mower,
    HandTool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationalEnvelope {
    pub max_fire_danger_bps: u16,
    pub min_localization_quality_bps: u16,
}
#[derive(Debug, Clone)]
pub struct Prescription {
    pub id: String,
    pub method: Method,
    pub tool: ToolKind,
    pub residual_fuel_target_bps: u16,
    pub envelope: OperationalEnvelope,
    pub authority_id: String,
    pub geometry_revision: u64,
}
impl Prescription {
    pub fn new(
        id: &str,
        method: Method,
        tool: ToolKind,
        target: u16,
        envelope: OperationalEnvelope,
        authority: &str,
        revision: u64,
    ) -> Result<Self, VegetationError> {
        if id.is_empty()
            || authority.is_empty()
            || target > 10_000
            || envelope.max_fire_danger_bps > 10_000
            || envelope.min_localization_quality_bps > 10_000
            || revision == 0
        {
            return Err(VegetationError::InvalidPlan);
        }
        Ok(Self {
            id: id.into(),
            method,
            tool,
            residual_fuel_target_bps: target,
            envelope,
            authority_id: authority.into(),
            geometry_revision: revision,
        })
    }
}
#[derive(Debug, Clone)]
pub struct TreatmentUnit {
    pub id: String,
    pub geometry: Polygon,
    pub geometry_revision: u64,
    pub survey_revision: String,
    pub fuel_revision: String,
    pub exclusions: Vec<Exclusion>,
}
impl TreatmentUnit {
    pub fn new(
        id: &str,
        geometry: Polygon,
        revision: u64,
        survey: &str,
        fuel: &str,
        exclusions: Vec<Exclusion>,
    ) -> Result<Self, VegetationError> {
        if id.is_empty() || revision == 0 || survey.is_empty() || fuel.is_empty() {
            return Err(VegetationError::InvalidPlan);
        }
        Ok(Self {
            id: id.into(),
            geometry,
            geometry_revision: revision,
            survey_revision: survey.into(),
            fuel_revision: fuel.into(),
            exclusions,
        })
    }
}
#[derive(Debug, Clone)]
pub struct MissionSnapshot {
    pub authority_id: String,
    pub lease_id: String,
    pub fence_digest: [u8; 32],
    pub geometry_revision: u64,
    pub issued_tick: u64,
    pub expires_tick: u64,
}
pub struct LogisticsSnapshot {
    pub capacity_kg: u64,
    pub custody_ready: bool,
}
pub struct CareSnapshot {
    pub robot_serviceable: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolState {
    Disarmed,
    Armed,
    Inhibited,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InhibitReason {
    Exclusion,
    Person,
    Wildlife,
    Utility,
    FireDanger,
    Localization,
    Tool,
    Communications,
}
#[derive(Debug, Clone, Copy)]
pub enum SafetyObservation {
    InsideExclusion,
    PersonDetected,
    WildlifeDetected,
    UtilityDetected,
    FireDanger { bps: u16 },
    LocalizationQuality { bps: u16 },
    ToolFault,
    CommunicationsLost,
    Clear,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafetyEvidence {
    pub sequence: u64,
    pub reason: InhibitReason,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionEvidence {
    pub planned_biomass_kg: u64,
    pub actual_biomass_kg: u64,
    pub geometry_revision: u64,
    pub envelope: OperationalEnvelope,
    pub trace_digest: [u8; 32],
    pub custody_digest: [u8; 32],
    pub missed_area_mm2: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkState {
    Planned,
    Authorized,
    Running,
    Complete,
}
pub struct WorkPackage {
    pub id: String,
    pub unit: TreatmentUnit,
    prescription: Prescription,
    planned_biomass_kg: u64,
    state: WorkState,
    tool: ToolState,
    inhibits: BTreeSet<InhibitReason>,
    evidence: Vec<SafetyEvidence>,
    completion: Option<ExecutionEvidence>,
    resources: Option<ResourceAdmission>,
    used_resets: BTreeSet<String>,
}
#[derive(Debug, Clone)]
struct ResourceAdmission {
    capacity_kg: u64,
    custody_digest: [u8; 32],
    care_digest: [u8; 32],
}
#[derive(Debug, Clone)]
pub struct ResetAuthorization {
    pub id: String,
    pub work_package_id: String,
    pub actor: String,
    pub inspection_digest: [u8; 32],
    pub authority_id: String,
    pub fence_digest: [u8; 32],
    pub issued_tick: u64,
    pub expires_tick: u64,
    pub cleared_causes: BTreeSet<InhibitReason>,
}
impl WorkPackage {
    pub fn plan(
        id: &str,
        unit: TreatmentUnit,
        prescription: Prescription,
        planned: u64,
    ) -> Result<Self, VegetationError> {
        if id.is_empty() || planned == 0 || unit.geometry_revision != prescription.geometry_revision
        {
            return Err(VegetationError::StaleRevision);
        }
        Ok(Self {
            id: id.into(),
            unit,
            prescription,
            planned_biomass_kg: planned,
            state: WorkState::Planned,
            tool: ToolState::Disarmed,
            inhibits: BTreeSet::new(),
            evidence: vec![],
            completion: None,
            resources: None,
            used_resets: BTreeSet::new(),
        })
    }
    pub fn authorize(&mut self, m: &MissionSnapshot, now: u64) -> Result<(), VegetationError> {
        if self.state != WorkState::Planned
            || m.authority_id != self.prescription.authority_id
            || m.lease_id.is_empty()
            || m.fence_digest == [0; 32]
            || m.issued_tick >= m.expires_tick
            || now < m.issued_tick
            || now >= m.expires_tick
        {
            return Err(VegetationError::MissingAuthority);
        }
        if m.geometry_revision != self.unit.geometry_revision {
            return Err(VegetationError::StaleRevision);
        }
        self.state = WorkState::Authorized;
        Ok(())
    }
    pub fn admit_resources(
        &mut self,
        l: &LogisticsSnapshot,
        c: &CareSnapshot,
    ) -> Result<(), VegetationError> {
        if !c.robot_serviceable {
            return Err(VegetationError::RobotUnserviceable);
        }
        if !l.custody_ready || l.capacity_kg < self.planned_biomass_kg {
            return Err(VegetationError::InsufficientBiomassCapacity);
        }
        self.resources = Some(ResourceAdmission {
            capacity_kg: l.capacity_kg,
            custody_digest: resource_digest(l.capacity_kg, 1),
            care_digest: resource_digest(u64::from(c.robot_serviceable), 2),
        });
        Ok(())
    }
    pub fn start(&mut self) -> Result<(), VegetationError> {
        if self.state != WorkState::Authorized || self.resources.is_none() {
            return Err(VegetationError::InvalidTransition);
        }
        self.state = WorkState::Running;
        self.tool = ToolState::Armed;
        Ok(())
    }
    pub fn observe(&mut self, o: SafetyObservation) {
        let reason = match o {
            SafetyObservation::InsideExclusion => Some(InhibitReason::Exclusion),
            SafetyObservation::PersonDetected => Some(InhibitReason::Person),
            SafetyObservation::WildlifeDetected => Some(InhibitReason::Wildlife),
            SafetyObservation::UtilityDetected => Some(InhibitReason::Utility),
            SafetyObservation::FireDanger { bps }
                if bps > self.prescription.envelope.max_fire_danger_bps =>
            {
                Some(InhibitReason::FireDanger)
            }
            SafetyObservation::LocalizationQuality { bps }
                if bps < self.prescription.envelope.min_localization_quality_bps =>
            {
                Some(InhibitReason::Localization)
            }
            SafetyObservation::ToolFault => Some(InhibitReason::Tool),
            SafetyObservation::CommunicationsLost => Some(InhibitReason::Communications),
            _ => None,
        };
        if let Some(r) = reason {
            self.inhibits.insert(r);
            self.tool = ToolState::Inhibited;
            self.evidence.push(SafetyEvidence {
                sequence: self.evidence.len() as u64 + 1,
                reason: r,
            });
        }
    }
    pub fn rearm(
        &mut self,
        r: ResetAuthorization,
        m: &MissionSnapshot,
        now: u64,
    ) -> Result<(), VegetationError> {
        if r.id.is_empty()
            || r.work_package_id != self.id
            || r.actor.is_empty()
            || r.inspection_digest == [0; 32]
            || r.authority_id != m.authority_id
            || r.fence_digest != m.fence_digest
            || r.issued_tick >= r.expires_tick
            || now < r.issued_tick
            || now >= r.expires_tick
            || now >= m.expires_tick
            || self.used_resets.contains(&r.id)
            || !self.inhibits.is_subset(&r.cleared_causes)
        {
            return Err(VegetationError::IndependentReleaseRequired);
        }
        if self.tool != ToolState::Inhibited {
            return Err(VegetationError::InvalidTransition);
        }
        self.inhibits.clear();
        self.used_resets.insert(r.id);
        self.tool = ToolState::Armed;
        Ok(())
    }
    pub fn tool_state(&self) -> ToolState {
        self.tool
    }
    pub fn evidence(&self) -> &[SafetyEvidence] {
        &self.evidence
    }
    pub fn prescription(&self) -> &Prescription {
        &self.prescription
    }
    pub fn record_completion(&mut self, e: ExecutionEvidence) -> Result<(), VegetationError> {
        let resources = self
            .resources
            .as_ref()
            .ok_or(VegetationError::InvalidTransition)?;
        if self.state != WorkState::Running
            || self.tool == ToolState::Armed
            || e.geometry_revision != self.unit.geometry_revision
            || e.envelope != self.prescription.envelope
            || e.trace_digest == [0; 32]
            || e.actual_biomass_kg > e.planned_biomass_kg
            || e.actual_biomass_kg == 0
            || e.planned_biomass_kg != self.planned_biomass_kg
            || e.custody_digest != resources.custody_digest
            || resources.care_digest == [0; 32]
            || resources.capacity_kg < e.actual_biomass_kg
        {
            return Err(VegetationError::InvalidPlan);
        }
        self.completion = Some(e);
        self.state = WorkState::Complete;
        self.tool = ToolState::Disarmed;
        Ok(())
    }
    pub fn end_execution(&mut self) -> Result<(), VegetationError> {
        if self.state != WorkState::Running {
            return Err(VegetationError::InvalidTransition);
        }
        self.tool = ToolState::Disarmed;
        Ok(())
    }
    pub fn is_running_armed(&self) -> bool {
        self.state == WorkState::Running && self.tool == ToolState::Armed
    }
    pub fn custody_digest(&self) -> Option<[u8; 32]> {
        self.resources.as_ref().map(|r| r.custody_digest)
    }
    pub fn is_complete(&self) -> bool {
        self.state == WorkState::Complete
    }
    pub fn effectiveness(&self) -> Option<&EffectivenessAssessment> {
        None
    }
}
pub trait GroundRobotPort {
    fn execute(&self, work: &WorkPackage, seed: u64) -> Result<ExecutionEvidence, VegetationError>;
}
pub struct DeterministicGroundRobot;
impl GroundRobotPort for DeterministicGroundRobot {
    fn execute(&self, w: &WorkPackage, seed: u64) -> Result<ExecutionEvidence, VegetationError> {
        if !w.is_running_armed() {
            return Err(VegetationError::SimulationUnavailable);
        }
        Ok(ExecutionEvidence {
            planned_biomass_kg: w.planned_biomass_kg,
            actual_biomass_kg: w.planned_biomass_kg.saturating_sub(seed % 101),
            geometry_revision: w.unit.geometry_revision,
            envelope: w.prescription.envelope,
            trace_digest: [seed.to_le_bytes()[0].max(1); 32],
            custody_digest: w
                .custody_digest()
                .ok_or(VegetationError::SimulationUnavailable)?,
            missed_area_mm2: w
                .unit
                .exclusions
                .iter()
                .map(|e| polygon_area(&e.area))
                .sum(),
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportAuthority {
    AdvisoryOnly,
}
pub struct SurveyReport {
    pub authority: SupportAuthority,
    pub geometry_revision: u64,
    pub evidence_digest: [u8; 32],
}
pub struct RelayStatus {
    pub available: bool,
}
pub trait DroneSupportPort {
    fn survey(&self, unit: &TreatmentUnit, seed: u64) -> Result<SurveyReport, VegetationError>;
    fn relay(&self, quality_bps: u16) -> RelayStatus;
}
pub struct DeterministicDroneSupport;
impl DroneSupportPort for DeterministicDroneSupport {
    fn survey(&self, u: &TreatmentUnit, seed: u64) -> Result<SurveyReport, VegetationError> {
        Ok(SurveyReport {
            authority: SupportAuthority::AdvisoryOnly,
            geometry_revision: u.geometry_revision,
            evidence_digest: [seed.to_le_bytes()[0].max(1); 32],
        })
    }
    fn relay(&self, q: u16) -> RelayStatus {
        RelayStatus {
            available: q >= 5000,
        }
    }
}
pub struct LongitudinalObservation {
    pub days_after: u32,
    pub residual_fuel_bps: u16,
    pub evidence_digest: [u8; 32],
}
pub struct EffectivenessAssessment {
    pub id: String,
    observations: Vec<LongitudinalObservation>,
    target: u16,
}
impl EffectivenessAssessment {
    pub fn assess(
        id: &str,
        w: &WorkPackage,
        mut observations: Vec<LongitudinalObservation>,
    ) -> Result<Self, VegetationError> {
        if !w.is_complete()
            || id.is_empty()
            || observations.is_empty()
            || observations.iter().any(|o| {
                o.days_after == 0 || o.residual_fuel_bps > 10_000 || o.evidence_digest == [0; 32]
            })
        {
            return Err(VegetationError::InvalidPlan);
        }
        observations.sort_by_key(|o| o.days_after);
        Ok(Self {
            id: id.into(),
            observations,
            target: w.prescription.residual_fuel_target_bps,
        })
    }
    pub fn target_met(&self) -> bool {
        self.observations
            .last()
            .is_some_and(|o| o.residual_fuel_bps <= self.target)
    }
}
