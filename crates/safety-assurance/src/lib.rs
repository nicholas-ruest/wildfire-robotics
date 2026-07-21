#![forbid(unsafe_code)]
//! Safety assurance domain: operational design domains, constraints, and promotion evidence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_kernel::{EntityId, TimeWindow};
use std::collections::BTreeSet;
use thiserror::Error;

/// Conditions under which a capability has been verified.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OperationalDesignDomain {
    terrains: BTreeSet<String>,
    weather: BTreeSet<String>,
    maximum_wind_kph: u16,
    requires_positioning: bool,
    allows_disconnected_operation: bool,
}

impl OperationalDesignDomain {
    /// Builds a validated operational design domain.
    pub fn new(
        terrains: impl IntoIterator<Item = String>,
        weather: impl IntoIterator<Item = String>,
        maximum_wind_kph: u16,
        requires_positioning: bool,
        allows_disconnected_operation: bool,
    ) -> Result<Self, SafetyError> {
        let terrains = normalize_set(terrains)?;
        let weather = normalize_set(weather)?;
        if maximum_wind_kph == 0 || maximum_wind_kph > 250 {
            return Err(SafetyError::InvalidWindLimit);
        }
        Ok(Self {
            terrains,
            weather,
            maximum_wind_kph,
            requires_positioning,
            allows_disconnected_operation,
        })
    }

    /// Returns true only when this verified ODD wholly contains the requested conditions.
    #[must_use]
    pub fn contains(&self, requested: &Self) -> bool {
        requested.terrains.is_subset(&self.terrains)
            && requested.weather.is_subset(&self.weather)
            && requested.maximum_wind_kph <= self.maximum_wind_kph
            && (!self.requires_positioning || requested.requires_positioning)
            && (self.allows_disconnected_operation || !requested.allows_disconnected_operation)
    }
}

fn normalize_set(
    values: impl IntoIterator<Item = String>,
) -> Result<BTreeSet<String>, SafetyError> {
    let values: BTreeSet<_> = values
        .into_iter()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .collect();
    if values.is_empty() {
        return Err(SafetyError::EmptyConditionSet);
    }
    Ok(values)
}

/// A signed and time-limited runtime safety constraint.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SafetyConstraint {
    /// Stable constraint identity.
    pub id: EntityId,
    /// Monotonically increasing policy version.
    pub version: u64,
    /// Operational interval.
    pub validity: TimeWindow,
    /// Identity of the approving safety authority.
    pub approved_by: EntityId,
    /// Detached signature reference/checksum, never secret key material.
    pub signature: String,
}

impl SafetyConstraint {
    /// Creates an enforceable constraint. Unsigned or zero-version constraints are rejected.
    pub fn new(
        id: EntityId,
        version: u64,
        validity: TimeWindow,
        approved_by: EntityId,
        signature: impl Into<String>,
    ) -> Result<Self, SafetyError> {
        let signature = signature.into();
        if version == 0 {
            return Err(SafetyError::InvalidConstraintVersion);
        }
        if signature.trim().len() < 16 {
            return Err(SafetyError::MissingSignature);
        }
        Ok(Self {
            id,
            version,
            validity,
            approved_by,
            signature,
        })
    }

    /// Enforces closed-open expiry semantics.
    #[must_use]
    pub fn is_active_at(&self, now: DateTime<Utc>) -> bool {
        self.validity.contains(now)
    }
}

/// Required independent evidence categories for physical promotion.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum EvidenceKind {
    /// Requirements are traced to tests.
    RequirementsTrace,
    /// Hazard controls have verified tests.
    HazardMitigation,
    /// Deterministic scenario simulation passed.
    Simulation,
    /// Hardware-in-the-loop validation passed.
    HardwareInLoop,
    /// Security review and artifact evidence passed.
    SecurityReview,
    /// Named independent safety approver signed.
    IndependentApproval,
}

/// Evidence case controlling release promotion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceCase {
    release_id: EntityId,
    evidence: BTreeSet<EvidenceKind>,
    open_blocking_findings: u32,
    near_miss_after_evidence: bool,
}

impl EvidenceCase {
    /// Starts an empty, non-promotable evidence case.
    #[must_use]
    pub fn new(release_id: EntityId) -> Self {
        Self {
            release_id,
            evidence: BTreeSet::new(),
            open_blocking_findings: 0,
            near_miss_after_evidence: false,
        }
    }

