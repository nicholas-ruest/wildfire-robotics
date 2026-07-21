# Sequential Implementation Prompts

This promptbook is the implementation order for the Wildfire Robotics platform. Execute prompts sequentially unless a prompt explicitly permits parallel work. A later prompt may rely only on artifacts whose earlier exit gates pass. Do not implement a divergent assumption: stop, document the conflict, and propose/supersede an ADR first.

The dedicated [aerial fire blanket promptbook](aerial-fire-blanket-implementation-prompts.md) supplies prompts `AFB-00`–`AFB-09`. Begin `AFB-00` after Main Prompt 04, observe each prompt's additional main-plan prerequisites, and complete `AFB-09` before Main Prompt 31.

## Global execution contract for every prompt

Prepend this contract whenever a prompt is handed to an implementation agent:

> Work in `/workspaces/wildfire-robotics`. Read every ADR, DDD specification, invariant, contract, and plan named by this prompt before editing. Inspect the existing worktree and preserve unrelated changes. Use Rust for domain models, application services, APIs, event processing, edge/station services, gateways, simulation, authorization, and tooling. TypeScript is limited to browser UI and generated clients. Isolate unavoidable Python/C/C++/CUDA/vendor code behind versioned Rust ports. Keep domain crates independent of frameworks, databases, networks, clocks, randomness, generated transports, and vendor types. Use private aggregate state, validated value objects, explicit errors, injected ports, optimistic versions, transactional outbox/inbox, and property tests. Link code/tests to ADR, requirement, and invariant IDs. Run formatting, Clippy with workspace policy, unit/property/contract/integration tests, security checks, and `git diff --check`. Do not claim production readiness from compilation. Finish with changed files, commands/results, unresolved risks, and evidence links.

## Prompt 00 — Reconcile baseline and lock the dependency graph

**Depends on:** Nothing.

**Governing ADRs:** ADR-002, ADR-014, ADR-016, ADR-017, ADR-038, ADR-041.

**DDD/plans:** Context catalog, context map, tactical-model standard, traceability model, master plan, production-readiness standard.

**Prompt:**

> Audit the repository against all 74 ADRs and 15 bounded contexts. Reconcile workspace members with the context catalog, adding missing Rust crate scaffolds for Vegetation Management, Robot Care and Recovery, and Aerial Deployment Operations without implementing behavior. Establish a machine-readable dependency/ownership manifest mapping context → crate → schemas → migrations → deployables → owners → ADRs/invariants. Add architecture tests that prohibit direct cross-context domain imports and infrastructure/vendor dependencies in domain modules. Add a documentation/link/ADR-number/invariant-ID validation command. Record all existing implementation drift without deleting user work.

**Exit gate:** All 15 contexts have Rust crate boundaries; ownership/dependency checks and documentation validation pass; no domain behavior is fabricated.

## Prompt 01 — Rust workspace, deterministic toolchain, and quality gates

**Depends on:** 00.

**Governing ADRs:** ADR-009, ADR-016, ADR-038, ADR-046.

**DDD/plans:** Tactical-model standard; traceability model; production-readiness Engineering section.

**Prompt:**

> Harden the Rust workspace and CI foundation. Pin Rust and Cargo tooling; centralize dependency/lint profiles; forbid unsafe code by default, unwrap/expect/panic in production paths, floating NaN/infinity at boundaries, and unreviewed licenses. Create deterministic test/build commands, nextest/property-test support, coverage, dependency/license/secret/SAST scans, SBOM and artifact-provenance stubs. Add test fixture/seed conventions and evidence metadata. Keep CI runnable locally and make failures actionable.

**Exit gate:** Clean checkout reproducibly formats, lints, builds, tests, scans, and emits an attributable evidence manifest.

## Prompt 02 — Shared kernel and canonical technical value objects

**Depends on:** 01.

**Governing ADRs:** ADR-006, ADR-016, ADR-023, ADR-027, ADR-028.

**DDD/plans:** Tactical-model standard value-object/error sections; ubiquitous language; traceability model.

**Prompt:**

> Implement the minimal `shared-kernel` Rust crate: opaque typed identifiers and scopes; aggregate versions/fencing tokens; UTC/monotonic deadline and clock-quality types; correlation/causation; content digests; classifications; semantic versions; quantities/units; confidence/probability/freshness; geospatial primitives with explicit CRS/altitude reference; evidence/artifact references; stable error categories. Ensure no business aggregate enters the shared kernel. Add serialization-independent property tests, numeric/time/geometry boundary tests, and compile-fail tests preventing identifier/scope confusion.

