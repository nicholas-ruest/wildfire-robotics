#![allow(clippy::cast_possible_truncation, clippy::manual_checked_ops)]

use chrono::{DateTime, Datelike, Utc};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataKind {
    Lightning,
    Weather,
    Fuels,
    Terrain,
    Hotspots,
}
#[derive(Debug, Clone)]
pub struct SnapshotPart {
    pub kind: DataKind,
    pub source: String,
    pub digest: [u8; 32],
    pub authoritative: bool,
    pub licensed: bool,
}
impl SnapshotPart {
    pub fn authoritative(kind: DataKind, source: &str, digest: [u8; 32]) -> Self {
        Self {
            kind,
            source: source.into(),
            digest,
            authoritative: true,
            licensed: true,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LearningError {
    #[error("incomplete or non-authoritative snapshot")]
    InvalidSnapshot,
    #[error("data leakage: {0}")]
    Leakage(&'static str),
    #[error("invalid learning record")]
    InvalidRecord,
    #[error("evaluation evidence insufficient")]
    InsufficientEvidence,
}
#[derive(Debug, Clone)]
pub struct AuthoritativeSnapshot {
    id: String,
    parts: Vec<SnapshotPart>,
}
impl AuthoritativeSnapshot {
    pub fn assemble(id: &str, mut parts: Vec<SnapshotPart>) -> Result<Self, LearningError> {
        let kinds = parts.iter().map(|p| p.kind).collect::<BTreeSet<_>>();
        let required = BTreeSet::from([
            DataKind::Lightning,
            DataKind::Weather,
            DataKind::Fuels,
            DataKind::Terrain,
            DataKind::Hotspots,
        ]);
        if id.is_empty()
            || kinds != required
            || parts.iter().any(|p| {
                !p.authoritative || !p.licensed || p.source.is_empty() || p.digest == [0; 32]
            })
        {
            return Err(LearningError::InvalidSnapshot);
        }
        parts.sort_by_key(|p| p.kind);
        Ok(Self {
            id: id.into(),
            parts,
        })
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn parts(&self) -> &[SnapshotPart] {
        &self.parts
    }
}

#[derive(Debug, Clone)]
pub struct LightningFeatures {
    pub strike_observed: bool,
    pub age_minutes: u32,
    pub weather_dryness_bps: u16,
    pub fuel_receptivity_bps: u16,
    pub terrain_slope_bps: u16,
    pub hotspot_observed: bool,
}
#[derive(Debug, Clone)]
pub struct LearningRecord {
    pub id: String,
    pub incident_id: String,
    pub geography: String,
    pub fire_year: i32,
    pub event_time: DateTime<Utc>,
    pub features: LightningFeatures,
    pub ignition_outcome: Option<bool>,
    pub location_uncertainty_m: u32,
    pub time_uncertainty_minutes: u32,
    pub reconnaissance_selected: bool,
    pub censored: bool,
    pub intervention: Option<String>,
}
impl LearningRecord {
    pub fn validate(&self) -> Result<(), LearningError> {
        if self.id.is_empty()
            || self.incident_id.is_empty()
            || self.geography.is_empty()
            || self.fire_year != self.event_time.year()
            || self.features.weather_dryness_bps > 10_000
            || self.features.fuel_receptivity_bps > 10_000
            || self.features.terrain_slope_bps > 10_000
            || self.censored != self.ignition_outcome.is_none()
        {
            return Err(LearningError::InvalidRecord);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BaselineThresholds {
    pub abstain_location_m: u32,
    pub abstain_time_minutes: u32,
}
impl Default for BaselineThresholds {
    fn default() -> Self {
        Self {
            abstain_location_m: 200,
            abstain_time_minutes: 10,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselinePrediction {
    pub holdover_probability_bps: u16,
    pub location_uncertainty_m: u32,
    pub time_uncertainty_minutes: u32,
    pub reconnaissance_value_bps: u16,
    pub abstained: bool,
    pub explanation: String,
    pub artifact_digest: [u8; 32],
}
pub trait IgnitionInference {
    fn infer(
        &self,
        f: &LightningFeatures,
        location_uncertainty_m: u32,
        time_uncertainty_minutes: u32,
    ) -> BaselinePrediction;
}
pub struct OperationalBaseline {
    digest: [u8; 32],
    thresholds: BaselineThresholds,
}
impl OperationalBaseline {
    pub fn new(digest: [u8; 32], thresholds: BaselineThresholds) -> Self {
        Self { digest, thresholds }
    }
}
impl IgnitionInference for OperationalBaseline {
    fn infer(&self, f: &LightningFeatures, l: u32, t: u32) -> BaselinePrediction {
        let mut score = u32::from(f.weather_dryness_bps) * 35 / 100
            + u32::from(f.fuel_receptivity_bps) * 35 / 100
            + u32::from(f.terrain_slope_bps) * 10 / 100
            + u32::from(f.hotspot_observed) * 1500
            + u32::from(f.strike_observed) * 500;
        score = score
            .saturating_sub(f.age_minutes.min(1440) * 2)
            .min(10_000);
        let abstained =
            l > self.thresholds.abstain_location_m || t > self.thresholds.abstain_time_minutes;
        BaselinePrediction {
            holdover_probability_bps: score as u16,
            location_uncertainty_m: l,
            time_uncertainty_minutes: t,
            reconnaissance_value_bps: if abstained {
                9000
            } else {
                (10_000 - score.abs_diff(5000) * 2).min(10_000) as u16
            },
            abstained,
            explanation: format!(
                "transparent baseline: dryness={}, fuel={}, slope={}, hotspot={}",
                f.weather_dryness_bps,
                f.fuel_receptivity_bps,
                f.terrain_slope_bps,
                f.hotspot_observed
            ),
            artifact_digest: self.digest,
        }
    }
}
pub trait CandidateInference {
    fn predict(&self, record: &LearningRecord) -> Result<BaselinePrediction, LearningError>;
}

pub struct LeakageGuard;
impl LeakageGuard {
    pub fn validate(
        train: &[LearningRecord],
        test: &[LearningRecord],
    ) -> Result<(), LearningError> {
        for a in train {
            for b in test {
                if a.incident_id == b.incident_id {
                    return Err(LearningError::Leakage("incident"));
                }
                if a.geography == b.geography {
                    return Err(LearningError::Leakage("geography"));
                }
                if a.fire_year == b.fire_year {
                    return Err(LearningError::Leakage("fire-year"));
                }
                if a.event_time.date_naive() == b.event_time.date_naive() {
                    return Err(LearningError::Leakage("time"));
                }
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy)]
pub struct PredictionOutcome {
    pub probability_bps: u16,
    pub outcome: bool,
}
#[derive(Debug, Clone, Copy)]
pub struct RareEventMetrics {
    pub precision_bps: u16,
    pub recall_bps: u16,
    pub brier_bps: u16,
}
#[derive(Debug, Clone, Copy)]
pub struct ShadowEvidence {
    pub prospective_cases: u64,
    pub observed_ignitions: u64,
    pub baseline_brier_bps: u16,
    pub candidate_brier_bps: u16,
}
pub struct LightningEvaluation {
    metrics: RareEventMetrics,
    calibration: Option<([u8; 32], u16)>,
    shadow: Option<ShadowEvidence>,
}
impl LightningEvaluation {
    pub fn evaluate(rows: &[PredictionOutcome]) -> Result<Self, LearningError> {
        if rows.is_empty() || rows.iter().any(|r| r.probability_bps > 10_000) {
            return Err(LearningError::InsufficientEvidence);
        }
        let tp = rows
            .iter()
            .filter(|r| r.outcome && r.probability_bps >= 5000)
            .count() as u32;
        let predicted = rows.iter().filter(|r| r.probability_bps >= 5000).count() as u32;
        let actual = rows.iter().filter(|r| r.outcome).count() as u32;
        let brier = rows
            .iter()
            .map(|r| {
                let y = if r.outcome { 10_000i64 } else { 0 };
                (i64::from(r.probability_bps) - y).unsigned_abs().pow(2)
            })
            .sum::<u64>()
            / rows.len() as u64
            / 10_000;
        Ok(Self {
            metrics: RareEventMetrics {
                precision_bps: if predicted == 0 {
                    0
                } else {
                    (tp * 10_000 / predicted) as u16
                },
                recall_bps: if actual == 0 {
                    0
                } else {
                    (tp * 10_000 / actual) as u16
                },
                brier_bps: brier.min(10_000) as u16,
            },
            calibration: None,
            shadow: None,
        })
    }
    pub fn attach_calibration(&mut self, d: [u8; 32], error_bps: u16) {
        if d != [0; 32] && error_bps <= 10_000 {
            self.calibration = Some((d, error_bps));
        }
    }
    pub fn record_shadow(&mut self, e: ShadowEvidence) {
        self.shadow = Some(e);
    }
    pub fn can_promote(&self) -> bool {
        self.calibration.is_some()
            && self.shadow.is_some_and(|s| {
                s.prospective_cases >= 100
                    && s.observed_ignitions > 0
                    && s.candidate_brier_bps < s.baseline_brier_bps
            })
            && self.metrics.precision_bps > 0
    }
    pub fn metrics(&self) -> RareEventMetrics {
        self.metrics
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitoringDecision {
    Continue,
    Withdraw,
}
pub struct ModelMonitor {
    lineage: [u8; 32],
    drift_limit_bps: u16,
}
impl ModelMonitor {
    pub fn new(lineage: [u8; 32], drift_limit_bps: u16) -> Self {
        Self {
            lineage,
            drift_limit_bps,
        }
    }
    pub fn observe(&mut self, drift_bps: u16, lineage: [u8; 32]) -> MonitoringDecision {
        if drift_bps > self.drift_limit_bps || lineage != self.lineage {
            MonitoringDecision::Withdraw
        } else {
            MonitoringDecision::Continue
        }
    }
}
