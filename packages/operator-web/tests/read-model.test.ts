import {describe, expect, it, vi} from "vitest";
import {
  ActionPolicyEngine,
  CommandReconciler,
  QueryCache,
  ReconnectReconciler,
  ScopedSnapshotStore,
  StatePartitions,
  adaptEnvelope,
  calculateFreshness,
  createQueryKey,
  settlePartialReads,
  type ReadEnvelope,
} from "../src/read-model";

describe("ADR-079 resilient read model", () => {
  it("should adapt the exact typed read envelope without substituting receipt time", () => {
    const envelope: ReadEnvelope<{count: number}> = adaptEnvelope({
      data: {count: 4}, sourceTime: "2026-07-23T11:59:00Z",
      receivedTime: "2026-07-23T12:00:00Z", asOfVersion: "v4", state: "current",
      provenance: [{source: "fleet", lineage: "sha256:4"}],
      uncertainty: {description: "±1"}, limitations: [], simulated: false,
    });
    expect(envelope).toEqual({
      data: {count: 4}, sourceTime: "2026-07-23T11:59:00Z",
      receivedTime: "2026-07-23T12:00:00Z", asOfVersion: "v4", state: "current",
      provenance: [{source: "fleet", lineage: "sha256:4"}],
      uncertainty: {description: "±1"}, limitations: [], simulated: false,
    });
  });

  it("should apply context freshness thresholds and reject future clock skew", () => {
    const now = Date.parse("2026-07-23T12:00:00Z");
    expect(calculateFreshness("fleet", "2026-07-23T11:59:40Z", now)).toBe("current");
    expect(calculateFreshness("hazard", "2026-07-23T11:55:00Z", now)).toBe("stale");
    expect(calculateFreshness("fleet", "2026-07-23T12:02:00Z", now)).toBe("unknown");
  });

  it("should include every authority and schema dimension in cache keys", () => {
    expect(createQueryKey({
      tenant: "t1", incident: "i9", context: "fleet", resource: "vehicles",
      filters: {cell: "alpha"}, schemaVersion: "v2", authorizationScope: "operator:west",
    })).toBe("t1|i9|fleet|vehicles|cell=alpha|v2|operator:west");
  });

  it("should deduplicate inflight reads and reject out-of-order snapshots", async () => {
    let resolve!: (value: ReadEnvelope<number>) => void;
    const read = vi.fn(() => new Promise<ReadEnvelope<number>>(done => { resolve = done; }));
    const cache = new QueryCache({maxReadRetries: 0, jitter: () => 0});
    const first = cache.read("key", read);
    const duplicate = cache.read("key", read);
    resolve(adaptEnvelope({
      data: 2, sourceTime: "2026-07-23T12:00:00Z", receivedTime: "2026-07-23T12:00:01Z",
      asOfVersion: "2", state: "current", provenance: [], limitations: [], simulated: false,
    }));
    await Promise.all([first, duplicate]);
    await cache.accept("key", adaptEnvelope({
      data: 1, sourceTime: "2026-07-23T11:00:00Z", receivedTime: "2026-07-23T12:00:02Z",
      asOfVersion: "1", state: "stale", provenance: [], limitations: [], simulated: false,
    }));

    expect(read).toHaveBeenCalledOnce();
    expect(cache.peek<number>("key")?.data).toBe(2);
    expect(cache.metrics().outOfOrderRejected).toBe(1);
  });

  it("should bound jittered retries to safe reads and support invalidation/coalescing", async () => {
    const read = vi.fn()
      .mockRejectedValueOnce(new Error("temporary"))
      .mockResolvedValue(adaptEnvelope({
        data: 3, sourceTime: null, receivedTime: "2026-07-23T12:00:00Z",
        asOfVersion: "3", state: "degraded", provenance: [], limitations: [], simulated: false,
      }));
    const cache = new QueryCache({maxReadRetries: 1, jitter: () => 0});
    await cache.read("key", read);
    cache.coalesce("key", 4);
    cache.coalesce("key", 5);
    cache.invalidate("key");

    expect(read).toHaveBeenCalledTimes(2);
    expect(cache.metrics()).toMatchObject({retries: 1, coalesced: 1, invalidations: 1});
  });

  it("should preserve successful regions during partial failure", async () => {
    const result = await settlePartialReads({
      fleet: Promise.resolve(4),
      hazard: Promise.reject(new Error("offline")),
    });
    expect(result.fleet).toEqual({status: "fulfilled", value: 4});
    expect(result.hazard.status).toBe("rejected");
  });

  it("should reconcile by cursor only when continuity is proven and never animate missed events", async () => {
    const changes = vi.fn(async () => ({cursor: "c2", snapshot: {count: 2}}));
    const snapshot = vi.fn(async () => ({cursor: "fresh", snapshot: {count: 9}}));
    const reconciler = new ReconnectReconciler(changes, snapshot);

    expect(await reconciler.reconnect({cursor: "c1", continuityProven: true})).toMatchObject({mode: "delta", animateMissed: false});
    expect(await reconciler.reconnect({cursor: "c1", continuityProven: false})).toMatchObject({mode: "snapshot", animateMissed: false});
  });

  it("should persist only allowed scoped snapshots and clear expired/tenant state", () => {
    const store = new ScopedSnapshotStore({now: () => 1_000});
    store.save({tenant: "t1", incident: "i1", key: "fleet", value: {count: 4}, expiresAt: 2_000, classification: "approved-read"});
    expect(store.load("t1", "i1", "fleet")).toEqual({count: 4});
    expect(() => store.save({tenant: "t1", incident: "i1", key: "token", value: "secret", expiresAt: 2_000, classification: "secret"})).toThrow();
    store.clearScope("t1", "i1");
    expect(store.load("t1", "i1", "fleet")).toBeNull();
  });

  it("should enforce connectivity policies without silent queue or auto-submit", () => {
    const policy = new ActionPolicyEngine();
    expect(policy.evaluate("online-required", {online: false, edgeAuthority: false}).allowed).toBe(false);
    expect(policy.evaluate("stage-offline", {online: false, edgeAuthority: false})).toMatchObject({allowed: true, submit: false, revalidate: true});
    expect(policy.evaluate("edge-authority-required", {online: false, edgeAuthority: true}).submit).toBe(true);
    expect(policy.reconnect()).toEqual({autoSubmitted: 0});
  });

  it("should separate four state partitions", () => {
    const state = new StatePartitions();
    expect(Object.keys(state)).toEqual(["authoritative", "view", "drafts", "commands"]);
  });

  it("should reconcile full command lifecycle and look up unknown submission before retry", async () => {
    const lookup = vi.fn(async () => ({
      idempotencyKey: "idem-1", receiptId: "r1", stage: "acknowledged" as const,
    }));
    const reconciler = new CommandReconciler(lookup);
    reconciler.recordTimeout("idem-1");
    const result = await reconciler.retry("idem-1");

    expect(lookup).toHaveBeenCalledWith("idem-1");
    expect(result.stage).toBe("acknowledged");
    expect(reconciler.stages).toEqual([
      "not-submitted", "submission-unknown", "accepted", "rejected", "acknowledged",
      "executing", "outcome-confirmed", "outcome-unknown", "held", "revoked", "failed",
    ]);
  });
});
