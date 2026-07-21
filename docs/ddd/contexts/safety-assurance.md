# Safety Assurance Context

## Purpose

Own hazards, ODDs, constraints, evidence cases, promotion decisions, and safety occurrences.

## Model

- **Aggregates:** Hazard, SafetyConstraint, ODD, EvidenceCase, SafetyOccurrence.
- **Core invariant:** Hazards have owners; constraints are versioned/signed; promotion requires complete evidence and independent approval; near misses block promotion.
- **Primary workflow:** Analyze -> mitigate -> verify -> assemble evidence -> approve/reject promotion -> monitor occurrences.

## Tactical model

| Aggregate | Lifecycle | Commands | Events |
|---|---|---|---|
| Hazard | identified → analyzed → controlled → accepted → monitoring/closed/reopened | RegisterHazard, AnalyzeHazard, AttachControl, AcceptResidualRisk, ReopenHazard | HazardRegistered, ResidualRiskAccepted, HazardReopened |
| SafetyConstraint | draft → verified → signed → effective → superseded/revoked/expired | DefineConstraint, VerifyConstraint, PublishConstraint, RevokeConstraint | ConstraintPublished, ConstraintRevoked |
| ODD | draft → validated → approved → restricted/superseded | DefineODD, AttachEvidence, ApproveODD, NarrowODD | ODDApproved, ODDNarrowed |
| EvidenceCase | assembling → review → complete → approved/rejected/stale | AddClaim, LinkEvidence, SubmitCase, ReviewCase, MarkStale | EvidenceCaseSubmitted, EvidenceCaseStale |
| SafetyOccurrence | reported → triaged → investigating → actions-open → closed/reopened | ReportOccurrence, TriageOccurrence, RecordFinding, AssignAction, CloseOccurrence | SafetyOccurrenceReported, SafetyActionAssigned |

Owned values include system boundary/intended use, hazard/threat cause, severity/exposure/controllability method, requirement/control, verification, evidence digest, assumption, residual risk, ODD, configuration/release, independent reviewer, approval scope/expiry, occurrence facts, findings, and corrective actions.

## Invariants

- `SA-INV-001`: Every hazard has accountable owner, analysis method, controls, verification, residual decision, scope, and review trigger.
- `SA-INV-002`: A constraint is immutable, signed, scoped, versioned, time-bounded where required, and only superseded by an authorized equal-or-stricter transition.
- `SA-INV-003`: Promotion identifies exact release/configuration/hardware/capability/ODD and requires complete traceability plus independent approval proportionate to risk.
- `SA-INV-004`: Missing/stale evidence, failed control, invalid assumption, severe occurrence, or overdue critical action blocks or revokes affected promotion.
- `SA-INV-005`: Residual risk acceptance is human, attributable, competent, scoped, expiring/reviewable, and cannot be inferred from deployment.

## Ports and read models

Ports connect requirements/test evidence, artifact provenance, configuration/deployment, incident/security systems, regulatory matrix, competency, and signing. Read models expose hazard log, control verification, ODD catalog, traceability gaps, promotion status, occurrences/actions, and expiring approvals.

## Boundary and failure policy

Owns release promotion and assurance traceability under ADR-045 and the [traceability model](../traceability-model.md). Missing evidence, failed control, new hazard, serious near miss, invalid assumption, or unaccepted residual risk revokes/blocks promotion or narrows ODD and triggers replanning.

## Implementation acceptance

Domain invariants must be executable and property-tested; API/event contracts require compatibility tests; persistence requires migration/rollback and concurrency tests; adapters require fault-injection and replay tests; operational promotion requires the applicable evidence in the [production readiness standard](../../operations/production-readiness.md).
