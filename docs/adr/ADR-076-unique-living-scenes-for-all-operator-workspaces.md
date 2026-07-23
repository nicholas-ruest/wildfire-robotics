# ADR-076: Unique living scenes for all operator workspaces

- **Status**: proposed
- **Date**: 2026-07-23
- **Deciders**:
- **Tags**: operator-console, threejs, workspaces, visualization, digital-twin

## Context

The operator console has fifteen left-sidebar workspaces. Reusing one map, one node graph, or one animation with different labels undermines comprehension and makes the product feel decorative. Operators need each tab to reveal an immediately recognizable domain model, current activity, exceptions, and management affordances.

Uniqueness must come from domain semantics and interaction—not arbitrary camera angles, colors, or visual effects. At the same time, the scenes must share enough conventions that operators do not relearn selection, freshness, uncertainty, alerts, confirmations, and command outcomes on every tab.

## Decision

Implement a distinct Three.js scene module and paired semantic management panel for every sidebar workspace. No production tab may fall back to the incident scene or a generic four-node topology. Each scene has a unique primary spatial metaphor, asset vocabulary, motion model, camera behavior, inspection model, and management workflow:

| Workspace | Unique living scene | Dynamic state | End-user management |
|---|---|---|---|
| Incident Command | Layered terrain command table with perimeter, divisions, objectives, resources, and exclusion volumes | perimeter revisions, objective progress, resource movement, weather and authority windows | select operational period, assign resources, stage objectives, publish/hold plan |
| Hazard Intelligence | Spatiotemporal observation field with sensor rays, confidence surface, provenance layers, and freshness decay | incoming observations, confidence/uncertainty, stale regions, superseded lineage | filter sources, scrub time, compare snapshots, publish bounded hazard picture |
| Predictive Planning | Branching scenario laboratory with animated fire-spread volumes and synchronized timelines | ensemble runs, probability envelopes, drift, horizon and model validity | change bounded assumptions, run/cancel simulation, compare scenarios, promote advisory |
| Mission Control | Spatial mission graph connecting objectives, geofences, leases, vehicles, conflicts, and outcomes | authorization stages, conflicts, lease expiry, dispatch and outcome states | compose mission, resolve conflicts, reserve assets, dispatch, hold or revoke intent |
| Fleet Operations | Hierarchical fleet-cell constellation with instanced vehicles clustered by readiness and epoch | eligibility, energy, health, membership, allocation and partitions | filter/select cohorts, rebalance cells, ground/release eligible assets, export capacity |
| Vehicle Integration | Exploded vehicle digital twin with subsystem telemetry and command-path pulses | gateway sessions, adapters, acknowledgements, actuator/sensor state, fault propagation | inspect subsystem, run diagnostic, stage adapter action, request safe-state transition |
| Station Operations | Live microgrid and habitat cutaway with energy particles, bays, reserves, and edge workloads | generation, load, charge sessions, reserve, service readiness and load shedding | prioritize loads, schedule bays, optimize charging, protect/release reserve |
| Logistics | Supply-network flow landscape with sources, custody nodes, carriers, routes, and destinations | reservations, inventory, custody transfers, route degradation, ETA distribution | create/release delivery, reroute carrier, reserve supply, reconcile arrival |
| Vegetation | Parcel-scale treatment twin with vegetation density, prescriptions, exclusions, and robot swaths | planned/actual geometry, treatment progress, exclusions, tool state, effectiveness | edit bounded prescription, approve work unit, pause cohort, inspect evidence |
| Suppression | Hydraulics and target-envelope scene with pumps, relay chain, nozzles, dose field, and stop path | measured flow/pressure, target error, cumulative dose, arming and independent-stop state | adjust within envelope, arm with required approval, inhibit flow, reconcile dose |
| Aerial Deployment | Airspace-to-ground deployment sequence showing aircraft, corridor, payload, cohorts, panels, tethers, and anchors | manifest, release gates, dispersion, formation, tension, alignment and recovery | inspect manifest, approve checklist, hold/advance phase, abort or command recovery |
| Safety Assurance | Assurance-case graph connected to hazards, constraints, evidence, occurrences, and promotion gates | evidence freshness, open occurrences, control verification and expiring reviews | open occurrence/review, apply constraint, request evidence, approve or block promotion |
| Identity & Access | Zero-trust authority graph with principals, devices, scoped grants, signatures, and command envelopes | grant expiry, device trust, approval chains, replay denial and revocation propagation | issue/revoke scoped grant, inspect authority path, review approval, verify envelope |
| Robot Care | Recovery and hospital scene with field hazard, medic pod route, quarantine bays, repair cells, and burn-in | case stage, hazards, custody, bay occupancy, repair proof and recertification | dispatch medic, reserve bay, transfer custody, quarantine, approve return to service |
| Commercial Ops | Tenant operations model with usage streams, entitlement boundaries, ledger, support health, and ROI distributions | rated events, budget/SLO burn, cases, invoices and sensitivity ranges | filter period, review adjustments, close billing period, inspect/export investment case |

