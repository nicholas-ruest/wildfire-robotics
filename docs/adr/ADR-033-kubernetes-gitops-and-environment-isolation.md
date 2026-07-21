# ADR-033: Kubernetes, GitOps, and environment isolation

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: platform, deployment

## Context

Cloud and station services require repeatable deployment, isolation, rollback, and evidence across development, test, simulation, staging, and production.

## Decision

Package services as signed OCI images and deploy cloud workloads to conformant Kubernetes through pull-based GitOps. Production accounts, clusters, credentials, encryption keys, data, and trust roots are isolated from non-production. Station deployments use a validated lightweight Kubernetes profile where appropriate. Desired state is reviewed, immutable, policy-checked, and promoted by digest; direct production mutation is emergency-only, time-bounded, audited, and reconciled. Infrastructure is declarative and recoverable from version control plus protected state and backups.

## Consequences

### Positive
- Reproducible promotion, drift detection, rollback, and portable operations.
### Negative
- Cluster and GitOps operations require specialist ownership and upgrade testing.
### Neutral
- Kubernetes is not used inside hard real-time vehicle control loops.

## Links
- [ADR-013](ADR-013-portable-cloud-platform-and-canadian-data-residency.md)
- [ADR-017](ADR-017-deployable-modular-services-with-context-owned-data.md)
