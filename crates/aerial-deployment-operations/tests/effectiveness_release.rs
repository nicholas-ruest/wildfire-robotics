#![allow(clippy::expect_used, missing_docs)]
use aerial_deployment_operations::*;
use chrono::{DateTime, Duration, Utc};
use std::collections::BTreeSet;

const CONFIG: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const ARTIFACT: &str = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
fn now() -> DateTime<Utc> {
    DateTime::from_timestamp(1_700_000_000, 0).expect("valid time")
}
fn evidence(id: &str) -> EvidenceRef {
    EvidenceRef::new(
        EvidenceId::new(id).expect("valid id"),
        ARTIFACT,
        &format!("object://afb09/{id}"),
        now() - Duration::hours(1),
        Some(now() + Duration::hours(1)),
    )
    .expect("valid evidence")
}
fn study() -> EffectivenessStudy {
    EffectivenessStudy::new(
        CONFIG,
        400,
        1_800,
        "radiant heat and ember exposure profile v3",
        "all panels sealed; panel-8 thermocouple degraded",
        vec!["blanket deployed at t+15s".into()],
        "unprotected matched plot",
        "no-deployment model and instrumented control",
        85_000,
        vec!["subscale geometry; no aircraft wake".into()],
        vec!["two anchor excursions; one sensor loss".into()],
        "rear-face heat dose reduced relative to the stated counterfactual",
        evidence("study-1"),
    )
    .expect("complete study")
}
fn physical() -> BTreeSet<PhysicalEvidenceStage> {
    BTreeSet::from([
        PhysicalEvidenceStage::Material,
        PhysicalEvidenceStage::Ground,
        PhysicalEvidenceStage::LowDrop,
        PhysicalEvidenceStage::Subscale,
        PhysicalEvidenceStage::AircraftIntegration,
        PhysicalEvidenceStage::FlightTest,
        PhysicalEvidenceStage::ControlledFire,
    ])
}
fn dossier() -> ReleaseDossier {
    ReleaseDossier {
        configuration_digest: CONFIG.into(),
        qualification_stage: QualificationStage::FullSystemCandidate,
        signed_configuration_manifest: evidence("manifest"),
        software_gates: EvidenceDomain::REQUIRED_SOFTWARE
            .into_iter()
            .map(|domain| GateEvidence {
                domain,
                artifact: evidence(&format!("gate-{domain:?}")),
                configuration_digest: CONFIG.into(),
                passed: true,
                signer: "independent-software-assurance".into(),
            })
            .collect(),
        effectiveness_studies: vec![study()],
        traceability: (1..=11)
            .map(|n| TraceLink {
                adr: format!("ADR-{:03}", 68 + n.min(6)),
                invariant: format!("AD-INV-{n:03}"),
                aggregate_behavior: "fail-closed transition".into(),
                contract: format!("aerial.v1.Invariant{n}"),
                test: format!("effectiveness_release::invariant_{n}"),
                evidence_digest: ARTIFACT.into(),
            })
            .collect(),
        unresolved_software_blockers: Vec::new(),
        outstanding_physical_evidence: physical(),
        claims: BTreeSet::from([CandidateClaim::SoftwareReleaseCandidate]),
        next_physical_gate: NextPhysicalGate {
            stage: PhysicalEvidenceStage::Material,
            test_protocol: "MAT-QUAL-001 controlled coupon protocol".into(),
            independent_approver: "independent materials review board".into(),
            measurable_criteria: vec![
                "rear-face heat dose below protocol threshold".into(),
                "all adverse outcomes reconciled".into(),
            ],
        },
    }
}

#[test]
fn effectiveness_requires_counterfactual_uncertainty_limitations_and_negative_outcomes() {
    assert_eq!(
        EffectivenessStudy::new(
            CONFIG,
            400,
            1_800,
            "exposure",
            "panel state",
            vec!["intervention".into()],
            "baseline",
            "",
            1,
            vec!["limit".into()],
            vec!["none observed".into()],
            "outcome",
            evidence("bad")
        ),
        Err(DomainError::InvalidEffectivenessStudy)
    );
    assert_eq!(study().configuration_digest(), CONFIG);
}

