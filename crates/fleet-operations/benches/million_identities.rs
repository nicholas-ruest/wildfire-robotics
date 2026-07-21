//! CI-safe and full-scale deterministic identity generation/placement benchmark.
#![allow(missing_docs, clippy::expect_used)]
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use fleet_operations::{PlacementIndex, SyntheticIdentityGenerator};

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("synthetic_identity_placement");
    for total in [10_000_u64, 1_000_000] {
        group.bench_with_input(BenchmarkId::from_parameter(total), &total, |b, &count| {
            b.iter(|| {
                let mut index = PlacementIndex::default();
                for item in SyntheticIdentityGenerator::new(0x5eed, count) {
                    index
                        .put(item.stable_id, item.cell, 1)
                        .expect("generated IDs are unique");
                }
                assert_eq!(
                    index.len(),
                    usize::try_from(count).expect("supported benchmark size")
                );
            });
        });
    }
    group.finish();
}
criterion_group!(benches, benchmark);
criterion_main!(benches);
