#![allow(missing_docs, clippy::panic)]
use std::time::Instant;
use vehicle_integration::{
    CohortPlan, ControllerKind, LinkQualityMap, LinkSample, Point3, RelayCandidate, ServiceClass,
    UavState,
};
fn main() {
    let uavs = (0..128)
        .map(|i| UavState {
            id: format!("u{i:03}"),
            position: Point3 {
                x_mm: (i % 8) * 5000,
                y_mm: (i / 8) * 5000,
                z_mm: 100_000,
            },
            energy_bps: 9000,
            link_quality_bps: 9000,
            controller: ControllerKind::Px4,
        })
        .collect::<Vec<_>>();
    let started = Instant::now();
    let mut valid = 0;
    for _ in 0..1000 {
        let p = CohortPlan::hierarchical(uavs.clone(), 8)
            .unwrap_or_else(|e| panic!("fixture failed: {e}"));
        if p.cells().len() == 16 && p.cells().iter().all(|c| c.members.len() <= 8) {
            valid += 1;
        }
    }
    let elapsed = started.elapsed();
    let links = LinkQualityMap::new(vec![LinkSample {
        from: "a".into(),
        to: "b".into(),
        quality_bps: 1000,
    }])
    .unwrap_or_else(|e| panic!("fixture failed: {e}"));
    let relay = links
        .route(
            ServiceClass::CommandAndControl,
            "a",
            "b",
            &[RelayCandidate {
                id: "r".into(),
                a_quality_bps: 9000,
                b_quality_bps: 8000,
            }],
        )
        .unwrap_or_else(|e| panic!("fixture failed: {e}"));
    println!(
        "{{\"schema_version\":1,\"fixture\":\"prompt22-cohort-relay-v1\",\"uavs\":128,\"max_cell\":8,\"iterations\":1000,\"valid_plans\":{valid},\"total_microseconds\":{},\"relay_effective_quality_bps\":{},\"scope\":\"deterministic topology/relay qualification; not radio or flight performance evidence\"}}",
        elapsed.as_micros(),
        relay.effective_quality_bps
    );
}
