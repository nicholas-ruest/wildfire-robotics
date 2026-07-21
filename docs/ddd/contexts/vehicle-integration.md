# Vehicle Integration Context

## Purpose

Isolate ROS 2, MAVLink, autopilot, and vendor protocols behind a stable vehicle capability contract.

## Model

- **Aggregates:** GatewaySession, CommandDelivery, TelemetryStream.
- **Core invariant:** Only authorized unexpired intents cross the gateway; acknowledgements are correlated; protocol types do not leak upstream.
- **Primary workflow:** Authenticate -> translate intent -> enforce local envelope -> deliver/ack -> normalize telemetry.

## Tactical model

| Aggregate | Lifecycle | Commands | Events |
|---|---|---|---|
| GatewaySession | negotiating → active → degraded → closed/compromised | OpenSession, AuthenticatePeer, NegotiateCapability, MarkDegraded, CloseSession | GatewaySessionOpened, LinkDegraded, GatewaySessionClosed |
| CommandDelivery | received → validated → queued → sent → acknowledged/executing/completed/rejected/expired/unknown | DeliverIntent, RecordProtocolAck, RecordExecution, ExpireIntent, RevokeIntent | IntentAcknowledged, IntentRejected, IntentOutcomeUnknown |
| TelemetryStream | opening → active → degraded → closed | OpenTelemetry, AcceptSample, ChangeTier, MarkGap, CloseTelemetry | TelemetryNormalized, TelemetryGapDetected |

Owned values include authenticated peer/session, protocol/adapter versions, negotiated capability, command envelope/digest, fencing token, sequence/ack class, retry budget, local constraint version, vehicle clock quality, normalized sample, quality flags, and link statistics.

## Invariants

- `VI-INV-001`: Only authenticated, authorized, unexpired, correctly scoped and signed intents with current fencing and safety versions cross the adapter.
- `VI-INV-002`: Duplicate delivery cannot create duplicate physical effect; unsupported idempotency requires a safer protocol-specific guard or capability prohibition.
- `VI-INV-003`: Transport acknowledgement, vehicle acceptance, execution start, and physical outcome are distinct states.
- `VI-INV-004`: Local envelopes can reject/narrow upstream intent and remain effective without cloud connectivity.
- `VI-INV-005`: Unknown outcome, link loss, adapter crash, invalid telemetry, or clock uncertainty is explicit and triggers capability-specific minimum-risk behavior.

## Ports and read models

Each ROS 2, DDS, MAVLink, autopilot, payload, ruv-drone, or vendor integration is a sandboxed anti-corruption adapter with conformance fixtures. The ruv-drone adapter exposes bounded cohort, formation, coverage, relay, allocation, deconfliction and policy-version outcomes without leaking its internal types; its MAPPO policy is disabled unless independently promoted. Ports include hardware trust, local policy/constraint cache, protocol transport, monotonic clock, safe-state controller, telemetry buffer, and immutable audit. Read models expose session, cohort/relay state, command outcome, link health, gaps, protocol faults, and local constraint freshness.

## Boundary and failure policy

Consumes authorized intents and publishes normalized facts through the [integration registry](../integration-contracts.md). Link loss, duplicate/reordered messages, clock drift, protocol ambiguity, adapter crash, or storage pressure invoke bounded retry/buffering, preserve independent local safety, and expose degradation without fabricating outcome.

## Implementation acceptance

Domain invariants must be executable and property-tested; API/event contracts require compatibility tests; persistence requires migration/rollback and concurrency tests; adapters require fault-injection and replay tests; operational promotion requires the applicable evidence in the [production readiness standard](../../operations/production-readiness.md).
