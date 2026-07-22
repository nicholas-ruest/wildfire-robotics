#![allow(missing_docs, clippy::must_use_candidate)]
use sha2::{Digest as _, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum VisualIndexError {
    #[error("invalid manifest")]
    InvalidManifest,
    #[error("invalid query")]
    InvalidQuery,
    #[error("atomic rebuild failed")]
    RebuildFailed,
    #[error("data leakage: {0}")]
    Leakage(&'static str),
    #[error("verification invalid")]
    InvalidVerification,
}
#[derive(Debug, Clone)]
pub struct VisualItem {
    pub id: String,
    pub media_digest: [u8; 32],
    pub keyframe_ms: u64,
    pub embedding: Vec<i16>,
}
impl VisualItem {
    pub fn new(
        id: &str,
        media_digest: [u8; 32],
        keyframe_ms: u64,
        embedding: Vec<i16>,
    ) -> Result<Self, VisualIndexError> {
        if id.is_empty() || media_digest == [0; 32] || embedding.is_empty() {
            return Err(VisualIndexError::InvalidManifest);
        }
        Ok(Self {
            id: id.into(),
            media_digest,
            keyframe_ms,
            embedding,
        })
    }
}
#[derive(Debug, Clone)]
pub struct IndexManifest {
    id: String,
    model_digest: [u8; 32],
    calibration_digest: [u8; 32],
    items: Vec<VisualItem>,
    digest: [u8; 32],
}
impl IndexManifest {
    pub fn new(
        id: &str,
        model_digest: [u8; 32],
        calibration_digest: [u8; 32],
        mut items: Vec<VisualItem>,
    ) -> Result<Self, VisualIndexError> {
        if id.is_empty()
            || model_digest == [0; 32]
            || calibration_digest == [0; 32]
            || items.is_empty()
        {
            return Err(VisualIndexError::InvalidManifest);
        }
        items.sort_by(|a, b| a.id.cmp(&b.id));
        let mut h = Sha256::new();
        h.update(id);
        h.update(model_digest);
        h.update(calibration_digest);
        for i in &items {
            h.update(&i.id);
            h.update(i.media_digest);
            h.update(i.keyframe_ms.to_be_bytes());
            for v in &i.embedding {
                h.update(v.to_be_bytes());
            }
        }
        Ok(Self {
            id: id.into(),
            model_digest,
            calibration_digest,
            items,
            digest: h.finalize().into(),
        })
    }
    pub fn digest(&self) -> [u8; 32] {
        self.digest
    }
    pub fn calibration_digest(&self) -> [u8; 32] {
        self.calibration_digest
    }
    pub fn model_digest(&self) -> [u8; 32] {
        self.model_digest
    }
    pub fn id(&self) -> &str {
        &self.id
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    pub id: String,
    pub distance: u64,
}
pub trait VisualIndexPort {
    fn search(&self, query: &[i16], limit: usize) -> Result<Vec<SearchHit>, VisualIndexError>;
}
pub struct ExactVisualIndex {
    manifest: IndexManifest,
}
impl ExactVisualIndex {
    pub fn build(manifest: IndexManifest) -> Result<Self, VisualIndexError> {
        Ok(Self { manifest })
    }
}
impl VisualIndexPort for ExactVisualIndex {
    fn search(&self, q: &[i16], limit: usize) -> Result<Vec<SearchHit>, VisualIndexError> {
        if q.is_empty()
            || limit == 0
            || self
                .manifest
                .items
                .iter()
                .any(|i| i.embedding.len() != q.len())
        {
            return Err(VisualIndexError::InvalidQuery);
        }
        let mut hits = self
            .manifest
            .items
            .iter()
            .map(|i| SearchHit {
                id: i.id.clone(),
                distance: i
                    .embedding
                    .iter()
                    .zip(q)
                    .map(|(a, b)| i64::from(*a).abs_diff(i64::from(*b)).pow(2))
                    .sum(),
            })
            .collect::<Vec<_>>();
        hits.sort_by(|a, b| a.distance.cmp(&b.distance).then(a.id.cmp(&b.id)));
        hits.truncate(limit);
        Ok(hits)
    }
}
pub struct AtomicIndexStore {
    active: IndexManifest,
    previous: Option<IndexManifest>,
}
impl AtomicIndexStore {
    pub fn activate(m: IndexManifest) -> Result<Self, VisualIndexError> {
        ExactVisualIndex::build(m.clone())?;
        Ok(Self {
            active: m,
            previous: None,
        })
    }
    pub fn rebuild(
        &mut self,
        candidate: Result<IndexManifest, VisualIndexError>,
    ) -> Result<(), VisualIndexError> {
        let candidate = candidate.map_err(|_| VisualIndexError::RebuildFailed)?;
        ExactVisualIndex::build(candidate.clone()).map_err(|_| VisualIndexError::RebuildFailed)?;
        self.previous = Some(std::mem::replace(&mut self.active, candidate));
        Ok(())
    }
    pub fn recover(&mut self) -> Result<(), VisualIndexError> {
        self.active = self
            .previous
            .take()
            .ok_or(VisualIndexError::RebuildFailed)?;
        Ok(())
    }
    pub fn active_digest(&self) -> [u8; 32] {
        self.active.digest()
    }
}

#[derive(Debug, Clone)]
pub struct GeoRegistration {
    pub method: String,
    pub error_m: u32,
}
#[derive(Debug, Clone)]
pub struct VerificationLabel {
    pub reviewer: String,
    pub method: String,
    pub confidence_bps: u16,
}
pub struct RetrievalCandidate {
    pub media_id: String,
    pub candidate_observation_id: String,
    pub similarity_bps: u16,
    georegistration: Option<GeoRegistration>,
    verification: Option<VerificationLabel>,
}
impl RetrievalCandidate {
    pub fn new(media: &str, obs: &str, similarity_bps: u16) -> Self {
        Self {
            media_id: media.into(),
            candidate_observation_id: obs.into(),
            similarity_bps,
            georegistration: None,
            verification: None,
        }
    }
    pub fn georegister(&mut self, g: GeoRegistration) {
        if !g.method.is_empty() {
            self.georegistration = Some(g);
        }
    }
    pub fn verify(&mut self, v: VerificationLabel) -> Result<(), VisualIndexError> {
        if self.georegistration.is_none()
            || v.reviewer.is_empty()
            || v.method.is_empty()
            || v.confidence_bps > 10_000
        {
            return Err(VisualIndexError::InvalidVerification);
        }
        self.verification = Some(v);
        Ok(())
    }
    pub fn is_verified_observation(&self) -> bool {
        self.georegistration.is_some() && self.verification.is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalIndexStatus {
    DisabledPendingEvaluation,
}
pub struct ExternalRupixel;
impl ExternalRupixel {
    pub fn status() -> ExternalIndexStatus {
        ExternalIndexStatus::DisabledPendingEvaluation
    }
}
