# Prompt 21 exact-index qualification

Run `cargo run -p hazard-intelligence --example exact_index_qualification --release` to emit machine-readable qualification evidence. The committed JSON fixes the evidence schema and expected deterministic quality/recovery results; measured latency is environment-dependent and must be taken from the current run rather than compared across machines.

The fixture has three labeled two-dimensional integer embeddings and one declared domain-shift query mixing both dimensions. A 100% result means exact agreement with these frozen labels only. It does not establish semantic retrieval quality, approximate-index scalability, wildfire-domain generalization, or production latency.
