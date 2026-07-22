# Deterministic Test and Evidence Conventions

All randomized tests and simulations accept an explicit unsigned 64-bit seed. The canonical CI
seed is `0x5749_4c44_4649_5245` (ASCII-inspired “WILDFIRE”). A failure must print its seed and
smallest reproducible input. Wall-clock time, entropy, network services, credentials, and hardware
are injected at boundaries; unit tests never read them implicitly.

Fixtures live below `tests/fixtures/<context>/<fixture-version>/`. Every fixture directory contains
`fixture.toml` with owner, source/license, classification, SHA-256 digest, creation procedure,
governing requirement/invariant IDs, and whether synthetic data is used. Binary and externally
sourced fixtures are immutable and content-addressed. Corrections create a new fixture version.

Run the complete quality gate with `cargo quality`. In addition to the Rust workspace, it validates
the generated contract client, API client, operator browser application, rendered deployment
manifests, and simulated recovery evidence. It emits logs, Rust and npm CycloneDX SBOMs, coverage,
and `target/evidence/quality-manifest.json`. The manifest is attributable to the exact Git revision,
toolchain, dependency lock digest, command outcomes, and produced artifacts. Generated evidence is
never committed and does not itself authorize cyber-physical promotion.
Local manifests contain content digests but remain unsigned. On `main`, the least-privilege
`release-evidence` workflow regenerates evidence from a clean checkout, creates a deterministic
bundle, and uses GitHub OIDC with Sigstore/Cosign to sign and immediately verify that bundle. It
uploads evidence only; it does not publish a release or grant production/field authority.

The integrated release path also runs the release traceability validator, Prompt 29 deterministic
scenario and fault campaign, privacy/consent checks, replay and recovery checks, and Prompt 30's
full-million workload in release mode. The labels “chaos” and “endurance” refer to deterministic
software qualification campaigns; they are not evidence of physical-system or field endurance.
