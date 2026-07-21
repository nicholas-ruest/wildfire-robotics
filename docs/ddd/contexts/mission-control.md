# Mission Control Context

## Purpose

Turn authorized assignments into constrained, deconflicted, auditable missions.

## Model

- **Aggregates:** Mission, Allocation, MissionLease, ConflictSet.
- **Core invariant:** One active lease controls a mission/vehicle intent; every command fits assignment and safety envelope; abort outranks progression.
- **Primary workflow:** Plan -> policy/safety check -> allocate -> authorize -> lease/dispatch -> monitor -> complete/abort.

## Tactical model

| Aggregate | Lifecycle | Commands | Events |
|---|---|---|---|
| Mission | draft → validating → authorized → dispatched → executing → completed/aborted/failed/expired | PlanMission, ValidateMission, AuthorizeMission, DispatchMission, AdvanceMission, AbortMission | MissionAuthorized, MissionStateChanged |
| Allocation | proposed → reserved → committed → released/expired | ProposeAllocation, ReserveCapability, CommitAllocation, ReleaseAllocation | AllocationReserved, AllocationReleased |
| MissionLease | offered → active → renewed → revoked/expired | AcquireLease, RenewLease, RevokeLease | MissionLeaseAcquired, MissionLeaseLost |
| ConflictSet | open → mitigated → accepted/closed | DetectConflict, ProposeMitigation, AcceptResidualConflict, CloseConflict | ConflictDetected, ConflictResolved |

Owned values include plan graph and version, assignment snapshot, operational envelope, capability requirements, reservations, deconfliction volume/time, policy/constraint/ODD versions, lease holder/term, checkpoints, execution state, abort reason, and evidence references.

## Invariants

- `MC-INV-001`: A mission is authorized only against a current assignment and exact policy, restriction, constraint, ODD, hazard, fleet, and plan snapshots.
- `MC-INV-002`: One active lease may advance a mission/vehicle intent; fencing tokens reject a prior holder after transfer or expiry.
- `MC-INV-003`: Allocation requires current attested capability and cannot double-commit exclusive resources.
- `MC-INV-004`: Unresolved collision, airspace, resource, or incompatible-objective conflicts prevent dispatch unless an authorized process explicitly accepts a permitted residual conflict.
- `MC-INV-005`: Abort, restriction, grounding, lease loss, or authority expiry outranks progression and initiates bounded revocation/minimum-risk handling.
- `MC-INV-006`: Fleet planning is hierarchical by partition/cell; global planning consumes bounded summaries and cannot synchronously lock or enumerate the million-asset fleet.
- `MC-INV-007`: A relay-drone plan declares required service class, coverage, duration, spectrum, energy/return reserve, airspace, handoff and fallback; connectivity benefit cannot relax vehicle-local safety.

## Ports and read models

Ports provide policy/constraint evaluation, hierarchical planner, deconfliction, partitioned fleet reservation, ruv-drone cohort/relay planning, link-quality forecasts, command gateway, clock, maps, weather, and notification. Read models expose regional/cell capacity, mission board, allocation/lease state, relay coverage, conflict timeline, command/ack chain, and freshness; command decisions reload authoritative aggregates.

## Boundary and failure policy

Owns the mission-authorization process manager and uses contracts in the [integration registry](../integration-contracts.md). Lease loss, conflict, stale input, partition, unknown command outcome, or policy denial stops dispatch, revokes reachable intent, requests the vehicle-specific minimum-risk condition, and escalates unresolved physical state.

## Implementation acceptance

Domain invariants must be executable and property-tested; API/event contracts require compatibility tests; persistence requires migration/rollback and concurrency tests; adapters require fault-injection and replay tests; operational promotion requires the applicable evidence in the [production readiness standard](../../operations/production-readiness.md).
