#![allow(missing_docs)]
#![allow(clippy::unwrap_used, clippy::duration_suboptimal_units)]

use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use chrono::{TimeZone, Utc};
use predictive_planning::*;

fn digest(byte: char) -> Digest {
    Digest::parse(&byte.to_string().repeat(64)).unwrap()
}
fn odd() -> OperationalDomain {
    OperationalDomain {
        id: "ODD-boreal".into(),
        regions: BTreeSet::from(["CA-AB".into()]),
        min_temperature_c: -30.0,
        max_temperature_c: 55.0,
        max_wind_speed_mps: 30.0,
        supported_fuel_models: BTreeSet::from(["C-2".into()]),
    }
}
fn release() -> ModelRelease {
    ModelRelease::register(ModelRegistration {
        id: "REL-fire-1".into(),
        artifact_digest: digest('a'),
        source_digest: digest('b'),
        dependency_digest: digest('c'),
        runtime_digest: digest('d'),
        training_data_digests: vec![digest('e')],
        odd: odd(),
        calibration: Calibration {
            method: "isotonic".into(),
            cohort_digest: digest('f'),
            valid_until: Utc.with_ymd_and_hms(2028, 1, 1, 0, 0, 0).unwrap(),
        },
        limitations: vec!["Not validated for crown-fire transitions".into()],
        license: "Apache-2.0".into(),
    })
    .unwrap()
}
fn context() -> RunContext {
    RunContext {
        region: "CA-AB".into(),
        temperature_c: 24.0,
        wind_speed_mps: 8.0,
        fuel_model: "C-2".into(),
        observed_at: Utc.with_ymd_and_hms(2027, 7, 1, 12, 0, 0).unwrap(),
    }
}
fn manifest(model: &ModelRelease) -> RunManifest {
    RunManifest::new(
        "RUN-1",
        model,
        vec![InputArtifact {
            id: "hazard-picture".into(),
            digest: digest('1'),
            license: Some("CC-BY-4.0".into()),
            superseded: false,
        }],
        42,
        BTreeMap::from([("cell_size_m".into(), Parameter::new(30.0, "m").unwrap())]),
        context(),
    )
    .unwrap()
}

#[test]
fn should_reject_operational_run_until_release_is_promoted() {
    let err = ForecastRun::request(manifest(&release()), RunMode::Operational).unwrap_err();
    assert_eq!(err, PlanningError::ModelNotPromoted);
}

#[test]
fn should_reject_operational_run_outside_validated_odd() {
    let mut model = release();
    model.approve("APR-1", "independent-reviewer").unwrap();
    model.promote().unwrap();
    let mut m = manifest(&model);
    m.context.wind_speed_mps = 31.0;
    assert_eq!(
        ForecastRun::request(m, RunMode::Operational).unwrap_err(),
        PlanningError::OutsideOperationalDomain
    );
}

#[test]
fn should_make_manifest_identity_stable_and_sensitive_to_seed() {
    let model = release();
    let first = manifest(&model);
    let mut second = manifest(&model);
    second.seed = 43;
    assert_eq!(first.digest(), manifest(&model).digest());
    assert_ne!(first.digest(), second.digest());
}

#[test]
fn should_reproduce_reference_output_for_same_manifest() {
    let model = release();
    let m = manifest(&model);
    let runner = DeterministicReferenceModel;
    let a = runner.execute(&m, &SandboxPolicy::strict()).unwrap();
    let b = runner.execute(&m, &SandboxPolicy::strict()).unwrap();
    assert!(a.within_tolerance(&b, 0.0));
}

#[test]
fn should_enforce_foreign_runner_sandbox_controls() {
    let policy = SandboxPolicy::strict();
    assert!(
        !policy.network_enabled
            && policy.read_only_root
            && policy.max_runtime <= Duration::from_secs(300)
    );
    assert_eq!(
        policy
            .validate(&ResourceRequest {
                cpu_cores: 2,
                memory_bytes: policy.max_memory_bytes + 1,
                gpu_count: 0,
                runtime: Duration::from_secs(1),
                network: false,
                output_bytes: 1
            })
            .unwrap_err(),
        PlanningError::SandboxLimitExceeded("memory")
    );
}

