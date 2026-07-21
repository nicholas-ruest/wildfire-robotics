# ADR-009: Simulation-Gated Cyber-Physical Delivery

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: simulation, validation, release

## Context

Software faults can move machines, interfere with aircraft, or apply suppressants. Ordinary CI is insufficient evidence.

## Decision

Every behavioral change progresses through deterministic unit/property tests, contract/integration tests, scenario simulation, fault injection, software-in-loop, hardware-in-loop, controlled field test, and authorized operations as applicable. Promotion evidence is signed and immutable. Safety invariants are independently implemented in simulation or monitors. A human approves each physical promotion; production has canary, isolation, and rollback.

## Consequences

### Positive
- Detects unsafe interactions before exposure and creates auditable evidence.

### Negative
- Simulator fidelity, scenario curation, hardware labs, and field facilities are major investments.

### Neutral
- Passing simulation is necessary but never sufficient for field authorization.

## Links
- [Production readiness](../operations/production-readiness.md)
- [ADR-001](ADR-001-safety-led-human-command-authority.md)
