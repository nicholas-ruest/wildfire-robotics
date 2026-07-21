# ADR-062: Vegetation-management robot operations

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: robots, vegetation, prevention

## Context

Fuel reduction and vegetation removal are persistent preventive operations with different targets, hazards, evidence, scheduling, and outcomes from active-fire suppression.

## Decision

Create a Vegetation Management bounded context for survey, prescription, permit/constraint, treatment unit, work package, biomass disposition, and effectiveness monitoring. Robot missions may mow, mulch, cut, remove, transport, or inspect only within an approved prescription and ODD containing ownership, ecology, cultural/environmental exclusions, people/wildlife/utility controls, slope/terrain, weather/fire-danger, tool envelope, debris/biomass plan, and stop conditions. Drones provide mapping, progress/effect imagery, communications relay, and route inspection. Every treated area links planned vs actual geometry, method, machine/tool configuration, quantity, exceptions, imagery, and subsequent fuel/fire outcomes. Learned recommendations cannot widen a prescription or operate cutting tools without authorization.

## Consequences

### Positive
- Gives preventive robotics a precise domain instead of forcing it into suppression.
### Negative
- Treatment types, environments, tools, and outcome horizons add substantial model diversity.
### Neutral
- Vegetation treatment effectiveness is measured over time and is not assumed from completion.

## Links
- [ADR-012](ADR-012-progressive-autonomy-and-safe-state-contract.md)
- [ADR-059](ADR-059-closed-loop-prediction-observation-outcome-learning.md)
