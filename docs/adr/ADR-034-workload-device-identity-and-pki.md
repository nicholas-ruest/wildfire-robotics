# ADR-034: Workload and device identity with managed PKI

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: security, identity

## Context

People, services, stations, vehicles, and controllers must authenticate across disconnected and hostile networks without shared credentials.

## Decision

Use separate managed trust domains for human, workload, and device identity. Workloads receive short-lived identities through workload attestation; devices use hardware-backed, non-exportable keys and manufacturer/onboarding evidence. Mutual TLS is mandatory across trust boundaries. Certificate issuance, renewal, revocation, rollover, compromise response, offline revocation bundles, root rotation, and cryptographic-agility procedures are automated and exercised. Identity proves a principal; authorization remains a separate policy decision.

## Consequences

### Positive
- Removes static shared credentials and makes trust attributable and revocable.
### Negative
- Offline renewal and root rotation add operational complexity.
### Neutral
- A valid certificate never implies permission to actuate.

## Links
- [ADR-010](ADR-010-zero-trust-identity-and-command-authorization.md)
- [ADR-023](ADR-023-canonical-authenticated-command-envelope.md)
