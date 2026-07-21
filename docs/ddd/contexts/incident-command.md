# Incident Command Context

## Purpose

Represent incident authority, objectives, restrictions, and operational assignments.

## Model

- **Aggregates:** Incident, OperationalPeriod, Objective, Assignment, Restriction.
- **Core invariant:** Assignments require authority, approver, geography, validity and constraints; restrictions can only narrow active authority without a new approval.
- **Primary workflow:** Open incident -> establish period/objectives -> approve restriction set -> issue/revoke assignments.

## Tactical model

| Aggregate | Lifecycle | Commands | Events |
|---|---|---|---|
| Incident | draft → active → contained → closed → archived | OpenIncident, ChangeCommand, SetGeography, ContainIncident, CloseIncident | IncidentOpened, IncidentCommandChanged, IncidentClosed |
| OperationalPeriod | draft → approved → active → expired/closed | DefinePeriod, ApprovePeriod, ActivatePeriod, ClosePeriod | OperationalPeriodActivated, OperationalPeriodExpired |
| Objective | proposed → approved → active → completed/cancelled | ProposeObjective, ApproveObjective, CompleteObjective, CancelObjective | ObjectiveApproved, ObjectiveCompleted |
| Assignment | draft → approved → issued → accepted → completed/revoked/expired | DraftAssignment, ApproveAssignment, IssueAssignment, RevokeAssignment | AssignmentIssued, AssignmentRevoked |
| Restriction | proposed → effective → superseded/expired | ProposeRestriction, ApproveRestriction, SupersedeRestriction | RestrictionChanged |

Owned values include command authority, organization, role/qualification, geography/altitude, validity, objective priority, resource/capability request, constraints, approvers, briefing acknowledgement, reason, and source authority.

## Invariants

- `IC-INV-001`: Exactly one effective command-authority record governs an incident scope at a time; transfers preserve chain of custody.
- `IC-INV-002`: An operational period cannot activate without approved authority, geography, time window, objectives, restrictions, and accountable commander.
- `IC-INV-003`: An assignment is a bounded objective authorization, never an actuator instruction, and cannot exceed incident/period authority.
- `IC-INV-004`: Authority expansion requires approval; a valid emergency restriction may narrow immediately and is never delayed by downstream acknowledgement.
- `IC-INV-005`: Expired, revoked, conflicting, or ambiguous authority fails closed and remains auditable.

## Ports and read models

Ports integrate authoritative incident-management systems through an anti-corruption layer, identity/qualification, maps, approvals, notification, and records export. Read models provide command board, operational-period plan, assignment board, restriction overlay, and acknowledgement gaps with source and freshness.

## Boundary and failure policy

Publishes authority through the [integration registry](../integration-contracts.md) and owns the incident-activation process manager. Authority ambiguity, expired period, qualification failure, missing acknowledgement, or conflicting restriction blocks issuance or narrows operation and escalates to incident command.

## Implementation acceptance

Domain invariants must be executable and property-tested; API/event contracts require compatibility tests; persistence requires migration/rollback and concurrency tests; adapters require fault-injection and replay tests; operational promotion requires the applicable evidence in the [production readiness standard](../../operations/production-readiness.md).
