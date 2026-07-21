# ADR-023: Canonical authenticated command envelope

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: implementation

## Context

A robot command must never be accepted without identity, authority, freshness, deduplication, and safety context.

## Decision

Every command contains command ID, idempotency key, issuer and approver identities, tenant/incident/mission/vehicle scope, issued/expiry times, expected aggregate version, capability, payload schema version, safety-constraint and ODD versions, correlation/causation IDs, and detached signature. Missing, stale, replayed, unauthorized, or mismatched commands fail closed.

## Consequences

### Positive
- Creates an auditable and mechanically enforceable control boundary.

### Negative
- Larger envelopes and clock/key dependencies increase integration work.

### Neutral
- Transport acknowledgement is not proof of physical execution.

## Links
- [ADR index](README.md)
