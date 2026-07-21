# ADR-005: Event-Driven Fleet Control Plane

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: events, orchestration, fleet

## Context

Command, stations, and vehicles are distributed and partition-prone. Commands cannot be confused with telemetry, and delivery is never exactly once.

## Decision

Use a durable event backbone for domain facts and telemetry, plus authenticated request/reply command channels. Contracts use a schema registry and compatibility policy. Commands carry identity, aggregate/version preconditions, idempotency key, deadline, authorization, operational envelope, and correlation/causation IDs. Aggregate updates use transactional outboxes; consumers are idempotent and maintain inbox/deduplication state.

## Consequences

### Positive
- Replay, auditability, loose coupling, backpressure, and partition tolerance.

### Negative
- Eventual consistency and operational complexity must be designed and tested.

### Neutral
- Ordering is guaranteed only within explicitly documented keys.

## Links
- [ADR-002](ADR-002-bounded-context-modular-platform.md)
- [ADR-011](ADR-011-observability-audit-and-evidence-by-design.md)
