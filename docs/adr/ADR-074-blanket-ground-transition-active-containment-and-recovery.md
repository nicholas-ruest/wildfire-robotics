# ADR-074: Blanket ground transition, active containment, and recovery

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: grounding, containment, recovery

## Context

After landing, robots must convert an unstable aerial system into a bounded ground installation, then perform anchoring, gap sealing, monitoring, edge suppression and vegetation work. Damaged or abandoned panels, parafoils, tethers, anchors and chemicals create ongoing hazards and custody obligations.

## Decision

Treat ground transition and active blanket operation as a distinct promotion stage. Robots land and establish safe tool/traction state before authorized anchor movement; validate terrain ownership, underground/overhead utilities, people/wildlife, slope, fuel/fire state and anchor compatibility. Panel contact, vent closure, tensioning and joints proceed in bounded zones with rollback/vent/isolation paths. Suppression agent use remains under Suppression Operations; vegetation removal remains under Vegetation Management; the aerial context coordinates installation only. Every section exposes top/bottom temperature, ember, contact, tension, wind/uplift, tear and gap state with freshness/uncertainty. Repositioning cannot drag fire, damage infrastructure, entrap people/animals, or expand the approved footprint. Recovery accounts for every serialized component and exposure; contaminated/damaged components enter Robot Care/Logistics quarantine, cleaning, repair, reuse, recycling or approved sacrificial disposition. Effectiveness compares protected versus counterfactual/baseline outcomes and never assumes containment from installation.

## Consequences

### Positive
- Preserves existing domain ownership and manages the blanket through its full physical lifecycle.
### Negative
- Terrain, utilities, wind, contamination and damaged-material recovery may dominate field operations.
### Neutral
- Installed panels provide temporary protection while durable containment is constructed.

## Links
- [ADR-060](ADR-060-wildfire-supply-chain-and-resource-digital-thread.md)
- [ADR-062](ADR-062-vegetation-management-robot-operations.md)
- [ADR-068](ADR-068-robot-care-recovery-and-maintenance-network.md)
