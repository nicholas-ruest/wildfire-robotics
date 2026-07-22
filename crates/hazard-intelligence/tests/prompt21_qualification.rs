#![allow(missing_docs, clippy::unwrap_used)]
use hazard_intelligence::{ExactVisualIndex, IndexManifest, VisualIndexPort, VisualItem};
#[test]
fn frozen_fixture_has_exact_agreement_including_declared_shift() {
    let m = IndexManifest::new(
        "fixture-v1",
        [9; 32],
        [8; 32],
        vec![
            VisualItem::new("dry-grass", [1; 32], 0, vec![100, 0]).unwrap(),
            VisualItem::new("forest", [2; 32], 0, vec![0, 100]).unwrap(),
            VisualItem::new("smoke", [3; 32], 0, vec![70, 70]).unwrap(),
        ],
    )
    .unwrap();
    let index = ExactVisualIndex::build(m).unwrap();
    for (q, label) in [
        (vec![100, 0], "dry-grass"),
        (vec![0, 100], "forest"),
        (vec![60, 60], "smoke"),
    ] {
        assert_eq!(index.search(&q, 1).unwrap()[0].id, label);
    }
}
