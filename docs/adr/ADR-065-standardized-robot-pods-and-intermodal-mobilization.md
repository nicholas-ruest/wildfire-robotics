# ADR-065: Standardized robot pods and intermodal mobilization

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: logistics, transport, robots

## Context

Robots need housing that can transition rapidly from protected storage/charging to road, rail, barge, airlift where feasible, and field staging. A single carrier for 100,000 robots is structurally and operationally infeasible.

## Decision

Use standardized lockable open-top/ventilated robot pods and racks derived from interoperable freight-container dimensions/fittings where practical. A pod is simultaneously habitat module, charging interface, inventory/custody unit, health/communications node, and intermodal load. Pods declare robot/tool compatibility, mass/centre-of-gravity, structural/securement limits, energy isolation, thermal/fire zones, charger/data interfaces, ingress protection, loading/unloading mode, and emergency access. Mobilize 100,000+ robots as a coordinated wave of independently routable pods across many autonomous carriers and, where advantageous, rail/barge transfer—not one articulated structure. Every pod/carrier remains independently safe, manually recoverable, geofenced and epoch-fenced; a convoy is partitioned into bounded platoons with planned staging, charging/refueling, handoff, separation, alternate routes and failure recovery.

## Consequences

### Positive
- Makes robot housing transportable, interchangeable, parallel, resilient and compatible with existing logistics assets.
### Negative
- Standard interfaces constrain robot dimensions and require substantial loading, staging and transfer infrastructure.
### Neutral
- “100,000 at once” means one scheduled mobilization wave with a declared arrival window.

## Links
- [ADR-060](ADR-060-wildfire-supply-chain-and-resource-digital-thread.md)
- [ISO freight-container standards](https://www.iso.org/sectors/transport/freight-containers)
