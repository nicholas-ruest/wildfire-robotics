#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use scale_qualification::{Campaign, Workload};

fn million_asset_campaign(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("prompt30_scale_qualification");
    group.sample_size(10);
    group.bench_function("million_assets_10x_reconnect", |bencher| {
        bencher.iter(|| Campaign::new(Workload::production()).run());
    });
    group.finish();
}

criterion_group!(benches, million_asset_campaign);
criterion_main!(benches);
