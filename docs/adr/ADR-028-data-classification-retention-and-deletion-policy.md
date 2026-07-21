# ADR-028: Data classification retention and deletion policy

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: implementation

## Context

Operational data spans public hazard feeds, personal information, sensitive infrastructure, imagery, telemetry, safety evidence, and commercial records with conflicting obligations.

## Decision

Assign classification, owner, residency, retention, deletion, legal-hold, and export policy at creation. Enforce lifecycle rules in PostgreSQL, object storage, streams, logs, and backups. Safety evidence and audit records use approved immutable retention; personal data is minimized and deletable where law permits.

## Consequences

### Positive
- Reduces privacy, cost, and compliance risk.

### Negative
- Policy propagation and deletion verification across replicas and backups are difficult.

### Neutral
- Retention is policy-driven, not an application default.

## Links
- [ADR index](README.md)
