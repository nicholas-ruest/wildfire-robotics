# Architecture and Assurance Traceability Model

## Identifier namespaces

Every governed artifact has an immutable identifier: `CAP` capability, `REQ` requirement, `INV` domain invariant, `HAZ` hazard, `THR` threat, `CTL` control, `ADR` decision, `API` contract, `EVT` event, `TST` test, `EVD` evidence, `REL` release, `CFG` configuration, `ODD` operational domain, `APR` approval, and `OCC` occurrence. Human-readable titles may change; identifiers and history do not.

## Required links

| Artifact | Must link to |
|---|---|
| Capability | intended user/authority, ODD, requirements, owner, commercial entitlement |
| Requirement | source, context, acceptance measure, tests; hazard/threat where applicable |
| Invariant | aggregate, requirement/control, enforcing methods, property tests |
| Hazard/threat | scope, causes, controls, verification, owner, residual decision |
| Contract | producer/consumer, requirement, classification, compatibility and contract tests |
| Release | source, dependencies, SBOM, artifacts, configuration, tests, evidence, approvals |
| Deployment | release digest, environment, tenant/region, time, actor, rollout and rollback result |
| Occurrence | deployed state, ODD, telemetry/audit evidence, controls taken, investigation and actions |

## Promotion query

A promotion decision must be mechanically able to answer: what capability is being promoted; for which exact hardware/software/configuration and ODD; under whose authority; which requirements, hazards, threats, and ADRs apply; what tests and field evidence passed; what exceptions and residual risks remain; who independently reviewed and accepted them; where the signed artifacts are; and how the release will be detected, contained, rolled back, and supported.

Missing, broken, expired, contradictory, or unapproved links fail the promotion gate. Evidence is content-addressed, immutable, access-controlled, retained under policy, and independently reproducible where feasible.
