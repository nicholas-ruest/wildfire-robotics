# ADR-031: Model registry with immutable releases and approval stages

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: implementation

## Context

Nowcasting, perception, and planning models can drift, be retrained, or change dependencies. A filename is not adequate production identity.

## Decision

Maintain immutable model releases with artifact digest, training code and data lineage, features, parameters, metrics, calibration, intended ODD, license, security scan, approvers, and stage. Only approved releases run operationally. Shadow, canary, rollback, and retirement are first-class transitions.

## Consequences

### Positive
- Reproducible decisions and controlled model changes.

### Negative
- Registry integration and evidence collection slow experimentation.

### Neutral
- Model approval does not authorize a mission.

## Links
- [ADR index](README.md)
