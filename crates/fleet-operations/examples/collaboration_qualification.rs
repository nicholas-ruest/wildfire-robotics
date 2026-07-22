#![allow(missing_docs, clippy::panic)]
use fleet_operations::{CollaborationRuntimePort, ConventionalCollaborationRuntime};
use std::time::Instant;
fn main() {
    let members = (0..128).map(|i| format!("v{i:03}")).collect::<Vec<_>>();
    let refs = members.iter().map(String::as_str).collect::<Vec<_>>();
    let started = Instant::now();
    let mut valid = 0;
    for _ in 0..10_000 {
        let r = ConventionalCollaborationRuntime
            .recommend(&refs, &[], 8, 10)
            .unwrap_or_else(|e| panic!("fixture failed: {e}"));
        if r.cohorts.len() == 16 && r.cohorts.iter().all(|c| c.len() <= 8) {
            valid += 1;
        }
    }
    println!(
        "{{\"schema_version\":1,\"fixture\":\"prompt25-conventional-v1\",\"members\":128,\"iterations\":10000,\"valid\":{valid},\"total_microseconds\":{},\"scope\":\"deterministic advisory partition timing; not mission allocation or RVM evidence\"}}",
        started.elapsed().as_micros()
    );
}
