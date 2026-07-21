# Full Implementation Program

- **Goal:** Deliver a commercially viable, Canada-first wildfire robotics platform that safely supports detection, planning, logistics, aerial mapping, and progressively supervised suppression.
- **Current state (2026-07-21):** Research and an initial roadmap exist; there is no application source, deployable infrastructure, verified hardware integration, safety case, or production evidence.
- **Terminal state:** Every in-scope bounded context is implemented, integrated, operated from reproducible infrastructure, independently verified, and promoted through the applicable safety and field gates.
- **Planning model:** GOAP with dependency gates. Lowest-risk information capabilities precede cyber-physical control. Replanning is mandatory when a precondition, regulation, supplier, validation result, or cost assumption changes.

## Non-negotiable definition of done

“Production ready” is not inferred from compilation or test count. The platform is done only when all applicable evidence exists:

- Functional behavior is traced from domain requirement to automated acceptance test.
- Unit, contract, integration, simulation, hardware-in-the-loop, fault-injection, endurance, recovery, and field tests pass at their declared gates.
- Safety hazards have owners, mitigations, residual-risk acceptance, and an auditable safety case. Emergency stop and safe-state behavior are independently verified.
- Threat models, SBOMs, signed artifacts, dependency policies, secrets management, least privilege, audit trails, penetration testing, and incident response are operational.
- SLOs, capacity limits, telemetry, alerting, runbooks, backup/restore, disaster recovery, and rollback are exercised rather than merely documented.
- Data licensing, privacy, retention, sovereignty, accessibility, procurement, export, environmental, aviation, radio, and incident-command obligations have documented approvals.
- Commercial operations have support ownership, maintenance plans, warranties/vendor obligations, training, deployment playbooks, cost model, and measured unit economics.
- No unresolved severity-1 or severity-2 defect exists; lower-severity accepted risks have owners and expiry dates.
- A release has reproducible provenance and the evidence bundle is approved by engineering, security, safety, operations, product, and the relevant operational authority.

Claims of “bug free” are not technically provable for a non-trivial distributed cyber-physical system. The enforceable substitute is zero known release-blocking defects plus systematic verification, monitored residual risk, rapid containment, and rollback.

## State model and optimal sequence

| Step | Action | Preconditions | Effect / exit evidence | Relative cost |
|---|---|---|---|---|
| 0 | Establish architecture and governance | Research baseline | Accepted decision set, context map, traceability model | Medium |
| 1 | Build platform foundation | Step 0 | Reproducible dev/test environments, identity, eventing, geospatial core, CI/CD, observability | High |
| 2 | Integrate hazard intelligence | Step 1; data terms approved | Live normalized feeds, provenance, quality monitoring, replayable history | High |
| 3 | Deliver decision support and digital twin | Steps 1–2 | Nowcasting/spread/ROI services validated against baselines; simulation promotion gate enforced | High |
| 4 | Deliver fleet command in simulation | Steps 1–3 | Policy-constrained missions, leases, deconfliction, safe-state behavior, audit replay | Very high |
| 5 | Prove single UAS operation | Step 4; aviation authorization | SITL/HITL and controlled field evidence; operator training and abort controls | Very high |
| 6 | Prove coordinated UAS operations | Step 5; multi-aircraft authorization | Conflict-free mapping with human airspace authority | Very high |
| 7 | Prove station and fixed-route carrier logistics | Step 4; approved hardware/site | Offline-capable station, degraded-GNSS route, recovery and maintenance evidence | Extreme |
| 8 | Prove remote-operated suppression | Step 7; live-fire facility and safety approval | Environmental qualification and controlled-burn evidence under human control | Extreme |
| 9 | Increment supervised autonomy | Step 8; approved safety case | Each autonomy increment separately bounded, simulated, tested, and authorized | Research |
| 10 | Commercial regional launch and scale | Relevant prior gates; positive economics | Supported regional service with measured SLOs, safety KPIs, and BCR | Extreme |

## Workstreams

Each workstream owns code, tests, infrastructure, documentation, and operational evidence—not just APIs.

