# Prompt 30 million-asset qualification

The Rust harness is an executable production gate, not a spreadsheet model. A
production run allocates all 1,000,000 asset records, emits one normalized
summary for each record, and mutates each record during each of ten regional
reconnect passes. The evidence reports `assets_exercised`,
`summaries_processed`, and `reconnect_attempts`; the gate requires exactly
1,000,000, 1,000,000, and 10,000,000 respectively.

Run the release qualification and benchmark with:

```sh
cargo run --release -p scale-qualification
cargo bench -p scale-qualification --bench million_assets
```

The campaign also executes a declared active mix with 40% hot-region skew,
bounded cell split/merge fencing, relay loss, one-region isolation, cell-at-a-
time rolling recovery, local charging budgets, supply/mobilization, a literal
100,000-robot arrival wave, and correlated hospital demand. Useful arrivals are
counted only after inspection, energization, connectivity, and eligibility.
Damaged robots are removed from charging and admitted or quarantined.

## Approved objectives

| Measure | Gate |
|---|---:|
| Logical processing p99 | <= 250 us |
| Measured summary throughput | >= 100,000/s |
| Summary lag/loss | 0 / 0 |
| Availability | >= 997,000 ppm |
| Regional recovery | <= 30 s |
| Processing headroom over 100,000/s | >= 20% |
| Failure containment | one region, no healthy-region loss |

Logical latency is an instrumented per-summary service-time model and wall-clock
throughput is measured from the actual run. The two are reported separately to
avoid presenting host timing as a network latency measurement. Resource
saturation is the peak per-cell budget utilization. Costs divide declared
campaign infrastructure cost by robots actually made ready or deployed.

## Locality proof and limitations

All mutable passes operate on `chunks_mut(cell_size)`. Instrumentation records
the largest touched set and counts global scans, global locks, consensus rounds,
and synchronous schedules; the gate requires all prohibited counters to remain
zero and the touched set never to exceed one cell. Regional fault injection
slices exactly one region and verifies no loss, duplicate dispatch, or stale
fence acceptance elsewhere.

The current measured bottleneck is destination inspection and energization.
This software qualification does not claim WAN, broker, database, carrier,
charger, hospital, field, or hardware capacity. Those environments must rerun
the same declared workload with production adapters before production approval.
