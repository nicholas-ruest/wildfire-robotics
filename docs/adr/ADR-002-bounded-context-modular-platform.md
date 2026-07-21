# ADR-002: Bounded-Context Modular Platform

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: ddd, modularity, ownership

## Context

Hazard science, incident command, fleet control, maintenance, and commercial administration evolve under different rules and failure modes. A shared domain model would couple safety-critical and business changes.

## Decision

Implement independently owned bounded contexts with explicit APIs/events, separate persistence ownership, versioned contracts, and anti-corruption layers for external systems. Begin as deployable modular services only where operational scaling or isolation requires it; avoid distributed transactions and use idempotent sagas/outboxes.

## Consequences

### Positive
- Clear ownership, fault containment, evolvable contracts, and focused testing.

### Negative
- Requires contract governance, eventual-consistency design, and duplicate read models.

### Neutral
- A bounded context is a semantic boundary, not automatically a microservice.

## Links
- [DDD context map](../ddd/context-map.md)
- [ADR-005](ADR-005-event-driven-fleet-control-plane.md)
