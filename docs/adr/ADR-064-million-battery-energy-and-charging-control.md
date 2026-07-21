# ADR-064: Million-battery energy and charging control

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: batteries, charging, scale

## Context

Tracking and charging millions of mobile batteries creates extreme telemetry, power, thermal-runaway, compatibility, degradation, scheduling, and simultaneous-recall risks.

## Decision

Assign every pack, module where serviceable, charger, dock, and energy store an immutable identity and digital twin. Vehicle BMS remains authoritative for cell-level safety; Fleet owns operational eligibility; Station owns charging sessions and local power constraints. Edge controllers ingest high-rate BMS data locally and publish event-driven exceptions plus bounded summaries. Track chemistry, form factor, firmware, certification, nominal/usable capacity, state of charge/health/power, temperature, insulation/isolation fault, cycles, throughput, fast-charge exposure, calibration uncertainty, location/custody, compatibility, warranty, predicted remaining life, reserve and quarantine state. Optimize charging by mission deadline, readiness class, degradation cost, site forecast, grid/generator constraint and thermal/fire zones. Commands are fenced, bounded, BMS-enforced and fail safe; bulk schedules remain partitioned by habitat/cell.

## Consequences

### Positive
- Enables safe readiness and lifecycle optimization without cloud-level cell telemetry.
### Negative
- Heterogeneous BMS semantics, estimation error and correlated charging demand require rigorous adapters and forecasting.
### Neutral
- State of charge is an estimate with time, method and uncertainty—not an exact quantity.

## Links
- [ADR-026](ADR-026-tiered-telemetry-and-bounded-backpressure.md)
- [ADR-053](ADR-053-million-asset-hierarchical-fleet-architecture.md)
