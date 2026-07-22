#![allow(clippy::unwrap_used)]
#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use digital_twin::{DeterministicTwin, Fault, Scenario, ScenarioLink};

fn campaign(c: &mut Criterion) {
    let twin = DeterministicTwin::standard();
    let scenarios = Fault::ALL
        .into_iter()
        .enumerate()
        .map(|(index, fault)| {
            Scenario::new(
                format!("BENCH-{index}"),
                index as u64,
                fault,
                ScenarioLink::new("REQ-29", "HAZ-29", "ADO-INV-005").unwrap(),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    c.bench_function("deterministic_full_fault_matrix", |b| {
        b.iter(|| {
            for scenario in &scenarios {
                std::hint::black_box(twin.run(scenario).unwrap());
            }
        });
    });
}

criterion_group!(benches, campaign);
criterion_main!(benches);
