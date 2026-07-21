# ADR-060: Wildfire supply chain and resource digital thread

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: logistics, supply-chain

## Context

Fire response depends on robots, batteries, fuel, suppressant, water, parts, sensors, payloads, stations, transport, maintenance, and custody across volatile demand and disrupted routes.

## Decision

Extend Logistics and Station Operations with an end-to-end resource digital thread: item/batch/serial identity, owner/custodian, quantity/unit, condition, location, compatibility, hazard class, expiry, calibration/maintenance, provenance, reservation, demand, lead time, supplier/route, handoff, consumption, recovery, and waste. Forecast demand from incident objectives, hazard scenarios, fleet state, burn rates, lead-time uncertainty, and service levels. Optimize multi-echelon stock, staging, replenishment, routing, charging/refueling, maintenance, spares, water/agent availability, and reverse logistics subject to safety and authority. Every recommendation shows assumptions, uncertainty, bottleneck, feasible alternatives, and human override; physical custody and inventory reconciliation remain authoritative.

## Consequences

### Positive
- Couples operational plans to feasible material, energy, maintenance, and transport capacity.
### Negative
- Data quality, supplier integration, substitutions, and disrupted lead times limit optimization.
### Neutral
- Optimization advises allocation; it does not override incident priorities or safety.

## Links
- [ADR-006](ADR-006-geospatial-temporal-data-and-provenance.md)
- [ADR-043](ADR-043-process-managers-compensation-and-human-escalation.md)
