# ADR-043: Process managers, compensation, and human escalation

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: distributed-systems, ddd

## Context

Cross-context workflows cannot use distributed transactions and must handle duplication, delay, reordering, partial completion, and irreversibility.

## Decision

Model every cross-context workflow as an explicitly owned, persisted process manager with state, correlation ID, initiating authority, deadlines, retries, idempotency keys, expected events, compensations, terminal outcomes, and operator escalation. Commands are at-least-once and effects are idempotent; events are immutable facts. Compensation is semantic and never described as rollback when physical action occurred. Stuck workflows are visible and repair actions are authorized and audited.

## Consequences

### Positive
- Makes partial failure and operational recovery designed behavior.
### Negative
- More persisted state, tests, and operator tooling are required.
### Neutral
- Some physical effects can only be mitigated, not undone.

## Links
- [ADR-005](ADR-005-event-driven-fleet-control-plane.md)
- [ADR-024](ADR-024-transactional-outbox-and-consumer-inbox.md)
