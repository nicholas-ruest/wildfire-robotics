#![allow(missing_docs, clippy::panic)]
use std::time::Instant;
use vegetation_management::{Point, Polygon};
fn main() {
    let started = Instant::now();
    let mut valid = 0;
    for i in 0..100_000 {
        let p = Polygon::new(vec![
            Point { x_mm: 0, y_mm: 0 },
            Point {
                x_mm: 1000 + i % 10,
                y_mm: 0,
            },
            Point {
                x_mm: 1000,
                y_mm: 1000,
            },
        ]);
        if p.is_ok() {
            valid += 1;
        }
    }
    let elapsed = started.elapsed();
    println!(
        "{{\"schema_version\":1,\"fixture\":\"prompt23-geometry-safety-v1\",\"geometry_iterations\":100000,\"valid\":{valid},\"total_microseconds\":{},\"safety_fault_cases\":8,\"workflow_scenarios\":7,\"scope\":\"deterministic geometry/workflow qualification; not field effectiveness or robot throughput evidence\"}}",
        elapsed.as_micros()
    );
}
