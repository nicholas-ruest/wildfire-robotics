# Vegetation Management Context

## Purpose

Plan, authorize, execute, and measure preventive vegetation/fuel treatments by robots and supporting drones without conflating them with active-fire suppression.

## Model

- **Aggregates:** TreatmentProgram, Prescription, TreatmentUnit, WorkPackage, EffectivenessAssessment.
- **Core invariant:** No cutting/removal tool operates outside an approved prescription, treatment geometry, ODD, environmental/utility exclusion, tool envelope, or human authority.
- **Primary workflow:** Survey → prescribe → review/approve → package work → allocate robots/drones → treat/inspect → reconcile biomass → measure effectiveness.

## Tactical model

| Aggregate | Lifecycle | Commands | Events |
|---|---|---|---|
| TreatmentProgram | proposed → approved → active → suspended/completed | CreateProgram, ApproveProgram, SuspendProgram, CompleteProgram | TreatmentProgramApproved, TreatmentProgramSuspended |
| Prescription | draft → surveyed → reviewed → approved → superseded/expired | DraftPrescription, AttachSurvey, ReviewPrescription, ApprovePrescription, SupersedePrescription | PrescriptionApproved, PrescriptionSuperseded |
| TreatmentUnit | planned → ready → reserved → treating → treated → verified/rework | DefineUnit, MarkReady, ReserveUnit, RecordTreatment, VerifyUnit, RequireRework | TreatmentStarted, TreatmentCompleted, TreatmentReworkRequired |
| WorkPackage | draft → authorized → allocated → dispatched → completed/aborted | CreateWorkPackage, AuthorizeWork, AllocateAssets, DispatchTreatment, AbortWork | WorkPackageAuthorized, WorkPackageStateChanged |
| EffectivenessAssessment | scheduled → collecting → assessed → accepted/superseded | ScheduleAssessment, AttachObservation, AssessEffectiveness, AcceptAssessment | EffectivenessAssessed |

Owned values include land/authority, program objective, source fuel map, prescription geometry/version, vegetation/fuel class, treatment/tool/method, target residual fuel, ecology/cultural/utility/wildlife exclusions, slope/terrain, weather/fire-danger limits, people separation, ODD, robot/drone allocation, planned/actual quantity and geometry, biomass custody/disposition, imagery/telemetry, exceptions, cost, and effectiveness horizon.

## Invariants

- `VM-INV-001`: An approved prescription identifies authority, owner, objective, geometry, source survey/fuel state, method/tool, desired residual state, exclusions, ODD, validity, and approvers.
- `VM-INV-002`: A work package is a non-overlapping, version-bound subset of a current prescription with feasible resource, route, communications, weather, fire-danger, and biomass plans.
- `VM-INV-003`: Tool activation requires current mission authority, promoted vehicle/tool capability, geofence, exclusion monitoring, independent stop, and operator/supervision mode required by its assurance level.
- `VM-INV-004`: Person, wildlife, utility, cultural/environmental exclusion, fire-danger excursion, tool anomaly, lost localization/supervision, or prescription uncertainty causes immediate tool inhibit and minimum-risk behavior.
- `VM-INV-005`: Completion records actual treated and missed geometry, method/tool configuration, material removed/left, biomass custody, exceptions, media/telemetry digests, and verification confidence.
- `VM-INV-006`: Effectiveness compares pre/post observations at declared horizons and never assumes reduced fire risk solely from work completion.

## Ports and read models

Ports cover cadastral/land authority, environmental/cultural/utility constraints, fuel and terrain maps, weather/fire danger, robot/tool capabilities, mission allocation, drone survey/relay, biomass logistics, evidence media/RuPixel, cost/usage, and outcome models. Read models expose program progress, prescription/version overlay, ready/blocked treatment units, robot/drone allocation, exclusions, biomass flow, planned-vs-actual treatment, cost, and measured effectiveness.

## Boundary and failure policy

Consumes provenance-aware hazard/fuel intelligence and publishes treatment/effectiveness facts through the [integration registry](../integration-contracts.md). Stale survey, changed restriction, high fire danger, unexpected person/wildlife/utility, tool fault, route/communications loss, or uncertain localization halts the affected work and preserves safe vehicle/tool state; it never automatically widens or relocates treatment.

## Implementation acceptance

Domain invariants must be executable and property-tested; geometry/exclusion/tool contracts require simulation and physical fail-safe tests; persistence requires migration/rollback and concurrency tests; adapters require fault injection and replay; promotion requires applicable evidence in the [production readiness standard](../../operations/production-readiness.md).
