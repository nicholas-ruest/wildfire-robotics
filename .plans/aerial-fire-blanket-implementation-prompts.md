# Aerial Fire Blanket Sequential Implementation Prompts

This companion promptbook implements the experimental aerial fire blanket described by ADR-069–ADR-074 and the Aerial Deployment Operations DDD. Execute prompts `AFB-00` through `AFB-09` in order. They supplement—not replace—the global execution contract and platform prerequisites in [implementation-prompts.md](implementation-prompts.md).

No prompt authorizes an aircraft test, payload release, wildfire deployment, or production-readiness claim. Software and simulation evidence may only promote the exact configuration to the next approved test stage.

## Global contract for every AFB prompt

> Work in `/workspaces/wildfire-robotics`. First read ADR-069 through ADR-074, `docs/ddd/contexts/aerial-deployment-operations.md`, the linked cross-context DDD specifications, `docs/operations/production-readiness.md`, and every additional document named by this prompt. Preserve unrelated worktree changes. Implement domain and application behavior in Rust wherever possible. Keep aircraft, simulation, flight-control, material-laboratory, sensor and vendor integrations behind versioned Rust ports. Do not embed vendor types or speculative physical constants in the domain. Use private aggregate state, validated units, explicit uncertainty and freshness, injected clocks, typed errors, optimistic versions, transactional outbox/inbox, idempotent commands and deterministic tests. Trace each behavior and test to its ADR and `AD-INV-*` identifiers. Treat dimensions, masses, aerodynamic coefficients, thermal performance, dispersion and aircraft limits as configuration-controlled evidence, never universal constants. Run formatting, strict Clippy, unit/property/state-machine/contract/integration tests and `git diff --check`. Report changed files, evidence, remaining assumptions and blocked physical-validation gates.

## AFB-00 — Establish the bounded context and dependency firewall

**Depends on:** Main Prompts 00–04.

**Governing ADRs:** ADR-016, ADR-038, ADR-069–ADR-074.

**DDD:** Aerial Deployment Operations context; all `AD-INV-001`–`AD-INV-011`; context catalog and context map.

**Prompt:**

> Scaffold the Rust `aerial-deployment-operations` domain, application, ports and adapter boundaries. Add opaque identifiers and validated value objects for blanket configurations, panels, joints, vents, anchors, tethers, reels, parafoils, cradles, aircraft configurations, manifests, corridors, footprints, zones, deployment phases and evidence references. Define typed commands, events and errors without implementing physical algorithms. Add architecture tests prohibiting direct imports from aircraft vendors, ruv-drone, databases, message brokers, web frameworks, simulation engines and other bounded-context domain crates. Publish a context ownership/dependency manifest and schema namespaces.

**Exit gate:** The crate builds independently; every aggregate/value/event has an owner; forbidden dependencies fail architecture tests; no assumed aircraft or blanket constants enter domain code.

## AFB-01 — Configuration, material qualification and staged promotion

**Depends on:** AFB-00 and Main Prompts 05–08.

**Governing ADRs:** ADR-009, ADR-019, ADR-045, ADR-069, ADR-071.

**DDD:** BlanketConfiguration and MembraneAssembly; `AD-INV-001`, `AD-INV-006`, `AD-INV-011`.

**Prompt:**

> Implement BlanketConfiguration and MembraneAssembly aggregates in Rust. Bind exact revisions of material, panels, joints, vents, anchors, tethers, reels, parafoils, cradles, robots, geometry, mass properties, ODD, qualification stage and signed evidence. Encode the promotion ladder: concept → coupon/material → component → ground multi-panel → low-drop → subscale extraction → SIL → HITL → instrumented range → aircraft ground/extraction → partial-scale flight → full-system candidate. Make substitution, expired evidence, unexplained test variance, unresolved occurrence or changed ODD invalidate promotion. Track panel isolation, sacrificial release and serialized recovery states without claiming “fireproof” or effectiveness from deployment.

**Exit gate:** Property/state-machine tests prove exact-configuration promotion, monotonic evidence requirements, suspension and requalification; no stage can be skipped or inherited by a changed configuration.

## AFB-02 — Payload manifest and aircraft-independent integration

**Depends on:** AFB-01 and Main Prompts 03, 06, 11 and 17.

**Governing ADRs:** ADR-014, ADR-047, ADR-070, ADR-073.

**DDD:** PayloadManifest and aircraft/loadmaster integration port; `AD-INV-002`, `AD-INV-010`.

**Prompt:**

> Implement PayloadManifest, loading-plan validation and `AerialPayloadInterface` in Rust. Represent packed geometry, measured mass/uncertainty, centre of gravity, moments, floor/ramp/roller point and distributed loads, restraint/extraction interfaces, opening loads, electrical/data/environmental limits, hazardous contents, loading stations and component serials. Create a versioned aircraft-adapter contract whose approved envelope is supplied by authoritative engineering evidence for an exact aircraft/tail/configuration; do not seed brochure maximum payload as an allowable airdrop value. Support simulated CC-177/C-17-class and C-130J/LM-100J-class adapters only through test fixtures until independently approved. Reconcile planned, inspected, loaded, retained/released and recovered manifests.

