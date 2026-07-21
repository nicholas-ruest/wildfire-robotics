# ADR-013: Portable Cloud Platform and Canadian Data Residency

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: cloud, kubernetes, sovereignty

## Context

Commercial and public-sector customers require resilient operations, procurement flexibility, controlled data location, and repeatable regional deployment.

## Decision

Package stateless cloud workloads as OCI images and operate them on managed Kubernetes with declarative infrastructure and GitOps. Prefer managed PostgreSQL/PostGIS, object storage, key management, and event services behind portable interfaces. Keep protected Canadian operational data and backups in approved Canadian regions unless a documented contract/law permits otherwise. Edge stations use a smaller signed, declarative deployment profile.

## Consequences

### Positive
- Reproducibility, regional isolation, supplier leverage, and controlled residency.

### Negative
- Portability abstractions and Kubernetes operations add complexity and cost.

### Neutral
- Portability means tested recovery paths, not identical clouds.

## Links
- [ADR-003](ADR-003-edge-first-intermittently-connected-operations.md)
- [ADR-010](ADR-010-zero-trust-identity-and-command-authorization.md)
