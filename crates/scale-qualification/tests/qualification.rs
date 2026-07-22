#![allow(missing_docs)]

use scale_qualification::{Campaign, Objectives, Workload};

#[test]
fn should_exercise_every_registered_asset_without_sampling()
-> Result<(), Box<dyn std::error::Error>> {
    let result = Campaign::new(Workload::production()).run()?;
    assert_eq!(result.assets_exercised, 1_000_000);
    assert_eq!(result.summaries_processed, 1_000_000);
    assert_eq!(result.reconnect_attempts, 10_000_000);
    Ok(())
}

#[test]
fn should_bound_faults_to_the_injected_region() -> Result<(), Box<dyn std::error::Error>> {
    let result = Campaign::new(Workload::test_fixture(40_000)).run()?;
    assert!(result.containment.all_bounded());
    assert_eq!(result.containment.healthy_region_loss, 0);
    Ok(())
}

#[test]
fn should_measure_all_required_operational_paths() -> Result<(), Box<dyn std::error::Error>> {
    let result = Campaign::new(Workload::test_fixture(40_000)).run()?;
    assert!(result.coverage.complete());
    assert_eq!(result.scenarios.hot_region_summaries, 16_000);
    assert_eq!(result.scenarios.relay_disconnected_assets, 40);
    assert_eq!(result.scenarios.region_isolated_assets, 4_000);
    assert_eq!(result.scenarios.recovered_assets, 4_000);
    assert_eq!(result.scenarios.rolling_upgrade_cells, 1_000);
    assert!(result.latency.p50_micros <= result.latency.p95_micros);
    assert!(result.latency.p95_micros <= result.latency.p99_micros);
    assert!(result.objectives_pass(&Objectives::approved()));
    Ok(())
}

#[test]
fn should_prove_locality_from_instrumented_accesses() -> Result<(), Box<dyn std::error::Error>> {
    let result = Campaign::new(Workload::test_fixture(40_000)).run()?;
    assert_eq!(result.architecture.global_scans, 0);
    assert_eq!(result.architecture.global_locks, 0);
    assert_eq!(result.architecture.consensus_rounds, 0);
    assert_eq!(result.architecture.synchronous_schedules, 0);
    assert!(result.architecture.max_touched_assets <= result.workload.cell_size);
    Ok(())
}

#[test]
fn should_make_prohibited_coordination_absent_from_the_harness_source() {
    let source = include_str!("../src/lib.rs");
    for prohibited in [
        "Mutex<",
        "RwLock<",
        "sort_unstable",
        "sort_by(",
        "consensus(",
        "global_schedule(",
    ] {
        assert!(
            !source.contains(prohibited),
            "prohibited primitive: {prohibited}"
        );
    }
}

#[test]
fn should_reject_non_divisible_or_too_small_workloads() {
    assert!(Campaign::new(Workload::test_fixture(999)).run().is_err());
}
