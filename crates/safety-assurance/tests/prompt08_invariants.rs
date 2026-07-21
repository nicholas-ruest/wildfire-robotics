//! Prompt 08 executable acceptance and property tests for SA-INV-001 through SA-INV-005.
#![allow(clippy::expect_used)]

use chrono::{DateTime, Duration, Utc};
use proptest::prelude::*;
use safety_assurance::*;
use shared_kernel::{EntityId, TimeWindow};

fn now() -> DateTime<Utc> {
    DateTime::<Utc>::UNIX_EPOCH + Duration::days(10)
}

struct Competent;
impl CompetencyPort for Competent {
    type Error = ();
    fn is_competent(
        &self,
        _: &EntityId,
        competency: &str,
        _: &str,
        _: DateTime<Utc>,
    ) -> Result<bool, Self::Error> {
        Ok(!competency.is_empty())
    }
}

fn odd() -> OperationalDesignDomain {
    OperationalDesignDomain::new(
        ["forest".into(), "road".into()],
        ["clear".into()],
        60,
        true,
        true,
    )
    .expect("fixture ODD")
}

fn scope(odd: &OperationalDesignDomain) -> PromotionScope {
    PromotionScope::new(
        [1; 32],
        [2; 32],
        [[3; 32]],
        "suppression",
        1,
        odd.id().clone(),
        1,
    )
    .expect("fixture scope")
}

fn accepted_hazard() -> Hazard {
    let instant = now();
    let mut hazard = Hazard::register(
        EntityId::new(),
        EntityId::new(),
        "suppression",
        instant + Duration::days(30),
        instant,
    )
    .expect("register");
    hazard.analyze("approved bow-tie v1").expect("analyze");
    hazard.attach_control("CTL-1").expect("control");
    hazard.verify_control("CTL-1").expect("verify");
    hazard
        .accept_residual(
            ResidualRiskAcceptance::new(
                EntityId::new(),
                "safety-authority",
                "controlled and monitored",
                "suppression",
                instant,
                instant + Duration::days(20),
            )
            .expect("risk"),
            instant,
            &Competent,
        )
        .expect("accept");
    hazard
}

fn complete_case(scope: PromotionScope) -> EvidenceCase {
    let instant = now();
    let author = EntityId::new();
    let mut case = EvidenceCase::open(EntityId::new(), scope, author.clone());
    for (index, kind) in [
        EvidenceKind::RequirementsTrace,
        EvidenceKind::HazardMitigation,
        EvidenceKind::Simulation,
        EvidenceKind::SoftwareInLoop,
        EvidenceKind::HardwareInLoop,
        EvidenceKind::ControlledField,
        EvidenceKind::SecurityReview,
        EvidenceKind::RollbackPlan,
        EvidenceKind::SupportPlan,
    ]
    .into_iter()
    .enumerate()
    {
        case.link(EvidenceRecord {
            id: format!("EVD-{index}"),
            kind,
            digest: [u8::try_from(index + 1).expect("small"); 32],
            status: EvidenceStatus::Current,
            valid_from: instant - Duration::days(1),
            expires_at: instant + Duration::days(10),
            assumption_valid: true,
        })
        .expect("link");
    }
    case.submit(instant).expect("submit");
    case.complete_review(
        IndependentApproval {
            id: "APR-1".into(),
            reviewer: EntityId::new(),
            case_author: author,
            competency: "independent-safety".into(),
            approved_at: instant,
            expires_at: instant + Duration::days(5),
            case_revision: 1,
        },
        instant,
    )
    .expect("review");
    case.approve(instant).expect("approve");
    case
}

struct Compliant;
impl ComplianceMatrixPort for Compliant {
    type Error = ();
    fn assess(
        &self,
        _: &PromotionScope,
        at: DateTime<Utc>,
    ) -> Result<ComplianceAssessment, Self::Error> {
        Ok(ComplianceAssessment {
            matrix_revision: "specialist-owned-v1".into(),
            applicable_items_complete: true,
            specialist_approval_expires_at: at + Duration::days(1),
        })
    }
}

