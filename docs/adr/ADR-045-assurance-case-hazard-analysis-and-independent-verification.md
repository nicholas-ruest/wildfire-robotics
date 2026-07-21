# ADR-045: Assurance case, hazard analysis, and independent verification

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: safety, assurance

## Context

Cyber-physical release requires evidence that hazards are controlled within an approved operational design domain.

## Decision

Maintain a versioned assurance case linking system definition, intended use, hazards, causal analysis, safety requirements, mitigations, verification, residual risk, assumptions, ODD, competency, approvals, occurrences, and deployed configuration. Use recognized system/software safety practices selected with the responsible authority. Safety-critical requirements have bidirectional traceability and independent verification proportionate to risk. Unmet evidence, invalid assumptions, near misses, or changed configuration suspend or narrow promotion. Only accountable human authorities accept residual risk.

## Consequences

### Positive
- Makes safety claims bounded, reviewable, and evidence-backed.
### Negative
- Independent analysis and field evidence impose time and cost.
### Neutral
- No document alone proves safe operation.

## Links
- [ADR-001](ADR-001-safety-led-human-command-authority.md)
- [ADR-009](ADR-009-simulation-gated-cyber-physical-delivery.md)
