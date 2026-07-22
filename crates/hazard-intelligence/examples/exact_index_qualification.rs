#![allow(missing_docs, clippy::panic)]

use hazard_intelligence::{
    AtomicIndexStore, ExactVisualIndex, IndexManifest, VisualIndexPort, VisualItem,
};
use std::time::Instant;
fn item(id: &str, n: u8, e: Vec<i16>) -> VisualItem {
    VisualItem::new(id, [n; 32], 0, e).unwrap_or_else(|e| panic!("frozen fixture invalid: {e}"))
}
fn manifest(id: &str) -> IndexManifest {
    IndexManifest::new(
        id,
        [9; 32],
        [8; 32],
        vec![
            item("dry-grass", 1, vec![100, 0]),
            item("forest", 2, vec![0, 100]),
            item("smoke", 3, vec![70, 70]),
        ],
    )
    .unwrap_or_else(|e| panic!("frozen fixture invalid: {e}"))
}
fn main() {
    let index = ExactVisualIndex::build(manifest("fixture-v1"))
        .unwrap_or_else(|e| panic!("index failed: {e}"));
    let labels = [
        (vec![100, 0], "dry-grass", false),
        (vec![0, 100], "forest", false),
        (vec![60, 60], "smoke", true),
    ];
    let start = Instant::now();
    let mut correct = 0usize;
    for _ in 0..1000 {
        for (q, label, _) in &labels {
            let hits = index
                .search(q, 1)
                .unwrap_or_else(|e| panic!("search failed: {e}"));
            if hits.first().is_some_and(|h| h.id == *label) {
                correct += 1;
            }
        }
    }
    let elapsed = start.elapsed();
    let total = labels.len() * 1000;
    let shifted_total = labels.iter().filter(|x| x.2).count();
    let shifted_correct = labels
        .iter()
        .filter(|x| x.2)
        .filter(|(q, l, _)| {
            index
                .search(q, 1)
                .ok()
                .and_then(|h| h.first().cloned())
                .is_some_and(|h| h.id == *l)
        })
        .count();
    let first = manifest("fixture-v1");
    let mut store = AtomicIndexStore::activate(first.clone())
        .unwrap_or_else(|e| panic!("activation failed: {e}"));
    let failed_rebuild_preserved = store
        .rebuild(Err(hazard_intelligence::VisualIndexError::InvalidManifest))
        .is_err()
        && store.active_digest() == first.digest();
    let second = manifest("fixture-v2");
    store
        .rebuild(Ok(second))
        .unwrap_or_else(|e| panic!("rebuild failed: {e}"));
    store
        .recover()
        .unwrap_or_else(|e| panic!("recovery failed: {e}"));
    let recovery_restored = store.active_digest() == first.digest();
    println!(
        "{{\"schema_version\":1,\"fixture\":\"prompt21-exact-v1\",\"queries\":{total},\"top1_correct\":{correct},\"top1_agreement_bps\":{},\"domain_shift_slice\":{{\"definition\":\"mixed-feature query absent from canonical endpoints\",\"queries\":{shifted_total},\"top1_correct\":{shifted_correct}}},\"latency\":{{\"total_microseconds\":{},\"mean_nanoseconds_per_query\":{}}},\"failed_rebuild_preserved_active\":{failed_rebuild_preserved},\"recovery_restored_previous\":{recovery_restored},\"scope\":\"3-item deterministic correctness fixture; not production-scale semantic evidence\"}}",
        correct * 10_000 / total,
        elapsed.as_micros(),
        elapsed.as_nanos() / total as u128
    );
}
