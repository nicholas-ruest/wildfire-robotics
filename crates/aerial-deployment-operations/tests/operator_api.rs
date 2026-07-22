#![allow(missing_docs)]
#![allow(clippy::expect_used)]
use aerial_deployment_operations::*;
use chrono::{TimeZone, Utc};
use std::collections::{BTreeSet, HashMap};

fn scope() -> OperationScope {
    OperationScope::new("tenant-a", "ca-bc", "fire-7").expect("fixture")
}
fn auth() -> OperatorAuthorization {
    OperatorAuthorization {
        principal: "operator-1".into(),
        tenant: "tenant-a".into(),
        incidents: BTreeSet::from(["fire-7".into()]),
        permissions: BTreeSet::from([
            OperatorPermission::Read,
            OperatorPermission::Command,
            OperatorPermission::IrreversibleCommand,
            OperatorPermission::AuditReplay,
        ]),
    }
}
fn command() -> OperatorCommand {
    OperatorCommand {
        id: "cmd-1".into(),
        scope: scope(),
        resource_id: "deployment-1".into(),
        action: "release".into(),
        authority_lane: AuthorityLane::Aircraft,
        expected_version: 8,
        irreversible: true,
    }
}
#[derive(Default)]
struct MemoryAudit {
    rows: HashMap<String, Vec<AuditRecord>>,
    fail: bool,
}
impl ImmutableAuditPort for MemoryAudit {
    fn head(&self, scope: &OperationScope) -> Result<Option<AuditRecord>, DomainError> {
        Ok(self
            .rows
            .get(&scope.incident)
            .and_then(|r| r.last())
            .cloned())
    }
    fn append(&mut self, record: AuditRecord) -> Result<(), DomainError> {
        if self.fail {
            return Err(DomainError::InvalidEvidence);
        }
        self.rows
            .entry(record.scope.incident.clone())
            .or_default()
            .push(record);
        Ok(())
    }
    fn replay(&self, scope: &OperationScope) -> Result<Vec<AuditRecord>, DomainError> {
        Ok(self.rows.get(&scope.incident).cloned().unwrap_or_default())
    }
}
struct RejectBulk;
impl BulkArtifactPort for RejectBulk {
    fn put_immutable(
        &mut self,
        _: &OperationScope,
        _: &str,
        _: &[u8],
    ) -> Result<String, DomainError> {
        Err(DomainError::InvalidEvidence)
    }
}

#[test]
fn contract_exposes_every_required_safety_view_with_provenance() {
    let all = [
        ViewKind::QualificationMatrix,
        ViewKind::AssemblyManifest,
        ViewKind::LoadApproval,
        ViewKind::CorridorMap,
        ViewKind::ExclusionMap,
        ViewKind::DispersionMap,
        ViewKind::ReleaseChecklist,
        ViewKind::DualDecisions,
        ViewKind::DeploymentPhase,
        ViewKind::CohortStability,
        ViewKind::TetherTension,
        ViewKind::PanelHealth,
        ViewKind::Footprint,
        ViewKind::DegradedZones,
        ViewKind::UnaccountedComponents,
        ViewKind::Disposition,
    ];
    assert_eq!(all.len(), 16);
}

#[test]
fn authorization_is_tenant_and_incident_isolated() {
    let mut wrong_tenant = auth();
    wrong_tenant.tenant = "tenant-b".into();
    assert_eq!(
        wrong_tenant.authorize(&scope(), OperatorPermission::Read),
        Err(DomainError::OperatorForbidden)
    );
    let mut wrong_incident = auth();
    wrong_incident.incidents.clear();
    assert_eq!(
        wrong_incident.authorize(&scope(), OperatorPermission::Read),
        Err(DomainError::OperatorForbidden)
    );
}

#[test]
fn stale_view_fails_closed_even_for_authorized_operator() {
    let now = Utc.timestamp_opt(100, 0).single().expect("time");
    let digest = format!("sha256:{}", "a".repeat(64));
    let view = SafetyView {
        api_version: "v1".into(),
        scope: scope(),
        resource_id: "mission-1".into(),
        kind: ViewKind::ReleaseChecklist,
        aggregate_version: 4,
        provenance: SafetyProvenance {
            source_time: Utc.timestamp_opt(50, 0).single().expect("time"),
            expires_at: now,
            uncertainty_bps: 100,
            configuration_digest: digest.clone(),
            evidence_digest: digest,
            authority: "incident-command".into(),
        },
        value: "ready",
    };
    assert_eq!(
        view.authorize(&auth(), now),
        Err(DomainError::UnsafeOperatorView)
    );
}

