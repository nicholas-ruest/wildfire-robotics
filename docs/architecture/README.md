# Wildfire Robotics Architecture

This directory is the governed architecture baseline for the Canada-first Wildfire Robotics platform. The research in `.plans/deep-research.md` is the evidence base; ADRs record binding technical decisions; the DDD context map defines ownership and integration boundaries; the implementation plan defines the evidence required to promote software or hardware.

## Authority order

1. Applicable law, regulator direction, incident-command authority, and approved safety case
2. Accepted ADRs and published interface contracts
3. Bounded-context domain models and service-level objectives
4. Implementation details

No autonomous action may override a human incident commander, an airspace restriction, a geofence, an emergency stop, or a safety invariant.

## Architecture views

- [Implementation program](../implementation/master-plan.md)
- [ADR index](../adr/README.md)
- [DDD context map](../ddd/context-map.md)
- [Tactical DDD standard](../ddd/tactical-model-standard.md)
- [Integration contract registry](../ddd/integration-contracts.md)
- [Cross-context process managers](../ddd/process-managers.md)
- [Assurance traceability model](../ddd/traceability-model.md)
- [Million-robot physical scale model](million-robot-physical-scale.md)
- [Ubiquitous language](../ddd/ubiquitous-language.md)
- [Production readiness](../operations/production-readiness.md)

## Decision state

ADRs begin as `proposed`. They become `accepted` only after named deciders approve them and any listed evidence prerequisites are met. An ADR governing safety-critical behavior cannot be accepted solely by code review.