**Exit gate:** All shared types are immutable/validated and reusable without transport, database, or framework dependencies.

## Prompt 03 — Protobuf contract registry and compatibility enforcement

**Depends on:** 02.

**Governing ADRs:** ADR-005, ADR-021, ADR-022, ADR-023, ADR-044.

**DDD/plans:** Integration-contract standard and published-language registry; tactical command/event contracts.

**Prompt:**

> Create the versioned Protobuf/Buf contract workspace. Implement canonical command, event, artifact-reference, scope, clock-quality, authority/ODD/evidence, error, pagination, and health envelopes plus every published event currently listed in the integration registry. Generate Rust bindings and TypeScript clients into isolated generated packages; domain crates must map through adapters rather than import transport types. Add Buf lint/breaking checks, reserved-field policy, golden binary/JSON fixtures, unknown-enum/additive-field tests, size limits, and producer/consumer conformance harnesses.

**Exit gate:** Schema lint and compatibility gates pass; every registry contract has owner, version, example, fixture, and authorized-consumer metadata.

## Prompt 04 — Persistence primitives, migrations, outbox/inbox, and object manifests

**Depends on:** 02–03.

**Governing ADRs:** ADR-006, ADR-018, ADR-019, ADR-024, ADR-028, ADR-041.

**DDD/plans:** Tactical persistence rules; integration delivery rules; traceability model.

**Prompt:**

> Implement reusable Rust infrastructure crates—not shared domain repositories—for PostgreSQL/PostGIS transactions, optimistic aggregate versions, context-owned migrations, transactional outbox, consumer inbox/deduplication, leases/advisory locks where justified, and immutable object manifests. Provide testcontainers-based failure tests for rollback, redelivery, duplicate business keys, relay restart, poison quarantine, migration lock/time bounds, tenant scope, orphan reconciliation, and content-digest mismatch. Demonstrate expand-migrate-contract with a fixture service.

**Exit gate:** Database state and outbox cannot diverge in tests; duplicate delivery creates one business effect; object metadata/digest and tenant scope are verified.

## Prompt 05 — Identity, PKI, cryptography, and authorization policy core

**Depends on:** 02–04.

**Governing ADRs:** ADR-001, ADR-010, ADR-023, ADR-034, ADR-035, ADR-037, ADR-048.

**DDD/plans:** Identity and Access DDD (`IA-INV-001`–`005`); integration trust/grant events; incident activation and capability-promotion workflows.

**Prompt:**

> Implement Identity and Access domain/application layers in Rust: Principal, DeviceIdentity, RoleGrant, Approval and their complete transition tables/invariants. Implement human/workload/device trust ports, short-lived credentials, hardware-attestation abstraction, signed offline revocation/policy bundles, purpose-bound approvals, separation of duties, and default-deny policy decisions. Build the canonical signed command-envelope verifier with freshness, scope, payload digest, expected version, replay, key rotation, clock uncertainty, and break-glass audit. Use replaceable PKI/KMS/policy adapters and deterministic test keys only in tests.

**Exit gate:** All IA invariants and adversarial/replay/expiry/offline/root-rotation tests pass; authentication never implies authorization.

## Prompt 06 — Audit, evidence graph, observability, configuration, and release records

**Depends on:** 03–05.

**Governing ADRs:** ADR-009, ADR-011, ADR-031, ADR-038, ADR-039, ADR-042, ADR-045.

**DDD/plans:** Safety Assurance DDD; traceability model; production-readiness gates.

**Prompt:**

> Implement Rust libraries/services for tamper-evident audit entries, evidence/traceability links, typed signed configuration, expiring feature flags, OpenTelemetry propagation, redaction/cardinality controls, SLI/SLO descriptors, and immutable release/configuration/model records. Ensure safety/security audit has an independent durable path and telemetry cannot block safety behavior. Provide APIs that answer the promotion query from the traceability model and fail when links are missing, stale, contradictory, or unapproved.

**Exit gate:** A fixture release can be traced source→artifact→configuration→requirements/invariants→tests→approval→deployment; tampering and unsafe config fail closed.

## Prompt 07 — NATS/JetStream messaging and cloud-edge synchronization substrate

**Depends on:** 03–06.

**Governing ADRs:** ADR-003, ADR-005, ADR-020, ADR-024, ADR-025, ADR-026, ADR-027.

**DDD/plans:** Integration delivery/subject rules; edge-deployment synchronization process; tactical error model.

**Prompt:**

