#![allow(missing_docs, clippy::unwrap_used, clippy::cast_possible_wrap)]
use aerial_deployment_operations::*;

struct DeterministicModel {
    contract: ModelContract,
    offset: i64,
}
impl SpecialistModel for DeterministicModel {
    fn contract(&self) -> &ModelContract {
        &self.contract
    }
    fn evaluate(&self, input: &ModelInputArtifact) -> Result<ModelOutputArtifact, DomainError> {
        let validity = self.contract.assess(&input.scenario().parameters);
        Ok(ModelOutputArtifact::new(
            input,
            &self.contract,
            500 + self.offset,
            validity,
        ))
    }
}
struct Rig(i64);
impl SilPort for Rig {
    fn exercise(&self, _: &Scenario) -> Result<i64, DomainError> {
        Ok(self.0)
    }
}
impl HitlPort for Rig {
    fn exercise(&self, _: &Scenario) -> Result<i64, DomainError> {
        Ok(self.0)
    }
}

fn harness(invalid: bool) -> QualificationHarness {
    let mut harness = QualificationHarness::new(20);
    for (index, subsystem) in CoupledSubsystem::ALL.into_iter().enumerate() {
        let maximum = if invalid { 1 } else { 1_000_000 };
        let domain = ValidityDomain::new("wind", 0, maximum, "micrometres/second").unwrap();
        let class = [
            SpecialistModelClass::Aeroelastic,
            SpecialistModelClass::ComputationalFluidDynamics,
            SpecialistModelClass::FiniteElement,
            SpecialistModelClass::Fire,
        ][index % 4];
        let contract =
            ModelContract::new(class, &format!("model-{index}"), "1.0.0", vec![domain]).unwrap();
        harness
            .register(
                subsystem,
                Box::new(DeterministicModel {
                    contract,
                    offset: index as i64,
                }),
            )
            .unwrap();
    }
    harness
}
fn scenario() -> Scenario {
    Scenario::seeded(42, 128, Fault::REQUIRED).unwrap()
}
fn next_test() -> BoundedPhysicalTest {
    BoundedPhysicalTest::new(BoundedPhysicalTestKind::InstrumentedRange, "range-001").unwrap()
}

#[test]
fn should_replay_identically_and_cover_every_invariant_and_fault() {
    let harness = harness(false);
    let scenario = scenario();
    let first = harness
        .run(&scenario, &Rig(506), &Rig(507), &next_test())
        .unwrap();
    let second = harness
        .run(&scenario, &Rig(506), &Rig(507), &next_test())
        .unwrap();
    assert_eq!(first.replay_digest, second.replay_digest);
    assert_eq!(first.fault_evidence.len(), Fault::REQUIRED.len());
    assert_eq!(
        first.invariant_coverage(),
        AdInvariant::ALL.into_iter().collect()
    );
    assert!(first.validity_visible());
}

#[test]
fn should_expose_disagreement_and_withhold_grant_outside_tolerance() {
    let report = harness(false)
        .run(&scenario(), &Rig(999), &Rig(500), &next_test())
        .unwrap();
    assert!(
        report
            .discrepancies
            .iter()
            .any(|item| !item.within_tolerance)
    );
    assert_eq!(report.grant, QualificationGrant::Withheld);
}

#[test]
fn should_withhold_grant_when_any_specialist_model_exits_validity_domain() {
    let report = harness(true)
        .run(&scenario(), &Rig(506), &Rig(507), &next_test())
        .unwrap();
    assert_eq!(report.grant, QualificationGrant::Withheld);
    assert!(
        !report
            .subsystem_outputs
            .values()
            .flatten()
            .all(|output| output.validity().values().all(|valid| *valid))
    );
}

#[test]
fn should_only_authorize_one_named_bounded_physical_test() {
    let report = harness(false)
        .run(
            &scenario(),
            &Rig(506),
            &Rig(507),
            &BoundedPhysicalTest::new(BoundedPhysicalTestKind::LowDrop, "drop-test-17").unwrap(),
        )
        .unwrap();
    assert_eq!(
        report.grant,
        QualificationGrant::NextBoundedPhysicalTest {
            test: BoundedPhysicalTest::new(BoundedPhysicalTestKind::LowDrop, "drop-test-17")
                .unwrap(),
            expires_after_runs: 1
        }
    );
}

#[test]
fn should_generate_seeded_uncertainty_sweeps_and_sensitivity() {
    let scenario = scenario();
    assert_eq!(scenario.uncertainty_sweep("wind", 5, 100).len(), 5);
    assert_eq!(
        Scenario::seeded(42, 128, Fault::REQUIRED)
            .unwrap()
            .parameters,
        scenario.parameters
    );
    assert!(
        !harness(false)
            .run(&scenario, &Rig(506), &Rig(507), &next_test())
            .unwrap()
            .sensitivities
            .is_empty()
    );
}

#[test]
fn should_reject_campaign_missing_any_named_fault_or_coupled_subsystem() {
    let incomplete = Scenario::seeded(42, 1, [Fault::Shock]).unwrap();
    assert!(matches!(
        harness(false).run(&incomplete, &Rig(0), &Rig(0), &next_test()),
        Err(DomainError::InvalidQualificationModel)
    ));
    let empty = QualificationHarness::new(1);
    assert!(matches!(
        empty.run(&scenario(), &Rig(0), &Rig(0), &next_test()),
        Err(DomainError::InvalidQualificationModel)
    ));
}

#[test]
fn immutable_artifacts_serialize_with_version_and_validity_provenance() {
    let report = harness(false)
        .run(&scenario(), &Rig(506), &Rig(507), &next_test())
        .unwrap();
    let encoded = serde_json::to_value(&report).unwrap();
    assert_eq!(
        encoded["subsystem_outputs"].as_object().unwrap().len(),
        CoupledSubsystem::ALL.len()
    );
}
