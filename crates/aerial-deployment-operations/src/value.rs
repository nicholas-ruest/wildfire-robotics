use crate::{DomainError, EvidenceId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRef {
    id: EvidenceId,
    digest: String,
    artifact_uri: String,
    valid_from: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
}
impl EvidenceRef {
    pub fn new(
        id: EvidenceId,
        digest: &str,
        artifact_uri: &str,
        valid_from: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Self, DomainError> {
        let valid_digest = digest.len() == 71
            && digest.starts_with("sha256:")
            && digest[7..]
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase());
        if !valid_digest {
            return Err(DomainError::InvalidDigest);
        }
        let artifact_uri = artifact_uri.trim();
        if artifact_uri.is_empty() || expires_at.is_some_and(|end| end <= valid_from) {
            return Err(DomainError::InvalidEvidence);
        }
        Ok(Self {
            id,
            digest: digest.into(),
            artifact_uri: artifact_uri.into(),
            valid_from,
            expires_at,
        })
    }
    #[must_use]
    pub fn is_current_at(&self, at: DateTime<Utc>) -> bool {
        at >= self.valid_from && self.expires_at.is_none_or(|end| at < end)
    }
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationScope {
    pub tenant: String,
    pub region: String,
    pub incident: String,
}
impl OperationScope {
    pub fn new(tenant: &str, region: &str, incident: &str) -> Result<Self, DomainError> {
        if [tenant, region, incident]
            .iter()
            .any(|v| v.trim().is_empty())
        {
            return Err(DomainError::InvalidScope);
        }
        Ok(Self {
            tenant: tenant.into(),
            region: region.into(),
            incident: incident.into(),
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualificationStage {
    Concept,
    CouponMaterial,
    Component,
    GroundMultiPanel,
    LowDrop,
    SubscaleExtraction,
    Sil,
    Hitl,
    InstrumentedRange,
    AircraftGroundExtraction,
    PartialScaleFlight,
    FullSystemCandidate,
    Suspended,
    Retired,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssemblyPhase {
    Planned,
    Assembling,
    Inspected,
    Packed,
    Deployed,
    Installed,
    Recovering,
    Recovered,
    Sacrificed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManifestPhase {
    Draft,
    Reconciled,
    LoadApproved,
    Loaded,
    Released,
    Retained,
    Accounted,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionPhase {
    Draft,
    Modeled,
    Reviewed,
    Authorized,
    Airborne,
    Completed,
    Aborted,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleasePhase {
    Requested,
    Checking,
    Armed,
    Released,
    Held,
    Aborted,
    Expired,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeploymentPhase {
    Retained,
    Extracted,
    Stabilized,
    CohortReleasing,
    ParafoilEstablished,
    FormationAcquired,
    SectionReefedRelease,
    TensionBalancedExpansion,
    TerrainAlignment,
    Landing,
    Landed,
    Isolated,
    Jettisoned,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallationPhase {
    Landed,
    Transitioning,
    Anchoring,
    Sealing,
    Active,
    Degraded,
    Recovering,
    Removed,
    TemporarilyLeft,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentDisposition {
    Pending,
    Recovered,
    Quarantined,
    Repair,
    Reuse,
    Recycle,
    Sacrificed,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    #[test]
    fn evidence_requires_sha256_and_positive_validity() {
        let id = EvidenceId::new("e-1").unwrap();
        let at = Utc.timestamp_opt(10, 0).single().unwrap();
        assert_eq!(
            EvidenceRef::new(id.clone(), "sha256:no", "object://e", at, None),
            Err(DomainError::InvalidDigest)
        );
        let digest = format!("sha256:{}", "a".repeat(64));
        assert!(EvidenceRef::new(id, &digest, "object://e", at, Some(at)).is_err());
    }
}
