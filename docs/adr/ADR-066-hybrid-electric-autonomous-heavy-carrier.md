# ADR-066: Hybrid-electric autonomous heavy carrier

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: transport, autonomy, energy

## Context

Remote long-distance robot transport needs electric low-speed control and regenerative braking, but northern range, payload, cold, disrupted charging and emergency mobilization make battery-only operation an availability risk.

## Decision

Use a modular flatbed/chassis carrier family with electric traction and a series-hybrid range extender: batteries drive traction; a small dispatchable liquid-fuel generator operates near efficient set points to replenish the DC bus during long travel or outage. Depot solar/grid energy supplies normal charging; vehicle-mounted PV supplies auxiliary/hotel/trickle loads and is not credited for propulsion range. Carriers expose swappable standardized pods, redundant braking/steering/power isolation, local obstacle and stability protection, load/axle/centre-of-gravity verification, tire/thermal health, degraded manual/remote recovery, V2X and convoy interfaces. Automated driving and platooning are separately promoted by route/season/weather/communications ODD. Each vehicle can stop safely without leader/cloud/V2X, and human-supervised modes remain available until higher autonomy is evidenced.

## Consequences

### Positive
- Combines electric control/efficiency with range and outage resilience.
### Negative
- Dual power systems add mass, maintenance, controls and failure modes.
### Neutral
- Fuel type and generator size are selected from route/energy evidence, not fixed by this ADR.

## Links
- [ADR-012](ADR-012-progressive-autonomy-and-safe-state-contract.md)
- [ADR-047](ADR-047-vehicle-firmware-ota-and-fleet-compatibility.md)
