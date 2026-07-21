# ADR-038: Secure software supply chain

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: security, delivery

## Context

Compromise or ambiguity in source, dependencies, builders, artifacts, firmware, or deployment manifests can create fleet-wide harm.

## Decision

Use protected source control, mandatory review, pinned toolchains and dependencies, hermetic/reproducible builds where feasible, ephemeral isolated builders, generated SBOMs, provenance attestations, malware/secret/license/vulnerability scans, and keyless or HSM-backed artifact signing. Deployments verify signature, provenance, policy, digest, and environment authorization. Critical dependencies have owners and replacement plans. Release records link source, builder, dependencies, tests, approvals, artifacts, and deployment.

## Consequences

### Positive
- Makes releases traceable and resists dependency and build compromise.
### Negative
- Exceptions and non-reproducible vendor firmware require explicit risk treatment.
### Neutral
- Signed malicious software remains malicious; review and verification still apply.

## Links
- [ADR-009](ADR-009-simulation-gated-cyber-physical-delivery.md)
- [ADR-014](ADR-014-open-standards-and-dependency-evaluation.md)
