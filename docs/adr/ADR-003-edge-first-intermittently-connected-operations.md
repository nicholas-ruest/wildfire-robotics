# ADR-003: Edge-First Intermittently Connected Operations

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: edge, resilience, connectivity

## Context

Wildfire locations frequently lack reliable backhaul. Cloud dependence would turn a routine partition into a safety failure.

## Decision

Safety, motion, actuation envelopes, current mission state, emergency stop, and minimum navigation execute locally. Stations provide incident-edge coordination and durable store-and-forward. Cloud services provide portfolio planning, long-horizon compute, aggregation, and fleet administration. Synchronization uses versioned, conflict-aware, replayable messages with bounded staleness.

## Consequences

### Positive
- Missions fail safely and can continue within authorization during partitions.

### Negative
- More complex reconciliation, edge deployment, capacity management, and testing.

### Neutral
- Cloud availability does not equal mission availability.

## Links
- [ADR-005](ADR-005-event-driven-fleet-control-plane.md)
- [ADR-012](ADR-012-progressive-autonomy-and-safe-state-contract.md)
