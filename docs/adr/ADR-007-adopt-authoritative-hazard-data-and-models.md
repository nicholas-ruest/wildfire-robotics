# ADR-007: Adopt Authoritative Hazard Data and Models

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: build-vs-buy, wildfire, data

## Context

Official and peer-reviewed systems already provide lightning, hotspot, danger-rating, and spread capabilities. Reimplementation would add validation and liability without differentiation.

## Decision

Integrate licensed/approved GOES GLM, WWLLN or commercial lightning, NASA FIRMS, CWFIS, CFFDRS, Cell2Fire, and selectively WRF-Fire through anti-corruption adapters. Build wildfire-specific fusion, confidence handling, mission planning, and operations. Community feeds are corroborative only. Each adapter has terms-of-use, attribution, quality, freshness, outage, replay, and replacement controls.

## Consequences

### Positive
- Concentrates engineering on differentiating operational capability.

### Negative
- External availability, schema, licensing, and scientific-version changes must be managed.

### Neutral
- No source alone authorizes an operational action.

## Links
- [Research](../../.plans/deep-research.md)
- [ADR-006](ADR-006-geospatial-temporal-data-and-provenance.md)
