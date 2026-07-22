#![allow(clippy::expect_used, clippy::unwrap_used, missing_docs)]

use chrono::{TimeZone, Utc};
use commercial_operations::*;

fn scope(tenant: &str) -> TenantScope {
    TenantScope::new(tenant, "ca-central").expect("valid fixture")
}

#[test]
fn should_reject_cross_tenant_usage_and_support_access() {
    let mut meter = Meter::open("meter-a", scope("tenant-a"), "robot-hour").expect("meter");
    let fact = UsageFact::new(
        "fact-1",
        scope("tenant-b"),
        10,
        Utc.timestamp_opt(10, 0).single().unwrap(),
        "digest-b",
    )
    .unwrap();
    assert_eq!(
        meter.record(fact),
        Err(CommercialError::TenantScopeMismatch)
    );

    let mut case = SupportCase::open(
        "case-a",
        scope("tenant-a"),
        "operator",
        Severity::High,
        "gateway down",
    )
    .unwrap();
    assert_eq!(
        case.authorize_diagnostic(
            &scope("tenant-b"),
            &DiagnosticGrant::new(
                "grant-a",
                "tech",
                Utc.timestamp_opt(20, 0).single().unwrap()
            )
            .unwrap()
        ),
        Err(CommercialError::TenantScopeMismatch)
    );
}

#[test]
fn should_deduplicate_usage_and_preserve_compensating_lineage() {
    let mut meter = Meter::open("meter-a", scope("tenant-a"), "robot-hour").unwrap();
    let at = Utc.timestamp_opt(10, 0).single().unwrap();
    let fact = UsageFact::new("fact-1", scope("tenant-a"), 10, at, "digest-a").unwrap();
    assert_eq!(meter.record(fact.clone()).unwrap(), RecordOutcome::Recorded);
    assert_eq!(meter.record(fact).unwrap(), RecordOutcome::Duplicate);
    let original = meter.entries()[0].id().to_owned();
    meter
        .adjust("adjust-1", &original, -2, "verified correction", at)
        .unwrap();
    assert_eq!(meter.net_quantity(), 8);
    assert_eq!(meter.entries().len(), 2);
}

#[test]
fn should_rate_historical_usage_with_effective_contract_version() {
    let start = Utc.timestamp_opt(100, 0).single().unwrap();
    let amendment = Utc.timestamp_opt(200, 0).single().unwrap();
    let mut contract = Contract::draft("contract-a", scope("tenant-a"), "CAD").unwrap();
    contract.approve("approver", start).unwrap();
    contract
        .add_terms(ContractTerms::new("v1", start, Some(amendment), 25, "CAD").unwrap())
        .unwrap();
    contract
        .add_terms(ContractTerms::new("v2", amendment, None, 40, "CAD").unwrap())
        .unwrap();
    assert_eq!(
        contract
            .rate(3, Utc.timestamp_opt(150, 0).single().unwrap())
            .unwrap()
            .minor_units(),
        75
    );
    assert_eq!(
        contract
            .rate(3, Utc.timestamp_opt(250, 0).single().unwrap())
            .unwrap()
            .minor_units(),
        120
    );
}

#[test]
fn should_never_interrupt_active_authorized_safety_work() {
    let mut entitlement = Entitlement::grant(
        "entitlement-a",
        scope("tenant-a"),
        "missions",
        1,
        Utc.timestamp_opt(500, 0).single().unwrap(),
    )
    .unwrap();
    entitlement.suspend_optional("billing unavailable").unwrap();
    assert_eq!(
        entitlement.decision(
            WorkClass::ActiveAuthorizedSafety,
            Utc.timestamp_opt(600, 0).single().unwrap()
        ),
        WorkDecision::ContinueAndReconcile
    );
    assert_eq!(
        entitlement.decision(
            WorkClass::OptionalNew,
            Utc.timestamp_opt(600, 0).single().unwrap()
        ),
        WorkDecision::DenyOptional
    );
}

#[test]
fn should_require_complete_offboarding_evidence_and_respect_legal_hold() {
    let mut tenant = Tenant::onboard(
        "tenant-aggregate-a",
        scope("tenant-a"),
        IsolationTier::Shared,
        "agency",
    )
    .unwrap();
    tenant.begin_offboarding().unwrap();
    tenant
        .record_offboarding(OffboardingEvidence::AccessRevoked {
            reference: "ev-access".into(),
        })
        .unwrap();
    tenant
        .record_offboarding(OffboardingEvidence::ExportCompleted {
            reference: "ev-export".into(),
        })
        .unwrap();
    tenant
        .record_offboarding(OffboardingEvidence::ResourcesIsolated {
            reference: "ev-isolation".into(),
        })
        .unwrap();
    tenant
        .record_offboarding(OffboardingEvidence::LegalHoldApplied {
            reference: "hold-7".into(),
        })
        .unwrap();
    assert_eq!(tenant.close(), Err(CommercialError::OffboardingIncomplete));
    tenant
        .record_offboarding(OffboardingEvidence::RetentionResolved {
            reference: "retained-under-hold".into(),
        })
        .unwrap();
    tenant.close().unwrap();
    assert_eq!(tenant.state(), TenantState::Closed);
}