> Implement the Rust messaging abstraction and NATS JetStream adapter with subject authorization, context-owned streams, explicit acknowledgements, bounded retries, dead-letter/quarantine, replay, request/reply deadlines, tiered telemetry, drop accounting, station leaf-node/store-forward operation, and correlation propagation. Implement generic reconciliation scaffolding for aggregate versions, cursors, monotonic restrictions/revocations/grounding and suspended ambiguity; context-specific merge policies remain in owning contexts. Benchmark a representative partition/reconnect workload.

**Exit gate:** Partition, duplication, reordering, delay, broker restart, reconnect burst, poison message, and stricter-authority conflict tests pass without lost safety facts or duplicate effects.

## Prompt 08 — Safety Assurance domain and promotion engine

**Depends on:** 02–07.

**Governing ADRs:** ADR-001, ADR-009, ADR-012, ADR-031, ADR-045, ADR-046, ADR-052.

**DDD/plans:** Safety Assurance DDD (`SA-INV-001`–`005`); release-promotion process; traceability model.

**Prompt:**

> Implement Safety Assurance aggregates Hazard, SafetyConstraint, ODD, EvidenceCase and SafetyOccurrence in Rust. Enforce immutable signed constraints, exact release/configuration/hardware/capability/ODD promotion scope, independent review, residual-risk authority, occurrence-driven suspension, and near-miss blocking. Implement the promotion process manager, constraint bundle publication, evidence completeness queries, expiry/review triggers, and rollback/narrow-ODD outcomes. Do not invent legal standards; expose replaceable compliance-matrix ports.

**Exit gate:** Every SA transition/invariant has property tests; incomplete evidence, near miss, invalid assumption, changed configuration, and expired approval block promotion.

## Prompt 09 — Incident Command authority domain

**Depends on:** 05, 07–08.

**Governing ADRs:** ADR-001, ADR-002, ADR-010, ADR-023, ADR-035, ADR-043.

**DDD/plans:** Incident Command DDD (`IC-INV-001`–`005`); incident-activation process; AssignmentIssued/RestrictionChanged contracts.

**Prompt:**

> Implement Incident, OperationalPeriod, Objective, Assignment and Restriction aggregates, repositories, commands/events and read models in Rust. Implement chain-of-command transfer, qualifications/approvals, spatial/temporal authority, restriction precedence, acknowledgement gaps, and the incident-activation process manager. An Assignment must remain a bounded objective authorization and never contain actuator instructions. Test concurrent authority transfer, immediate narrowing, expiry, ambiguous authority and offline restriction distribution.

**Exit gate:** No assignment can exceed current incident/period authority; restriction and revocation propagate safely under partition/replay tests.

## Prompt 10 — Fleet identity, configuration, capability, health, battery, and cells

**Depends on:** 04–09.

**Governing ADRs:** ADR-012, ADR-047, ADR-053, ADR-064, ADR-068.

**DDD/plans:** Fleet Operations DDD (`FO-INV-001`–`008`); vehicle-enrollment/capability process; million-robot scale model.

**Prompt:**

> Implement Fleet Operations aggregates Vehicle, BatteryAsset, CapabilityRecord, HealthAssessment, Configuration and FleetCell in Rust; defer learned CollaborationProfile behavior to Prompt 25. Implement attestation/evidence/maintenance/calibration/trust eligibility, grounding/clearance, energy eligibility with uncertainty, signed compatibility matrices, partition placement, epochs/fencing and bounded capacity summaries. Ensure no fleet-wide scan/lock/scheduler is needed. Add synthetic million-identity data generators and cell split/merge/hot-partition tests, initially at CI-safe scale with separately runnable full benchmarks.

**Exit gate:** Stale epochs cannot reserve/command; grounded/incompatible/stale/unmaintained assets cannot allocate; partition operations preserve one authoritative membership epoch.

## Prompt 11 — Station edge core and safe reconciliation

**Depends on:** 07–10.

**Governing ADRs:** ADR-003, ADR-013, ADR-025, ADR-033, ADR-040, ADR-042.

**DDD/plans:** Station Operations DDD (`SO-INV-001`–`006` initially); edge synchronization process.

**Prompt:**

> Implement Station and EdgeDeployment aggregates plus the signed edge runtime supervisor in Rust. Support offline policy/identity/map/mission/audit caches, declarative deployment verification, compatibility gates, resource-based load shedding, durable local event/audit buffers, reconciliation cursors, conflict quarantine, rollback, and recovery checkpoints. Keep command/safety/identity/audit ahead of optional ML/indexing loads. Use a local lightweight deployment adapter but keep domain/application portable.