#[test]
fn exact_complete_case_promotes_and_changed_configuration_suspends() {
    let mut approved_odd = odd();
    approved_odd.approve();
    let exact = scope(&approved_odd);
    let case = complete_case(exact.clone());
    let hazard = accepted_hazard();
    let mut manager = PromotionProcessManager::new(exact);
    assert_eq!(
        manager.evaluate(&case, &[hazard], &approved_odd, &[], &Compliant, now()),
        PromotionOutcome::Promoted
    );
    assert!(
        matches!(manager.configuration_changed([9;32]), PromotionOutcome::Suspended(reasons) if reasons.contains(&PromotionBlocker::ConfigurationChanged))
    );
}

#[test]
fn every_exit_gate_condition_fails_closed() {
    let mut approved_odd = odd();
    approved_odd.approve();
    let exact = scope(&approved_odd);
    let incomplete = EvidenceCase::open(EntityId::new(), exact.clone(), EntityId::new());
    let mut manager = PromotionProcessManager::new(exact.clone());
    assert!(
        matches!(manager.evaluate(&incomplete, &[accepted_hazard()], &approved_odd, &[], &Compliant, now()), PromotionOutcome::Blocked(reasons) if reasons.contains(&PromotionBlocker::Evidence))
    );

    let mut near_miss = SafetyOccurrence::report(
        EntityId::new(),
        "suppression",
        OccurrenceSeverity::NearMiss,
        now(),
    )
    .expect("occurrence");
    near_miss.triage().expect("triage");
    let case = complete_case(exact.clone());
    let mut manager = PromotionProcessManager::new(exact);
    assert!(
        matches!(manager.evaluate(&case, &[accepted_hazard()], &approved_odd, &[near_miss], &Compliant, now()), PromotionOutcome::Blocked(reasons) if reasons.contains(&PromotionBlocker::Occurrence))
    );

    let mut invalid_assumption = complete_case(manager.scope.clone());
    invalid_assumption.mark_stale();
    assert!(matches!(
        manager.evaluate(
            &invalid_assumption,
            &[accepted_hazard()],
            &approved_odd,
            &[],
            &Compliant,
            now()
        ),
        PromotionOutcome::Blocked(_) | PromotionOutcome::Suspended(_)
    ));
    assert!(
        !complete_case(manager.scope.clone()).is_current_approved(now() + Duration::days(6)),
        "expired approval must block"
    );
}

#[test]
fn constraints_only_supersede_with_exact_lineage_and_equal_or_stricter_scope() {
    let instant = now();
    let id = EntityId::new();
    let odd_id = EntityId::new();
    let base = SafetyConstraint::define(
        id.clone(),
        1,
        TimeWindow::new(instant, instant + Duration::days(10)).expect("window"),
        EntityId::new(),
        "verified-signature-1",
        ConstraintScope {
            tenant: "tenant-a".into(),
            capability: "suppression".into(),
            odd_id: odd_id.clone(),
            maximum_authority: 50,
        },
        [1; 32],
    )
    .expect("constraint");
    let strict = SafetyConstraint::define(
        id.clone(),
        2,
        TimeWindow::new(instant, instant + Duration::days(9)).expect("window"),
        EntityId::new(),
        "verified-signature-2",
        ConstraintScope {
            tenant: "tenant-a".into(),
            capability: "suppression".into(),
            odd_id: odd_id.clone(),
            maximum_authority: 40,
        },
        [2; 32],
    )
    .expect("constraint");
    assert_eq!(base.clone().supersede(strict.clone()), Ok(strict));
    let broader = SafetyConstraint::define(
        id,
        2,
        TimeWindow::new(instant, instant + Duration::days(11)).expect("window"),
        EntityId::new(),
        "verified-signature-3",
        ConstraintScope {
            tenant: "tenant-a".into(),
            capability: "suppression".into(),
            odd_id,
            maximum_authority: 60,
        },
        [3; 32],
    )
    .expect("constraint");
    assert_eq!(
        base.supersede(broader),
        Err(SafetyError::AuthorityExpansion)
    );
}

#[test]
fn occurrence_requires_findings_and_verified_actions_before_closure() {
    let mut occurrence = SafetyOccurrence::report(
        EntityId::new(),
        "suppression",
        OccurrenceSeverity::Serious,
        now(),
    )
    .expect("report");
    assert_eq!(
        occurrence.investigate(),
        Err(SafetyError::InvalidTransition)
    );
    occurrence.triage().expect("triage");
    occurrence.investigate().expect("investigate");
    occurrence
        .record_finding("loss of positioning")
        .expect("finding");
    occurrence.assign_action("ACT-1").expect("action");
    assert_eq!(occurrence.close(), Err(SafetyError::OpenSafetyAction));
    occurrence.complete_action("ACT-1").expect("complete");
    occurrence.close().expect("close");
    assert!(!occurrence.blocks_scope("suppression"));
    occurrence.reopen().expect("reopen");
    assert!(occurrence.blocks_scope("suppression"));
}

