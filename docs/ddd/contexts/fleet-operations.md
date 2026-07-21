# Fleet Operations Context

## Purpose

Own the operational identity, capability, configuration eligibility, and health of managed vehicles.

## Model

- **Aggregates:** Vehicle, BatteryAsset, CapabilityRecord, HealthAssessment, Configuration, FleetCell, CollaborationProfile.
- **Core invariant:** A grounded vehicle cannot be allocated; capability claims require signed evidence and compatible configuration; health freshness is explicit.
- **Primary workflow:** Enroll -> attest configuration/capabilities -> assess health -> mark eligible -> monitor/ground.

## Tactical model

| Aggregate | Lifecycle | Commands | Events |
|---|---|---|---|
| Vehicle | candidate → enrolled → active → grounded → retired | RegisterVehicle, ActivateVehicle, GroundVehicle, ClearGrounding, RetireVehicle | VehicleRegistered, VehicleGrounded, VehicleCleared |
| BatteryAsset | registered → available → assigned → charging/in-use → quarantined/servicing → retired | RegisterBattery, BindBattery, RecordBmsSummary, ChangeEligibility, QuarantineBattery, RetireBattery | BatteryEligibilityChanged, BatteryHealthChanged |
| CapabilityRecord | claimed → testing → attested → suspended/expired | ClaimCapability, AttachEvidence, AttestCapability, SuspendCapability | CapabilityAttested, CapabilitySuspended |
| HealthAssessment | collecting → assessed → stale/superseded | RecordHealthEvidence, AssessHealth, MarkStale | VehicleHealthChanged |
| Configuration | draft → validated → approved → installed → superseded | RegisterConfiguration, ValidateCompatibility, ApproveConfiguration, AttestInstallation | ConfigurationApproved, ConfigurationInstalled |
| FleetCell | forming → active → splitting/merging → degraded/closed | FormCell, AssignMember, ChangeEpoch, SplitCell, MergeCell, DegradeCell | FleetCellChanged, FleetCellRepartitioned |
| CollaborationProfile | collecting → evaluated → promoted → stale/revoked | RecordCooperationOutcome, EvaluateRelationship, PromoteProfile, RevokeProfile | CollaborationProfilePromoted |

Owned values include asset serial/ownership, hardware/software BOM, configuration digest, capability and assurance level, ODD, evidence, maintenance/calibration state, health observations/freshness, eligibility, grounding reason, clearance authority, cell/epoch/membership, cohort capacity, and time-decayed relationship evidence. Battery assets own chemistry/form factor, nominal/usable capacity, BMS/firmware, state-of-health summary/uncertainty, cycles/throughput, compatibility, custody, warranty, quarantine and predicted life. Relationship features include context, cooperative task, communication, handoff, complementarity, outcome, safety result, source and model version—not subjective trust.

## Invariants

- `FO-INV-001`: Physical identity, device identity, tenant/owner, and configuration are uniquely bound with attestation evidence.
- `FO-INV-002`: Eligibility requires non-expired trust, maintenance, calibration, compatible configuration, fresh health, promoted capability, and matching ODD.
- `FO-INV-003`: Grounded or retired vehicles and suspended capabilities cannot be allocated.
- `FO-INV-004`: Health is an assessment with source/time/quality, never an unqualified boolean.
- `FO-INV-005`: Clearance identifies the resolved cause, evidence, authorized approver, and new aggregate version.
- `FO-INV-006`: Every active asset belongs to one authoritative fleet partition epoch per purpose; stale membership/fencing cannot command or reserve it.
- `FO-INV-007`: Relationship/coherence scores are advisory, time-decayed, outcome-sourced and poison-resistant; they never create identity, trust, authority, capability, or allocation.
- `FO-INV-008`: A vehicle is energy-eligible only with a compatible, non-quarantined battery/fuel system whose estimated usable energy, power, thermal state and uncertainty satisfy mission departure and minimum-risk reserves.

## Ports and read models

Ports integrate inventory/CMMS, attestation, vulnerability intelligence, telemetry summaries, test evidence, safety promotion, partition placement, and the optional RVM relationship/coherence adapter. Read models are hierarchically partitioned and expose fleet inventory, cell load/epochs, eligibility, health/freshness, maintenance forecast, compatibility, collaboration evidence, and grounding queue without fleet-wide scans.

## Boundary and failure policy

Owns enrollment/capability promotion with Identity and Safety through the [process-manager rules](../process-managers.md). Stale health, incompatible or vulnerable firmware, maintenance/calibration overdue, lost trust, or unexplained attestation change grounds affected capability and requires explicit evidence-backed clearance.

## Implementation acceptance

Domain invariants must be executable and property-tested; API/event contracts require compatibility tests; persistence requires migration/rollback and concurrency tests; adapters require fault-injection and replay tests; operational promotion requires the applicable evidence in the [production readiness standard](../../operations/production-readiness.md).