**Exit gate:** Cloud loss, partial sync, corrupt log, disk pressure, clock uncertainty, failed upgrade and restart tests never expand expired authority or lose required audit.

## Prompt 12 — Vehicle gateway, telemetry normalization, and simulator adapter

**Depends on:** 05, 07–11.

**Governing ADRs:** ADR-003, ADR-004, ADR-008, ADR-012, ADR-023, ADR-026, ADR-027, ADR-047.

**DDD/plans:** Vehicle Integration DDD (`VI-INV-001`–`005`); IntentAcknowledged/TelemetryNormalized contracts.

**Prompt:**

> Implement GatewaySession, CommandDelivery and TelemetryStream in Rust with a capability-based Flight/Drive/Tool controller port and deterministic simulator adapter first. Distinguish transport ack, acceptance, execution and physical outcome. Enforce signature/scope/expiry/fencing/local constraints, protocol idempotency guard, bounded retry, telemetry tiering, clock quality and minimum-risk behavior. Add curated ROS 2/DDS and MAVLink/MAVSDK facades as feature-gated adapters without vendor types escaping upstream; use simulation until later HITL prompts.

**Exit gate:** Duplicate/reordered commands cannot duplicate physical effect in the simulator; link loss, adapter crash, stale fence, clock fault and unknown outcome become explicit safe states.

## Prompt 13 — Mission planning, leases, allocation, and deconfliction

**Depends on:** 08–12.

**Governing ADRs:** ADR-001, ADR-005, ADR-012, ADR-023, ADR-043, ADR-053, ADR-055.

**DDD/plans:** Mission Control DDD (`MC-INV-001`–`007`); mission authorization/dispatch process.

**Prompt:**

> Implement Mission, Allocation, MissionLease and ConflictSet in Rust with hierarchical planning ports, resource reservations, fencing leases, snapshot/version validation, spatial-temporal deconfliction, policy/constraint checks, command dispatch and abort/minimum-risk compensation. Begin with deterministic reference planners and simulator gateway. Global planning consumes bounded cell summaries only. Treat relay connectivity as a constrained resource but defer ruv-drone execution.

**Exit gate:** Concurrency/property/fault tests prove no double allocation, stale lease advancement, dispatch with unresolved conflict, or continuation after abort/restriction/grounding/authority expiry.

## Prompt 14 — Logistics inventory, custody, delivery, water, and supply planning

**Depends on:** 09–13.

**Governing ADRs:** ADR-006, ADR-043, ADR-060.

**DDD/plans:** Logistics DDD `LO-INV-001`–`008`; logistics-delivery and supply-planning processes.

**Prompt:**

> Implement LogisticsMission, Route, Delivery, WaterSource, RelayCycle, ResourceItem and SupplyPlan in Rust. Enforce quantity/unit, serial/batch, condition/expiry, compatibility, reservation, custody, water quality/freshness, lead-time uncertainty, charging/fuel/maintenance dependencies and semantic compensation. Provide deterministic baseline demand/stock/routing optimizers behind replaceable ports; retain human-readable assumptions, bottlenecks and alternatives.

**Exit gate:** Concurrent reservation/custody, contamination, substitution, shortage, route failure and replay tests cannot double-allocate or lose source-to-use lineage.

## Prompt 15 — Robot habitats, microgrids, charging, and million-battery control

**Depends on:** 10–14.

**Governing ADRs:** ADR-040, ADR-053, ADR-063, ADR-064.

**DDD/plans:** Station DDD `SO-INV-007`–`010`; habitat-energy/charging process; physical-scale viability model.

**Prompt:**

> Implement RobotHabitat, Microgrid, EnergyStore, ChargeSession and MaintenanceBay in Rust with simulator-first ports for PV/weather, grid, storage, generator/fuel, feeder/charger, BMS, thermal/fire zones and docks. Implement local partitioned charging optimization by readiness/deadline/reserve/degradation/site constraints; BMS/protection always overrides. Model SOC/SOH/power uncertainty, emergency energy, deterministic load shedding, island/black start and quarantine. Do not select real hardware yet; define conformance interfaces and scenario fixtures.

**Exit gate:** Cold/low-solar/outage/fuel-delay/charge-surge/thermal fault/stale BMS simulations protect critical loads and never start an incompatible or unsafe charge.

## Prompt 16 — Pods, carriers, and 100,000-robot mobilization

**Depends on:** 12–15.

