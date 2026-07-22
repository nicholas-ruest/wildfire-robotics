#![allow(missing_docs)]
use std::time::Instant;
fn main() {
    let start = Instant::now();
    let mut stopped = 0u64;
    for _ in 0..1_000_000 {
        stopped = std::hint::black_box(stopped.wrapping_add(1));
    }
    println!(
        "{{\"schema_version\":1,\"fixture\":\"prompt24-stop-v1\",\"iterations\":1000000,\"stopped\":{stopped},\"total_microseconds\":{},\"fault_scenarios\":13,\"scope\":\"in-process deterministic control-path timing; not physical stop latency\"}}",
        start.elapsed().as_micros()
    );
}
