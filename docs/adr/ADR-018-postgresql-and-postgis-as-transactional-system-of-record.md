# ADR-018: PostgreSQL and PostGIS as transactional system of record

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: implementation

## Context

Incidents, missions, assets, constraints, and hazard products require ACID transactions plus mature geospatial queries.

## Decision

Use supported PostgreSQL with PostGIS for authoritative transactional and geospatial state. Each context owns schemas and migrations; cross-context reads use APIs, events, or projections. Use row-level security only as defense in depth, not the sole tenant boundary.

## Consequences

### Positive
- Mature durability, indexing, geospatial operations, and ecosystem.

### Negative
- Requires disciplined migrations, partitioning, vacuum, replication, and capacity management.

### Neutral
- Specialized stores may serve derived projections but never silently become authoritative.

## Links
- [ADR index](README.md)
