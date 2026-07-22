#![allow(missing_docs, clippy::unwrap_used)]
use predictive_planning::*;

fn facts() -> Vec<OperationalCostFact> {
    vec![
        OperationalCostFact::new(
            "capex",
            "tenant-a",
            "ca-bc",
            0,
            CostCategory::Capital,
            Strategy::RobotAssisted,
            900_000,
            800_000,
            1_100_000,
        )
        .unwrap(),
        OperationalCostFact::new(
            "robot-ops",
            "tenant-a",
            "ca-bc",
            1,
            CostCategory::Energy,
            Strategy::RobotAssisted,
            90_000,
            70_000,
            120_000,
        )
        .unwrap(),
        OperationalCostFact::new(
            "human-ops",
            "tenant-a",
            "ca-bc",
            1,
            CostCategory::Staffing,
            Strategy::HumanOnly,
            500_000,
            420_000,
            650_000,
        )
        .unwrap(),
        OperationalCostFact::new(
            "maintenance",
            "tenant-a",
            "ca-bc",
            1,
            CostCategory::Maintenance,
            Strategy::RobotAssisted,
            80_000,
            60_000,
            130_000,
        )
        .unwrap(),
    ]
}
fn scenario(seed: u64) -> InvestmentScenario {
    let mut scenario = InvestmentScenario::define(
        "case-1",
        1,
        "tenant-a",
        "ca-bc",
        "CAD",
        2026,
        5,
        500,
        seed,
        2_000,
        Counterfactual {
            version: 3,
            human_only: true,
            approved_causal_method: None,
        },
        facts(),
    )
    .unwrap();
    scenario.calibrate().unwrap();
    scenario
}

#[test]
fn should_be_seeded_reproducible_and_expose_uncertainty() {
    let a = scenario(42).run().unwrap();
    let b = scenario(42).run().unwrap();
    assert_eq!(a, b);
    assert!(a.npv.p05 <= a.npv.median && a.npv.median <= a.npv.p95);
}
#[test]
fn should_separate_capital_recurring_and_tco() {
    let r = scenario(7).run().unwrap();
    assert!(r.robot_tco.capital > 0);
    assert!(r.robot_tco.recurring > 0);
    assert_eq!(
        r.robot_tco.total,
        r.robot_tco.capital + r.robot_tco.recurring
    );
}
#[test]
fn should_report_irr_payback_and_sensitivity_without_causal_claim() {
    let r = scenario(9).run().unwrap();
    assert!(!r.causal_claim);
    assert!(!r.sensitivity.is_empty());
    assert!(matches!(r.irr, IrrStatus::Unique { .. }));
    assert!(matches!(r.payback, PaybackStatus::Year(_)));
}

#[test]
fn should_reject_overflow_and_report_never_statuses() {
    let huge = vec![
        OperationalCostFact::new(
            "huge",
            "tenant-a",
            "ca-bc",
            1,
            CostCategory::Maintenance,
            Strategy::RobotAssisted,
            i64::MAX,
            i64::MAX,
            i64::MAX,
        )
        .unwrap(),
    ];
    let mut s = InvestmentScenario::define(
        "overflow",
        1,
        "tenant-a",
        "ca-bc",
        "CAD",
        2026,
        100,
        0,
        1,
        2,
        Counterfactual {
            version: 1,
            human_only: true,
            approved_causal_method: None,
        },
        huge,
    )
    .unwrap();
    s.calibrate().unwrap();
    assert!(s.run().is_err());
    let mut no_return = InvestmentScenario::define(
        "never",
        1,
        "tenant-a",
        "ca-bc",
        "CAD",
        2026,
        2,
        0,
        1,
        2,
        Counterfactual {
            version: 1,
            human_only: true,
            approved_causal_method: None,
        },
        vec![
            OperationalCostFact::new(
                "cost",
                "tenant-a",
                "ca-bc",
                0,
                CostCategory::Capital,
                Strategy::RobotAssisted,
                10,
                10,
                10,
            )
            .unwrap(),
        ],
    )
    .unwrap();
    no_return.calibrate().unwrap();
    let r = no_return.run().unwrap();
    assert_eq!(r.payback, PaybackStatus::Never);
    assert_eq!(r.irr, IrrStatus::UndefinedNoSignChange);
}
#[test]
fn should_match_constant_distribution_and_digest_inputs() {
    let mut a = scenario(11);
    let r = a.run().unwrap();
    assert_ne!(a.content_digest(), scenario(12).content_digest());
    a.mark_simulated(&r).unwrap();
    a.review().unwrap();
    assert_eq!(a.publish().unwrap(), r.result_digest);
}
#[test]
fn should_match_constant_distribution_analytic_npv() {
    let f = vec![
        OperationalCostFact::new(
            "cap",
            "tenant-a",
            "ca-bc",
            0,
            CostCategory::Capital,
            Strategy::RobotAssisted,
            100,
            100,
            100,
        )
        .unwrap(),
        OperationalCostFact::new(
            "baseline",
            "tenant-a",
            "ca-bc",
            1,
            CostCategory::Staffing,
            Strategy::HumanOnly,
            60,
            60,
            60,
        )
        .unwrap(),
    ];
    let mut s = InvestmentScenario::define(
        "analytic",
        1,
        "tenant-a",
        "ca-bc",
        "CAD",
        2026,
        2,
        0,
        5,
        2,
        Counterfactual {
            version: 1,
            human_only: true,
            approved_causal_method: None,
        },
        f,
    )
    .unwrap();
    s.calibrate().unwrap();
    let r = s.run().unwrap();
    assert_eq!(
        r.npv,
        Range {
            p05: 20,
            median: 20,
            p95: 20
        }
    );
}
#[test]
fn should_gate_structured_causality() {
    assert!(ApprovedCausalMethod::new("did", "approval", "bad").is_err());
    let approved =
        ApprovedCausalMethod::new("difference-in-differences", "approval-7", &"a".repeat(64))
            .unwrap();
    assert_eq!(approved.approval_id, "approval-7");
}
#[test]
fn should_reject_cross_tenant_future_or_duplicate_facts() {
    let mut f = facts();
    f.push(
        OperationalCostFact::new(
            "foreign",
            "tenant-b",
            "ca-bc",
            1,
            CostCategory::Logistics,
            Strategy::RobotAssisted,
            1,
            1,
            1,
        )
        .unwrap(),
    );
    assert!(
        InvestmentScenario::define(
            "bad",
            1,
            "tenant-a",
            "ca-bc",
            "CAD",
            2026,
            5,
            500,
            1,
            100,
            Counterfactual {
                version: 1,
                human_only: true,
                approved_causal_method: None
            },
            f
        )
        .is_err()
    );
}
#[test]
fn should_require_versioned_human_only_counterfactual() {
    let bad = Counterfactual {
        version: 0,
        human_only: false,
        approved_causal_method: None,
    };
    assert!(
        InvestmentScenario::define(
            "bad",
            1,
            "tenant-a",
            "ca-bc",
            "CAD",
            2026,
            5,
            500,
            1,
            100,
            bad,
            facts()
        )
        .is_err()
    );
}
