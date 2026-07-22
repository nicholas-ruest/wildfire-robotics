use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Digest(String);
impl Digest {
    pub fn parse(value: &str) -> Result<Self, PlanningError> {
        if value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit()) {
            Ok(Self(value.to_ascii_lowercase()))
        } else {
            Err(PlanningError::InvalidDigest)
        }
    }
    pub(crate) fn hash(bytes: &[u8]) -> Self {
        Self(format!("{:x}", Sha256::digest(bytes)))
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PlanningError {
    #[error("invalid digest")]
    InvalidDigest,
    #[error("invalid registration: {0}")]
    InvalidRegistration(&'static str),
    #[error("model is not promoted")]
    ModelNotPromoted,
    #[error("run is outside the operational domain")]
    OutsideOperationalDomain,
    #[error("invalid transition")]
    InvalidTransition,
    #[error("invalid output")]
    InvalidOutput,
    #[error("publication gate failed: {0}")]
    PublicationGateFailed(&'static str),
    #[error("sandbox limit exceeded: {0}")]
    SandboxLimitExceeded(&'static str),
    #[error("duplicate identifier")]
    DuplicateIdentifier,
    #[error("invalid recommendation")]
    InvalidRecommendation,
    #[error("invalid scenario: {0}")]
    InvalidScenario(&'static str),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationalDomain {
    pub id: String,
    pub regions: BTreeSet<String>,
    pub min_temperature_c: f64,
    pub max_temperature_c: f64,
    pub max_wind_speed_mps: f64,
    pub supported_fuel_models: BTreeSet<String>,
}
impl OperationalDomain {
    pub fn contains(&self, c: &RunContext) -> bool {
        self.regions.contains(&c.region)
            && c.temperature_c.is_finite()
            && c.temperature_c >= self.min_temperature_c
            && c.temperature_c <= self.max_temperature_c
            && c.wind_speed_mps.is_finite()
            && (0.0..=self.max_wind_speed_mps).contains(&c.wind_speed_mps)
            && self.supported_fuel_models.contains(&c.fuel_model)
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Calibration {
    pub method: String,
    pub cohort_digest: Digest,
    pub valid_until: DateTime<Utc>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelRegistration {
    pub id: String,
    pub artifact_digest: Digest,
    pub source_digest: Digest,
    pub dependency_digest: Digest,
    pub runtime_digest: Digest,
    pub training_data_digests: Vec<Digest>,
    pub odd: OperationalDomain,
    pub calibration: Calibration,
    pub limitations: Vec<String>,
    pub license: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelStage {
    Registered,
    Approved,
    Promoted,
    Shadow,
    Canary,
    Suspended,
    Retired,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Approval {
    pub id: String,
    pub reviewer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRelease {
    content: ModelRegistration,
    content_digest: Digest,
    stage: ModelStage,
    approvals: Vec<Approval>,
    rollback_reason: Option<String>,
}
impl ModelRelease {
    pub fn register(content: ModelRegistration) -> Result<Self, PlanningError> {
        if content.id.trim().is_empty()
            || content.license.trim().is_empty()
            || content.limitations.is_empty()
            || content.odd.regions.is_empty()
            || content.training_data_digests.is_empty()
        {
            return Err(PlanningError::InvalidRegistration("required provenance"));
        }
        let bytes = serde_json::to_vec(&content)
            .map_err(|_| PlanningError::InvalidRegistration("serialization"))?;
        Ok(Self {
            content,
            content_digest: Digest::hash(&bytes),
            stage: ModelStage::Registered,
            approvals: vec![],
            rollback_reason: None,
        })
    }
    pub fn approve(&mut self, id: &str, reviewer: &str) -> Result<(), PlanningError> {
        if self.stage != ModelStage::Registered || id.is_empty() || reviewer.is_empty() {
            return Err(PlanningError::InvalidTransition);
        }
        self.approvals.push(Approval {
            id: id.into(),
            reviewer: reviewer.into(),
        });
        self.stage = ModelStage::Approved;
        Ok(())
    }
    pub fn promote(&mut self) -> Result<(), PlanningError> {
        if self.stage != ModelStage::Approved {
            return Err(PlanningError::InvalidTransition);
        }
        self.stage = ModelStage::Promoted;
        Ok(())
    }
    pub fn begin_shadow(&mut self) -> Result<(), PlanningError> {
        if self.stage != ModelStage::Promoted {
            return Err(PlanningError::InvalidTransition);
        }
        self.stage = ModelStage::Shadow;
        Ok(())
    }
    pub fn begin_canary(&mut self) -> Result<(), PlanningError> {
        if self.stage != ModelStage::Shadow {
            return Err(PlanningError::InvalidTransition);
        }
        self.stage = ModelStage::Canary;
        Ok(())
    }
    pub fn rollback(&mut self, reason: &str) -> Result<(), PlanningError> {
        if !matches!(self.stage, ModelStage::Shadow | ModelStage::Canary)
            || reason.trim().is_empty()
        {
            return Err(PlanningError::InvalidTransition);
        }
        self.rollback_reason = Some(reason.into());
        self.stage = ModelStage::Promoted;
        Ok(())
    }
    pub fn suspend(&mut self) {
        self.stage = ModelStage::Suspended;
    }
    pub fn stage(&self) -> ModelStage {
        self.stage
    }
    pub fn content_digest(&self) -> &Digest {
        &self.content_digest
    }
    pub fn registration(&self) -> &ModelRegistration {
        &self.content
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunContext {
    pub region: String,
    pub temperature_c: f64,
    pub wind_speed_mps: f64,
    pub fuel_model: String,
    pub observed_at: DateTime<Utc>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputArtifact {
    pub id: String,
    pub digest: Digest,
    pub license: Option<String>,
    pub superseded: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    value: f64,
    unit: String,
}
impl Parameter {
    pub fn new(value: f64, unit: &str) -> Result<Self, PlanningError> {
        if !value.is_finite() || unit.trim().is_empty() {
            Err(PlanningError::InvalidRegistration("parameter"))
        } else {
            Ok(Self {
                value,
                unit: unit.into(),
            })
        }
    }
    pub fn value(&self) -> f64 {
        self.value
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunManifest {
    pub id: String,
    model_release_digest: Digest,
    model_stage: ModelStage,
    model_odd: OperationalDomain,
    artifact_digest: Digest,
    source_digest: Digest,
    dependency_digest: Digest,
    runtime_digest: Digest,
    pub inputs: Vec<InputArtifact>,
    pub seed: u64,
    pub parameters: BTreeMap<String, Parameter>,
    pub context: RunContext,
}
impl RunManifest {
    pub fn new(
        id: &str,
        model: &ModelRelease,
        mut inputs: Vec<InputArtifact>,
        seed: u64,
        parameters: BTreeMap<String, Parameter>,
        context: RunContext,
    ) -> Result<Self, PlanningError> {
        if id.trim().is_empty() || inputs.is_empty() {
            return Err(PlanningError::InvalidRegistration("manifest"));
        }
        inputs.sort_by(|a, b| a.id.cmp(&b.id).then(a.digest.cmp(&b.digest)));
        let c = model.registration();
        Ok(Self {
            id: id.into(),
            model_release_digest: model.content_digest.clone(),
            model_stage: model.stage,
            model_odd: c.odd.clone(),
            artifact_digest: c.artifact_digest.clone(),
            source_digest: c.source_digest.clone(),
            dependency_digest: c.dependency_digest.clone(),
            runtime_digest: c.runtime_digest.clone(),
            inputs,
            seed,
            parameters,
            context,
        })
    }
    pub fn digest(&self) -> Digest {
        Digest::hash(&serde_json::to_vec(self).unwrap_or_default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Experimental,
    Shadow,
    Operational,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForecastStatus {
    Requested,
    Running,
    Validating,
    Validated,
    Published,
    Rejected,
    Failed,
    Withdrawn,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForecastCell {
    pub x: i32,
    pub y: i32,
    pub probability: f64,
    pub arrival_minutes: Option<f64>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelOutput {
    pub cells: Vec<ForecastCell>,
    pub uncertainty: f64,
    pub artifact_digest: Digest,
}
impl ModelOutput {
    pub fn is_valid(&self) -> bool {
        !self.cells.is_empty()
            && self.uncertainty.is_finite()
            && (0.0..=1.0).contains(&self.uncertainty)
            && self.cells.iter().all(|c| {
                c.probability.is_finite()
                    && (0.0..=1.0).contains(&c.probability)
                    && c.arrival_minutes.is_none_or(|v| v.is_finite() && v >= 0.0)
            })
    }
    pub fn within_tolerance(&self, other: &Self, tolerance: f64) -> bool {
        self.cells.len() == other.cells.len()
            && self.cells.iter().zip(&other.cells).all(|(a, b)| {
                a.x == b.x
                    && a.y == b.y
                    && (a.probability - b.probability).abs() <= tolerance
                    && match (a.arrival_minutes, b.arrival_minutes) {
                        (Some(x), Some(y)) => (x - y).abs() <= tolerance,
                        (None, None) => true,
                        _ => false,
                    }
            })
            && (self.uncertainty - other.uncertainty).abs() <= tolerance
            && self.artifact_digest == other.artifact_digest
    }
}
#[derive(Debug, Clone, Copy)]
pub struct PublicationEvidence {
    pub schema: bool,
    pub physical_plausibility: bool,
    pub calibration: bool,
    pub completeness: bool,
    pub licensing: bool,
    pub domain_validity: bool,
    pub reproducible: bool,
}
impl PublicationEvidence {
    pub fn all_passed() -> Self {
        Self {
            schema: true,
            physical_plausibility: true,
            calibration: true,
            completeness: true,
            licensing: true,
            domain_validity: true,
            reproducible: true,
        }
    }
    fn first_failure(self) -> Option<&'static str> {
        [
            ("schema", self.schema),
            ("physical-plausibility", self.physical_plausibility),
            ("calibration", self.calibration),
            ("completeness", self.completeness),
            ("licensing", self.licensing),
            ("domain-validity", self.domain_validity),
            ("reproducibility", self.reproducible),
        ]
        .into_iter()
        .find_map(|(n, v)| (!v).then_some(n))
    }
}

#[derive(Debug)]
pub struct ForecastRun {
    manifest: RunManifest,
    status: ForecastStatus,
    output: Option<ModelOutput>,
}
impl ForecastRun {
    pub fn request(manifest: RunManifest, mode: RunMode) -> Result<Self, PlanningError> {
        if mode == RunMode::Operational && manifest.model_stage != ModelStage::Promoted {
            return Err(PlanningError::ModelNotPromoted);
        }
        if !manifest.model_odd.contains(&manifest.context) {
            return Err(PlanningError::OutsideOperationalDomain);
        }
        if manifest
            .inputs
            .iter()
            .any(|i| i.superseded || i.license.as_deref().is_none_or(str::is_empty))
        {
            return Err(PlanningError::PublicationGateFailed("input-lineage"));
        }
        Ok(Self {
            manifest,
            status: ForecastStatus::Requested,
            output: None,
        })
    }
    pub fn start(&mut self) -> Result<(), PlanningError> {
        if self.status != ForecastStatus::Requested {
            return Err(PlanningError::InvalidTransition);
        }
        self.status = ForecastStatus::Running;
        Ok(())
    }
    pub fn record_output(&mut self, output: ModelOutput) -> Result<(), PlanningError> {
        if self.status != ForecastStatus::Running {
            return Err(PlanningError::InvalidTransition);
        }
        self.output = Some(output);
        self.status = ForecastStatus::Validating;
        Ok(())
    }
    pub fn validate(&mut self, e: PublicationEvidence) -> Result<(), PlanningError> {
        if self.status != ForecastStatus::Validating {
            return Err(PlanningError::InvalidTransition);
        }
        if !self.output.as_ref().is_some_and(ModelOutput::is_valid) {
            self.status = ForecastStatus::Rejected;
            return Err(PlanningError::InvalidOutput);
        }
        if let Some(g) = e.first_failure() {
            self.status = ForecastStatus::Rejected;
            return Err(PlanningError::PublicationGateFailed(g));
        }
        self.status = ForecastStatus::Validated;
        Ok(())
    }
    pub fn publish(&mut self) -> Result<(), PlanningError> {
        if self.status != ForecastStatus::Validated {
            return Err(PlanningError::InvalidTransition);
        }
        self.status = ForecastStatus::Published;
        Ok(())
    }
    pub fn status(&self) -> ForecastStatus {
        self.status
    }
    pub fn manifest(&self) -> &RunManifest {
        &self.manifest
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Authority {
    AdvisoryOnly,
}
#[derive(Debug, Clone)]
pub struct Recommendation {
    id: String,
    run_id: String,
    text: String,
    confidence: f64,
    uncertainty: f64,
    expires_at: DateTime<Utc>,
    limitations: Vec<String>,
    alternatives: Vec<String>,
    freshness: String,
}
impl Recommendation {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &str,
        run_id: &str,
        text: &str,
        confidence: f64,
        uncertainty: f64,
        expires_at: DateTime<Utc>,
        limitations: Vec<String>,
        alternatives: Vec<String>,
        freshness: &str,
    ) -> Result<Self, PlanningError> {
        if id.is_empty()
            || run_id.is_empty()
            || text.is_empty()
            || freshness.is_empty()
            || limitations.is_empty()
            || alternatives.is_empty()
            || !(0.0..=1.0).contains(&confidence)
            || !(0.0..=1.0).contains(&uncertainty)
        {
            return Err(PlanningError::InvalidRecommendation);
        }
        Ok(Self {
            id: id.into(),
            run_id: run_id.into(),
            text: text.into(),
            confidence,
            uncertainty,
            expires_at,
            limitations,
            alternatives,
            freshness: freshness.into(),
        })
    }
    pub fn authority(&self) -> Authority {
        Authority::AdvisoryOnly
    }

    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn run_id(&self) -> &str {
        &self.run_id
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn confidence(&self) -> f64 {
        self.confidence
    }
    pub fn uncertainty(&self) -> f64 {
        self.uncertainty
    }
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }
    pub fn limitations(&self) -> &[String] {
        &self.limitations
    }
    pub fn alternatives(&self) -> &[String] {
        &self.alternatives
    }
    pub fn freshness(&self) -> &str {
        &self.freshness
    }
}
#[derive(Debug, Clone, Copy)]
pub enum Invalidation {
    MaterialDrift,
    InvalidAssumption,
    SupersededInput,
}
#[derive(Default)]
pub struct LineageRegistry {
    edges: BTreeMap<String, BTreeSet<String>>,
}
impl LineageRegistry {
    pub fn register(&mut self, source: &str, dependent: &str) {
        self.edges
            .entry(source.into())
            .or_default()
            .insert(dependent.into());
    }
    pub fn invalidate(&self, source: &str, _reason: Invalidation) -> BTreeSet<String> {
        let mut found = BTreeSet::new();
        let mut queue = VecDeque::from([source.to_owned()]);
        while let Some(item) = queue.pop_front() {
            if let Some(next) = self.edges.get(&item) {
                for dep in next {
                    if found.insert(dep.clone()) {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }
        found
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvaluationStatus {
    Designed,
    Frozen,
    Running,
    Reviewed,
    Accepted,
    Rejected,
}
#[derive(Debug)]
pub struct EvaluationStudy {
    pub id: String,
    pub cohort_digest: Digest,
    pub status: EvaluationStatus,
    pub metrics: BTreeMap<String, f64>,
}
impl EvaluationStudy {
    pub fn design(id: &str, cohort_digest: Digest) -> Result<Self, PlanningError> {
        if id.is_empty() {
            return Err(PlanningError::InvalidRegistration("evaluation"));
        }
        Ok(Self {
            id: id.into(),
            cohort_digest,
            status: EvaluationStatus::Designed,
            metrics: BTreeMap::new(),
        })
    }
    pub fn freeze(&mut self) -> Result<(), PlanningError> {
        if self.status != EvaluationStatus::Designed {
            return Err(PlanningError::InvalidTransition);
        }
        self.status = EvaluationStatus::Frozen;
        Ok(())
    }
    pub fn record(&mut self, metrics: BTreeMap<String, f64>) -> Result<(), PlanningError> {
        if self.status != EvaluationStatus::Frozen || metrics.values().any(|v| !v.is_finite()) {
            return Err(PlanningError::InvalidTransition);
        }
        self.metrics = metrics;
        self.status = EvaluationStatus::Reviewed;
        Ok(())
    }
    pub fn accept(&mut self) -> Result<(), PlanningError> {
        if self.status != EvaluationStatus::Reviewed {
            return Err(PlanningError::InvalidTransition);
        }
        self.status = EvaluationStatus::Accepted;
        Ok(())
    }
}