#[test]
fn self_review_expired_review_and_missing_each_evidence_kind_are_rejected() {
    let approved_odd = odd();
    let exact = scope(&approved_odd);
    let author = EntityId::new();
    let mut case = EvidenceCase::open(EntityId::new(), exact.clone(), author.clone());
    assert_eq!(case.submit(now()), Err(PromotionError::IncompleteEvidence));
    let complete = complete_case(exact);
    assert!(!complete.is_current_approved(now() + Duration::days(5)));

    let mut review = EvidenceCase::open(EntityId::new(), complete.scope.clone(), author.clone());
    for (index, kind) in [
        EvidenceKind::RequirementsTrace,
        EvidenceKind::HazardMitigation,
        EvidenceKind::Simulation,
        EvidenceKind::SoftwareInLoop,
        EvidenceKind::HardwareInLoop,
        EvidenceKind::ControlledField,
        EvidenceKind::SecurityReview,
        EvidenceKind::RollbackPlan,
        EvidenceKind::SupportPlan,
    ]
    .into_iter()
    .enumerate()
    {
        review
            .link(EvidenceRecord {
                id: format!("R-{index}"),
                kind,
                digest: [u8::try_from(index + 1).expect("small"); 32],
                status: EvidenceStatus::Current,
                valid_from: now() - Duration::hours(1),
                expires_at: now() + Duration::days(1),
                assumption_valid: true,
            })
            .expect("link");
    }
    review.submit(now()).expect("submit");
    assert_eq!(
        review.complete_review(
            IndependentApproval {
                id: "APR-SELF".into(),
                reviewer: author.clone(),
                case_author: author,
                competency: "safety".into(),
                approved_at: now(),
                expires_at: now() + Duration::hours(1),
                case_revision: 1
            },
            now()
        ),
        Err(PromotionError::InvalidApproval)
    );
}

#[test]
fn expiry_query_is_deterministic_and_uses_exclusive_boundary() {
    let instant = now();
    let due = due_reviews(
        [
            ExpiringItem {
                id: "APR-2".into(),
                kind: ExpiringKind::Approval,
                expires_at: instant + Duration::seconds(1),
            },
            ExpiringItem {
                id: "EVD-1".into(),
                kind: ExpiringKind::Evidence,
                expires_at: instant,
            },
            ExpiringItem {
                id: "APR-1".into(),
                kind: ExpiringKind::Approval,
                expires_at: instant,
            },
        ],
        instant,
    );
    assert_eq!(
        due.into_iter().map(|item| item.id).collect::<Vec<_>>(),
        ["EVD-1", "APR-1"]
    );
}

proptest! {
    #[test]
    fn sa_inv_001_acceptance_never_precedes_all_verified_controls(control_count in 1usize..20) {
        let instant = now(); let mut hazard = Hazard::register(EntityId::new(), EntityId::new(), "suppression", instant + Duration::days(2), instant).expect("register"); hazard.analyze("analysis-v1").expect("analyze");
        for i in 0..control_count { hazard.attach_control(format!("CTL-{i}")).expect("attach"); }
        for i in 0..control_count.saturating_sub(1) { hazard.verify_control(&format!("CTL-{i}")).expect("verify"); }
        let decision = ResidualRiskAcceptance::new(EntityId::new(), "competent", "bounded", "suppression", instant, instant + Duration::days(1)).expect("decision");
        prop_assert_eq!(hazard.accept_residual(decision, instant, &Competent), Err(SafetyError::HazardIncomplete));
    }

    #[test]
    fn odd_narrowing_never_increases_wind_limit(wide in 20u16..200, narrow in 1u16..20) {
        let mut approved = OperationalDesignDomain::new(["forest".into()], ["clear".into()], wide, true, true).expect("wide"); approved.approve();
        let candidate = OperationalDesignDomain::new(["forest".into()], ["clear".into()], narrow, true, false).expect("narrow");
        prop_assert!(approved.narrow_to(&candidate).is_ok());
        prop_assert!(approved.contains(&candidate));
    }
}
