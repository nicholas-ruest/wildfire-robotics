# ADR-063: Solar microgrid robot habitats

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: energy, stations, northern-operations

## Context

Millions of robots distributed through northern regions require secure housing, charging, thermal management, maintenance, communications, and immediate readiness despite seasonal solar variation, cold, smoke, snow, isolation, and grid outages.

## Decision

Build a hierarchy of standardized robot habitats: individual dock/pod → local solar habitat → sector depot → regional mobilization hub. Each habitat is an islandable DC-first microgrid with site-modeled PV, stationary battery, bidirectional robot/vehicle charging where safe, grid/hydro/wind connection where available, and a dispatchable low-carbon liquid-fuel range/resilience generator sized to critical winter autonomy. Solar is never the sole critical energy source. Habitats provide weather/fire hardening, snow management, ventilation, battery thermal conditioning, detection/suppression, segmented charging, maintenance/calibration, spares, communications, physical security, and safe evacuation. Energy control forecasts site resource, load, mission demand and outages; it reserves emergency departure/return energy and sheds training/indexing/optional loads before readiness, safety, identity, command, audit, heating, and communications.

## Consequences

### Positive
- Keeps robots charged, protected, serviced, distributed, and deployable with resilient local energy.
### Negative
- Northern winter generation, heating, snow, battery degradation, fuel logistics, and fire separation dominate sizing and cost.
### Neutral
- Final PV/storage/generator capacity is site- and robot-profile-specific, not a universal ratio.

## Links
- [ADR-003](ADR-003-edge-first-intermittently-connected-operations.md)
- [ADR-040](ADR-040-backup-recovery-and-continuity.md)
