# Suppression Operations Context

## Purpose

Control human-approved teleoperation and progressively bounded suppressant application.

## Model

- **Aggregates:** SuppressionPlan, ActuationEnvelope, Target, Operation.
- **Core invariant:** No application without signed envelope; agent/dose/target/environment stay within limits; emergency stop is independent; every action is recorded.
- **Primary workflow:** Approve target/envelope -> arm under two-person control -> operate/monitor -> disarm -> reconcile usage/effect.

## Tactical model

| Aggregate | Lifecycle | Commands | Events |
|---|---|---|---|
| SuppressionPlan | draft → reviewed → approved → active → complete/revoked/expired | DraftPlan, ReviewPlan, ApprovePlan, RevokePlan | SuppressionPlanApproved, SuppressionPlanRevoked |
| ActuationEnvelope | draft → signed → armed → active → inhibited/expired/closed | DefineEnvelope, SignEnvelope, ArmEnvelope, InhibitEnvelope, CloseEnvelope | EnvelopeArmed, EnvelopeViolated, EnvelopeInhibited |
| Target | proposed → verified → approved → engaged → assessed/withdrawn | ProposeTarget, VerifyTarget, ApproveTarget, RecordEffect, WithdrawTarget | TargetApproved, TargetWithdrawn |
| Operation | prepared → armed → operating → paused/disarmed → completed/aborted | PrepareOperation, StartTeleoperation, ApplyAgent, PauseOperation, EmergencyStop, CompleteOperation | SuppressionStateChanged, AgentApplied |

Owned values include target geometry/classification, protected/exclusion volumes, suppressant identity/batch/safety data, dose/rate/pressure/direction, environmental limits, people/aircraft detection, operator/supervisor qualifications, independent stop path, sensor quality, actual application, and effect evidence.

## Invariants

- `SU-INV-001`: No actuation without current incident authority, promoted capability, approved target/agent, signed envelope, two distinct qualified approvers, and verified independent emergency stop.
- `SU-INV-002`: Location, aim, dose, rate, pressure, environment, visibility, exclusion, and supervision remain inside the envelope continuously.
- `SU-INV-003`: Person/aircraft intrusion, envelope uncertainty/breach, lost supervision, critical sensor/actuator fault, or stop request immediately inhibits energy/flow through an independent path.
- `SU-INV-004`: Every physical application records commanded and measured quantity, uncertainty, place/time, configuration, operator, and outcome; it is never represented as reversible.
- `SU-INV-005`: Autonomous behavior cannot select a new target, widen an envelope, re-arm, or resume after inhibit without the approved authority path.

## Ports and read models

Ports cover certified actuator controller, independent emergency stop, perception/exclusion monitoring, weather, agent inventory/SDS, vehicle state, operator console, dose measurement, and evidence recorder. Read models expose arming checklist, live envelope margins, independent-stop health, application ledger, effect assessment, and occurrence timeline.

## Boundary and failure policy

Owns the suppression process manager in [process managers](../process-managers.md). Any breach, intrusion, pressure/aim fault, uncertain sensor, lost supervision, or audit failure causes immediate independent inhibit/disarm, preserves evidence, and enters the safety-occurrence workflow before re-arming.

## Implementation acceptance

Domain invariants must be executable and property-tested; API/event contracts require compatibility tests; persistence requires migration/rollback and concurrency tests; adapters require fault-injection and replay tests; operational promotion requires the applicable evidence in the [production readiness standard](../../operations/production-readiness.md).
