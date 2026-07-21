# Contract registry governance

[`contracts/registry.toml`](../../contracts/registry.toml) is the machine-readable inventory of the published language defined by the [integration contract standard](../ddd/integration-contracts.md). It is metadata, not a replacement for Protobuf schemas or producer/consumer conformance tests.

## What belongs in the registry

Register a contract only when its owning context intentionally publishes it across a context boundary. Names in a context's internal aggregate lifecycle table are not public commands or events by implication. A synchronous command is added when its owning API and authorization boundary are explicitly published; it uses `kind = "command"` and the same metadata requirements as an event.

Each entry must declare a unique `(name, version)`, owner, SemVer version, canonical JSON example, golden binary fixture, authorized consumers, data classification, replay policy, and failure policy. Consumer aliases such as `authorized-read-models` denote separately approved principals, not wildcard access.

## Change control

- The producer owns semantics, schema, examples, fixtures, compatibility tests, access policy, retention, SLO, and deprecation.
- Additive compatible changes retain the major version and require updated examples and conformance coverage. Incompatible changes receive a new major version and run on a parallel subject through a measured migration window.
- Removed Protobuf fields and enum values are reserved permanently. Unknown enum values and additive fields must remain consumable.
- Every consumer registers its purpose, fields used, tolerated lateness, replay behavior, classification approval, and failure policy before access is granted.
- Registry review must include the producer and affected consumers. Safety-, authority-, identity-, or privacy-relevant changes also require the corresponding assurance/security/privacy review.

## Delivery and replay

Published events are delivered at least once. Consumers deduplicate by `message_id`, preserve semantic idempotency, detect aggregate-version gaps, and follow the entry's replay and failure policies. Invalid signatures, scope, classification, digests, or safety metadata are quarantined. Missing or ambiguous authority fails closed.

Before production promotion, every path in the registry must exist and schema lint, breaking-change checks, golden fixture tests, producer/consumer conformance, access-control tests, bounded-payload tests, replay tests, and rollback/migration evidence must pass.