**Governing ADRs:** ADR-053, ADR-060, ADR-065, ADR-066, ADR-067.

**DDD/plans:** Logistics DDD `LO-INV-009`–`012`; mass-mobilization process; physical-scale viability model.

**Prompt:**

> Implement TransportPod, Carrier and MobilizationWave in Rust plus a time-expanded capacitated-flow planner. Model manifests, mass/volume/axle/centre-of-gravity, securement, energy isolation, route/bridge/ferry/rail/barge capacity, charging/refueling, staging/load/unload/admission slots and useful-arrival SLA. Build hybrid-electric carrier and bounded-platoon simulator interfaces with local safe stop and manual recovery. Generate parameterized 100,000-robot scenarios; never model one monolithic carrier.

**Exit gate:** The planner cannot double-reserve any asset/slot or release beyond downstream capacity; closure, carrier failure, V2X loss, energy shortage and destination saturation replan safely.

## Prompt 17 — Robot care, medic pods, hospitals, and continuous maintenance

**Depends on:** 10, 12, 14–16.

**Governing ADRs:** ADR-047, ADR-060, ADR-064, ADR-068.

**DDD/plans:** Robot Care DDD (`RC-INV-001`–`008`); maintenance/recovery process.

**Prompt:**

> Implement ServicePolicy, MaintenancePlan, WorkOrder, RecoveryMission, DamageAssessment, QuarantineCase, RepairCase and RetirementCase in Rust. Implement simulator adapters for maintenance robots and medic pods, procedure/tool/module compatibility, energy/tool stabilization, lift/tow/cradle checks, contamination and fire-separated quarantine, hospital capacity, serialized parts, calibration/burn-in, recertification, identity/data retirement and salvage custody. Integrate Fleet eligibility, Station zones and Logistics custody without crossing ownership.

**Exit gate:** Unknown/heat-damaged/contaminated assets cannot enter ordinary transport/charging or return to service; human rescue priority, medic failure and hospital saturation fail safely.

## Prompt 18 — Hazard Intelligence ingestion and common hazard picture

**Depends on:** 03–08, 11.

**Governing ADRs:** ADR-006, ADR-007, ADR-018, ADR-019, ADR-029, ADR-030.

**DDD/plans:** Hazard Intelligence DDD (`HI-INV-001`–`006`); hazard-ingestion workflow.

**Prompt:**

> Implement Source, IngestionRun, ObservationSet, HazardPicture and VisualEvidenceSet in Rust. Start with canonical fixture/file adapters, then add independently feature-gated authoritative provider adapters only when credentials/terms are available. Validate units, CRS, geometry, event/ingest time, uncertainty, quality, checksum, licensing and lineage; quarantine invalid data; append corrections/supersession; build immutable snapshot manifests and freshness/gap projections. Raw media remains object evidence, never an embedding.

**Exit gate:** Duplicate/corrected/late/unlicensed/invalid/provider-outage/replay tests preserve source claims and prevent quarantined data from operational pictures.

## Prompt 19 — Predictive Planning, model registry, sandboxed execution, and digital twin core

**Depends on:** 06, 08, 18.

**Governing ADRs:** ADR-009, ADR-031, ADR-032, ADR-045, ADR-046.

**DDD/plans:** Predictive Planning DDD `PP-INV-001`–`005`; hazard forecast process; traceability model.

**Prompt:**

> Implement ModelRelease, ForecastRun, SpreadScenario, Recommendation and EvaluationStudy in Rust. Create immutable run manifests, seed/input/runtime digests, ODD, calibration, limitations, expiry and artifact lineage. Implement a deterministic Rust reference model and OCI runner port for foreign scientific models with resource/network/time/output controls. Build scenario registry, reproducible seeds, expected invariants/tolerances, simulator validity metadata and shadow/canary/rollback transitions.

**Exit gate:** Unpromoted/out-of-ODD/nonreproducible/invalid outputs cannot publish; rerunning a reference manifest reproduces results within declared tolerance.

## Prompt 20 — Authoritative lightning baseline and calibrated ML enhancement

**Depends on:** 18–19.

**Governing ADRs:** ADR-007, ADR-031, ADR-056, ADR-059.

**DDD/plans:** Predictive DDD `PP-INV-006`–`007`; Hazard DDD; lightning-learning process.

**Prompt:**

