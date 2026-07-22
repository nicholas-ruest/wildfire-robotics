//! Hazard source, observation, picture, and visual-evidence aggregates.
#![allow(missing_docs)]
use chrono::{DateTime, Utc};
use shared_kernel::{EntityId, GeoPoint, Quantity};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
pub type Digest = [u8; 32];
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum HazardError {
    #[error("source terms, license, or lifecycle deny use")]
    SourceDenied,
    #[error("observation is incomplete or invalid")]
    InvalidObservation,
    #[error("batch is quarantined")]
    Quarantined,
    #[error("duplicate identity contradicts prior content")]
    ContradictoryDuplicate,
    #[error("observation correction is invalid")]
    InvalidCorrection,
    #[error("picture manifest is incomplete, stale, or contaminated")]
    InvalidPicture,
    #[error("visual evidence is unverified or misaligned")]
    UnverifiedVisual,
    #[error("ingestion transition is invalid")]
    InvalidTransition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceState {
    Candidate,
    Active,
    Suspended,
    Retired,
}
#[derive(Clone, Debug)]
pub struct Source {
    pub id: EntityId,
    pub provider: String,
    pub license: String,
    pub terms_digest: Digest,
    pub coverage: String,
    pub state: SourceState,
    pub credential_generation: u64,
}
impl Source {
    pub fn register(
        id: EntityId,
        provider: impl Into<String>,
        license: impl Into<String>,
        terms: Digest,
        coverage: impl Into<String>,
    ) -> Result<Self, HazardError> {
        let provider = provider.into();
        let license = license.into();
        let coverage = coverage.into();
        if provider.trim().is_empty()
            || license.trim().is_empty()
            || coverage.trim().is_empty()
            || terms == [0; 32]
        {
            return Err(HazardError::SourceDenied);
        }
        Ok(Self {
            id,
            provider,
            license,
            terms_digest: terms,
            coverage,
            state: SourceState::Candidate,
            credential_generation: 1,
        })
    }
    pub fn activate(&mut self, terms_approved: bool) -> Result<(), HazardError> {
        if self.state != SourceState::Candidate || !terms_approved {
            return Err(HazardError::SourceDenied);
        }
        self.state = SourceState::Active;
        Ok(())
    }
    pub fn suspend(&mut self) {
        if self.state != SourceState::Retired {
            self.state = SourceState::Suspended;
        }
    }
    #[must_use]
    pub fn usable(&self) -> bool {
        self.state == SourceState::Active
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunState {
    Created,
    Fetching,
    Validating,
    Accepted,
    Quarantined,
    Failed,
}
#[derive(Clone, Debug)]
pub struct IngestionRun {
    pub id: EntityId,
    pub source_id: EntityId,
    pub state: RunState,
    pub fetched_checksum: Option<Digest>,
    pub accepted: u64,
    pub rejected: u64,
    pub reason: Option<String>,
}
impl IngestionRun {
    #[must_use]
    pub fn start(id: EntityId, source: &Source) -> Self {
        Self {
            id,
            source_id: source.id.clone(),
            state: RunState::Created,
            fetched_checksum: None,
            accepted: 0,
            rejected: 0,
            reason: None,
        }
    }
    pub fn record_fetch(&mut self, checksum: Digest) -> Result<(), HazardError> {
        if self.state != RunState::Created || checksum == [0; 32] {
            return Err(HazardError::InvalidTransition);
        }
        self.fetched_checksum = Some(checksum);
        self.state = RunState::Validating;
        Ok(())
    }
    pub fn finish(&mut self, accepted: u64, rejected: u64) -> Result<(), HazardError> {
        if self.state != RunState::Validating {
            return Err(HazardError::InvalidTransition);
        }
        self.accepted = accepted;
        self.rejected = rejected;
        self.state = if rejected == 0 {
            RunState::Accepted
        } else {
            RunState::Quarantined
        };
        Ok(())
    }
    pub fn fail(&mut self, reason: impl Into<String>) {
        self.reason = Some(reason.into());
        self.state = RunState::Failed;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Observation {
    pub id: EntityId,
    pub provider_identity: String,
    pub source_id: EntityId,
    pub source_version: String,
    pub license: String,
    pub event_time: DateTime<Utc>,
    pub ingested_at: DateTime<Utc>,
    pub geometry: GeoPoint,
    pub quantity: Quantity,
    pub uncertainty: u64,
    pub quality_bps: u16,
    pub lineage: Vec<Digest>,
    pub content_digest: Digest,
    pub correction_of: Option<EntityId>,
    pub operational: bool,
}
impl Observation {
    pub fn validate(&self, source: &Source) -> Result<(), HazardError> {
        if !source.usable()
            || self.source_id != source.id
            || self.license != source.license
            || self.provider_identity.trim().is_empty()
            || self.source_version.trim().is_empty()
            || self.event_time > self.ingested_at
            || self.quality_bps > 10_000
            || self.lineage.is_empty()
            || self.lineage.contains(&[0; 32])
            || self.content_digest == [0; 32]
        {
            return Err(HazardError::InvalidObservation);
        }
        Ok(())
    }
    #[must_use]
    pub fn semantic_key(&self) -> String {
        format!(
            "{}:{}:{:02x?}",
            self.provider_identity, self.source_version, self.content_digest
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SetState {
    Open,
    Sealed,
    Superseded,
}
#[derive(Clone, Debug)]
pub struct ObservationSet {
    pub id: EntityId,
    pub state: SetState,
    observations: BTreeMap<String, Observation>,
    provider_versions: BTreeMap<String, Digest>,
    quarantine: Vec<Observation>,
    superseded: BTreeMap<String, String>,
    pub manifest_digest: Option<Digest>,
}
impl ObservationSet {
    #[must_use]
    pub fn open(id: EntityId) -> Self {
        Self {
            id,
            state: SetState::Open,
            observations: BTreeMap::new(),
            provider_versions: BTreeMap::new(),
            quarantine: Vec::new(),
            superseded: BTreeMap::new(),
            manifest_digest: None,
        }
    }
    pub fn append(
        &mut self,
        observation: Observation,
        source: &Source,
    ) -> Result<bool, HazardError> {
        if self.state != SetState::Open {
            return Err(HazardError::InvalidTransition);
        }
        if observation.validate(source).is_err() || !observation.operational {
            self.quarantine.push(observation);
            return Err(HazardError::Quarantined);
        }
        let identity = format!(
            "{}:{}",
            observation.provider_identity, observation.source_version
        );
        if let Some(digest) = self.provider_versions.get(&identity) {
            return if digest == &observation.content_digest {
                Ok(false)
            } else {
                Err(HazardError::ContradictoryDuplicate)
            };
        }
        self.provider_versions
            .insert(identity, observation.content_digest);
        self.observations
            .insert(observation.semantic_key(), observation);
        Ok(true)
    }
    pub fn correct(
        &mut self,
        original: &EntityId,
        correction: Observation,
        source: &Source,
    ) -> Result<(), HazardError> {
        let prior = self
            .observations
            .values()
            .find(|o| &o.id == original)
            .ok_or(HazardError::InvalidCorrection)?;
        if correction.correction_of.as_ref() != Some(original)
            || correction.event_time < prior.event_time
            || correction.content_digest == prior.content_digest
        {
            return Err(HazardError::InvalidCorrection);
        }
        let prior_key = prior.semantic_key();
        let next_key = correction.semantic_key();
        correction.validate(source)?;
        self.superseded.insert(prior_key, next_key.clone());
        self.provider_versions.insert(
            format!(
                "{}:{}",
                correction.provider_identity, correction.source_version
            ),
            correction.content_digest,
        );
        self.observations.insert(next_key, correction);
        Ok(())
    }
    pub fn seal(&mut self, digest: Digest) -> Result<(), HazardError> {
        if self.state != SetState::Open || digest == [0; 32] || self.observations.is_empty() {
            return Err(HazardError::InvalidTransition);
        }
        self.manifest_digest = Some(digest);
        self.state = SetState::Sealed;
        Ok(())
    }
    pub fn operational(&self) -> impl Iterator<Item = &Observation> {
        self.observations
            .iter()
            .filter(|(key, _)| !self.superseded.contains_key(*key))
            .map(|(_, v)| v)
    }
    #[must_use]
    pub fn quarantined_count(&self) -> usize {
        self.quarantine.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Gap {
    pub area: String,
    pub reason: String,
    pub since: DateTime<Utc>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PictureState {
    Building,
    Published,
    Stale,
    Superseded,
}
#[derive(Clone, Debug)]
pub struct HazardPicture {
    pub id: EntityId,
    pub state: PictureState,
    pub observation_manifest: Digest,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub freshness_seconds: u64,
    pub uncertainty_summary: u64,
    pub gaps: Vec<Gap>,
    pub algorithm_version: String,
    pub observation_digests: BTreeSet<Digest>,
}
impl HazardPicture {
    pub fn build(
        id: EntityId,
        set: &ObservationSet,
        valid_from: DateTime<Utc>,
        valid_until: DateTime<Utc>,
        freshness: u64,
        gaps: Vec<Gap>,
        algorithm: impl Into<String>,
    ) -> Result<Self, HazardError> {
        let algorithm = algorithm.into();
        let manifest = set.manifest_digest.ok_or(HazardError::InvalidPicture)?;
        let observations = set.operational().collect::<Vec<_>>();
        if set.state != SetState::Sealed
            || valid_from >= valid_until
            || freshness == 0
            || algorithm.trim().is_empty()
            || observations.is_empty()
        {
            return Err(HazardError::InvalidPicture);
        }
        Ok(Self {
            id,
            state: PictureState::Building,
            observation_manifest: manifest,
            valid_from,
            valid_until,
            freshness_seconds: freshness,
            uncertainty_summary: observations
                .iter()
                .map(|o| o.uncertainty)
                .max()
                .unwrap_or(0),
            gaps,
            algorithm_version: algorithm,
            observation_digests: observations.iter().map(|o| o.content_digest).collect(),
        })
    }
    pub fn publish(&mut self, now: DateTime<Utc>) -> Result<(), HazardError> {
        if self.state != PictureState::Building || now < self.valid_from || now >= self.valid_until
        {
            return Err(HazardError::InvalidPicture);
        }
        self.state = PictureState::Published;
        Ok(())
    }
    pub fn project_freshness(&mut self, now: DateTime<Utc>) {
        if self.state == PictureState::Published && now >= self.valid_until {
            self.state = PictureState::Stale;
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectEvidence {
    pub object_key: String,
    pub media_type: String,
    pub checksum: Digest,
    pub captured_at: DateTime<Utc>,
    pub footprint_digest: Digest,
    pub sensor_calibration_digest: Digest,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VisualState {
    Ingesting,
    Indexed,
    Verified,
    Superseded,
}
#[derive(Clone, Debug)]
pub struct VisualEvidenceSet {
    pub id: EntityId,
    pub state: VisualState,
    pub raw: Vec<ObjectEvidence>,
    pub index_version: Option<String>,
    pub verified_observations: BTreeMap<String, u16>,
}
impl VisualEvidenceSet {
    #[must_use]
    pub fn new(id: EntityId) -> Self {
        Self {
            id,
            state: VisualState::Ingesting,
            raw: Vec::new(),
            index_version: None,
            verified_observations: BTreeMap::new(),
        }
    }
    pub fn register(&mut self, e: ObjectEvidence) -> Result<(), HazardError> {
        if self.state != VisualState::Ingesting
            || e.object_key.trim().is_empty()
            || e.media_type.trim().is_empty()
            || e.checksum == [0; 32]
            || e.footprint_digest == [0; 32]
            || e.sensor_calibration_digest == [0; 32]
        {
            return Err(HazardError::UnverifiedVisual);
        }
        self.raw.push(e);
        Ok(())
    }
    pub fn index(&mut self, version: impl Into<String>) -> Result<(), HazardError> {
        let version = version.into();
        if self.raw.is_empty() || version.trim().is_empty() {
            return Err(HazardError::UnverifiedVisual);
        }
        self.index_version = Some(version);
        self.state = VisualState::Indexed;
        Ok(())
    }
    pub fn verify(
        &mut self,
        candidate: impl Into<String>,
        spatial_temporal_aligned: bool,
        method: impl Into<String>,
        confidence_bps: u16,
    ) -> Result<(), HazardError> {
        let candidate = candidate.into();
        let method = method.into();
        if self.state != VisualState::Indexed
            || !spatial_temporal_aligned
            || method.trim().is_empty()
            || confidence_bps > 10_000
        {
            return Err(HazardError::UnverifiedVisual);
        }
        self.verified_observations.insert(candidate, confidence_bps);
        self.state = VisualState::Verified;
        Ok(())
    }
}
