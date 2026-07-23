# ADR-079: Resilient operator read model and offline UI state

- **Status**: proposed
- **Date**: 2026-07-23
- **Deciders**:
- **Tags**: operator-console, read-model, offline, resilience, state-management

## Context

The operator UI combines telemetry, commands, forecasts, approvals, safety constraints, and business data owned by different bounded contexts. These inputs update at different rates and may be current, delayed, incomplete, superseded, or unavailable during field connectivity loss.

A single loading spinner or global “online” indicator hides this distinction. Retaining the last successful response without its age can present stale information as current. Clearing the interface on reconnection loses operator context. Optimistically changing operational state after command submission can falsely imply that a robot or physical process has responded.

The UI needs one consistent model for server state, local view state, drafts, and command state. It must remain useful during partial failure while clearly limiting actions that cannot be safely authorized or delivered.

## Decision

Adopt a resilient, query-oriented operator read-model layer. Keep four state classes separate:

1. **Authoritative server state** — immutable snapshots and event deltas received from owning bounded contexts.
2. **Local view state** — route, filters, sort, density, camera, panel layout, and selection.
3. **Draft action state** — unsubmitted operator input, validation, scope, and expected version.
4. **Command lifecycle state** — submission, receipt, acknowledgement, execution, and observed outcome.

Local reducers may derive presentation state but may not rewrite authoritative facts. Operational commands do not use optimistic updates. Only local view preferences may update optimistically.

### Read envelope

Every query exposed to the UI returns or is adapted into a typed envelope:

```ts
interface ReadEnvelope<T> {
  data: T | null;
  sourceTime: string | null;
  receivedTime: string;
  asOfVersion: string | null;
  state: "current" | "stale" | "degraded" | "gap" | "unknown";
  provenance: ProvenanceRef[];
  uncertainty?: Uncertainty;
  limitations: Limitation[];
  simulated: boolean;
}
```

Freshness is calculated from the owning context's declared policy, not a global timeout. The interface displays source time and freshness at the point where a fact is used. Receipt time may not substitute for observation time.

Aggregate views retain per-source state. One failed query does not blank unaffected regions or convert a partially known situation into a single generic error.

### Query cache and synchronization

Use a typed query cache behind an application-owned adapter so a library can be introduced or replaced without leaking its API into workspace components. Query keys include tenant, incident, bounded context, resource identity, relevant filters, schema version, and authorization scope.

The cache:

- deduplicates in-flight reads and cancels obsolete requests;
- applies bounded retries with jitter only to safe, idempotent reads;
- rejects out-of-order snapshots using version and source-time rules;
- coalesces high-rate telemetry before rendering;
- invalidates from authoritative events and command receipts;
- preserves the previous snapshot during refresh while visibly marking it stale;
- records cache hit, age, retry, gap, and reconciliation metrics without sensitive payloads.

A reconnect performs version-aware reconciliation. The UI first restores the last known snapshot as stale, then requests changes since its cursor or fetches a new snapshot when continuity cannot be proven. It never animates through missed events as if they occurred live.

### Offline and degraded operation

Persist only explicitly approved, minimum-necessary read snapshots and harmless view preferences. Persistence is tenant- and incident-scoped, encrypted by the platform where available, bounded by classification and retention policy, and cleared on sign-out, scope loss, or expiry. Secrets, bearer tokens, raw command envelopes, confirmations, and protected bulk evidence are not placed in browser persistence.

Each action declares one connectivity policy:

- `online-required` — disabled offline with a reason and recovery guidance;
- `stage-offline` — may save a local draft but requires revalidation and explicit submission after reconnection;
- `edge-authority-required` — available only when a trusted local authority service confirms scope and can issue a durable receipt.

The browser does not silently queue operational commands for later transmission. Reconnection never auto-submits a draft.

The global connection indicator reports transport reachability separately from data freshness. Workspace regions expose their own current, stale, degraded, gap, or unknown state. Operators can always identify the last known time, missing capability, effect on decisions, and suggested recovery.

### Command and outcome reconciliation

Command submission includes the expected resource version, authorization scope, idempotency key, and constraint snapshot required by the owning API. A durable receipt becomes a separate command-lifecycle record; it does not mutate the read model.

After reconnection or refresh, command records reconcile by idempotency key and authoritative receipt ID. The UI distinguishes:

- not submitted;
- submission status unknown;
- accepted or rejected;
- acknowledged;
- executing;
- physical outcome confirmed;
- physical outcome unknown;
- held, revoked, or failed.

If submission status is unknown, the UI queries by idempotency key before allowing a retry. A transport timeout is never shown as a command rejection or successful outcome.

### Verification

Contract tests cover envelope parsing, schema mismatch, freshness transitions, authorization-scope changes, and context-specific thresholds. Deterministic tests cover out-of-order events, duplicate receipts, partial query failure, offline restart, reconnect with and without cursor continuity, clock skew, cache expiry, tenant switching, and revocation.

Browser tests verify that stale data cannot appear current, essential cached data remains inspectable offline, prohibited actions cannot be submitted, staged drafts require revalidation, and no command is silently replayed.

## Consequences

### Positive

- Keeps useful last-known information visible without disguising its age or quality.
- Prevents optimistic UI state from implying an unobserved physical outcome.
- Makes partial failure and reconnection behavior consistent across workspaces.
- Separates harmless UI persistence from sensitive or consequential state.
- Provides testable contracts for freshness, gaps, command receipts, and offline capability.

### Negative

- Requires APIs and read models to expose versions, source times, provenance, and resumable cursors.
- Adds cache invalidation, reconciliation, persistence, and clock-quality complexity.
- Some offline workflows remain intentionally unavailable.

### Neutral

- This ADR does not make the browser a system of record.
- A third-party query library may implement the cache behind the adapter after dependency review.
- Visual presentation of these states is governed by ADR-078 and scene behavior by ADR-075 through ADR-077.

## Links

- [ADR-003](ADR-003-edge-first-intermittently-connected-operations.md)
- [ADR-005](ADR-005-event-driven-fleet-control-plane.md)
- [ADR-023](ADR-023-canonical-authenticated-command-envelope.md)
- [ADR-025](ADR-025-version-vector-edge-reconciliation-with-authority-precedence.md)
- [ADR-026](ADR-026-tiered-telemetry-and-bounded-backpressure.md)
- [ADR-027](ADR-027-utc-plus-monotonic-time-and-clock-quality.md)
- [ADR-028](ADR-028-data-classification-retention-and-deletion-policy.md)
- [ADR-042](ADR-042-configuration-feature-flags-and-runtime-change.md)
- [ADR-049](ADR-049-privacy-consent-accessibility-and-records.md)
- [ADR-078](ADR-078-task-oriented-operator-shell-and-design-system.md)