#[test]
fn should_reject_invalid_outputs_and_publish_valid_calibrated_output() {
    let mut model = release();
    model.approve("APR-1", "reviewer").unwrap();
    model.promote().unwrap();
    let mut run = ForecastRun::request(manifest(&model), RunMode::Operational).unwrap();
    run.start().unwrap();
    run.record_output(ModelOutput {
        cells: vec![ForecastCell {
            x: 0,
            y: 0,
            probability: 1.2,
            arrival_minutes: Some(-1.0),
        }],
        uncertainty: 0.2,
        artifact_digest: digest('9'),
    })
    .unwrap();
    assert_eq!(
        run.validate(PublicationEvidence::all_passed()).unwrap_err(),
        PlanningError::InvalidOutput
    );
    let mut run = ForecastRun::request(manifest(&model), RunMode::Operational).unwrap();
    run.start().unwrap();
    run.record_output(
        DeterministicReferenceModel
            .execute(run.manifest(), &SandboxPolicy::strict())
            .unwrap(),
    )
    .unwrap();
    run.validate(PublicationEvidence::all_passed()).unwrap();
    run.publish().unwrap();
    assert_eq!(run.status(), ForecastStatus::Published);
}

#[test]
fn should_reject_nonreproducible_output_before_publication() {
    let mut model = release();
    model.approve("APR-1", "reviewer").unwrap();
    model.promote().unwrap();
    let mut run = ForecastRun::request(manifest(&model), RunMode::Operational).unwrap();
    run.start().unwrap();
    let output = DeterministicReferenceModel
        .execute(run.manifest(), &SandboxPolicy::strict())
        .unwrap();
    run.record_output(output).unwrap();
    let mut evidence = PublicationEvidence::all_passed();
    evidence.reproducible = false;
    assert_eq!(
        run.validate(evidence).unwrap_err(),
        PlanningError::PublicationGateFailed("reproducibility")
    );
}

#[test]
fn should_never_grant_authority_through_recommendation() {
    let rec = Recommendation::new(
        "REC-1",
        "RUN-1",
        "Monitor eastern flank",
        0.7,
        0.2,
        Utc.with_ymd_and_hms(2027, 7, 1, 13, 0, 0).unwrap(),
        vec!["Sparse observations".into()],
        vec!["Ground survey".into()],
        "20 minutes old",
    )
    .unwrap();
    assert_eq!(rec.authority(), Authority::AdvisoryOnly);
}

#[test]
fn should_withdraw_outputs_when_lineage_is_materially_invalidated() {
    let mut graph = LineageRegistry::default();
    graph.register("hazard-picture", "RUN-1");
    graph.register("RUN-1", "REC-1");
    let affected = graph.invalidate("hazard-picture", Invalidation::MaterialDrift);
    assert_eq!(affected, BTreeSet::from(["REC-1".into(), "RUN-1".into()]));
}

#[test]
fn should_register_reproducible_scenario_and_control_canary_rollback() {
    let scenario = SpreadScenario::define(ScenarioDefinition {
        id: "SCN-wind-shift".into(),
        seed: 99,
        simulator: SimulatorValidity {
            artifact_digest: digest('7'),
            valid_odd: odd(),
            calibrated_against: vec![digest('8')],
            simulation_to_reality_gap: 0.12,
        },
        requirements: BTreeSet::from(["PP-INV-001".into()]),
        hazards: BTreeSet::from(["HAZ-fire-spread".into()]),
        expected: vec![ExpectedInvariant {
            name: "burned_area_ha".into(),
            expected: 5.0,
            tolerance: 0.01,
        }],
    })
    .unwrap();
    let mut registry = ScenarioRegistry::default();
    registry.register(scenario).unwrap();
    assert_eq!(registry.get("SCN-wind-shift").unwrap().seed(), 99);
    let mut model = release();
    model.approve("APR-1", "reviewer").unwrap();
    model.promote().unwrap();
    model.begin_shadow().unwrap();
    model.begin_canary().unwrap();
    model.rollback("drift threshold exceeded").unwrap();
    assert_eq!(model.stage(), ModelStage::Promoted);
}
