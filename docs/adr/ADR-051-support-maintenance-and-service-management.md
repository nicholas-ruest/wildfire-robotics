# ADR-051: Support, maintenance, and service management

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: commercial, operations

## Context

Commercial cyber-physical service needs staffed support, field maintenance, customer communication, and controlled remote assistance.

## Decision

Define service tiers, hours, response/restoration targets, severity, escalation, customer responsibilities, exclusions, and status communication in versioned contracts. Integrate support cases with tenant-safe diagnostics, incident/problem/change management, known errors, parts, warranties, calibration, preventive maintenance, training, and field dispatch. Remote support is consented, least-privileged, time-bounded, recorded, and unable to issue vehicle commands outside operational authority. Safety occurrences always enter Safety Assurance regardless of commercial severity.

## Consequences

### Positive
- Makes product operation supportable and contractually measurable.
### Negative
- Staffing, spares, training, and regional field response are major fixed costs.
### Neutral
- Restoring service never outranks safe operation.

## Links
- [ADR-015](ADR-015-commercial-multi-tenant-product-boundary.md)
- [ADR-039](ADR-039-observability-slos-and-operational-ownership.md)
