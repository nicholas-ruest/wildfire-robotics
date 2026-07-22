//! Effectiveness evidence and fail-closed release-candidate assessment (AFB-09).
use crate::{DomainError, EvidenceRef, QualificationStage};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

fn digest(value: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(value.as_bytes()))
}

fn valid_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceDomain {
    Domain,
    Contract,
    Replay,
    Security,
    Recovery,
    Simulation,
    Fault,
    Performance,
    Endurance,
    Sbom,
    Provenance,
    CapacitySlo,
    RiskRegister,
}
impl EvidenceDomain {
    pub const REQUIRED_SOFTWARE: [Self; 13] = [
        Self::Domain,
        Self::Contract,
        Self::Replay,
        Self::Security,
        Self::Recovery,
        Self::Simulation,
        Self::Fault,
        Self::Performance,
        Self::Endurance,
        Self::Sbom,
        Self::Provenance,
        Self::CapacitySlo,
        Self::RiskRegister,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PhysicalEvidenceStage {
    Material,
    Ground,
    LowDrop,
    Subscale,
    AircraftIntegration,
    FlightTest,
    ControlledFire,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectivenessStudy {
    configuration_digest: String,
    pub protected_area_m2: u64,
    pub protected_duration_seconds: u64,
    pub exposure_conditions: String,
    pub panel_state: String,
    pub interventions: Vec<String>,
    pub baseline: String,
    pub counterfactual: String,
    pub uncertainty_ppm: u32,
    pub limitations: Vec<String>,
    pub negative_outcomes: Vec<String>,
    pub outcome: String,
    artifact: EvidenceRef,
}
impl EffectivenessStudy {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        configuration_digest: &str,
        protected_area_m2: u64,
        protected_duration_seconds: u64,
        exposure_conditions: &str,
        panel_state: &str,
        interventions: Vec<String>,
        baseline: &str,
        counterfactual: &str,
        uncertainty_ppm: u32,
        limitations: Vec<String>,
        negative_outcomes: Vec<String>,
        outcome: &str,
        artifact: EvidenceRef,
    ) -> Result<Self, DomainError> {
        let required_text = [
            exposure_conditions,
            panel_state,
            baseline,
            counterfactual,
            outcome,
        ];
        if !valid_digest(configuration_digest)
            || protected_area_m2 == 0
            || protected_duration_seconds == 0
            || uncertainty_ppm > 1_000_000
            || required_text.iter().any(|v| v.trim().is_empty())
            || interventions.is_empty()
            || limitations.is_empty()
            || negative_outcomes.is_empty()
            || interventions
                .iter()
                .chain(&limitations)
                .chain(&negative_outcomes)
                .any(|v| v.trim().is_empty())
        {
            return Err(DomainError::InvalidEffectivenessStudy);
        }
        Ok(Self {
            configuration_digest: configuration_digest.into(),
            protected_area_m2,
            protected_duration_seconds,
            exposure_conditions: exposure_conditions.trim().into(),
            panel_state: panel_state.trim().into(),
            interventions,
            baseline: baseline.trim().into(),
            counterfactual: counterfactual.trim().into(),
            uncertainty_ppm,
            limitations,
            negative_outcomes,
            outcome: outcome.trim().into(),
            artifact,
        })
    }
    #[must_use]
    pub fn configuration_digest(&self) -> &str {
        &self.configuration_digest
    }
    #[must_use]
    pub fn artifact(&self) -> &EvidenceRef {
        &self.artifact
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceLink {
    pub adr: String,
    pub invariant: String,
    pub aggregate_behavior: String,
    pub contract: String,
    pub test: String,
    pub evidence_digest: String,
}
impl TraceLink {
    #[must_use]
    pub fn complete(&self) -> bool {
        [
            &self.adr,
            &self.invariant,
            &self.aggregate_behavior,
            &self.contract,
            &self.test,
        ]
        .iter()
        .all(|v| !v.trim().is_empty())
            && valid_digest(&self.evidence_digest)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateEvidence {
    pub domain: EvidenceDomain,
    pub artifact: EvidenceRef,
    pub configuration_digest: String,
    pub passed: bool,
    pub signer: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CandidateClaim {
    SoftwareReleaseCandidate,
    Fireproof,
    Operational,
    AircraftApproved,
    CommerciallyReady,
    ProductionReady,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextPhysicalGate {
    pub stage: PhysicalEvidenceStage,
    pub test_protocol: String,
    pub independent_approver: String,
    pub measurable_criteria: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseDossier {
    pub configuration_digest: String,
    pub qualification_stage: QualificationStage,
    pub signed_configuration_manifest: EvidenceRef,
    pub software_gates: Vec<GateEvidence>,
    pub effectiveness_studies: Vec<EffectivenessStudy>,
    pub traceability: Vec<TraceLink>,
    pub unresolved_software_blockers: Vec<String>,
    pub outstanding_physical_evidence: BTreeSet<PhysicalEvidenceStage>,
    pub claims: BTreeSet<CandidateClaim>,
    pub next_physical_gate: NextPhysicalGate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateDecision {
    pub candidate_digest: String,
    pub software_release_candidate: bool,
    pub outstanding_physical_evidence: BTreeSet<PhysicalEvidenceStage>,
    pub statement: String,
}

impl ReleaseDossier {
    pub fn evaluate(&self, at: DateTime<Utc>) -> Result<CandidateDecision, DomainError> {
        let required = BTreeSet::from(EvidenceDomain::REQUIRED_SOFTWARE);
        let supplied = self
            .software_gates
            .iter()
            .map(|g| g.domain)
            .collect::<BTreeSet<_>>();
        let required_invariants = (1..=11)
            .map(|number| format!("AD-INV-{number:03}"))
            .collect::<BTreeSet<_>>();
        let supplied_invariants = self
            .traceability
            .iter()
            .map(|link| link.invariant.clone())
            .collect::<BTreeSet<_>>();
        let all_physical = BTreeSet::from([
            PhysicalEvidenceStage::Material,
            PhysicalEvidenceStage::Ground,
            PhysicalEvidenceStage::LowDrop,
            PhysicalEvidenceStage::Subscale,
            PhysicalEvidenceStage::AircraftIntegration,
            PhysicalEvidenceStage::FlightTest,
            PhysicalEvidenceStage::ControlledFire,
        ]);
        if !valid_digest(&self.configuration_digest)
            || self.qualification_stage != QualificationStage::FullSystemCandidate
            || !self.signed_configuration_manifest.is_current_at(at)
            || self.software_gates.iter().any(|g| {
                !g.passed
                    || g.configuration_digest != self.configuration_digest
                    || g.signer.trim().is_empty()
                    || !g.artifact.is_current_at(at)
            })
            || supplied != required
            || self.software_gates.len() != required.len()
            || self.effectiveness_studies.is_empty()
            || self.effectiveness_studies.iter().any(|s| {
                s.configuration_digest() != self.configuration_digest
                    || !s.artifact().is_current_at(at)
            })
            || self.traceability.len() != required_invariants.len()
            || self.traceability.iter().any(|t| !t.complete())
            || supplied_invariants != required_invariants
            || self.outstanding_physical_evidence != all_physical
            || !self
                .outstanding_physical_evidence
                .contains(&self.next_physical_gate.stage)
            || self.next_physical_gate.test_protocol.trim().is_empty()
            || self
                .next_physical_gate
                .independent_approver
                .trim()
                .is_empty()
            || self.next_physical_gate.measurable_criteria.is_empty()
            || self
                .next_physical_gate
                .measurable_criteria
                .iter()
                .any(|criterion| criterion.trim().is_empty())
        {
            return Err(DomainError::ReleaseEvidenceIncomplete);
        }
        if !self.unresolved_software_blockers.is_empty()
            || self
                .claims
                .iter()
                .any(|claim| *claim != CandidateClaim::SoftwareReleaseCandidate)
        {
            return Err(DomainError::ReleasePromotionInhibited);
        }
        Ok(CandidateDecision {
            candidate_digest: digest(&format!("{}:{:?}:{:?}", self.configuration_digest, self.software_gates, self.traceability)),
            software_release_candidate: true,
            outstanding_physical_evidence: self.outstanding_physical_evidence.clone(),
            statement: "Software release candidate only; no material, operational, aircraft, commercial, production, or fireproof approval is conferred.".into(),
        })
    }
}
