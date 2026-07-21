# ADR-036: Tenant isolation and dedicated deployment tiers

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: tenancy, commercial, security

## Context

Commercial service requires economical shared operation while some customers require physical or administrative isolation.

## Decision

Make tenant and region mandatory authenticated scopes in APIs, events, storage keys, cache keys, metrics, logs, exports, jobs, and support tooling. Shared deployments use database row-level security plus application enforcement and automated cross-tenant tests. Dedicated database, cluster, account, and offline deployment tiers are product options. Encryption keys are at least environment-scoped and tenant-scoped when contract or classification requires it. No global operator query exists without explicit privileged purpose and audit.

## Consequences

### Positive
- Provides defensible isolation with commercially tiered deployment choices.
### Negative
- Dedicated tiers increase release, support, and cost-management complexity.
### Neutral
- Tenant isolation applies equally to derived data and observability.

## Links
- [ADR-015](ADR-015-commercial-multi-tenant-product-boundary.md)
- [ADR-028](ADR-028-data-classification-retention-and-deletion-policy.md)
