# Integration Contract Standard and Registry

## Canonical event envelope

Every published event contains `message_id`, `event_type`, `schema_version`, `occurred_at`, `recorded_at`, `producer`, `producer_version`, `aggregate_type`, `aggregate_id`, `aggregate_version`, `tenant_id`, optional `region_id` and `incident_id`, `correlation_id`, `causation_id`, `trace_context`, `classification`, `subject`, `content_type`, payload digest, and payload. Safety-relevant events also carry authority, ODD, constraint, evidence, and clock-quality references. Optional scope is absent rather than fabricated.

Subjects follow `wr.<environment>.<region>.<tenant>.<context>.<aggregate>.<event>.v<major>`. Access is least-privilege by subject and environment. Payload schemas are Protobuf under ADR-021. Large artifacts are immutable object references with digest, size, media type, classification, license, and expiry—not embedded bytes.

## Delivery rules

- At-least-once delivery; consumers must deduplicate and remain semantically idempotent.
- Ordering is guaranteed only per aggregate stream; consumers use aggregate version to detect gaps.
- Producers never wait for consumers in their aggregate transaction.
- Consumers define retry limits, exponential jitter, dead-letter/quarantine, replay safety, and operator repair.
- Event deletion or retention never invalidates the authoritative aggregate/audit retention policy.
- Major incompatible versions use parallel subjects and a measured migration window.

## Published-language registry

