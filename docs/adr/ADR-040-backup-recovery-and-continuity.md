# ADR-040: Backup, recovery, and continuity

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: resilience, operations

## Context

Wildfire operations must survive regional cloud loss, station isolation, corruption, operator error, and ransomware without unsafe continuation.

## Decision

Define per-data-product RPO, RTO, restore order, minimum viable service, and authority for disaster declaration. Use encrypted, immutable, access-isolated backups with cross-failure-domain copies and tested point-in-time recovery. Stations retain bounded local mission, policy, identity, map, and audit capability and enter defined degraded modes during cloud loss. Recovery never resurrects expired authority or blindly replays commands. Conduct scheduled restore, failover, station-loss, regional-loss, and communications exercises with measured evidence.

## Consequences

### Positive
- Converts backup claims into demonstrated continuity.
### Negative
- Isolated copies and exercises add cost and operational load.
### Neutral
- Recovery restores data and service, not operational authority.

## Links
- [ADR-003](ADR-003-edge-first-intermittently-connected-operations.md)
- [ADR-025](ADR-025-version-vector-edge-reconciliation-with-authority-precedence.md)
