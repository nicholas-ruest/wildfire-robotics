#![allow(missing_docs)]
//! Representative station partition/reconnect burst benchmark.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use messaging_core::store_forward::{StoreForwardQueue, TelemetryTier};
use std::hint::black_box;

fn partition_reconnect(c: &mut Criterion) {
    let mut group = c.benchmark_group("partition_reconnect");
    for messages in [10_000_usize, 100_000] {
        group.throughput(Throughput::Elements(messages as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(messages),
            &messages,
            |bencher, &count| {
                bencher.iter(|| {
                    let Ok(mut queue) = StoreForwardQueue::new(count, count * 256, count / 4)
                    else {
                        return;
                    };
                    for index in 0..count {
                        let tier = if index % 10 == 0 {
                            TelemetryTier::SafetyCritical
                        } else if index % 3 == 0 {
                            TelemetryTier::Operational
                        } else {
                            TelemetryTier::Diagnostic
                        };
                        let payload_byte = index.to_le_bytes()[0];
                        if queue.enqueue(tier, vec![payload_byte; 128]).is_err() {
                            return;
                        }
                    }
                    let mut drained = 0_usize;
                    while let Some(item) = queue.pop() {
                        black_box(item);
                        drained += 1;
                    }
                    let _ = black_box(drained);
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, partition_reconnect);
criterion_main!(benches);