Every scene must include:

1. A stable overview camera and a guided “reset view” action.
2. At least three meaningful selectable entity types and a semantic details panel.
3. At least one time-varying operational signal driven by the workspace read model.
4. Visible current, stale, degraded, gap, and unknown representations where applicable.
5. A legend, units, timestamps, uncertainty, provenance, limitations, and simulation/live-data labeling.
6. One complete management workflow that produces a staged action, confirmation where required, receipt, lifecycle updates, and an explicit physical-outcome state.
7. Deterministic demo fixtures and a non-animated semantic fallback containing equivalent essential information.

Scenes share design-system tokens for status semantics, typography, selection, focus, alerts, and command stages. They must differ in scene graph and domain behavior. Acceptance review rejects a scene if changing its title and labels would make it plausibly represent another workspace.

Implement scenes incrementally behind per-workspace feature flags. The recommended delivery slices are:

1. Core platform plus Incident Command, Fleet Operations, and Station Operations.
2. Hazard Intelligence, Predictive Planning, and Mission Control.
3. Vehicle Integration, Logistics, Vegetation, and Suppression.
4. Aerial Deployment, Safety Assurance, Identity & Access, Robot Care, and Commercial Ops.

Existing HTML/SVG views remain available until the corresponding scene passes its acceptance gates. Feature flags support rollback per workspace without disabling the entire console.

## Consequences

### Positive

- Makes every sidebar selection visibly and functionally distinct.
- Aligns spatial metaphors and controls with the owning bounded context.
- Provides a concrete acceptance catalog and prevents generic-scene reuse.
- Supports incremental delivery and per-workspace rollback.

### Negative

- Fifteen scene modules require significant design, domain, engineering, asset, and test investment.
- Cross-scene consistency reviews and domain-expert validation become ongoing work.
- Rich scenes may reveal weak or missing read models and management APIs.

### Neutral

- A workspace may use 2.5D rather than perspective 3D when that better supports accurate operator judgment.
- Visual distinctiveness does not authorize autonomous action or relax command controls.

## Links

- [ADR-002](ADR-002-bounded-context-modular-platform.md)
- [ADR-005](ADR-005-event-driven-fleet-control-plane.md)
- [ADR-009](ADR-009-simulation-gated-cyber-physical-delivery.md)
- [ADR-012](ADR-012-progressive-autonomy-and-safe-state-contract.md)
- [ADR-023](ADR-023-canonical-authenticated-command-envelope.md)
- [ADR-043](ADR-043-process-managers-compensation-and-human-escalation.md)
- [ADR-046](ADR-046-digital-twin-scenario-and-test-evidence.md)
- [ADR-053](ADR-053-million-asset-hierarchical-fleet-architecture.md)
- [ADR-062](ADR-062-vegetation-management-robot-operations.md)
- [ADR-063](ADR-063-solar-microgrid-robot-habitats.md)
- [ADR-068](ADR-068-robot-care-recovery-and-maintenance-network.md)
- [ADR-069](ADR-069-experimental-modular-aerial-fire-blanket-capability.md)
- [ADR-075](ADR-075-threejs-operator-workspace-rendering-platform.md)
- [ADR-077](ADR-077-threejs-interaction-performance-accessibility-and-verification.md)
- [ADR-078](ADR-078-task-oriented-operator-shell-and-design-system.md)
- [ADR-079](ADR-079-resilient-operator-read-model-and-offline-ui-state.md)
