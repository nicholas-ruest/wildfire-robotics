# Robot Care and Recovery Context

## Purpose

Maintain robot readiness continuously, recover disabled or damaged robots safely, operate distributed service automation, and decide evidence-backed repair, recertification, salvage, or retirement.

## Model

- **Aggregates:** ServicePolicy, MaintenancePlan, WorkOrder, RecoveryMission, DamageAssessment, QuarantineCase, RepairCase, RetirementCase.
- **Core invariant:** A damaged or uncertain robot cannot return to a habitat charge rack or operational eligibility until energy, contamination, structural, tool, configuration, calibration, and functional risks are classified and cleared.
- **Primary workflow:** Monitor → predict/schedule → inspect/service → detect failure → stabilize/recover → triage/quarantine → diagnose/repair → verify/burn-in → recertify or retire.

## Tactical model

| Aggregate | Lifecycle | Commands | Events |
|---|---|---|---|
| ServicePolicy | draft → validated → approved → effective → superseded | DefineServicePolicy, ValidatePolicy, ApprovePolicy, SupersedePolicy | ServicePolicyPublished |
| MaintenancePlan | proposed → scheduled → active → complete/overdue/suspended | CreateMaintenancePlan, ScheduleMaintenance, StartPlan, CompletePlan, SuspendPlan | MaintenanceDue, MaintenanceCompleted |
| WorkOrder | reported → triaged → assigned → servicing → testing → closed/rework | OpenWorkOrder, TriageWork, AssignMaintainer, RecordService, RunServiceTest, CloseWork | WorkOrderChanged |
| RecoveryMission | requested → assessed → authorized → en-route → stabilizing → recovering → transferred/aborted | RequestRecovery, AssessScene, AuthorizeRecovery, StabilizeRobot, RecoverRobot, TransferCustody | RecoveryMissionChanged, RobotRecovered |
| DamageAssessment | collecting → classified → reviewed → superseded | RecordDamageEvidence, ClassifyDamage, ReviewAssessment | DamageClassified |
| QuarantineCase | open → isolated → monitoring → cleared/escalated/disposed | OpenQuarantine, IsolateAsset, RecordMonitoring, ClearQuarantine, EscalateQuarantine | RobotQuarantined, QuarantineCleared |
| RepairCase | admitted → diagnosing → repairing → calibrating → burn-in → recertified/failed | AdmitRobot, DiagnoseRobot, AuthorizeRepair, InstallPart, CalibrateRobot, RunBurnIn, RecertifyRobot | RobotRepaired, RobotRecertified |
| RetirementCase | proposed → approved → depowered → sanitized → salvaged/recycled → closed | ProposeRetirement, ApproveRetirement, DepowerAsset, SanitizeData, SalvageParts, CompleteRetirement | RobotRetired |

Owned values include robot/battery/tool identity, location/custody, service policy/version, duty/thermal/exposure history, predicted degradation, fault codes, imagery/telemetry, damage class, stored-energy/tool/pressure state, contamination, structural/lift/tow points, medic pod/capability, recovery route/ODD, quarantine zone, diagnosis, part serial/provenance, repair procedure, calibration, tests/burn-in, residual limitation, recertification, salvage and destruction evidence.

## Invariants

- `RC-INV-001`: Maintenance intervals derive from approved policy, robot configuration, duty/exposure, calendar, cycles, condition and prior repairs; overdue critical work removes affected capability eligibility.
- `RC-INV-002`: A maintenance robot executes only promoted procedures for compatible modules/tools, verifies isolation, records parts/measurements, and escalates ambiguity or work beyond its assurance level.
- `RC-INV-003`: Recovery authorization requires scene/route/communications ODD, medic capability, robot mass/lift/tow compatibility, known or conservatively isolated energy/tool hazards, custody destination and fallback.
- `RC-INV-004`: Heat-exposed, swollen/leaking, electrically unsafe, contaminated, structurally unstable or unknown-state robots/batteries enter declared fire-separated quarantine transport and storage.
- `RC-INV-005`: Recovery never outranks human rescue, exclusion, evacuation or responder safety and cannot widen an incident assignment.
- `RC-INV-006`: Repair preserves serialized part provenance, approved procedure/configuration, measurements, calibration and independent required tests; cannibalized parts are never returned silently to stock.
- `RC-INV-007`: Return to service requires resolved/quarantined faults, compatible promoted configuration, completed maintenance/calibration/burn-in, fresh health assessment, evidence-linked recertification and Fleet eligibility update.
- `RC-INV-008`: Retirement proves energy/tool neutralization, identity/trust revocation, data sanitization, hazardous-material disposition, salvage custody and immutable asset history.

## Ports and read models

Ports cover fleet health/BMS, medic-pod and maintenance-robot adapters, mission/recovery routing, habitat/hospital/quarantine capacity, lifting/towing sensors, thermal/gas/electrical diagnostics, CMMS/procedure library, parts/inventory, calibration rigs, digital twin/simulation, identity revocation, evidence/media and waste/recycling. Read models expose readiness risk, due/overdue work, maintainer/medic coverage, disabled-robot queue, recovery ETA, quarantine hazards/capacity, hospital bays/throughput, parts bottlenecks, repeat faults, repair yield, mean time to repair, recertification and retirement/salvage.

## Boundary and failure policy

Fleet Operations owns operational eligibility and asset configuration; Station owns habitat energy/physical zones; Logistics owns pods, custody movement, spares and waste; Robot Care owns service/recovery/repair disposition. Unknown scene, unsafe stored energy, contamination, incompatible recovery tooling, unavailable quarantine, lost custody, failed calibration/burn-in or ambiguous fault fails to isolation and escalation—not optimistic movement or return to service.

## Implementation acceptance

Invariant/property, procedure compatibility, work-order concurrency, custody, parts provenance, quarantine, recovery simulation/HITL, lift/tow/load, energy isolation, thermal event, contamination, communications loss, medic failure, calibration, burn-in, recertification and retirement tests must pass. Hospital and medic capacity must meet approved correlated-incident damage scenarios and recovery SLOs.