**Exit gate:** Unit/dimensional, uncertainty, mass-balance, CG/moment, floor-load, interface-version, substitution, over-limit and reconciliation tests pass; missing or stale aircraft evidence prevents load approval.

## AFB-03 — Corridor, dispersion and two-key release authorization

**Depends on:** AFB-02 and Main Prompts 09, 13 and 18–20.

**Governing ADRs:** ADR-001, ADR-012, ADR-023, ADR-045, ADR-070, ADR-073.

**DDD:** AerialDropMission and ReleaseAuthorization; `AD-INV-003`, `AD-INV-004`.

**Prompt:**

> Implement AerialDropMission and ReleaseAuthorization in Rust. Bind the exact payload digest to the route, release corridor, nominal and failed-component dispersion footprints, three-dimensional exclusion volume, jettison sectors, emergency landing zones, ground boundary, point of no return, alternate/abort plan, ODD and expiring source observations. Require separate current aircraft-authority and incident/safety decisions over the identical command digest; either party can hold, veto or abort, and neither decision can be inferred from mission approval. Continuously re-evaluate aircraft/payload health, weather, wind, turbulence, smoke, icing, airspace, terrain, fire position, people/vehicles/aircraft, navigation/time, communications, surveillance confidence and ground readiness. After release, allow only pre-authorized bounded least-harm contingencies.

**Exit gate:** Race, replay, stale-data, mismatched-digest, veto, intrusion, communications-loss, surveillance-loss and point-of-no-return tests prove that automation cannot compel or broaden a release.

## AFB-04 — Coupled airborne deployment state machine

**Depends on:** AFB-03 and Main Prompts 12, 13, 22 and 25.

**Governing ADRs:** ADR-012, ADR-023, ADR-054, ADR-058, ADR-072, ADR-073.

**DDD:** AirborneDeployment; `AD-INV-005`, `AD-INV-006`, `AD-INV-009`.

**Prompt:**

> Implement the deterministic airborne deployment protocol in Rust: retained → extracted → stabilized → cohort release → parafoil established → formation acquired → section reefed release → tension-balanced expansion → terrain alignment → landing, with isolated/jettisoned terminal branches. Require measured margins for separation, tether routing/rate/tension, reefing/vent state, stability, wind, navigation/time, communications, clearance and remaining contingencies at every transition. Model small bounded cohorts with local controllers and hierarchical summaries; prohibit a global safe-flight dependency. Implement idempotent retain, pause, reef, vent, isolate, breakaway, emergency-land and safe-sector jettison commands. Put ruv-drone and RVM behind optional advisory adapters; their output cannot authorize a transition, assign trust or override local safety.

**Exit gate:** Model-based and property tests cover every legal/illegal transition and concurrent fault; one robot, panel, tether, parafoil, controller, network or learned-adapter failure cannot command an unsafe transition in another cohort.

## AFB-05 — Ground transition, anchoring and temporary protection

**Depends on:** AFB-04 and Main Prompts 10, 13, 18, 23 and 24.

**Governing ADRs:** ADR-012, ADR-045, ADR-062, ADR-068, ADR-074.

**DDD:** GroundInstallation; Vegetation Management and Suppression Operations boundaries; `AD-INV-007`, `AD-INV-008`, `AD-INV-009`.

**Prompt:**

> Implement GroundInstallation in Rust for landed → transitioning → anchoring → sealing → active → degraded → recovering → removed/temporarily-left. Require landed safe robot/tool state and current terrain rights, utilities, people/wildlife, slope, wind/uplift, fuel/fire and anchor compatibility before work in each bounded zone. Track panel contact, top/bottom temperature, ember exposure, tension, uplift, tears, vents, gaps and sensor freshness with explicit uncertainty. Implement bounded pause, vent, isolate, reposition, suppression-request and escalation policies. Aerial Deployment may coordinate installation but must request independently authorized suppressant application, vegetation work, mission expansion and robot capability from their owning contexts.

**Exit gate:** Terrain/utility intrusion, stale sensing, anchor pullout, uplift, tear, heat transfer, gap, robot/tool fault and lost-communications tests inhibit only affected zones and never fabricate containment effectiveness.

## AFB-06 — Custody, recovery, quarantine and disposition

**Depends on:** AFB-05 and Main Prompts 14, 17 and 26.

**Governing ADRs:** ADR-019, ADR-060, ADR-068, ADR-071, ADR-073, ADR-074.

**DDD:** PayloadManifest, MembraneAssembly and GroundInstallation; Logistics and Robot Care contracts; `AD-INV-006`, `AD-INV-010`.

**Prompt:**

> Implement end-to-end serialized accounting for every robot, panel, joint, parafoil, tether, reel, anchor, cradle section and chemical payload. Record custody, location/confidence, thermal/smoke/chemical exposure, damage, contamination, energized hazards, sacrificial releases, searches and recovery. Hand off quarantined components to Logistics and Robot Care through idempotent contracts for decontamination, inspection, repair, calibration, burn-in, reuse, recycling, retirement or explicitly approved temporary abandonment. Prevent mission closure while an item is silently missing; support formally accepted unlocated/sacrificed dispositions with authority, search evidence and continuing hazard notices.