| Producer | Contract | Required semantic content | Authorized consumers |
|---|---|---|---|
| Identity & Access | `DeviceTrustChanged` | device, trust state, reason, effective time, policy version | Fleet, Vehicle, Station, Safety |
| Identity & Access | `AuthorityGrantChanged` | principal, scopes, permissions, validity, approvers | command-owning contexts |
| Hazard Intelligence | `ObservationAccepted` | source, spacetime, measurement, unit, quality, provenance, license | Prediction, read models |
| Hazard Intelligence | `HazardPictureUpdated` | region, valid time, artifact digest, confidence, freshness | Prediction, Incident Command |
| Hazard Intelligence | `VisualEvidenceIndexed` | media/keyframe manifest, footprint/time, sensor quality, index/model versions | Prediction, Vegetation, authorized search |
| Hazard Intelligence | `LightningRiskAreaPublished` | input/baseline/model releases, area/time, ignition probability, uncertainty, priority, expiry | Prediction, Incident, Mission |
| Predictive Planning | `ForecastPublished` | model/input releases, run, horizon, uncertainty, artifact, expiry | Incident Command |
| Predictive Planning | `RecommendationPublished` | evidence, alternatives, confidence, limitations, expiry | Incident Command only as advice |
| Incident Command | `AssignmentIssued` | authority, objective, geography, validity, constraints, approvers | Mission Control, Safety |
| Incident Command | `RestrictionChanged` | restriction, scope, effective time, superseded version | all operational contexts |
| Mission Control | `MissionAuthorized` | assignment, plan digest, allocation, ODD/constraints, validity | Fleet, Vehicle, Logistics, Suppression |
| Mission Control | `MissionStateChanged` | prior/current state, reason, actor, time | Incident, Fleet, Safety, read models |
| Fleet Operations | `CapabilityAttested` | vehicle, capability, assurance, ODD, configuration, evidence, expiry | Mission Control |
| Fleet Operations | `VehicleGrounded` | vehicle, reason, effective time, clearance requirement | Mission, Vehicle, Station, Safety |
| Fleet Operations | `FleetCellChanged` | cell, epoch, scope, capacity summary, members artifact/digest, split/merge cause | Mission, Station |
| Fleet Operations | `CollaborationProfilePromoted` | cohort/context, relationship model, features/evidence manifest, validity | Mission as advice only |
| Fleet Operations | `BatteryEligibilityChanged` | battery, compatible vehicles, energy/power/health summaries with uncertainty, quarantine/expiry | Station, Mission, Logistics |
| Fleet Operations | `RobotRecoveryRequested` | robot/location, last state/exposure, estimated hazards, mass/configuration, urgency, evidence | Robot Care, Station, Logistics |
| Vehicle Integration | `IntentAcknowledged` | command, vehicle, adapter, ack class, vehicle time/quality | Mission Control, audit |
| Vehicle Integration | `TelemetryNormalized` | vehicle, sample time, quality, position/state or artifact reference | Fleet, Mission, Safety |
| Station Operations | `StationAvailabilityChanged` | station, services, energy, connectivity, capacity, freshness | Mission, Fleet, Logistics |
| Station Operations | `ConnectivityMapUpdated` | cell/region, service classes, coverage/quality, validity, source manifest | Mission, Vehicle |
| Station Operations | `HabitatReadinessChanged` | habitat, deployable robots/pods by class, energy/charge/maintenance/communications constraints, freshness | Fleet, Mission, Logistics |
| Station Operations | `ChargeSessionChanged` | battery/vehicle/charger, state, scheduled/actual energy and power, anomaly/reason, time | Fleet, Logistics, Commercial |
| Station Operations | `EnergyShortfallPredicted` | habitat/period, critical/available energy distributions, cause, shed state, alternatives | Fleet, Mission, Logistics |
| Logistics | `DeliveryStateChanged` | mission, custody, route/source, state, quantity, exceptions | Mission, Incident, Commercial |
| Logistics | `SupplyShortagePredicted` | resource, place/time, demand/stock/lead-time distributions, bottleneck, alternatives | Incident, Mission, Station |
| Logistics | `PodCustodyChanged` | pod, manifest digest, origin/destination/custodian, mass/energy isolation, state/time | Station, Fleet, Mission |
| Logistics | `MobilizationWaveChanged` | wave, capability counts, pod/carrier/corridor plan, useful-arrival distribution, bottlenecks/exceptions | Incident, Mission, Fleet, Station |
| Vegetation Management | `TreatmentCompleted` | prescription/unit, planned/actual geometry, method/tool, quantities, exceptions, evidence | Hazard, Prediction, Commercial |
| Vegetation Management | `EffectivenessAssessed` | treatment, observation horizons, pre/post fuel/outcome, uncertainty, evidence | Hazard, Prediction, Commercial |
| Robot Care | `MaintenanceCompleted` | robot/configuration, policy/work order, procedures/parts, measurements/tests, limitations | Fleet, Station |
| Robot Care | `RobotRecovered` | robot, medic/recovery mission, scene/energy/contamination state, custody destination/evidence | Fleet, Station, Logistics, Safety |
| Robot Care | `RobotQuarantined` | robot/battery, hazard classes, isolation requirements, location, monitoring/release authority | Fleet, Station, Logistics, Safety |
| Robot Care | `RobotRecertified` | robot/configuration, repair/parts/calibration/burn-in evidence, approved capability limitations | Fleet, Safety |
| Robot Care | `RobotRetired` | robot, depower/sanitization/trust-revocation/salvage/disposition evidence | Fleet, Identity, Logistics, Commercial |
| Aerial Deployment | `BlanketConfigurationPromoted` | exact payload/material/panel/tether/parafoil/cradle/robot configuration, stage, ODD, evidence, expiry | Mission, Fleet, Logistics, Safety |
| Aerial Deployment | `PayloadLoadApproved` | aircraft/configuration reference, reconciled manifest/digest, mass properties, loading/release envelope, approvers | Aircraft adapter, Mission, Safety |
| Aerial Deployment | `PayloadReleased` | mission/release IDs, dual authorization digests, aircraft/payload configuration, release spacetime/conditions, predicted footprint | Incident, Mission, Safety, Logistics |
| Aerial Deployment | `DeploymentPhaseChanged` | prior/current phase, cohort/panel scope, stability/tension/vent state, navigation/time quality, contingency margin | Mission, Vehicle, Safety |
| Aerial Deployment | `BlanketActivated` | installed footprint/panels, gaps/omissions, sensor freshness, exposure envelope, temporary validity | Incident, Suppression, Vegetation, Hazard |
| Aerial Deployment | `ComponentDispositionChanged` | serialized component, exposure/damage/contamination, custody, recover/reuse/repair/recycle/sacrifice disposition | Logistics, Robot Care, Safety |
| Suppression Operations | `SuppressionStateChanged` | operation, envelope, state, agent/dose summary, reason | Mission, Safety, Incident |
| Safety Assurance | `ConstraintPublished` | constraint set, scope, ODD, validity, signature, supersedes | all constrained contexts |
| Safety Assurance | `PromotionChanged` | subject release/capability, stage, ODD, evidence, decision | deployment and fleet systems |
| Safety Assurance | `SafetyOccurrenceReported` | class, scope, time, immediate controls, evidence references | Incident, Security where applicable |
| Commercial Operations | `EntitlementChanged` | tenant, product, limits, effective interval, contract version | API/work admission only |
| Commercial Operations | `SupportEscalated` | case, severity, tenant, affected service/assets, safe contact data | operations and safety when applicable |
| Commercial Operations | `InvestmentCasePublished` | scenario/model versions, fact manifest, strategies, NPV/IRR/payback/TCO ranges, assumptions | authorized product/portfolio users |

## Synchronous request rules

Synchronous gRPC is restricted to queries requiring a current answer or commands sent directly to the owning context. Calls have deadlines, bounded payloads, authorization context, idempotency where effects occur, and explicit degraded behavior. No chain longer than one required downstream dependency may sit on a safety command path. Cached answers declare source version, observation time, expiry, and whether they are authoritative.

## Contract ownership and acceptance

The producer owns semantic meaning, schema, examples, conformance fixtures, compatibility tests, retention, access policy, SLO, and deprecation. Consumers register purpose, fields used, tolerated lateness, replay behavior, classification approval, and failure policy. A contract cannot reach production until producer and consumer tests, access controls, capacity limits, observability, replay, and rollback/migration evidence pass.
