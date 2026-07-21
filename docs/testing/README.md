# Deterministic Test and Evidence Conventions

All randomized tests and simulations accept an explicit unsigned 64-bit seed. The canonical CI
seed is `0x5749_4c44_4649_5245` (ASCII-inspired “WILDFIRE”). A failure must print its seed and
smallest reproducible input. Wall-clock time, entropy, network services, credentials, and hardware
are injected at boundaries; unit tests never read them implicitly.

Fixtures live below `tests/fixtures/<context>/<fixture-version>/`. Every fixture directory contains
`fixture.toml` with owner, source/license, classification, SHA-256 digest, creation procedure,
governing requirement/invariant IDs, and whether synthetic data is used. Binary and externally
sourced fixtures are immutable and content-addressed. Corrections create a new fixture version.

Run the complete quality gate with `./scripts/quality.sh`. It emits logs, CycloneDX SBOMs, coverage,
and `target/evidence/quality-manifest.json`. The manifest is attributable to the exact Git revision,
toolchain, dependency lock digest, command outcomes, and produced artifacts. Generated evidence is
never committed and does not itself authorize cyber-physical promotion.
