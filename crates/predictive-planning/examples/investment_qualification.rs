#![allow(missing_docs, clippy::expect_used)]

use predictive_planning::{
    CostCategory, Counterfactual, InvestmentScenario, OperationalCostFact, Strategy,
};
use std::time::Instant;

fn fact(
    id: &str,
    category: CostCategory,
    strategy: Strategy,
    mode: i64,
    low: i64,
    high: i64,
) -> OperationalCostFact {
    OperationalCostFact::new(
        id,
        "qualification-tenant",
        "ca-central-1",
        0,
        category,
        strategy,
        mode,
        low,
        high,
    )
    .expect("static fact is valid")
}

fn standard_facts() -> Vec<OperationalCostFact> {
    vec![
        fact(
            "robot-capital",
            CostCategory::Capital,
            Strategy::RobotAssisted,
            4_000_000,
            3_600_000,
            4_600_000,
        ),
        fact(
            "robot-maintenance",
            CostCategory::Maintenance,
            Strategy::RobotAssisted,
            500_000,
            400_000,
            700_000,
        ),
        fact(
            "robot-energy",
            CostCategory::Energy,
            Strategy::RobotAssisted,
            250_000,
            200_000,
            350_000,
        ),
        fact(
            "robot-logistics",
            CostCategory::Logistics,
            Strategy::RobotAssisted,
            300_000,
            250_000,
            450_000,
        ),
        fact(
            "human-capital",
            CostCategory::Capital,
            Strategy::HumanOnly,
            1_000_000,
            800_000,
            1_300_000,
        ),
        fact(
            "human-staffing",
            CostCategory::Staffing,
            Strategy::HumanOnly,
            1_600_000,
            1_300_000,
            2_100_000,
        ),
        fact(
            "human-logistics",
            CostCategory::Logistics,
            Strategy::HumanOnly,
            700_000,
            550_000,
            950_000,
        ),
        fact(
            "human-downtime",
            CostCategory::Downtime,
            Strategy::HumanOnly,
            350_000,
            200_000,
            600_000,
        ),
    ]
}

fn main() {
    let mut scenario = InvestmentScenario::define(
        "qualification-standard",
        1,
        "qualification-tenant",
        "ca-central-1",
        "CAD",
        2026,
        10,
        500,
        0x5eed_2026,
        10_000,
        Counterfactual {
            version: 1,
            human_only: true,
            approved_causal_method: None,
        },
        standard_facts(),
    )
    .expect("static scenario is valid")
    .with_protected_units(25_000)
    .expect("protected units are valid");
    scenario.calibrate().expect("draft scenario calibrates");

    let started = Instant::now();
    let result = scenario.run().expect("qualified simulation succeeds");
    let elapsed = started.elapsed().as_micros();
    let runs = result.runs;
    let seed = result.seed;
    let npv_p05 = result.npv.p05;
    let npv_median = result.npv.median;
    let npv_p95 = result.npv.p95;
    let robot_tco = result.robot_tco.total;
    let human_tco = result.human_tco.total;
    let causal_claim = result.causal_claim;
    let result_digest = result.result_digest;
    println!(
        "{{\"schema_version\":1,\"fixture\":\"prompt26-investment-standard-v1\",\"runs\":{runs},\"facts\":8,\"seed\":{seed},\"npv_p05\":{npv_p05},\"npv_median\":{npv_median},\"npv_p95\":{npv_p95},\"robot_tco\":{robot_tco},\"human_tco\":{human_tco},\"causal_claim\":{causal_claim},\"result_digest\":\"{result_digest}\",\"total_microseconds\":{elapsed},\"scope\":\"deterministic single-process Monte Carlo standard profile; excludes scheduler, persistence, and distributed execution\"}}",
    );
}