#[test]
fn exact_complete_current_software_evidence_yields_only_a_software_candidate() {
    let decision = dossier().evaluate(now()).expect("complete exact dossier");
    assert!(decision.software_release_candidate);
    assert_eq!(decision.outstanding_physical_evidence, physical());
    assert!(decision.statement.contains("no material"));
}

#[test]
fn apparent_coverage_and_survival_cannot_substitute_for_effectiveness_evidence() {
    let mut candidate = dossier();
    candidate.effectiveness_studies.clear();
    assert_eq!(
        candidate.evaluate(now()),
        Err(DomainError::ReleaseEvidenceIncomplete)
    );
}

#[test]
fn fails_closed_for_missing_gate_wrong_configuration_stale_artifact_or_trace_gap() {
    let mut missing = dossier();
    missing.software_gates.pop();
    assert_eq!(
        missing.evaluate(now()),
        Err(DomainError::ReleaseEvidenceIncomplete)
    );
    let mut wrong = dossier();
    wrong.software_gates[0].configuration_digest = ARTIFACT.into();
    assert_eq!(
        wrong.evaluate(now()),
        Err(DomainError::ReleaseEvidenceIncomplete)
    );
    let mut stale = dossier();
    stale.signed_configuration_manifest = EvidenceRef::new(
        EvidenceId::new("stale").expect("id"),
        ARTIFACT,
        "object://stale",
        now() - Duration::hours(2),
        Some(now()),
    )
    .expect("ref");
    assert_eq!(
        stale.evaluate(now()),
        Err(DomainError::ReleaseEvidenceIncomplete)
    );
    let mut trace = dossier();
    trace.traceability[0].contract.clear();
    assert_eq!(
        trace.evaluate(now()),
        Err(DomainError::ReleaseEvidenceIncomplete)
    );
}

#[test]
fn blockers_and_unsupported_readiness_claims_inhibit_promotion() {
    let mut blocked = dossier();
    blocked.unresolved_software_blockers.push("SEC-42".into());
    assert_eq!(
        blocked.evaluate(now()),
        Err(DomainError::ReleasePromotionInhibited)
    );
    let mut overclaim = dossier();
    overclaim.claims.insert(CandidateClaim::AircraftApproved);
    assert_eq!(
        overclaim.evaluate(now()),
        Err(DomainError::ReleasePromotionInhibited)
    );
}

#[test]
fn next_physical_stage_requires_independent_measurable_approval_criteria() {
    let mut candidate = dossier();
    candidate.next_physical_gate.measurable_criteria.clear();
    assert_eq!(
        candidate.evaluate(now()),
        Err(DomainError::ReleaseEvidenceIncomplete)
    );
}

#[test]
fn duplicate_gates_or_invariants_cannot_mask_missing_coverage() {
    let mut gates = dossier();
    gates.software_gates.push(gates.software_gates[0].clone());
    assert_eq!(
        gates.evaluate(now()),
        Err(DomainError::ReleaseEvidenceIncomplete)
    );
    let mut trace = dossier();
    trace.traceability[10].invariant = "AD-INV-010".into();
    assert_eq!(
        trace.evaluate(now()),
        Err(DomainError::ReleaseEvidenceIncomplete)
    );
}

#[test]
fn digests_and_physical_criteria_are_canonical_and_fail_closed() {
    let mut digest = dossier();
    digest.configuration_digest =
        "sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".into();
    assert_eq!(
        digest.evaluate(now()),
        Err(DomainError::ReleaseEvidenceIncomplete)
    );
    let mut trace = dossier();
    trace.traceability[0].evidence_digest =
        "sha256:BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB".into();
    assert_eq!(
        trace.evaluate(now()),
        Err(DomainError::ReleaseEvidenceIncomplete)
    );
    let mut criteria = dossier();
    criteria.next_physical_gate.measurable_criteria[0] = " ".into();
    assert_eq!(
        criteria.evaluate(now()),
        Err(DomainError::ReleaseEvidenceIncomplete)
    );
}