#[test]
fn maps_require_explicit_crs84_and_valid_coordinates() {
    assert!(
        GeoView::new(
            "urn:ogc:def:crs:OGC::CRS84",
            vec![(-123_000_000, 49_000_000), (-122_000_000, 50_000_000)]
        )
        .is_ok()
    );
    assert_eq!(
        GeoView::new("EPSG:4326", vec![(0, 0), (1, 1)]),
        Err(DomainError::InvalidMapReference)
    );
}

#[test]
fn irreversible_action_requires_exact_digest_confirmation_and_detects_race() {
    let cmd = command();
    let mut audit = MemoryAudit::default();
    let at = Utc.timestamp_opt(100, 0).single().expect("time");
    assert_eq!(
        record_command(
            &mut audit,
            &auth(),
            &cmd,
            8,
            None,
            ActionOutcome::Requested,
            at
        ),
        Err(DomainError::ConfirmationRequired)
    );
    let confirmation = ExplicitConfirmation {
        command_digest: cmd.digest(),
        statement: "CONFIRM IRREVERSIBLE ACTION".into(),
    };
    assert_eq!(
        record_command(
            &mut audit,
            &auth(),
            &cmd,
            9,
            Some(&confirmation),
            ActionOutcome::Requested,
            at
        ),
        Err(DomainError::VersionConflict)
    );
}

#[test]
fn aircraft_and_incident_decisions_and_physical_outcomes_remain_separate() {
    let mut audit = MemoryAudit::default();
    let mut cmd = command();
    let confirmation = ExplicitConfirmation {
        command_digest: cmd.digest(),
        statement: "CONFIRM IRREVERSIBLE ACTION".into(),
    };
    let at = Utc.timestamp_opt(100, 0).single().expect("time");
    let aircraft = record_command(
        &mut audit,
        &auth(),
        &cmd,
        8,
        Some(&confirmation),
        ActionOutcome::Accepted,
        at,
    )
    .expect("aircraft");
    cmd.id = "cmd-2".into();
    cmd.authority_lane = AuthorityLane::Incident;
    cmd.irreversible = false;
    let incident = record_command(
        &mut audit,
        &auth(),
        &cmd,
        8,
        None,
        ActionOutcome::Observed,
        at,
    )
    .expect("incident");
    assert_ne!(
        (aircraft.authority_lane, aircraft.outcome),
        (incident.authority_lane, incident.outcome)
    );
    assert!(verify_audit_replay(
        &audit.replay(&scope()).expect("replay")
    ));
}

#[test]
fn audit_persistence_fails_closed_and_tamper_breaks_replay() {
    let cmd = command();
    let confirmation = ExplicitConfirmation {
        command_digest: cmd.digest(),
        statement: "CONFIRM IRREVERSIBLE ACTION".into(),
    };
    let at = Utc.timestamp_opt(100, 0).single().expect("time");
    let mut audit = MemoryAudit {
        fail: true,
        ..MemoryAudit::default()
    };
    assert_eq!(
        record_command(
            &mut audit,
            &auth(),
            &cmd,
            8,
            Some(&confirmation),
            ActionOutcome::Executed,
            at
        ),
        Err(DomainError::OperatorPersistenceFailed)
    );
    audit.fail = false;
    let record = record_command(
        &mut audit,
        &auth(),
        &cmd,
        8,
        Some(&confirmation),
        ActionOutcome::Executed,
        at,
    )
    .expect("record");
    let mut scope_tamper = record.clone();
    scope_tamper.scope.tenant = "other-tenant".into();
    assert!(!verify_audit_replay(&[scope_tamper]));
    let mut record = record;
    record.outcome = ActionOutcome::Failed;
    assert!(!verify_audit_replay(&[record]));
}

#[test]
fn bulk_artifact_owning_service_failure_is_closed() {
    assert_eq!(
        persist_bulk_artifact(&mut RejectBulk, &scope(), b"map"),
        Err(DomainError::OperatorPersistenceFailed)
    );
}
