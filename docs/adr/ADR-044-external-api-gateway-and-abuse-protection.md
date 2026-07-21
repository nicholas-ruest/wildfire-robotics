# ADR-044: External API gateway and abuse protection

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: api, security

## Context

Customer, partner, support, and public integrations need stable interfaces without exposing internal services or allowing overload into safety paths.

## Decision

Terminate external APIs at managed gateways enforcing TLS, identity, authorization context, schema/size limits, rate and concurrency quotas, request deadlines, idempotency, threat filtering, and audit. Publish OpenAPI/OGC contracts, lifecycle policy, compatibility guarantees, SDKs, sandbox, and deprecation dates. Separate command, bulk ingestion, query, export, and public traffic pools. Shed noncritical work before operational control traffic and never retry unsafe non-idempotent operations automatically.

## Consequences

### Positive
- Creates a governable customer boundary and protects internal capacity.
### Negative
- Gateway policy and client compatibility become product responsibilities.
### Neutral
- Internal authorization is still enforced by the owning context.

## Links
- [ADR-022](ADR-022-grpc-internally-and-rest-ogc-interfaces-externally.md)
- [ADR-026](ADR-026-tiered-telemetry-and-bounded-backpressure.md)
