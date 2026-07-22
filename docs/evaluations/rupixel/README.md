# RuPixel evaluation status

Evaluated upstream revision: `bb1269fb9f4b0920de5944d248e4e79a01f3b80c` (2026-06-26), MIT licensed.

Adoption is deferred. The evaluated repository has no releases or packages, and its Rust crates explicitly require path dependencies from the RuVector monorepo. Its published benchmarks use a synthetic embedder/subset and demonstrate plumbing rather than semantic retrieval quality. These results are not production evidence.

The production boundary therefore remains the neutral Rust `VisualIndexPort`. The deterministic exact-search implementation is the enabled fallback. External RuPixel integration remains `DisabledPendingEvaluation`; no dependency, compatibility, quality, latency, or safety claim is made.

## Local benchmark evidence

The fallback uses stable manifest ordering and integer squared-distance ranking with identifier tie-breaking. Complexity is `O(n*d + n log n)` per query and memory is `O(n*d)`, where `n` is indexed keyframes and `d` is embedding width. It is intended as the reproducible correctness baseline, not a large-scale approximate-nearest-neighbor claim.

Before enabling an external adapter, record immutable upstream/source/dependency digests and evaluate representative wildfire media for recall, precision, calibration, subgroup/geography bias, censored outcomes, rebuild recovery, latency, and memory under the intended ODD.
