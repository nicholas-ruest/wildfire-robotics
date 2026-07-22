# Integrated software release candidate

`WR-RC-2026-07-22` is a software release candidate. It is not a field-ready or production-ready declaration.

The `release-gate` executable regenerates the effective inventory from repository sources and fails closed when the expected 74 ADRs, 15 contexts, 107 invariants, 49 integration contracts, or 18 process managers drift. It also verifies the release identifier, required cross-cutting artifacts, unresolved-risk dispositions, architecture deviations, and the non-software evidence boundary. Its JSON output contains content digests suitable for inclusion in the immutable quality evidence bundle.

Run `cargo run --locked -p release-gate -- .` from a clean checkout. A successful result establishes repository consistency only. The quality workflow must separately establish formatting, compilation, strict linting, tests, security analysis, dependency policy, SBOM, deployment validation, coverage, scenario and scale results.

## Promotion decision

- Known software release blockers: zero at assembly time, subject to the complete quality gate remaining green.
- Architecture deviations: none.
- Software label: `software-release-candidate`.
- Field and production claims: prohibited.
- External HITL, controlled-field, aircraft, material, regulatory and operational evidence: outstanding and listed in `release/field-evidence-status.toml`.

Any failed technical gate, new severity-one or severity-two defect, broken traceability link, unapproved deviation, expired risk decision, or contradictory evidence invalidates this candidate. Physical behavior remains governed by the existing simulation, dual-authority, local inhibit and safe-state controls.

