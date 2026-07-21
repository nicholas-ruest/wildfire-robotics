# ADR-072: Guided parafoil, tether, and progressive deployment control

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: parafoil, formation, control

## Context

Pre-connected robots descending under separate parafoils while progressively unfolding a shared flexible membrane create tightly coupled multi-body dynamics. Guidance or tether error can cause collision, entanglement, canopy collapse, shock loading, uncontrolled sail behavior, or broad ground hazard.

## Decision

Implement deployment as a certified finite-state protocol: retained → extracted/stabilized → cohort release → parafoil established → formation acquired → section reefed release → tension-balanced expansion → terrain alignment → landing → ground transition → anchoring/sealing. Every transition requires measured stability margins, navigation/time quality, formation separation, tether routing/tension, panel/vent state, wind envelope, drop-zone clearance, communications and remaining abort options. Use small bounded cohorts with local control and hierarchical coordination; no global controller is a single point of safe flight. Tethers have unique routing, length/rate/tension limits, cross/entanglement detection, powered-reel brakes, breakaway thresholds and safe-release sectors. Progressive reefing/vent control prevents instantaneous full inflation. Each robot has an independently reachable emergency landing/minimum-risk state; the cradle/panel system has independently safe retain, pause, vent, isolate and jettison states. ruv-drone may advise bounded coordination only behind the ADR-054 adapter and must be proven for this coupled load.

## Consequences

### Positive
- Converts deployment into testable transitions with explicit local containment and abort options.
### Negative
- Coupled aeroelastic/parafoil/tether control and validation are substantially harder than ordinary cooperative UAV flight.
### Neutral
- Existing guided parafoils are evidence of a component pattern, not evidence for this combined system.

## Links
- [ADR-012](ADR-012-progressive-autonomy-and-safe-state-contract.md)
- [ADR-054](ADR-054-ruv-drone-cooperative-uav-coordination.md)
