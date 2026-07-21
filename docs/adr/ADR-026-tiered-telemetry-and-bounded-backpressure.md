# ADR-026: Tiered telemetry and bounded backpressure

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: implementation

## Context

National fleets can produce more telemetry than links, brokers, storage, and operators can consume, especially during incidents.

## Decision

Classify telemetry as safety-critical, operational, diagnostic, and bulk. Reserve bandwidth and storage for higher tiers; aggregate and sample lower tiers; use bounded buffers with explicit drop counters. Safety events and command acknowledgements are never silently dropped. Raw high-rate data is retained locally for a configurable incident window.

## Consequences

### Positive
- Predictable behavior under overload and constrained links.

### Negative
- Some low-priority detail is intentionally unavailable centrally.

### Neutral
- Every display exposes telemetry freshness and completeness.

## Links
- [ADR index](README.md)
