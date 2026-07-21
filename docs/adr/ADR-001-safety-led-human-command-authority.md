# ADR-001: Safety-Led Human Command Authority

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: safety, governance, incident-command

## Context

Robots will operate around responders, civilians, aircraft, infrastructure, and active fire. Existing wildfire robotics is human-supervised, and incident command and aviation authorities retain legal and operational control.

## Decision

The platform is advisory or human-supervised by default. Every mission and actuation requires an authenticated authority, approved operational envelope, expiring authorization, local safety monitor, and auditable chain of command. Emergency stop, geofence, airspace restriction, loss-of-control-link policy, and incident-commander override are independent hard constraints. Autonomy may optimize only inside them.

## Consequences

### Positive
- Preserves accountable human authority and enables incremental certification.

### Negative
- Adds approval latency, safety engineering, training, and operational staffing.

### Neutral
- “Autonomous” denotes bounded execution, not transfer of legal authority.

## Links
- [ADR-009](ADR-009-simulation-gated-cyber-physical-delivery.md)
- [ADR-012](ADR-012-progressive-autonomy-and-safe-state-contract.md)