    /// Adds independently persisted evidence.
    pub fn record(&mut self, kind: EvidenceKind) {
        self.evidence.insert(kind);
    }

    /// Updates the authoritative number of release-blocking findings.
    pub fn set_open_blocking_findings(&mut self, count: u32) {
        self.open_blocking_findings = count;
    }

    /// A new near miss invalidates promotion until the case is reviewed and rebuilt.
    pub fn report_near_miss(&mut self) {
        self.near_miss_after_evidence = true;
    }

    /// Promotes only a complete, clean evidence case.
    pub fn approve(self) -> Result<Promotion, SafetyError> {
        let required = BTreeSet::from([
            EvidenceKind::RequirementsTrace,
            EvidenceKind::HazardMitigation,
            EvidenceKind::Simulation,
            EvidenceKind::HardwareInLoop,
            EvidenceKind::SecurityReview,
            EvidenceKind::IndependentApproval,
        ]);
        if self.evidence != required {
            return Err(SafetyError::IncompleteEvidence);
        }
        if self.open_blocking_findings > 0 {
            return Err(SafetyError::BlockingFindings);
        }
        if self.near_miss_after_evidence {
            return Err(SafetyError::NearMissRequiresReview);
        }
        Ok(Promotion {
            release_id: self.release_id,
        })
    }
}

/// Proof that a release passed the domain promotion invariant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Promotion {
    /// Promoted release.
    pub release_id: EntityId,
}

/// Safety domain failures. Callers must fail closed on every variant.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum SafetyError {
    /// Terrain or weather conditions were absent.
    #[error("operational design domain condition sets cannot be empty")]
    EmptyConditionSet,
    /// Wind limit is zero or physically implausible.
    #[error("maximum wind must be between 1 and 250 km/h")]
    InvalidWindLimit,
    /// Constraint versions begin at one.
    #[error("constraint version must be positive")]
    InvalidConstraintVersion,
    /// Constraint lacks a durable signature reference.
    #[error("constraint must include a signature")]
    MissingSignature,
    /// Required evidence categories are missing or contain unrecognized state.
    #[error("promotion evidence is incomplete")]
    IncompleteEvidence,
    /// Release-blocking findings remain open.
    #[error("promotion has blocking findings")]
    BlockingFindings,
    /// A near miss requires safety review before promotion.
    #[error("near miss requires evidence review")]
    NearMissRequiresReview,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn odd(
        terrains: &[&str],
        wind: u16,
        disconnected: bool,
    ) -> Result<OperationalDesignDomain, SafetyError> {
        OperationalDesignDomain::new(
            terrains.iter().map(|value| (*value).to_owned()),
            ["clear".to_owned(), "smoke".to_owned()],
            wind,
            true,
            disconnected,
        )
    }

    #[test]
    fn broader_verified_odd_contains_narrower_request() -> Result<(), SafetyError> {
        assert!(odd(&["road", "forest"], 60, true)?.contains(&odd(&["forest"], 30, false)?));
        Ok(())
    }

    #[test]
    fn connected_only_odd_rejects_disconnected_request() -> Result<(), SafetyError> {
        assert!(!odd(&["forest"], 60, false)?.contains(&odd(&["forest"], 30, true)?));
        Ok(())
    }

    #[test]
    fn evidence_case_fails_closed_when_incomplete() {
        let mut case = EvidenceCase::new(EntityId::new());
        case.record(EvidenceKind::Simulation);
        assert_eq!(case.approve(), Err(SafetyError::IncompleteEvidence));
    }

    #[test]
    fn near_miss_blocks_otherwise_complete_case() {
        let mut case = EvidenceCase::new(EntityId::new());
        for kind in [
            EvidenceKind::RequirementsTrace,
            EvidenceKind::HazardMitigation,
            EvidenceKind::Simulation,
            EvidenceKind::HardwareInLoop,
            EvidenceKind::SecurityReview,
            EvidenceKind::IndependentApproval,
        ] {
            case.record(kind);
        }
        case.report_near_miss();
        assert_eq!(case.approve(), Err(SafetyError::NearMissRequiresReview));
    }
}
