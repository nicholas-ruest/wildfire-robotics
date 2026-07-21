# ADR-017: Deployable modular services with context-owned data

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: implementation

## Context

The bounded contexts need independent ownership and fault isolation without paying the operational cost of a microservice for every aggregate.

## Decision

Implement each bounded context as a Rust crate with a clean application port. Package a context as an independent service only when scaling, security, availability, regulatory isolation, or release cadence requires it. Co-located modules still communicate through application interfaces and never share domain persistence.

## Consequences

### Positive
- Clear extraction path and lower initial operational complexity.

### Negative
- Deployment topology can vary by environment and requires architecture tests.

### Neutral
- Bounded context and deployable unit remain distinct concepts.

## Links
- [ADR index](README.md)
