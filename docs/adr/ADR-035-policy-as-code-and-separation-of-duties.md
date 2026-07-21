# ADR-035: Policy as code and separation of duties

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: authorization, safety

## Context

Authorization combines tenant, incident, role, assignment, geography, time, ODD, vehicle capability, and safety constraints.

## Decision

Evaluate versioned, signed policy bundles locally at every command boundary. Default deny applies to absent, stale, indeterminate, or conflicting policy. Decisions return policy version, inputs, outcome, reason, and obligations for audit. Safety-critical arming, authority expansion, release promotion, trust-root changes, and emergency override require declared separation of duties. Break-glass access is scoped, expiring, monitored, retrospectively reviewed, and cannot bypass independent physical safety controls.

## Consequences

### Positive
- Authorization is testable, explainable, consistent, and usable offline.
### Negative
- Policy compatibility and distribution require formal governance.
### Neutral
- Policy narrows authority; it does not create incident authority.

## Links
- [ADR-001](ADR-001-safety-led-human-command-authority.md)
- [ADR-010](ADR-010-zero-trust-identity-and-command-authorization.md)
