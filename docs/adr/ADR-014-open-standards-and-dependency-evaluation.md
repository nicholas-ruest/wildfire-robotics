# ADR-014: Open Standards and Dependency Evaluation

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: supply-chain, open-source, procurement

## Context

The research identifies mature dependencies and experimental ruvnet components with unverified fit or licensing. Critical capability cannot rest on marketing claims or a single-maintainer project without an exit path.

## Decision

Use published contracts and adapters at dependency boundaries. Admit a dependency only after license, provenance, maintenance, security, performance, determinism, support, hardware fit, safety impact, total cost, and replacement-path review. Independently benchmark experimental components against representative workloads. Maintain SBOMs, version/update policy, escrow or source availability where needed, and tested fallback implementations for critical paths.

## Consequences

### Positive
- Limits lock-in and supply-chain surprises.

### Negative
- Evaluation and adapter maintenance slow initial adoption.

### Neutral
- No named AI/agent stack is preselected.

## Links
- [ADR-007](ADR-007-adopt-authoritative-hazard-data-and-models.md)
- [Research](../../.plans/deep-research.md)
