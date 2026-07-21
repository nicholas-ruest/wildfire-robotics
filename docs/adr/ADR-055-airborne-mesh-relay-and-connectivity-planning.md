# ADR-055: Airborne mesh relay and connectivity planning

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: networking, drones, edge

## Context

Terrain, fire, infrastructure loss, congestion, and moving robots create intermittent links. Relay drones can improve coverage but cannot become a hidden single point of command or safety.

## Decision

Treat connectivity as a measured, forecast, and allocated mission resource. Build time-versioned link-quality/coverage maps from station, robot, drone, terrain, spectrum, weather, congestion, and energy observations. Mission Control may allocate bounded ruv-drone relay cohorts and reposition them within approved airspace/energy/ODD to meet declared service classes. Use store-and-forward, multipath, disruption-tolerant queues, priority admission, congestion control, cryptographic peer identity, loop prevention, and bounded routing convergence. Commands remain expiring/idempotent and robots retain local safety through total network loss. Relay optimization never overrides airspace, collision, return-energy, or emergency constraints.

## Consequences

### Positive
- Makes communications availability observable and actively improvable.
### Negative
- RF modeling, spectrum coexistence, energy limits, and moving topology complicate guarantees.
### Neutral
- Relay drones improve reach; they do not make networks reliable enough for remote safety dependence.

## Links
- [ADR-003](ADR-003-edge-first-intermittently-connected-operations.md)
- [ADR-054](ADR-054-ruv-drone-cooperative-uav-coordination.md)
