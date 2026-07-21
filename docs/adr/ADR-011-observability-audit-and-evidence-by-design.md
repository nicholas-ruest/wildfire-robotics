# ADR-011: Observability, Audit, and Evidence by Design

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: observability, audit, safety-case

## Context

Operators must distinguish stale data, degraded models, communications loss, unsafe hardware, and service failure. Regulators and investigators need a reconstructable record.

## Decision

Instrument services, models, stations, gateways, and vehicles with correlated metrics, logs, traces, domain events, data-quality signals, and time synchronization status. Keep a tamper-evident audit ledger for command/approval/configuration/model/release actions. Define SLOs and error budgets; preserve incident evidence under governed retention and privacy rules. Safety evidence links requirements, hazards, mitigations, tests, artifacts, and approvals.

## Consequences

### Positive
- Faster diagnosis, accountable operations, and evidence-backed releases.

### Negative
- Telemetry cost, privacy, retention, and disconnected buffering require careful controls.

### Neutral
- Observability data is operationally critical data.

## Links
- [ADR-005](ADR-005-event-driven-fleet-control-plane.md)
- [ADR-009](ADR-009-simulation-gated-cyber-physical-delivery.md)
