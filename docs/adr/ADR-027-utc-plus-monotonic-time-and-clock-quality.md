# ADR-027: UTC plus monotonic time and clock quality

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: implementation

## Context

Expiry, ordering, sensor fusion, audit reconstruction, and deconfliction fail when device clocks drift or jump.

## Decision

Represent external timestamps in UTC and durations/deadlines using monotonic clocks. Synchronize with authenticated NTP, PTP, or GNSS sources as appropriate, publish clock offset and uncertainty, reject safety actions beyond configured uncertainty, and preserve source event time plus ingestion time.

## Consequences

### Positive
- Makes temporal uncertainty explicit and testable.

### Negative
- Time hardware, holdover behavior, and leap-second testing add cost.

### Neutral
- Sequence numbers and causation complement timestamps; time alone never establishes order.

## Links
- [ADR index](README.md)
