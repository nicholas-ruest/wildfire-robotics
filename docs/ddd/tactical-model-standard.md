# Tactical Domain Model Standard

This standard is normative for every bounded context. A context is not implementation-ready until its aggregate specifications, contracts, tests, and operational controls satisfy this document.

## Domain module shape

Each Rust context crate separates `domain`, `application`, `ports`, and `infrastructure`. The domain has no network, database, clock, random-number, framework, or vendor dependency. Those capabilities enter through ports. Only the context's public application API and published contracts are importable by another context. The shared kernel is limited to stable technical types: identifiers, time-quality, geospatial primitives, money, units, correlation/causation metadata, classification, and authenticated envelopes. It contains no business aggregate.

## Aggregate specification

Every aggregate root has a specification containing:

| Required element | Rule |
|---|---|
| Identity | Globally unique, opaque, tenant-scoped where applicable, never reused |
| State | Closed state enumeration and legal transition table |
| Invariants | Numbered predicates enforced at construction and every transition |
| Commands | Actor, authorization, preconditions, input, effect, errors, emitted events |
| Concurrency | Expected version; one transaction changes one aggregate and appends its outbox |
| Time | Injected trusted clock; UTC plus monotonic ordering and declared clock quality |
| Deletion | Retention, tombstone/anonymization behavior, legal hold, referential effect |
| Audit | Actor, reason, policy/evidence versions, correlation and causation |
| Repository | Domain port for load/save by ID and purpose-specific queries only |

Aggregate fields are private. Construction and mutation use validated value objects and domain methods. Invalid states are unrepresentable where practical. Child entities do not escape mutable aggregate ownership. Cross-aggregate invariants use reservations, uniqueness constraints, or process managers, never an in-memory transaction illusion.

## Command contract

Every application command includes the ADR-023 envelope and declares:

- stable command type and schema version;
- principal, tenant, incident, assignment, mission, and target scopes as applicable;
- expected aggregate version and idempotency key;
- issued-at, expiry, clock quality, and policy/safety/ODD versions;
- validated payload with units, CRS, ranges, and classification;
- success result, stable machine-readable rejection codes, and retry classification.

Command handling performs authentication, schema validation, authorization, freshness, deduplication, aggregate load, invariant evaluation, persistence, and outbox append atomically. Rejection is an auditable outcome but not a domain event unless the domain needs the fact.

## Event contract

Domain events are immutable past-tense facts. Published events use the integration envelope in [integration contracts](integration-contracts.md), carry no secrets, and expose only the minimum stable published language. Producers own schemas; consumers tolerate additive fields and unknown enum values. Corrections append superseding facts rather than rewriting history.

## Errors

All contexts use these top-level categories with context-specific codes:

| Category | Retry | Meaning |
|---|---:|---|
| `INVALID_ARGUMENT` | No | Schema, unit, CRS, range, or semantic validation failed |
| `UNAUTHENTICATED` | No | Identity or signature cannot be established |
| `FORBIDDEN` | No | Authenticated principal lacks authority |
| `STALE_AUTHORITY` | No | Assignment, policy, approval, ODD, lease, or command expired |
| `CONFLICT` | Re-read | Aggregate version or exclusive resource conflict |
| `FAILED_PRECONDITION` | After change | Domain invariant prevents the action |
| `RESOURCE_EXHAUSTED` | Backoff | Bounded quota or capacity reached |
| `DEPENDENCY_UNAVAILABLE` | Bounded | Required port unavailable; degraded policy applies |
| `INTERNAL` | Controlled | Unexpected failure; no sensitive detail returned |

Unsafe ambiguity maps to denial or minimum-risk behavior, never optimistic continuation.

## Value-object minimums

Contexts reuse or define validated types for identifiers, non-empty names, geo points/polygons with CRS, altitude reference, distance, speed, heading, time window, freshness, confidence, probability, quantity with unit, content digest, semantic version, classification, tenant scope, incident scope, ODD version, policy version, and evidence reference. Floating-point NaN/infinity and implicit unit conversion are rejected at boundaries.

## Persistence and messaging

- One aggregate transaction writes current state, version, audit metadata, and outbox.
- Inbox uniqueness is `(consumer, message_id)`; business idempotency uses the declared idempotency key.
- Optimistic locking is mandatory; retries rerun authorization and invariants.
- Repositories never expose cross-context joins or infrastructure types.
- Read models are rebuildable, freshness-labelled, and not authoritative for commands.
- Personal and classified fields follow ADR-028/049; tenant isolation follows ADR-036.

## Required verification

Each aggregate has example and property tests for construction, every legal and illegal transition, authorization-relevant conditions, concurrency, idempotency, serialization compatibility, time boundaries, numeric/unit/CRS boundaries, and invariant preservation. Each context adds contract, persistence, migration, replay, fault-injection, load, security, privacy, and recovery tests. Cyber-physical contexts also require scenario, SIL/HITL, communications-loss, clock-fault, emergency-stop, and minimum-risk-condition evidence.

## Definition of domain-ready

A context is domain-ready only when every catalogued aggregate and process manager has an approved specification, code and tests are traceable to numbered invariants, public contracts pass compatibility checks, threat and hazard controls are linked, SLOs and runbooks have owners, and no unresolved domain ambiguity can expand authority or create uncontrolled physical action.
