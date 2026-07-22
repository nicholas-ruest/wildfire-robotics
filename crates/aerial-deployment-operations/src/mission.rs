//! Mission and independent two-key release authority (ADR-070/073; AD-INV-003/004).
use crate::{
    AerialDropMissionId, AircraftConfigurationId, DomainError, EmergencyLandingZoneId, EvidenceRef,
    ExclusionZoneId, FootprintId, JettisonZoneId, OddId, PayloadManifestId, ReleaseCorridorId,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashSet};

fn text(value: &str) -> Result<String, DomainError> {
    let value = value.trim();
    if value.is_empty() || value.len() > 256 {
        return Err(DomainError::InvalidMissionBinding);
    }
    Ok(value.to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AircraftBinding {
    pub configuration: AircraftConfigurationId,
    tail: String,
}
impl AircraftBinding {
    pub fn new(configuration: AircraftConfigurationId, tail: &str) -> Result<Self, DomainError> {
        Ok(Self {
            configuration,
            tail: text(tail)?,
        })
    }
    #[must_use]
    pub fn tail(&self) -> &str {
        &self.tail
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissionBindings {
    pub payload: PayloadManifestId,
    pub payload_digest: String,
    pub aircraft: AircraftBinding,
    pub route: EvidenceRef,
    pub corridor: ReleaseCorridorId,
    pub nominal_footprint: FootprintId,
    pub failed_component_footprints: Vec<FootprintId>,
    pub exclusion_volume: ExclusionZoneId,
    pub jettison_sectors: Vec<JettisonZoneId>,
    pub emergency_landing_zones: Vec<EmergencyLandingZoneId>,
    pub ground_boundary: FootprintId,
    pub point_of_no_return: EvidenceRef,
    pub alternate_abort_plan: EvidenceRef,
    pub odd: OddId,
    pub odd_evidence: EvidenceRef,
}
impl MissionBindings {
    pub fn validate(self) -> Result<Self, DomainError> {
        let valid_digest = self.payload_digest.len() == 71
            && self.payload_digest.starts_with("sha256:")
            && self.payload_digest[7..]
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase());
        let unique_failures = self
            .failed_component_footprints
            .iter()
            .collect::<HashSet<_>>()
            .len();
        let unique_jettison = self.jettison_sectors.iter().collect::<HashSet<_>>().len();
        let unique_emergency = self
            .emergency_landing_zones
            .iter()
            .collect::<HashSet<_>>()
            .len();
        if !valid_digest
            || self.failed_component_footprints.is_empty()
            || unique_failures != self.failed_component_footprints.len()
            || self.jettison_sectors.is_empty()
            || unique_jettison != self.jettison_sectors.len()
            || self.emergency_landing_zones.is_empty()
            || unique_emergency != self.emergency_landing_zones.len()
        {
            return Err(DomainError::InvalidMissionBinding);
        }
        Ok(self)
    }
    #[must_use]
    pub fn payload_digest(&self) -> &str {
        &self.payload_digest
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReleaseCondition {
    AircraftHealth,
    PayloadHealth,
    Weather,
    Wind,
    Turbulence,
    Smoke,
    Icing,
    Airspace,
    Terrain,
    FirePosition,
    PeopleVehiclesAircraft,
    NavigationTime,
    Communications,
    SurveillanceConfidence,
    GroundReadiness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceObservation {
    pub condition: ReleaseCondition,
    pub source: EvidenceRef,
    pub observed_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub safe: bool,
    /// Integer basis points avoids floating-point ambiguity in safety decisions.
    pub confidence_bps: u16,
}
impl SourceObservation {
    pub fn new(
        condition: ReleaseCondition,
        source: EvidenceRef,
        observed_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
        safe: bool,
        confidence_bps: u16,
    ) -> Result<Self, DomainError> {
        if expires_at <= observed_at || confidence_bps > 10_000 {
            return Err(DomainError::UnsafeOrStaleObservation);
        }
        Ok(Self {
            condition,
            source,
            observed_at,
            expires_at,
            safe,
            confidence_bps,
        })
    }
    fn permits(&self, now: DateTime<Utc>, minimum_confidence_bps: u16) -> bool {
        self.safe
            && self.source.is_current_at(now)
            && now >= self.observed_at
            && now < self.expires_at
            && self.confidence_bps >= minimum_confidence_bps
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorityRole {
    AircraftAuthority,
    IncidentSafety,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionOutcome {
    Approve,
    Hold,
    Veto,
    Abort,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityDecision {
    pub authority: AuthorityRole,
    pub outcome: DecisionOutcome,
    pub command_digest: String,
    pub decided_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub evidence: EvidenceRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LeastHarmContingency {
    Retain,
    Reef,
    Vent,
    Isolate,
    EmergencyLand,
    SafeSectorJettison,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AerialDropMission {
    pub id: AerialDropMissionId,
    pub bindings: MissionBindings,
    observations: BTreeMap<ReleaseCondition, SourceObservation>,
    preauthorized_contingencies: BTreeSet<LeastHarmContingency>,
    version: u64,
    point_of_no_return_crossed: bool,
    released: bool,
    release_fingerprint: Option<String>,
}
impl AerialDropMission {
    pub fn new(
        id: AerialDropMissionId,
        bindings: MissionBindings,
        preauthorized_contingencies: BTreeSet<LeastHarmContingency>,
    ) -> Result<Self, DomainError> {
        if preauthorized_contingencies.is_empty() {
            return Err(DomainError::InvalidMissionBinding);
        }
        Ok(Self {
            id,
            bindings: bindings.validate()?,
            observations: BTreeMap::new(),
            preauthorized_contingencies,
            version: 0,
            point_of_no_return_crossed: false,
            released: false,
            release_fingerprint: None,
        })
    }
    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }
    #[must_use]
    pub const fn released(&self) -> bool {
        self.released
    }
    pub fn observe(
        &mut self,
        expected_version: u64,
        observation: SourceObservation,
    ) -> Result<(), DomainError> {
        if expected_version != self.version {
            return Err(DomainError::VersionConflict);
        }
        self.observations.insert(observation.condition, observation);
        self.version += 1;
        Ok(())
    }
    pub fn cross_point_of_no_return(&mut self, expected_version: u64) -> Result<(), DomainError> {
        if expected_version != self.version {
            return Err(DomainError::VersionConflict);
        }
        self.point_of_no_return_crossed = true;
        self.version += 1;
        Ok(())
    }
    fn conditions_permit(&self, now: DateTime<Utc>, min_confidence_bps: u16) -> bool {
        const REQUIRED: [ReleaseCondition; 15] = [
            ReleaseCondition::AircraftHealth,
            ReleaseCondition::PayloadHealth,
            ReleaseCondition::Weather,
            ReleaseCondition::Wind,
            ReleaseCondition::Turbulence,
            ReleaseCondition::Smoke,
            ReleaseCondition::Icing,
            ReleaseCondition::Airspace,
            ReleaseCondition::Terrain,
            ReleaseCondition::FirePosition,
            ReleaseCondition::PeopleVehiclesAircraft,
            ReleaseCondition::NavigationTime,
            ReleaseCondition::Communications,
            ReleaseCondition::SurveillanceConfidence,
            ReleaseCondition::GroundReadiness,
        ];
        REQUIRED.iter().all(|key| {
            self.observations
                .get(key)
                .is_some_and(|o| o.permits(now, min_confidence_bps))
        })
    }
    fn binding_evidence_is_current(&self, now: DateTime<Utc>) -> bool {
        [
            &self.bindings.route,
            &self.bindings.point_of_no_return,
            &self.bindings.alternate_abort_plan,
            &self.bindings.odd_evidence,
        ]
        .into_iter()
        .all(|evidence| evidence.is_current_at(now))
    }
    #[must_use]
    pub fn canonical_release_digest(&self) -> String {
        let b = &self.bindings;
        let mut fields = vec![
            self.id.as_str(),
            b.payload.as_str(),
            b.payload_digest(),
            b.aircraft.configuration.as_str(),
            b.aircraft.tail(),
            b.route.digest(),
            b.corridor.as_str(),
            b.nominal_footprint.as_str(),
            b.exclusion_volume.as_str(),
            b.ground_boundary.as_str(),
            b.point_of_no_return.digest(),
            b.alternate_abort_plan.digest(),
            b.odd.as_str(),
            b.odd_evidence.digest(),
        ];
        fields.extend(
            b.failed_component_footprints
                .iter()
                .map(FootprintId::as_str),
        );
        fields.extend(b.jettison_sectors.iter().map(JettisonZoneId::as_str));
        fields.extend(
            b.emergency_landing_zones
                .iter()
                .map(EmergencyLandingZoneId::as_str),
        );
        let mut hasher = Sha256::new();
        for field in fields {
            hasher.update(field.len().to_be_bytes());
            hasher.update(field.as_bytes());
        }
        format!("sha256:{:x}", hasher.finalize())
    }
    pub fn commit_release(
        &mut self,
        expected_version: u64,
        now: DateTime<Utc>,
        minimum_confidence_bps: u16,
        aircraft: &AuthorityDecision,
        incident: &AuthorityDecision,
    ) -> Result<(), DomainError> {
        const DOMAIN_CONFIDENCE_FLOOR_BPS: u16 = 9_000;
        let digest = self.canonical_release_digest();
        if aircraft.authority != AuthorityRole::AircraftAuthority
            || incident.authority != AuthorityRole::IncidentSafety
            || aircraft.command_digest != digest
            || incident.command_digest != digest
        {
            return Err(DomainError::ReleaseDigestMismatch);
        }
        if aircraft.evidence.id() == incident.evidence.id() {
            return Err(DomainError::ReleaseDigestMismatch);
        }
        let fingerprint = decision_fingerprint(aircraft, incident);
        if self.released {
            return if expected_version.checked_add(1) == Some(self.version)
                && self.release_fingerprint.as_deref() == Some(&fingerprint)
            {
                Ok(())
            } else {
                Err(DomainError::ReplayConflict)
            };
        }
        if expected_version != self.version {
            return Err(DomainError::VersionConflict);
        }
        if !(DOMAIN_CONFIDENCE_FLOOR_BPS..=10_000).contains(&minimum_confidence_bps) {
            return Err(DomainError::UnsafeOrStaleObservation);
        }
        for decision in [aircraft, incident] {
            if decision.outcome != DecisionOutcome::Approve {
                return Err(DomainError::ReleaseInhibited);
            }
            if now < decision.decided_at
                || now >= decision.expires_at
                || !decision.evidence.is_current_at(now)
            {
                return Err(DomainError::UnsafeOrStaleObservation);
            }
        }
        if !self.binding_evidence_is_current(now)
            || !self.conditions_permit(now, minimum_confidence_bps)
        {
            return Err(DomainError::UnsafeOrStaleObservation);
        }
        self.released = true;
        self.release_fingerprint = Some(fingerprint);
        self.version += 1;
        Ok(())
    }
    pub fn execute_post_release(
        &self,
        contingency: LeastHarmContingency,
    ) -> Result<(), DomainError> {
        if !self.released || !self.preauthorized_contingencies.contains(&contingency) {
            return Err(DomainError::ContingencyNotAuthorized);
        }
        Ok(())
    }
    pub fn broaden_or_reroute(&self) -> Result<(), DomainError> {
        if self.released || self.point_of_no_return_crossed {
            Err(DomainError::OperationalBoundaryCrossed)
        } else {
            Ok(())
        }
    }
}

fn decision_fingerprint(aircraft: &AuthorityDecision, incident: &AuthorityDecision) -> String {
    let mut hasher = Sha256::new();
    for decision in [aircraft, incident] {
        for field in [
            format!("{:?}", decision.authority),
            format!("{:?}", decision.outcome),
            decision.command_digest.clone(),
            decision.decided_at.to_rfc3339(),
            decision.expires_at.to_rfc3339(),
            decision.evidence.id().as_str().to_owned(),
            decision.evidence.digest().to_owned(),
        ] {
            hasher.update(field.len().to_be_bytes());
            hasher.update(field.as_bytes());
        }
    }
    format!("sha256:{:x}", hasher.finalize())
}
