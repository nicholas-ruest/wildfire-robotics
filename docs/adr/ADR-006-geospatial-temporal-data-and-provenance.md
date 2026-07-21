# ADR-006: Geospatial-Temporal Data and Provenance

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: geospatial, temporal, provenance

## Context

Hazards, missions, observations, routes, restrictions, and models are spatial and time-dependent. Safety decisions require knowledge of source, freshness, coordinate reference, uncertainty, and transformation history.

## Decision

Use PostgreSQL/PostGIS as the authoritative transactional geospatial store, object storage for immutable bulk artifacts, and purpose-built time-series/search projections where justified. Store event time and ingestion time, CRS, units, uncertainty, quality flags, license, source checksum, lineage, and processing version. Adopt OGC interfaces at external boundaries and canonical internal geometry conventions.

## Consequences

### Positive
- Reproducible analysis and defensible operational decisions.

### Negative
- Provenance and temporal corrections increase storage and pipeline complexity.

### Neutral
- Derived products remain linked to immutable inputs.

## Links
- [ADR-007](ADR-007-adopt-authoritative-hazard-data-and-models.md)
- [ADR-011](ADR-011-observability-audit-and-evidence-by-design.md)