#[test]
fn should_expire_diagnostic_consent_and_prevent_vehicle_command_authority() {
    let case = SupportCase::open(
        "case-a",
        scope("tenant-a"),
        "operator",
        Severity::Critical,
        "link fault",
    )
    .unwrap();
    let grant = DiagnosticGrant::new(
        "grant-a",
        "tech",
        Utc.timestamp_opt(100, 0).single().unwrap(),
    )
    .unwrap();
    assert!(!case.can_access_diagnostics(
        "tech",
        Utc.timestamp_opt(101, 0).single().unwrap(),
        &grant
    ));
    assert!(!grant.permits_vehicle_commands());
}

#[test]
fn should_quarantine_same_fact_identity_with_changed_content() {
    let at = Utc.timestamp_opt(10, 0).single().unwrap();
    let mut meter = Meter::open("meter-1", scope("tenant-a"), "robot-hour").unwrap();
    meter
        .record(UsageFact::new("fact-1", scope("tenant-a"), 10, at, "digest-a").unwrap())
        .unwrap();
    assert_eq!(
        meter.record(UsageFact::new("fact-1", scope("tenant-a"), 11, at, "digest-b").unwrap()),
        Err(CommercialError::ReplayConflict)
    );
}

#[test]
fn should_reconcile_rate_and_finalize_immutable_invoice() {
    let at = Utc.timestamp_opt(100, 0).single().unwrap();
    let mut contract = Contract::draft("contract-1", scope("tenant-a"), "CAD").unwrap();
    contract.approve("finance-approver", at).unwrap();
    contract
        .add_terms(ContractTerms::new("terms-v1", at, None, 25, "CAD").unwrap())
        .unwrap();
    let mut meter = Meter::open("meter-1", scope("tenant-a"), "robot-hour").unwrap();
    meter
        .record(UsageFact::new("fact-1", scope("tenant-a"), 4, at, "digest-a").unwrap())
        .unwrap();
    meter.begin_closing(vec!["station-gap".into()]).unwrap();
    assert_eq!(meter.reconcile(), Err(CommercialError::ReconciliationGap));
    meter.reconcile_gap("station-gap").unwrap();
    meter.reconcile().unwrap();
    meter.rate_window(&contract, at).unwrap();
    let invoice = meter.finalize_invoice("invoice-1").unwrap();
    assert_eq!(invoice.amount().minor_units(), 100);
    assert_eq!(meter.state(), MeterState::Invoiced);
}

#[test]
fn should_require_facts_human_counterfactual_prediction_and_review_before_publication() {
    let at = Utc.timestamp_opt(100, 0).single().unwrap();
    let mut case =
        InvestmentCase::create("investment-1", scope("tenant-a"), "station expansion").unwrap();
    case.link_fact(InvestmentFact {
        id: "energy-fact".into(),
        digest: "sha256:energy".into(),
        observed_at: at,
    })
    .unwrap();
    case.define_human_counterfactual(HumanCounterfactual {
        version: "human-v1".into(),
        digest: "sha256:human".into(),
        description: "human-only baseline".into(),
    })
    .unwrap();
    case.link_predictive_result("sha256:monte-carlo-result")
        .unwrap();
    case.review().unwrap();
    case.publish(PublicationEvidence {
        reviewer: "independent-reviewer".into(),
        evidence_digest: "sha256:evidence".into(),
        published_at: at,
    })
    .unwrap();
    assert_eq!(case.state(), InvestmentState::Published);
    assert_eq!(case.version(), 5);
}

#[test]
fn should_enforce_support_case_transition_order_and_audited_consent() {
    let expires = Utc.timestamp_opt(200, 0).single().unwrap();
    let mut case = SupportCase::open(
        "case-1",
        scope("tenant-a"),
        "operator",
        Severity::High,
        "fault",
    )
    .unwrap();
    case.triage(Severity::Critical).unwrap();
    case.escalate().unwrap();
    let grant = DiagnosticGrant::new("grant-1", "tech", expires).unwrap();
    case.authorize_diagnostic(&scope("tenant-a"), &grant)
        .unwrap();
    assert!(case.can_access_diagnostics(
        "tech",
        Utc.timestamp_opt(150, 0).single().unwrap(),
        &grant
    ));
    case.resolve().unwrap();
    case.close().unwrap();
    case.reopen().unwrap();
}
