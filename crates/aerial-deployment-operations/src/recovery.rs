//! Serialized recovery accounting and the Logistics/Robot Care anti-corruption boundary.
use crate::{ComponentId, DomainError, HandoffRequestId, ObservationId, SearchId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SerializedKind {
    Robot,
    Panel,
    Joint,
    Parafoil,
    Tether,
    Reel,
    Anchor,
    CradleSection,
    ChemicalPayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedItem {
    component: ComponentId,
    pub kind: SerializedKind,
    serial: String,
}
impl SerializedItem {
    pub fn new(
        component: ComponentId,
        kind: SerializedKind,
        serial: &str,
    ) -> Result<Self, DomainError> {
        let serial = serial.trim();
        if serial.is_empty() || serial.len() > 128 {
            return Err(DomainError::InvalidComponent);
        }
        Ok(Self {
            component,
            kind,
            serial: serial.into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Custodian {
    Aircraft,
    RecoveryTeam(String),
    Logistics,
    RobotCare,
    IncidentCommand,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocationFix {
    pub reference: String,
    pub confidence_basis_points: u16,
}
impl LocationFix {
    pub fn new(reference: &str, confidence_basis_points: u16) -> Result<Self, DomainError> {
        if reference.trim().is_empty() || confidence_basis_points > 10_000 {
            return Err(DomainError::InvalidRecoveryRecord);
        }
        Ok(Self {
            reference: reference.into(),
            confidence_basis_points,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationStatus {
    Known(LocationFix),
    Unknown,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExposureRecord {
    pub maximum_temperature_millicelsius: Option<i64>,
    pub smoke_dose_milligram_minutes_m3: Option<u64>,
    pub chemical_dose_milligram_minutes_m3: Option<u64>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageAssessment {
    Unknown,
    None,
    Suspected(Vec<String>),
    Confirmed(Vec<String>),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContaminationStatus {
    Unknown,
    Clear,
    Suspected { agents: Vec<String> },
    Confirmed { agents: Vec<String> },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnergizedHazard {
    Electrical { volts: u32 },
    MechanicalTension,
    Pressure,
    Thermal,
    Chemical(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryObservation {
    pub observation_id: ObservationId,
    pub component: ComponentId,
    pub observed_at: DateTime<Utc>,
    pub custody: Custodian,
    pub location: LocationStatus,
    pub exposure: ExposureRecord,
    pub damage: DamageAssessment,
    pub contamination: ContaminationStatus,
    pub energized_hazards: Vec<EnergizedHazard>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustodyState {
    Unaccounted,
    Held(Custodian),
    Disputed { claims: Vec<Custodian> },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryDisposition {
    Pending,
    Recovered,
    Quarantined,
    Decontamination,
    Inspection,
    Repair,
    Calibration,
    BurnIn,
    Reuse,
    Recycling,
    Retirement,
    TemporarilyAbandoned,
    AcceptedUnlocated,
    Sacrificed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordOutcome {
    Applied,
    Duplicate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchRecord {
    pub id: SearchId,
    pub component: ComponentId,
    pub evidence: String,
    pub searched_at: DateTime<Utc>,
}
impl SearchRecord {
    pub fn new(
        id: SearchId,
        component: ComponentId,
        evidence: &str,
        searched_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if evidence.trim().is_empty() {
            return Err(DomainError::InvalidRecoveryRecord);
        }
        Ok(Self {
            id,
            component,
            evidence: evidence.into(),
            searched_at,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispositionAuthority {
    pub actor: String,
    pub authorization_reference: String,
}
impl DispositionAuthority {
    pub fn new(actor: &str, authorization_reference: &str) -> Result<Self, DomainError> {
        if actor.trim().is_empty() || authorization_reference.trim().is_empty() {
            return Err(DomainError::InvalidRecoveryRecord);
        }
        Ok(Self {
            actor: actor.into(),
            authorization_reference: authorization_reference.into(),
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HazardNotice {
    pub id: String,
    pub message: String,
    pub issued_at: DateTime<Utc>,
    pub cleared_at: Option<DateTime<Utc>>,
}
impl HazardNotice {
    pub fn new(
        id: &str,
        message: &str,
        issued_at: DateTime<Utc>,
        cleared_at: Option<DateTime<Utc>>,
    ) -> Result<Self, DomainError> {
        if id.trim().is_empty()
            || message.trim().is_empty()
            || cleared_at.is_some_and(|at| at < issued_at)
        {
            return Err(DomainError::InvalidRecoveryRecord);
        }
        Ok(Self {
            id: id.into(),
            message: message.into(),
            issued_at,
            cleared_at,
        })
    }
    fn active(&self) -> bool {
        self.cleared_at.is_none()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissingReason {
    Unlocated,
    Sacrificed,
    TemporarilyAbandoned,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingDisposition {
    pub component: ComponentId,
    pub reason: MissingReason,
    pub authority: DispositionAuthority,
    pub searches: Vec<SearchId>,
    pub notice: HazardNotice,
}
impl MissingDisposition {
    pub fn new(
        component: ComponentId,
        reason: MissingReason,
        authority: DispositionAuthority,
        searches: Vec<SearchId>,
        notice: HazardNotice,
    ) -> Result<Self, DomainError> {
        if matches!(
            reason,
            MissingReason::Unlocated | MissingReason::TemporarilyAbandoned
        ) && searches.is_empty()
        {
            return Err(DomainError::RecoveryClosureInhibited);
        }
        if !notice.active() {
            return Err(DomainError::RecoveryClosureInhibited);
        }
        Ok(Self {
            component,
            reason,
            authority,
            searches,
            notice,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SacrificialRelease {
    pub id: String,
    pub component: ComponentId,
    pub authority: DispositionAuthority,
    pub released_at: DateTime<Utc>,
    pub notice: HazardNotice,
}
impl SacrificialRelease {
    pub fn new(
        id: &str,
        component: ComponentId,
        authority: DispositionAuthority,
        released_at: DateTime<Utc>,
        notice: HazardNotice,
    ) -> Result<Self, DomainError> {
        if id.trim().is_empty() || !notice.active() {
            return Err(DomainError::InvalidRecoveryRecord);
        }
        Ok(Self {
            id: id.into(),
            component,
            authority,
            released_at,
            notice,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandoffTarget {
    Logistics,
    RobotCare,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestedTreatment {
    Decontamination,
    Inspection,
    Repair,
    Calibration,
    BurnIn,
    Reuse,
    Recycling,
    Retirement,
    TemporaryAbandonment,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffRequest {
    pub id: HandoffRequestId,
    pub component: ComponentId,
    pub target: HandoffTarget,
    pub treatment: RequestedTreatment,
    pub requested_at: DateTime<Utc>,
}
impl HandoffRequest {
    pub fn new(
        id: HandoffRequestId,
        component: ComponentId,
        target: HandoffTarget,
        treatment: RequestedTreatment,
        requested_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if target == HandoffTarget::Logistics
            && matches!(
                treatment,
                RequestedTreatment::Calibration | RequestedTreatment::BurnIn
            )
        {
            return Err(DomainError::InvalidHandoff);
        }
        Ok(Self {
            id,
            component,
            target,
            treatment,
            requested_at,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandoffDecision {
    Accepted,
    Rejected { reason: String },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffAck {
    pub request_id: HandoffRequestId,
    pub acknowledgement_id: String,
    pub acknowledged_at: DateTime<Utc>,
    pub decision: HandoffDecision,
}
impl HandoffAck {
    pub fn accepted(
        request_id: HandoffRequestId,
        acknowledgement_id: &str,
        acknowledged_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if acknowledgement_id.trim().is_empty() {
            return Err(DomainError::InvalidHandoff);
        }
        Ok(Self {
            request_id,
            acknowledgement_id: acknowledgement_id.into(),
            acknowledged_at,
            decision: HandoffDecision::Accepted,
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandoffOutcome {
    Queued,
    Duplicate,
    Acknowledged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryItem {
    pub item: SerializedItem,
    observations: Vec<RecoveryObservation>,
    custody: CustodyState,
    disposition: RecoveryDisposition,
    active_notices: Vec<HazardNotice>,
}
impl RecoveryItem {
    #[must_use]
    pub fn custody(&self) -> &CustodyState {
        &self.custody
    }
    #[must_use]
    pub const fn disposition(&self) -> RecoveryDisposition {
        self.disposition
    }
    #[must_use]
    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }
    #[must_use]
    pub fn latest_observation(&self) -> Option<&RecoveryObservation> {
        self.observations.last()
    }
    fn hazardous(&self) -> bool {
        self.active_notices.iter().any(HazardNotice::active)
            || self.observations.last().is_some_and(|o| {
                !o.energized_hazards.is_empty()
                    || !matches!(
                        o.contamination,
                        ContaminationStatus::Clear | ContaminationStatus::Unknown
                    )
            })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoverySummary {
    pub total: usize,
    pub recovered: usize,
    pub unlocated: usize,
    pub quarantined: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosureReport {
    pub can_close: bool,
    pub unresolved: Vec<ComponentId>,
    pub blocking_hazards: Vec<ComponentId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryLedger {
    items: HashMap<ComponentId, RecoveryItem>,
    observations: HashMap<ObservationId, RecoveryObservation>,
    searches: HashMap<SearchId, SearchRecord>,
    handoffs: HashMap<HandoffRequestId, HandoffRequest>,
    acknowledgements: HashMap<HandoffRequestId, HandoffAck>,
}
impl RecoveryLedger {
    pub fn new(items: Vec<SerializedItem>) -> Result<Self, DomainError> {
        if items.is_empty() {
            return Err(DomainError::InvalidRecoveryRecord);
        }
        let mut mapped = HashMap::new();
        for item in items {
            let key = item.component.clone();
            if mapped
                .insert(
                    key,
                    RecoveryItem {
                        item,
                        observations: vec![],
                        custody: CustodyState::Unaccounted,
                        disposition: RecoveryDisposition::Pending,
                        active_notices: vec![],
                    },
                )
                .is_some()
            {
                return Err(DomainError::InvalidComponent);
            }
        }
        Ok(Self {
            items: mapped,
            observations: HashMap::new(),
            searches: HashMap::new(),
            handoffs: HashMap::new(),
            acknowledgements: HashMap::new(),
        })
    }
    #[must_use]
    pub fn items(&self) -> &HashMap<ComponentId, RecoveryItem> {
        &self.items
    }
    #[must_use]
    pub fn item(&self, component: &ComponentId) -> Option<&RecoveryItem> {
        self.items.get(component)
    }
    pub fn record(
        &mut self,
        observation: RecoveryObservation,
    ) -> Result<RecordOutcome, DomainError> {
        if let Some(existing) = self.observations.get(&observation.observation_id) {
            return if existing == &observation {
                Ok(RecordOutcome::Duplicate)
            } else {
                Err(DomainError::ReplayConflict)
            };
        }
        let item = self
            .items
            .get_mut(&observation.component)
            .ok_or(DomainError::UnknownSerializedItem)?;
        item.observations.push(observation.clone());
        item.observations.sort_by(|left, right| {
            left.observed_at.cmp(&right.observed_at).then_with(|| {
                left.observation_id
                    .as_str()
                    .cmp(right.observation_id.as_str())
            })
        });
        let latest_at = item
            .observations
            .last()
            .map(|value| value.observed_at)
            .ok_or(DomainError::InvalidRecoveryRecord)?;
        let latest = item
            .observations
            .iter()
            .filter(|value| value.observed_at == latest_at)
            .collect::<Vec<_>>();
        let mut claims = latest
            .iter()
            .map(|value| value.custody.clone())
            .collect::<Vec<_>>();
        claims.sort_by_key(|claim| format!("{claim:?}"));
        claims.dedup();
        item.custody = if claims.len() == 1 {
            CustodyState::Held(claims[0].clone())
        } else {
            CustodyState::Disputed { claims }
        };
        let hazardous = latest.iter().any(|value| {
            !value.energized_hazards.is_empty()
                || !matches!(value.contamination, ContaminationStatus::Clear)
                || !matches!(value.damage, DamageAssessment::None)
        });
        item.disposition = if hazardous {
            RecoveryDisposition::Quarantined
        } else if latest
            .iter()
            .all(|value| matches!(value.location, LocationStatus::Known(_)))
        {
            RecoveryDisposition::Recovered
        } else {
            RecoveryDisposition::Pending
        };
        self.observations
            .insert(observation.observation_id.clone(), observation);
        Ok(RecordOutcome::Applied)
    }
    pub fn record_search(&mut self, search: SearchRecord) -> Result<RecordOutcome, DomainError> {
        if !self.items.contains_key(&search.component) {
            return Err(DomainError::UnknownSerializedItem);
        }
        if let Some(existing) = self.searches.get(&search.id) {
            return if existing == &search {
                Ok(RecordOutcome::Duplicate)
            } else {
                Err(DomainError::ReplayConflict)
            };
        }
        self.searches.insert(search.id.clone(), search);
        Ok(RecordOutcome::Applied)
    }
    pub fn accept_missing(&mut self, missing: MissingDisposition) -> Result<(), DomainError> {
        if !missing.searches.iter().all(|id| {
            self.searches
                .get(id)
                .is_some_and(|s| s.component == missing.component)
        }) {
            return Err(DomainError::RecoveryClosureInhibited);
        }
        let item = self
            .items
            .get_mut(&missing.component)
            .ok_or(DomainError::UnknownSerializedItem)?;
        item.disposition = match missing.reason {
            MissingReason::Unlocated => RecoveryDisposition::AcceptedUnlocated,
            MissingReason::Sacrificed => RecoveryDisposition::Sacrificed,
            MissingReason::TemporarilyAbandoned => RecoveryDisposition::TemporarilyAbandoned,
        };
        item.active_notices.push(missing.notice);
        Ok(())
    }
    pub fn record_sacrifice(&mut self, release: SacrificialRelease) -> Result<(), DomainError> {
        let item = self
            .items
            .get_mut(&release.component)
            .ok_or(DomainError::UnknownSerializedItem)?;
        item.disposition = RecoveryDisposition::Sacrificed;
        item.active_notices.push(release.notice);
        Ok(())
    }
    pub fn request_handoff(
        &mut self,
        request: HandoffRequest,
    ) -> Result<HandoffOutcome, DomainError> {
        if let Some(existing) = self.handoffs.get(&request.id) {
            return if existing == &request {
                Ok(HandoffOutcome::Duplicate)
            } else {
                Err(DomainError::ReplayConflict)
            };
        }
        let item = self
            .items
            .get(&request.component)
            .ok_or(DomainError::UnknownSerializedItem)?;
        if item.disposition != RecoveryDisposition::Quarantined {
            return Err(DomainError::InvalidHandoff);
        }
        self.handoffs.insert(request.id.clone(), request);
        Ok(HandoffOutcome::Queued)
    }
    pub fn acknowledge_handoff(&mut self, ack: &HandoffAck) -> Result<HandoffOutcome, DomainError> {
        let request = self
            .handoffs
            .get(&ack.request_id)
            .ok_or(DomainError::InvalidHandoff)?;
        if matches!(&ack.decision, HandoffDecision::Rejected { reason } if reason.trim().is_empty())
        {
            return Err(DomainError::InvalidHandoff);
        }
        if let Some(existing) = self.acknowledgements.get(&ack.request_id) {
            return if existing == ack {
                Ok(HandoffOutcome::Duplicate)
            } else {
                Err(DomainError::ReplayConflict)
            };
        }
        if ack.decision == HandoffDecision::Accepted {
            let item = self
                .items
                .get_mut(&request.component)
                .ok_or(DomainError::UnknownSerializedItem)?;
            item.custody = CustodyState::Held(match request.target {
                HandoffTarget::Logistics => Custodian::Logistics,
                HandoffTarget::RobotCare => Custodian::RobotCare,
            });
            item.disposition = match request.treatment {
                RequestedTreatment::Decontamination => RecoveryDisposition::Decontamination,
                RequestedTreatment::Inspection => RecoveryDisposition::Inspection,
                RequestedTreatment::Repair => RecoveryDisposition::Repair,
                RequestedTreatment::Calibration => RecoveryDisposition::Calibration,
                RequestedTreatment::BurnIn => RecoveryDisposition::BurnIn,
                RequestedTreatment::Reuse => RecoveryDisposition::Reuse,
                RequestedTreatment::Recycling => RecoveryDisposition::Recycling,
                RequestedTreatment::Retirement => RecoveryDisposition::Retirement,
                RequestedTreatment::TemporaryAbandonment => RecoveryDisposition::Quarantined,
            };
        }
        self.acknowledgements
            .insert(ack.request_id.clone(), ack.clone());
        Ok(HandoffOutcome::Acknowledged)
    }
    #[must_use]
    pub fn summary(&self) -> RecoverySummary {
        RecoverySummary {
            total: self.items.len(),
            recovered: self
                .items
                .values()
                .filter(|i| i.disposition == RecoveryDisposition::Recovered)
                .count(),
            unlocated: self
                .items
                .values()
                .filter(|i| i.disposition == RecoveryDisposition::Pending)
                .count(),
            quarantined: self
                .items
                .values()
                .filter(|i| i.disposition == RecoveryDisposition::Quarantined)
                .count(),
        }
    }
    #[must_use]
    pub fn closure_report(&self) -> ClosureReport {
        let terminal = |d| {
            matches!(
                d,
                RecoveryDisposition::Recovered
                    | RecoveryDisposition::Reuse
                    | RecoveryDisposition::Recycling
                    | RecoveryDisposition::Retirement
                    | RecoveryDisposition::TemporarilyAbandoned
                    | RecoveryDisposition::AcceptedUnlocated
                    | RecoveryDisposition::Sacrificed
            )
        };
        let mut unresolved = self
            .items
            .values()
            .filter(|i| {
                !terminal(i.disposition) || matches!(i.custody, CustodyState::Disputed { .. })
            })
            .map(|i| i.item.component.clone())
            .collect::<Vec<_>>();
        let mut blocking_hazards = self
            .items
            .values()
            .filter(|i| {
                i.hazardous()
                    && !matches!(
                        i.disposition,
                        RecoveryDisposition::TemporarilyAbandoned
                            | RecoveryDisposition::AcceptedUnlocated
                            | RecoveryDisposition::Sacrificed
                    )
            })
            .map(|i| i.item.component.clone())
            .collect::<Vec<_>>();
        unresolved.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        blocking_hazards.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        ClosureReport {
            can_close: unresolved.is_empty() && blocking_hazards.is_empty(),
            unresolved,
            blocking_hazards,
        }
    }
}
