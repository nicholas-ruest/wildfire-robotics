# ADR-025: Version-vector edge reconciliation with authority precedence

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: implementation

## Context

Stations can operate offline and later reconnect with concurrent cloud changes. Naive last-write-wins could expand expired authority or discard safety restrictions.

## Decision

Use per-aggregate versions plus synchronization cursors and explicit merge policies. Safety restrictions, revocations, aborts, and grounding are monotonic and win conflicts. Mission progression requires a valid lease. Ambiguous conflicts enter a suspended state for human resolution; synchronization can never expand authority.

## Consequences

### Positive
- Deterministic, safety-biased recovery after partitions.

### Negative
- Context-specific merge policies and reconciliation tooling are complex.

### Neutral
- Telemetry projections may use last-write-wins only when no control authority is affected.

## Links
- [ADR index](README.md)