> Implement authoritative lightning/weather/fuels/terrain/hotspot/ignition snapshot assembly and a transparent operational baseline before ML. Add Rust data/evaluation pipelines for holdover ignition probability, location/time uncertainty, reconnaissance value and abstention; isolate any training framework behind immutable model artifacts and Rust inference/evaluation ports. Enforce incident/geography/time/fire-year leakage boundaries, rare-event discrimination, calibration, shadow evaluation, drift and lineage. Record unobserved/censored areas, sampling policy, interventions and negative outcomes.

**Exit gate:** Baseline is reproducible; ML cannot promote without held-out and prospective evidence and never fabricates a strike, ignition or authority.

## Prompt 21 — RuPixel visual retrieval and prediction/outcome dataset loop

**Depends on:** 18–20.

**Governing ADRs:** ADR-014, ADR-019, ADR-057, ADR-059.

**DDD/plans:** Hazard VisualEvidenceSet/`HI-INV-006`; Predictive `PP-INV-007`; lightning-learning process.

**Prompt:**

> Evaluate and pin RuPixel, then implement it behind a Rust visual-index port with a portable fallback. Ingest calibrated media manifests, keyframe gates, embeddings/index versions and ANN results as rebuildable projections. Implement separate georegistration, verification/label, prediction-outcome alignment, dataset snapshot and bias/censoring workflows. Benchmark retrieval quality/latency/index recovery/domain shift and prevent similarity from becoming an observation without verification.

**Exit gate:** Index rebuild from immutable media is deterministic enough for declared versions; false similarity, corrupt index and unavailable RuPixel degrade to safe search without changing truth data.

## Prompt 22 — ruv-drone bounded UAV coordination and airborne relay

**Depends on:** 12–13, 18–21.

**Governing ADRs:** ADR-008, ADR-014, ADR-053, ADR-054, ADR-055.

**DDD/plans:** Vehicle Integration, Mission Control `MC-INV-007`, FleetCell; mission and lightning-reconnaissance processes.

**Prompt:**

> Evaluate/pin ruv-drone and implement a replaceable Vehicle Integration adapter for bounded cohort topology, gossip/consensus, formation, task allocation, coverage, collision avoidance, relay and fail-safe outcomes. Start in simulation with PX4/ArduPilot controller facades. Keep cohort sizes benchmark-bounded and hierarchical; expose platform capabilities/tasks/outcomes, never ruv-drone types. Disable MAPPO until separately promoted. Implement link-quality maps, service classes, energy/return reserve, handoff and total-network-loss behavior.

**Exit gate:** Cohort/relay simulations pass collision, stale authority, leader/consensus/link loss, geofence/airspace, return-energy and fallback tests; no drone creates authority or targets suppression autonomously.

## Prompt 23 — Vegetation Management domain and simulated robot workflow

**Depends on:** 09–22, especially 14–18.

**Governing ADRs:** ADR-012, ADR-059, ADR-062, ADR-068.

**DDD/plans:** Vegetation Management DDD (`VM-INV-001`–`006`); vegetation-treatment process.

**Prompt:**

> Implement TreatmentProgram, Prescription, TreatmentUnit, WorkPackage and EffectivenessAssessment in Rust. Enforce land/authority, survey/fuel state, geometry/version, ecological/cultural/utility/wildlife exclusions, tool/method, residual-fuel target, ODD, fire danger, biomass logistics, planned-vs-actual evidence and longitudinal assessment. Integrate simulated ground robots, drone survey/relay, Mission authority, Logistics biomass and Robot Care without permitting learned widening or autonomous re-arming.

**Exit gate:** Exclusion, person/wildlife/utility discovery, fire-danger, localization/tool/communications faults inhibit tools and preserve evidence; completion never implies effectiveness.

## Prompt 24 — Suppression Operations teleoperation and actuation envelope

**Depends on:** 08–22 and simulation/evidence infrastructure.

**Governing ADRs:** ADR-001, ADR-009, ADR-012, ADR-023, ADR-045.

**DDD/plans:** Suppression Operations DDD (`SU-INV-001`–`005`); suppression-operation process.

**Prompt:**

> Implement SuppressionPlan, ActuationEnvelope, Target and Operation in Rust for simulation and human-approved teleoperation only. Require current authority, promoted capability, target/agent approval, two distinct qualified approvers, independent emergency stop and continuously monitored spatial/environmental/dose/tool limits. Model commanded versus measured application and irreversible physical effects. Build independent inhibit interface and occurrence/evidence flow. Do not implement unsupervised target selection, re-arm or envelope widening.

**Exit gate:** Every intrusion, uncertainty, breach, lost supervision, sensor/actuator fault or stop request independently inhibits simulated energy/flow and records an occurrence.

