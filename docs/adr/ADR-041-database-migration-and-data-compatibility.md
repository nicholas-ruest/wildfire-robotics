# ADR-041: Database migration and data compatibility

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: data, delivery

## Context

Independent context deployment and edge lag require safe mixed-version operation and recoverable data evolution.

## Decision

Each context exclusively owns ordered, immutable migrations and a schema compatibility window. Apply expand-migrate-contract: add compatible shape, dual-read/write only when explicitly bounded, backfill with checkpoints and reconciliation, switch readers, then remove after fleet-version evidence. Migrations are idempotent, lock/time bounded, observable, rehearsed on production-scale anonymized data, and have roll-forward plus restore plans. Destructive changes require retention/legal review and verified backups.

## Consequences

### Positive
- Supports zero/low-downtime upgrades and intermittently connected versions.
### Negative
- Compatibility periods increase code and storage complexity.
### Neutral
- Database rollback may be unsafe; roll-forward is normally preferred.

## Links
- [ADR-017](ADR-017-deployable-modular-services-with-context-owned-data.md)
- [ADR-018](ADR-018-postgresql-and-postgis-as-transactional-system-of-record.md)
