# ADR-019: Immutable object storage for bulk artifacts

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: implementation

## Context

Imagery, model artifacts, simulation outputs, maps, evidence bundles, and logs are too large for transactional databases and require durable provenance.

## Decision

Store bulk artifacts in versioned, encrypted, content-addressed object storage with checksums, retention class, tenant/incident scope, legal hold, and immutable evidence buckets. Database records contain metadata and stable object references.

## Consequences

### Positive
- Scalable storage, reproducibility, lifecycle policies, and WORM evidence support.

### Negative
- Object/database consistency and orphan cleanup require explicit workflows.

### Neutral
- Objects are immutable; corrections produce new versions.

## Links
- [ADR index](README.md)