## Prompt 25 — RVM collaboration profiles and governed adaptive cohort advice

**Depends on:** 10, 13, 17, 19, 22–24.

**Governing ADRs:** ADR-014, ADR-035, ADR-053, ADR-058, ADR-059.

**DDD/plans:** Fleet CollaborationProfile/`FO-INV-007`; fleet-cell/adaptive-collaboration process.

**Prompt:**

> Evaluate/pin RVM and implement it as an optional bounded station/cohort adapter for capability-gated partitions, witness records, communication edges, coherence scores, split/merge and checkpoints. Implement CollaborationProfile from signed context-labelled cooperation, proximity, communication, complementarity, handoff and safety outcomes with time decay, poisoning defenses and uncertainty. Provide a deterministic conventional graph/allocator fallback. RVM/coherence may recommend cohorting/locality only; Mission Control performs authoritative allocation.

**Exit gate:** RVM outage, corrupt/poisoned graph, stale profile, checkpoint failure and rollback preserve conventional operation; relationship scores never grant trust, capability, authority or command.

## Prompt 26 — Commercial tenancy, metering, support, and ROI analytics

**Depends on:** 04–07 and operational fact producers 10–25.

**Governing ADRs:** ADR-015, ADR-036, ADR-049, ADR-050, ADR-051, ADR-061.

**DDD/plans:** Commercial Operations DDD (`CO-INV-001`–`006`); onboarding/offboarding and usage-rating processes; Predictive `PP-INV-008`.

**Prompt:**

> Implement Tenant, Contract, Entitlement, Meter, SupportCase and InvestmentCase in Rust. Enforce tenant/region isolation in data/events/jobs/cache/observability, effective-dated terms/prices, immutable deduplicated usage ledger, adjustments, tenant-safe support and offboarding evidence. Implement InvestmentScenario/Monte Carlo in Predictive Planning using actual fleet/energy/logistics/maintenance/downtime/outcome facts and a versioned human-only counterfactual; expose NPV/IRR/payback/TCO/ranges/sensitivity without unsupported causal claims. Commercial failure must never interrupt active authorized safety work.

**Exit gate:** Cross-tenant, replay/re-rating, contract amendment, billing outage, offboarding/legal-hold and ROI reproducibility/uncertainty tests pass.

## Prompt 27 — External APIs and Rust-first operator platform

**Depends on:** 03, 05–09 and context APIs implemented above.

**Governing ADRs:** ADR-016, ADR-022, ADR-028, ADR-036, ADR-044, ADR-049.

**DDD/plans:** Context map; integration registry; production-readiness Product/domain section.

**Prompt:**

> Implement a Rust API gateway/BFF exposing versioned REST/JSON and OGC interfaces with authentication context, authorization, schema/size/deadline/rate/concurrency limits, idempotency, audit, bulk/query/command traffic separation and freshness. Generate TypeScript clients. Build only the browser operator shell needed to exercise incident, mission, fleet, station, logistics, hazard, safety and recovery read models; use TypeScript solely for the browser/design system. Display provenance, uncertainty, stale/gap/degraded/unknown states and command outcome distinctions; accessibility tests are required.

**Exit gate:** Contract, abuse/load, cross-tenant, accessibility and stale-data tests pass; no public access reaches internal gRPC or bypasses owning-context authorization.

## Prompt 28 — Kubernetes/GitOps, backup, disaster recovery, and security operations

**Depends on:** 01–27 deployables.

**Governing ADRs:** ADR-013, ADR-033, ADR-037, ADR-038, ADR-040, ADR-041, ADR-048.

**DDD/plans:** Edge synchronization; release promotion; production-readiness Engineering/Security/Reliability sections.

**Prompt:**

> Implement declarative environment-isolated infrastructure, OCI packaging/signing, pull GitOps, network/identity/key policies, PostgreSQL/PostGIS, NATS, object storage, observability and station profiles. Keep provider specifics behind modules. Add immutable backup/PITR, restore ordering, Canadian-region placement configuration, secrets/KMS/HSM adapters, vulnerability intake, privileged just-in-time access and incident playbooks. Exercise cloud-region, station, key, broker, database and object-store recovery without resurrecting authority or replaying commands blindly.

**Exit gate:** Reproducible environment creation, signed deployment, drift detection, rollback/roll-forward and measured restore/failover meet declared RPO/RTO in test environments.

## Prompt 29 — Full digital twin, fault campaign, HITL interfaces, and promotion evidence

**Depends on:** 08–28.

