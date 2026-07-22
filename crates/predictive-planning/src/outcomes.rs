use sha2::{Digest as _, Sha256};
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OutcomeError {
    #[error("invalid dataset")]
    InvalidDataset,
    #[error("data leakage: {0}")]
    Leakage(&'static str),
}
#[derive(Debug, Clone)]
pub struct AlignedOutcome {
    pub prediction_id: String,
    pub evidence_id: String,
    pub incident: String,
    pub geography: String,
    pub fire_year: i32,
    pub outcome: Option<bool>,
    pub selected: bool,
    pub sampling_policy: String,
    pub intervention: Option<String>,
    pub censored: bool,
}
impl AlignedOutcome {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &str,
        evidence_id: &str,
        incident: &str,
        geo: &str,
        year: i32,
        outcome: Option<bool>,
        selected: bool,
        sampling: &str,
        intervention: Option<&str>,
    ) -> Result<Self, OutcomeError> {
        if id.is_empty()
            || evidence_id.is_empty()
            || incident.is_empty()
            || geo.is_empty()
            || sampling.is_empty()
        {
            return Err(OutcomeError::InvalidDataset);
        }
        Ok(Self {
            prediction_id: id.into(),
            evidence_id: evidence_id.into(),
            incident: incident.into(),
            geography: geo.into(),
            fire_year: year,
            outcome,
            selected,
            sampling_policy: sampling.into(),
            intervention: intervention.map(Into::into),
            censored: outcome.is_none(),
        })
    }
}
#[derive(Debug, Clone)]
pub struct DatasetSnapshot {
    id: String,
    rows: Vec<AlignedOutcome>,
    digest: [u8; 32],
}
impl DatasetSnapshot {
    pub fn freeze(
        id: &str,
        train: &[AlignedOutcome],
        test: &[AlignedOutcome],
    ) -> Result<Self, OutcomeError> {
        for a in train {
            for b in test {
                if a.incident == b.incident {
                    return Err(OutcomeError::Leakage("incident"));
                }
                if a.geography == b.geography {
                    return Err(OutcomeError::Leakage("geography"));
                }
                if a.fire_year == b.fire_year {
                    return Err(OutcomeError::Leakage("fire-year"));
                }
            }
        }
        if id.is_empty() || train.is_empty() || test.is_empty() {
            return Err(OutcomeError::InvalidDataset);
        }
        let rows = train.iter().chain(test).cloned().collect::<Vec<_>>();
        let mut h = Sha256::new();
        h.update(id);
        for r in &rows {
            h.update(&r.prediction_id);
            h.update(&r.evidence_id);
            h.update(&r.incident);
            h.update(&r.geography);
            h.update(r.fire_year.to_be_bytes());
            h.update([
                u8::from(r.outcome.unwrap_or(false)),
                u8::from(r.censored),
                u8::from(r.selected),
            ]);
            h.update(&r.sampling_policy);
            if let Some(i) = &r.intervention {
                h.update(i);
            }
        }
        Ok(Self {
            id: id.into(),
            rows,
            digest: h.finalize().into(),
        })
    }
    pub fn digest(&self) -> [u8; 32] {
        self.digest
    }
    pub fn rows(&self) -> &[AlignedOutcome] {
        &self.rows
    }
    pub fn id(&self) -> &str {
        &self.id
    }
}
