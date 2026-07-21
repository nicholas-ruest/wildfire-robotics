# ADR-054: ruv-drone cooperative UAV coordination

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: drones, coordination, dependency

## Context

Drones must map lightning/fire areas, relay communications, inspect robot routes, and collect imagery while remaining deconflicted and subordinate to incident and aviation authority.

## Decision

Adopt `ruvnet/ruv-drone` behind a Vehicle Integration anti-corruption adapter as the preferred cooperative-UAV coordination engine above PX4/ArduPilot. Use only evaluated modules: bounded-cohort topology/consensus, gossip, formation, cooperative task allocation, coverage planning, collision avoidance, relay missions, fail-safe integration, simulation, and optionally promoted MAPPO policies. Upstream contracts expose capabilities/tasks/outcomes rather than ruv-drone types. Cohorts remain bounded and hierarchical under ADR-053. Pin source/digest/license, maintain an internal compatibility facade and replacement implementation, threat-model every feature, and independently benchmark, simulate, HITL-test, and field-validate before promotion. Military targeting, threat-adaptive swarming, autonomous suppression target selection, and safety-authority creation remain out of scope.

## Consequences

### Positive
- Reuses a Rust-native cooperative UAV stack aligned with PX4/ArduPilot and relay/mapping missions.
### Negative
- The young dependency has no published releases and its small-cohort targets do not establish wildfire or million-fleet suitability.
### Neutral
- Autopilots retain flight control and independent safety functions.

## Links
- [ADR-008](ADR-008-px4-first-uas-integration-with-ardupilot-adapters.md)
- [ADR-014](ADR-014-open-standards-and-dependency-evaluation.md)
- [ruv-drone](https://github.com/ruvnet/ruv-drone)
