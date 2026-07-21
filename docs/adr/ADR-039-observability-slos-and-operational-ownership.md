# ADR-039: Observability, SLOs, and operational ownership

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: reliability, operations

## Context

Production behavior cannot be controlled without service, data-quality, model-quality, safety, security, and physical-system signals tied to accountable owners.

## Decision

Use OpenTelemetry-compatible metrics, traces, and structured logs with propagated tenant, incident, correlation, causation, release, and location-safe identifiers. Each deployable owns SLIs, SLOs, error budgets, dashboards, actionable alerts, runbooks, capacity limits, synthetic checks, and an on-call team. Safety/security events use independent durable paths. Telemetry is classified, redacted, cardinality-bounded, sampled by policy, and never allowed to block a safety action. SLO breach halts risky promotion and triggers review.

## Consequences

### Positive
- Makes reliability measurable and operational accountability explicit.
### Negative
- Storage cost, privacy, sampling, and alert quality require continuous governance.
### Neutral
- Absence of an alert is not proof of safety.

## Links
- [ADR-011](ADR-011-observability-audit-and-evidence-by-design.md)
- [ADR-026](ADR-026-tiered-telemetry-and-bounded-backpressure.md)
