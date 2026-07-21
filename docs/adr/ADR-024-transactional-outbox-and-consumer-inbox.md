# ADR-024: Transactional outbox and consumer inbox

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: implementation

## Context

Database state and published events must not diverge, and redelivery must not duplicate business effects.

## Decision

Write aggregate changes and outbox records in one PostgreSQL transaction. Relays publish with stable event IDs. Consumers record inbox/deduplication state in the same transaction as effects. Define retry, poison-message quarantine, replay, and operator repair procedures.

## Consequences

### Positive
- Prevents lost events and duplicate effects under normal failures.

### Negative
- Adds storage, cleanup, relay lag, and operational repair complexity.

### Neutral
- Delivery remains at least once; effects are idempotent.

## Links
- [ADR index](README.md)