**Exit gate:** Partial recovery, duplicate scans, conflicting custody, disconnected operations, contaminated equipment, unknown location and adapter outage tests converge without losing components or prematurely clearing hazards.

## AFB-07 — Operator APIs, geospatial views and immutable audit

**Depends on:** AFB-03–AFB-06 and Main Prompts 07 and 27.

**Governing ADRs:** ADR-016, ADR-019, ADR-022, ADR-028, ADR-036, ADR-044, ADR-073.

**DDD:** All Aerial Deployment read models and events; `AD-INV-003`–`AD-INV-010`.

**Prompt:**

> Expose versioned Rust APIs and operator read models for qualification matrices, assembly/manifests, load approval, corridor/exclusion/dispersion maps, release checklist and dual decisions, deployment phase/cohort stability/tether tension, panel health/footprint, degraded zones, unaccounted components and disposition. Show source time, freshness, uncertainty, configuration/evidence digest and authority on every safety-relevant view. Require explicit confirmation for irreversible actions, render aircraft and incident decisions separately, and distinguish requested/accepted/executed/observed/failed outcomes. Persist immutable audit and bulk artifacts through their owning platform services; generate browser clients rather than duplicating domain rules in TypeScript.

**Exit gate:** Contract, authorization, tenant/incident isolation, stale-view, accessibility, map-reference-system, command-race and audit-replay tests pass; the UI cannot bypass or synthesize authorization.

## AFB-08 — Coupled digital twin, fault campaign and HITL ports

**Depends on:** AFB-01–AFB-07 and Main Prompt 29 prerequisites.

**Governing ADRs:** ADR-009, ADR-045, ADR-046, ADR-069–ADR-074.

**DDD:** All `AD-INV-001`–`AD-INV-011`; aerial fire blanket qualification/deployment process manager.

**Prompt:**

> Build a deterministic Rust-led qualification harness coupling aircraft extraction inputs, payload dynamics, parafoils, robots, tethers/reels, flexible membrane/reefing/vents, wind/turbulence/smoke, terrain/obstacles, landing/anchors, heat/embers/fire spread, communications/navigation/time and component failures. Keep specialist aeroelastic, CFD, finite-element and fire models behind versioned ports with immutable input/output artifacts and declared validity domains. Add SIL and standardized HITL ports, seeded scenario generation, uncertainty sweeps, sensitivity analysis, model correlation and discrepancy tracking. Exercise off-nominal extraction, shock/oscillation, collision/entanglement, canopy/tether/panel/robot failures, control disagreement, delayed telemetry, unsafe dispersion, jettison, landing, anchor failure, uplift, thermal breach, contamination and incomplete recovery.

**Exit gate:** Required scenarios replay deterministically; every invariant has fault evidence; model validity and disagreement are visible; passing simulation authorizes only the next bounded physical test, never aircraft or wildfire operation.

## AFB-09 — Effectiveness evaluation and evidence-gated release candidate

**Depends on:** AFB-08, Main Prompts 26 and 28–30.

**Governing ADRs:** ADR-009, ADR-011, ADR-038, ADR-045, ADR-048, ADR-069–ADR-074.

**DDD:** BlanketConfiguration effectiveness evidence; integration contracts and process manager; `AD-INV-001`–`AD-INV-011`, especially `AD-INV-011`.

**Prompt:**

> Assemble an aerial-fire-blanket software release candidate and trace ADR → invariant → aggregate behavior → contract → test → evidence. Implement effectiveness studies that bind protected area/time, exposure conditions, panel state, interventions, baseline/counterfactual, uncertainty, limitations and negative outcomes; coverage area or apparent survival alone is not proof. Run all domain, contract, replay, security, recovery, simulation, fault, performance and endurance gates. Produce signed configuration manifests, SBOM/provenance, capacity/SLO report, unresolved-risk register and a promotion dossier that clearly separates completed software evidence from material, ground, low-drop, subscale, aircraft-integration, flight-test and controlled-fire evidence still required.

**Exit gate:** Zero known software release blockers and complete traceability for the exact candidate configuration; no unsupported “fireproof,” operational, aircraft-approved, commercially ready or production-ready claim; the next physical test stage and its independent approval criteria are explicit.

## Dependency summary

```text
Main 00–08 foundation/evidence/safety
          └── AFB-00 context boundary
                └── AFB-01 configuration/material promotion
                      └── AFB-02 payload/aircraft interface
                            └── AFB-03 mission/release authority
                                  └── AFB-04 airborne deployment
                                        └── AFB-05 ground installation
                                              └── AFB-06 recovery/disposition
AFB-03–06 + Main 27 ──> AFB-07 operator platform
AFB-01–07 + Main 29 prerequisites ──> AFB-08 digital twin/HITL
AFB-08 + Main 26, 28–30 ──> AFB-09 evidence-gated candidate
AFB-09 ──> Main 31 integrated release candidate
```