1. Platform engineering: monorepo/toolchain, environments, identity, policy, event backbone, geospatial storage, observability, supply-chain security.
2. Hazard intelligence: lightning, hotspot, weather/FWI, fuel, terrain, and provenance ingestion; quality and licensing controls.
3. Prediction and planning: ignition nowcasting, tactical/strategic spread adapters, confidence/calibration, ROI and station siting.
4. Mission and fleet: command authorization, allocation, leases, geofencing, deconfliction, telemetry, digital twin, and immutable audit.
5. UAS: PX4-first integration, MAVLink/MAVSDK, payloads, mapping, regulated airspace workflows, and human abort.
6. Stations and carriers: edge operation, intermittent synchronization, energy/maintenance, fixed-route logistics, non-GNSS localization, water relay.
7. Suppression R&D: perception, thermal/environmental qualification, teleoperation, actuation envelopes, and supervised-autonomy safety cases.
8. Product and operations: incident-command workflows, accessibility, training, support, billing/economics, procurement, release and incident management.
9. Cooperative aerial systems: ruv-drone integration, bounded UAV cohorts, mapping/reconnaissance, airborne relay, coverage/deconfliction and validated learning policies.
10. Adaptive fleet collaboration: hierarchical cells, optional RVM partitions/coherence, relationship evidence, poisoning controls, conventional fallback and million-asset benchmarks.
11. Visual learning loop: calibrated media ingestion, RuPixel indexing, lightning prediction/outcome alignment, dataset governance, retraining and champion/challenger promotion.
12. Vegetation management: survey, treatment prescriptions, cutting/removal robots, biomass logistics and longitudinal effectiveness measurement.
13. Robot habitats and energy: site-specific solar microgrids, stationary storage, resilient generation, docks, million-battery identity/charging, thermal/fire safety and readiness forecasting.
14. Mass mobility: standardized habitat/transport pods, hybrid-electric autonomous carriers, bounded platoons, intermodal transfers and 100,000-robot useful-arrival mobilization exercises.
15. Robot care and recovery: autonomous maintenance robots at habitats, small medic/recovery pods, quarantine and decontamination, regional robot hospitals, repair/calibration/burn-in, recertification, salvage and retirement.
16. Aerial fire blanket R&D: material/panel/tether/parafoil/cradle qualification, aircraft-independent payload interface, dual-authority release, coupled deployment simulation, staged ground/low-drop/subscale/aircraft tests, ground installation, temporary containment and full component recovery/accounting.

## Quality gates

| Gate | Required evidence |
|---|---|
| Merge | Formatting, static analysis, unit/property tests, secret/SAST/dependency scan, ADR/interface checks |
| Integration | Consumer/provider contract tests, schema compatibility, migration rollback, deterministic replay |
| Simulation | Scenario coverage, safety invariants, chaos/fault injection, latency/capacity budgets, reproducible seeds |
| Hardware-in-loop | Real controller/sensors, communications loss, clock drift, brownout/restart, emergency stop |
| Controlled field | Approved test plan, trained staff, exclusion zones, weather envelope, incident/near-miss capture |
| Production | Signed evidence bundle, change approval, canary/rollback, staffed support, exercised recovery |

## Quantitative release objectives

Initial objectives must be refined with operators and regulators before acceptance:

- Command control-plane availability: 99.95% monthly; safety functions remain local during cloud loss.
- Safety-critical command authorization and audit persistence: p99 ≤250 ms inside the incident edge network.
- Fleet scale: manage at least 1,000,000 registered and concurrently connected asset identities using hierarchical cells, with no global consensus, scan, lock, or synchronous scheduler.
- Telemetry scale: sustain 1,000,000 assets at a 1 Hz normalized operational-summary baseline and a tested 10× regional reconnect burst; raw high-rate streams remain edge-local or use tiered artifact paths.
- Active coordination scale: validate declared mixes up to 1,000,000 active assets through multi-region/cell simulation, including hot-incident skew, cell split/merge, relay loss, regional isolation, rolling upgrade, and recovery. Per-cell consensus/cooperative groups remain bounded by benchmarked limits.
- Scale acceptance: publish p50/p95/p99 command, allocation, ingest, query and reconciliation latency; throughput, lag, loss, availability, recovery, infrastructure saturation and cost per asset under the approved million-asset workload model.
- Recovery: cloud RPO ≤5 minutes and RTO ≤30 minutes; station mission state recovers locally without cloud dependency.
- Every command is authenticated, authorized, idempotent, time-bounded, attributable, and replay-auditable.
- No single network, cloud, GNSS, model, or operator-console failure may cause uncontrolled motion or actuation.

## Replanning triggers

Replan immediately on failed validation, changed law/airspace rules, unavailable data rights, supplier end-of-life, security incident, unacceptable near miss, model drift, SLO breach, cost overrun above the approved threshold, or discovery that an adopted component cannot meet its safety envelope.

## Program risks and fallback

The primary path uses proven data/autopilot/middleware components and builds wildfire-specific orchestration. If an experimental dependency fails evaluation, replace it behind an adapter with a mature conventional component. If autonomous suppression cannot achieve an acceptable safety case, the commercially supported terminal product remains human-supervised/teleoperated suppression; autonomy is never promoted merely to satisfy a schedule.
