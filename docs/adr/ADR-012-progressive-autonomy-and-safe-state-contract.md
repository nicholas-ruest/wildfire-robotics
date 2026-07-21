# ADR-012: Progressive Autonomy and Safe-State Contract

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: autonomy, safe-state, odD

## Context

No production-proven autonomous wildland suppression system exists. Capability and confidence vary by vehicle, environment, sensor health, communications, and mission.

## Decision

Represent autonomy as explicit levels per capability and operational design domain (ODD): advisory, teleoperated, supervised execution, and only then bounded autonomous execution. Each vehicle advertises verified capabilities and health. Authorization is valid only inside its ODD and time window. Boundary breach, stale command, localization uncertainty, safety-monitor disagreement, or critical fault triggers a vehicle-specific minimum-risk condition and human notification.

## Consequences

### Positive
- Prevents blanket autonomy claims and supports evidence-based progression.

### Negative
- ODD modeling, runtime assurance, and safe-state verification are substantial work.

### Neutral
- Full autonomy is not a mandatory commercial release criterion.

## Links
- [ADR-001](ADR-001-safety-led-human-command-authority.md)
- [ADR-003](ADR-003-edge-first-intermittently-connected-operations.md)
