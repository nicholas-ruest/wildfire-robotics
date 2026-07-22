# RVM dependency-admission record

Evaluation date: 2026-07-22. Upstream: `https://github.com/ruvnet/rvm` at immutable commit `af97d18f29d5704f2fbbeecccce192d712bb9a80`.

## Decision

Adoption is deferred and no RVM dependency is present in the product graph. The platform-owned `CollaborationRuntimePort` remains the replacement boundary, the deterministic conventional graph runtime is enabled, and RVM reports `DisabledPendingEvaluation`.

The upstream tree declares `MIT OR Apache-2.0` in workspace metadata and its README, but the evaluated commit does not contain the referenced `LICENSE`, `LICENSE-MIT`, or `LICENSE-APACHE` files. Its root workspace version is `0.1.1`, while repository tags run through `v1.5.0`. Those provenance and versioning conflicts fail ADR-014 dependency admission before runtime claims can be relied upon.

## Reproduction

```bash
git ls-remote https://github.com/ruvnet/rvm.git HEAD refs/heads/main 'refs/tags/*'
git clone --filter=blob:none --no-checkout https://github.com/ruvnet/rvm.git rvm-evaluation
git -C rvm-evaluation show af97d18f29d5704f2fbbeecccce192d712bb9a80:Cargo.toml
git -C rvm-evaluation ls-tree --name-only af97d18f29d5704f2fbbeecccce192d712bb9a80
```

## Admission matrix

| Gate | Result | Evidence / required follow-up |
|---|---|---|
| Immutable provenance | Partial | Commit is fetchable; no signed tag or release attestation was established. |
| License | Fail | Declared dual license has no corresponding license text in the evaluated root tree. |
| Version/release policy | Fail | Workspace `0.1.1` conflicts with tags through `v1.5.0`; compatible update policy is undefined. |
| Maintenance/support | Not established | No commercial support or response-time commitment was established. |
| Security/SBOM | Not established | A product-scoped dependency/SBOM/advisory review is required after licensing is resolved. |
| Determinism/performance | Not established | Representative cohort, checkpoint, fault, and hardware benchmarks are required. |
| Safety/hardware fit | Not established | RVM is prohibited from flight, drive, actuation, authority, or other safety loops. |
| Replacement path | Pass | The neutral runtime port and conventional graph allocator remain independently usable. |

## Operational boundary

Collaboration output is advisory split/merge/locality information only. External results are validated for exact membership and cohort bounds before use; outage or invalid output invokes the conventional runtime. No relationship or coherence result can grant trust, capability, identity, authority, mission allocation, or command.
