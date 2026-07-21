# ADR-042: Configuration, feature flags, and runtime change

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: operations, safety

## Context

Uncontrolled runtime configuration can bypass review, fragment behavior, or expand cyber-physical authority.

## Decision

Use typed, schema-versioned configuration with declared owner, safe default, bounds, classification, rollout scope, and restart semantics. Configuration and flags are reviewed, signed, auditable, staged, observable, and automatically expired when temporary. Unknown or invalid safety-relevant values fail closed. Feature flags may isolate or narrow behavior but cannot enable unapproved capability, ODD, authority, or suppression. Vehicle safety limits require the same evidence and promotion controls as code.

## Consequences

### Positive
- Enables controlled rollout without a hidden safety bypass.
### Negative
- Flag inventory and compatibility require active cleanup.
### Neutral
- A configuration-only change may still be a formal release.

## Links
- [ADR-012](ADR-012-progressive-autonomy-and-safe-state-contract.md)
- [ADR-009](ADR-009-simulation-gated-cyber-physical-delivery.md)
