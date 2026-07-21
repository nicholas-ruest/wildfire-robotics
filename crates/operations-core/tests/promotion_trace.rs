//! Outside-in promotion-query acceptance tests.

use chrono::{DateTime, Duration, Utc};
use operations_core::traceability::{
    Approval, ApprovalKind, ArtifactKind, DeploymentRecord, EvidenceGraph, EvidenceLink,
    EvidenceStatus, GovernedArtifact, PromotionRequest, TraceabilityError,
};

fn now() -> DateTime<Utc> {
    DateTime::<Utc>::UNIX_EPOCH + Duration::days(1)
}

fn artifact(id: &str, kind: ArtifactKind) -> Result<GovernedArtifact, TraceabilityError> {
    GovernedArtifact::new(id, kind, [7; 32], now())
}

fn complete_graph() -> Result<EvidenceGraph, TraceabilityError> {
    let mut graph = EvidenceGraph::new();
    for item in [
        artifact("REL-1", ArtifactKind::Release)?,
        artifact("SRC-1", ArtifactKind::Source)?,
        artifact("BIN-1", ArtifactKind::BuildArtifact)?,
        artifact("CFG-1", ArtifactKind::Configuration)?,
        artifact("CAP-1", ArtifactKind::Capability)?,
        artifact("REQ-1", ArtifactKind::Requirement)?,
        artifact("INV-1", ArtifactKind::Invariant)?,
        artifact("HAZ-1", ArtifactKind::Hazard)?,
        artifact("THR-1", ArtifactKind::Threat)?,
        artifact("ADR-1", ArtifactKind::Decision)?,
        artifact("TST-1", ArtifactKind::Test)?,
        artifact("EVD-1", ArtifactKind::Evidence)?,
        artifact("ODD-1", ArtifactKind::OperationalDomain)?,
        artifact("SBOM-1", ArtifactKind::Sbom)?,
        artifact("DEP-1", ArtifactKind::Deployment)?,
    ] {
        graph.insert_artifact(item)?;
    }

    for target in [
        "SRC-1", "BIN-1", "CFG-1", "CAP-1", "REQ-1", "INV-1", "HAZ-1", "THR-1", "ADR-1", "TST-1",
        "EVD-1", "ODD-1", "SBOM-1", "DEP-1",
    ] {
        graph.insert_link(EvidenceLink::current("REL-1", target, now()))?;
    }
    graph.insert_approval(Approval::new(
        "APR-1",
        "REL-1",
        "human:safety-reviewer",
        ApprovalKind::IndependentSafety,
        "release-authority",
        now(),
        now() + Duration::days(30),
    )?)?;
    Ok(graph)
}

#[test]
fn should_answer_promotion_query_when_release_trace_is_complete() -> Result<(), TraceabilityError> {
    let graph = complete_graph()?;
    let decision = graph.evaluate_promotion(&PromotionRequest {
        release_id: "REL-1".into(),
        configuration_id: "CFG-1".into(),
        capability_id: "CAP-1".into(),
        odd_id: "ODD-1".into(),
        hardware_ids: vec!["HW-1".into()],
        authority: "release-authority".into(),
        evaluated_at: now() + Duration::hours(1),
    })?;
    assert_eq!(decision.release_id, "REL-1");
    Ok(())
}

#[test]
fn should_fail_closed_when_evidence_is_stale_or_contradictory() -> Result<(), TraceabilityError> {
    let mut stale = complete_graph()?;
    stale.set_link_status("REL-1", "EVD-1", EvidenceStatus::Stale)?;
    assert!(matches!(
        stale.evaluate_promotion(&PromotionRequest::fixture(
            "REL-1",
            "CFG-1",
            "CAP-1",
            "ODD-1",
            "release-authority",
            now()
        )),
        Err(TraceabilityError::StaleLink { .. })
    ));

    let mut contradictory = complete_graph()?;
    contradictory.set_link_status("REL-1", "TST-1", EvidenceStatus::Contradictory)?;
    assert!(matches!(
        contradictory.evaluate_promotion(&PromotionRequest::fixture(
            "REL-1",
            "CFG-1",
            "CAP-1",
            "ODD-1",
            "release-authority",
            now()
        )),
        Err(TraceabilityError::ContradictoryLink { .. })
    ));
    Ok(())
}

#[test]
fn should_fail_closed_when_link_or_independent_approval_is_missing() -> Result<(), TraceabilityError>
{
    let mut missing = complete_graph()?;
    missing.set_link_status("REL-1", "REQ-1", EvidenceStatus::Revoked)?;
    assert!(matches!(
        missing.evaluate_promotion(&PromotionRequest::fixture(
            "REL-1",
            "CFG-1",
            "CAP-1",
            "ODD-1",
            "release-authority",
            now()
        )),
        Err(TraceabilityError::RevokedLink { .. })
    ));

    let graph = complete_graph()?;
    assert!(matches!(
        graph.evaluate_promotion(&PromotionRequest::fixture(
            "REL-1",
            "CFG-1",
            "CAP-1",
            "ODD-1",
            "different-authority",
            now()
        )),
        Err(TraceabilityError::Unapproved(_))
    ));
    Ok(())
}

#[test]
fn should_reject_mutation_of_immutable_deployment_record() -> Result<(), TraceabilityError> {
    let record = DeploymentRecord::new(
        "DEP-1",
        "REL-1",
        "CFG-1",
        "production-ca",
        "tenant-1",
        "human:operator",
        now(),
        [9; 32],
    )?;
    assert!(matches!(
        record.verify_identity([8; 32]),
        Err(TraceabilityError::DigestMismatch { .. })
    ));
    Ok(())
}