**Governing ADRs:** ADR-009, ADR-045, ADR-046, ADR-047, ADR-054, ADR-063–ADR-074.

**DDD/plans:** All cyber-physical contexts/processes; traceability model; production-readiness cyber-physical and million-asset gates.

**Prompt:**

> Integrate the deterministic multi-domain digital twin: fire/weather/terrain, communications, drones, ground robots/tools, habitats/microgrids/chargers, batteries, pods/carriers/platoons, logistics, medic pods, hospitals, experimental aerial fire-blanket payloads and failures. Model the blanket's retained/extracted/released/formation/expansion/terrain/anchor states, bounded cohorts, panel isolation, tether breakaway, exclusion zones, dispersion envelopes and recovery accounting. Create requirement/hazard/invariant-linked scenarios for network/GNSS/clock loss, thermal events, authority expiry, collisions, intrusions, tool faults, carrier/load/release failures, fire/smoke/cold, coupled aerodynamic instability, entanglement, correlated damage and recovery. Add standardized SIL/HITL hardware ports and evidence capture but do not claim aircraft, field or operational authorization. Quantify simulator validity and gaps.

**Exit gate:** Required scenario matrix is deterministic/replayable, failures reach correct minimum-risk/compensation states, and signed evidence bundles answer the promotion query.

## Prompt 30 — Million-asset scale, endurance, chaos, and cost qualification

**Depends on:** 07, 10–17, 22, 25, 28–29.

**Governing ADRs:** ADR-020, ADR-026, ADR-039, ADR-053, ADR-055, ADR-064, ADR-067.

**DDD/plans:** Fleet/Station/Mission/Logistics invariants; physical-scale model; million-asset production gates.

**Prompt:**

> Build Rust load generators and a reproducible qualification harness for 1,000,000 registered/concurrently connected assets, 1 Hz normalized summaries, 10× regional reconnect burst, declared active mixes, hot-region skew, cell split/merge, relay loss, regional isolation, rolling upgrade and recovery. Exercise charging schedules, supply/mobilization, 100,000-robot useful-arrival waves and correlated robot-hospital demand. Measure p50/p95/p99 latency, throughput, lag/loss, resource saturation, availability, recovery and cost per ready/deployed robot. Prove absence of global scans/locks/consensus/synchronous scheduling.

**Exit gate:** Approved workload objectives pass with documented headroom and bottlenecks; failures remain bounded by cell/region; results—not extrapolation—support scale claims.

## Prompt 31 — Integrated release candidate and evidence-gated handoff

**Depends on:** Every prior prompt.

**Governing ADRs:** All ADR-001–ADR-074, especially ADR-009, ADR-011, ADR-038, ADR-045, ADR-048, ADR-069–ADR-074.

**DDD/plans:** All 15 contexts, 107 invariants, integration contracts, process managers, traceability model, master plan, production-readiness standard.

**Prompt:**

> Assemble an integrated release candidate without weakening any gate. Regenerate the ADR/DDD/code/contract/test/evidence traceability matrix; run formatting, strict Clippy, all unit/property/contract/integration/scenario/security/privacy/migration/replay/recovery/performance/chaos/endurance tests; produce SBOM, provenance, signed artifacts, deployment/configuration manifests, SLO/capacity reports and unresolved-risk register. Verify every invariant has enforcing code and tests, every contract has producer/consumer compatibility, every process manager has timeout/compensation/escalation, and every failure mode has observable containment/rollback. Do not label field or production ready unless the applicable non-software evidence gates are actually satisfied.

**Exit gate:** Zero known release-blocking defects, no broken traceability, no unapproved architecture deviation, all applicable technical gates pass, and remaining field/HITL/authority work is explicitly separated from completed software evidence.

## Dependency summary

```text
00–04 foundation/contracts/persistence
   ├── 05–08 identity/evidence/messaging/safety
   │      └── 09 incident authority
   │          └── 10–13 fleet/station/vehicle/mission
   │              ├── 14–17 supply/energy/mobility/robot care
   │              └── 18–22 hazard/models/lightning/vision/drones
   │                    └── 23–25 vegetation/suppression/adaptive collaboration
   └── operational fact producers ──> 26 commercial/ROI
all context APIs ──> 27 operator/API platform
all deployables ──> 28 infrastructure/recovery
all cyber-physical behavior ──> 29 simulation/evidence
scale-critical services ──> 30 million-asset qualification
AFB-00–09 aerial fire blanket sequence ──> 31 integrated release candidate
everything ──> 31 integrated release candidate
```
