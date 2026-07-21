# ADR-010: Zero-Trust Identity and Command Authorization

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: security, identity, authorization

## Context

Compromised credentials, devices, or links could create physical harm. Network location cannot establish trust.

## Decision

Assign cryptographic workload/device identities rooted in managed hardware where feasible. Use short-lived credentials, mutual authentication, least privilege, tenant and incident scoping, policy-as-code, separation of duties, step-up approval for actuation, signed commands/artifacts, revocation, and offline-verifiable authorization envelopes. Encrypt data in transit and at rest and maintain key rotation and recovery.

## Consequences

### Positive
- Limits compromise and creates attributable control actions.

### Negative
- Offline identity, revocation, key custody, and field replacement are operationally demanding.

### Neutral
- Safety validation remains necessary even for authenticated commands.

## Links
- [ADR-001](ADR-001-safety-led-human-command-authority.md)
- [ADR-005](ADR-005-event-driven-fleet-control-plane.md)
