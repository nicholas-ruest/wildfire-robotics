# ADR-067: Mass mobilization capacity and flow control

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: scale, logistics, optimization

## Context

Dispatching tens or hundreds of thousands of robots can gridlock depots, routes, chargers, unload zones, communications and incident staging even when enough carriers exist.

## Decision

Model mobilization as a capacitated time-expanded flow network over habitats, pods, robots, carriers, routes, bridges/ferries/rail/barge, charging/refueling, drivers/supervisors where applicable, staging/unload lanes, maintenance, communications and destination absorption. Plans optimize time-to-useful-capability rather than departure count and reserve corridor/time slots, energy, pods, carriers, loading equipment and destination capacity. Release bounded waves with admission control, rendezvous checkpoints, independent convoy cells and continuously recomputed ETA/capacity. Never launch assets that cannot be safely routed, staged, energized, communicated with, unloaded, maintained or assigned. Simulate evacuation conflicts, road closure, bridge/load constraints, carrier failure, charging queues, smoke/weather and incident-priority changes.

## Consequences

### Positive
- Converts spectacular fleet size into deployable operational throughput.
### Negative
- Physical bottlenecks and uncertain routes may make the fastest safe plan counterintuitive.
### Neutral
- Readiness is measured at destination and available for work, not merely stored or moving.

## Links
- [ADR-043](ADR-043-process-managers-compensation-and-human-escalation.md)
- [ADR-065](ADR-065-standardized-robot-pods-and-intermodal-mobilization.md)
