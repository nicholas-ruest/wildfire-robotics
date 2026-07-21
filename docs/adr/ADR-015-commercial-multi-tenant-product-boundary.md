# ADR-015: Commercial Multi-Tenant Product Boundary

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: commercial, tenancy, product

## Context

Commercial viability requires multiple agencies/regions, contractual isolation, entitlements, support, metering, and evidence without allowing business concerns to contaminate safety-critical control.

## Decision

Separate customer administration and commercial operations from incident operations. Tenant, region, incident, and authority are explicit scopes enforced in identity, storage, events, observability, and export. Billing/metering consumes operational facts asynchronously and can never authorize, delay, or revoke an active safety action. Offer dedicated deployments where regulation or contract requires stronger isolation.

## Consequences

### Positive
- Supports commercial operation and public-sector isolation requirements.

### Negative
- Tenant-aware testing, support, data lifecycle, and deployment variants expand scope.

### Neutral
- Safety and incident authority outrank commercial entitlement during an authorized incident.

## Links
- [ADR-002](ADR-002-bounded-context-modular-platform.md)
- [ADR-010](ADR-010-zero-trust-identity-and-command-authorization.md)
