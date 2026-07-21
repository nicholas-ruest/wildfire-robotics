# ADR-046: Digital twin, scenario, and test evidence

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: simulation, testing

## Context

Safe and economical validation requires reproducible simulation without treating simulation as equivalent to field evidence.

## Decision

Version simulator, environment, vehicle, sensor, communications, weather, fire, and fault models as evidence-bearing artifacts with provenance and declared validity. Scenarios have stable IDs, seeds, requirements/hazards covered, expected invariants, tolerances, and result retention. Calibrate models against hardware and field observations, quantify simulation-to-reality gaps, and prevent evidence use outside validated domains. Promotion requires increasing fidelity through unit/property, contract, integration, scenario, SIL, HITL, controlled field, and operational monitoring gates.

## Consequences

### Positive
- Enables repeatable regression and broad hazardous-scenario exploration.
### Negative
- Model calibration and scenario maintenance are substantial products themselves.
### Neutral
- Simulation supports but never replaces required physical validation.

## Links
- [ADR-009](ADR-009-simulation-gated-cyber-physical-delivery.md)
- [ADR-045](ADR-045-assurance-case-hazard-analysis-and-independent-verification.md)
