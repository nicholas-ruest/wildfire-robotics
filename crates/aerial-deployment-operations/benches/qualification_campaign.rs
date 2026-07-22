#![allow(missing_docs, clippy::unwrap_used, clippy::wildcard_imports)]
use aerial_deployment_operations::*;
use criterion::{Criterion, black_box, criterion_group, criterion_main};

struct Model(ModelContract);
impl SpecialistModel for Model {
    fn contract(&self) -> &ModelContract {
        &self.0
    }
    fn evaluate(&self, input: &ModelInputArtifact) -> Result<ModelOutputArtifact, DomainError> {
        Ok(ModelOutputArtifact::new(
            input,
            &self.0,
            500,
            self.0.assess(&input.scenario().parameters),
        ))
    }
}
struct Rig;
impl SilPort for Rig {
    fn exercise(&self, _: &Scenario) -> Result<i64, DomainError> {
        Ok(500)
    }
}
impl HitlPort for Rig {
    fn exercise(&self, _: &Scenario) -> Result<i64, DomainError> {
        Ok(500)
    }
}
fn benchmark(c: &mut Criterion) {
    let mut harness = QualificationHarness::new(1);
    for subsystem in CoupledSubsystem::ALL {
        let domain = ValidityDomain::new("wind", 0, 1_000_000, "scaled").unwrap();
        harness
            .register(
                subsystem,
                Box::new(Model(
                    ModelContract::new(
                        SpecialistModelClass::CoupledDeterministic,
                        "deterministic",
                        "1.0.0",
                        vec![domain],
                    )
                    .unwrap(),
                )),
            )
            .unwrap();
    }
    let scenario = Scenario::seeded(7, 1_000, Fault::REQUIRED).unwrap();
    let next = BoundedPhysicalTest::new(
        BoundedPhysicalTestKind::InstrumentedRange,
        "bounded-range-test",
    )
    .unwrap();
    c.bench_function("afb08_1000_sample_coupled_campaign", |b| {
        b.iter(|| {
            black_box(
                harness
                    .run(black_box(&scenario), &Rig, &Rig, &next)
                    .unwrap(),
            )
        });
    });
}
criterion_group!(benches, benchmark);
criterion_main!(benches);
