import {describe, expect, it, vi} from "vitest";
import {
  ACTION_STAGES,
  ContextRecoveryController,
  DegradationController,
  MotionPreference,
  PerformanceTelemetry,
  ResourceTracker,
  SceneSelectionModel,
  SemanticActionController,
  SemanticFallbackWorkflow,
  TruthfulSnapshotProjector,
  renderEntityControls,
  renderCommandStages,
} from "../src/visualization";

describe("ADR-077 interaction and verification contract", () => {
  it("should synchronize pointer, keyboard, and HTML entity selection", () => {
    const model = new SceneSelectionModel();
    const observed: Array<string | null> = [];
    model.subscribe(selection => observed.push(selection?.id ?? null));

    model.select({id: "fleet:vehicle:1", type: "vehicle"}, "pointer");
    model.select({id: "fleet:cell:1", type: "cell"}, "keyboard");
    model.select({id: "fleet:vehicle:2", type: "vehicle"}, "semantic-html");

    expect(observed).toEqual(["fleet:vehicle:1", "fleet:cell:1", "fleet:vehicle:2"]);
  });

  it("should submit only confirmed semantic actions with complete safety context", async () => {
    const gateway = {dispatch: vi.fn(async () => ({
      idempotencyKey: "key-1", stage: "accepted" as const, physicalOutcome: "unknown" as const,
    }))};
    const actions = new SemanticActionController("suppression", gateway);
    actions.stage({
      idempotencyKey: "key-1", scope: "pump:p-7", constraints: ["flow<=40L/min"],
      expectedVersion: "v19", approval: "dual-approved", payload: {flow: 35},
    });

    await expect(actions.submit()).rejects.toThrow("confirmation");
    actions.confirm();
    const receipt = await actions.submit();

    expect(gateway.dispatch).toHaveBeenCalledWith("suppression", expect.objectContaining({
      scope: "pump:p-7", expectedVersion: "v19", approval: "dual-approved",
    }));
    expect(receipt.physicalOutcome).toBe("unknown");
  });

  it("should preserve view and draft state but clear confirmation on workspace suspension", () => {
    const actions = new SemanticActionController("mission", {dispatch: vi.fn()});
    actions.stage({
      idempotencyKey: "key-2", scope: "mission:m-2", constraints: ["lease=current"],
      expectedVersion: "v2", approval: "approved", payload: {},
    });
    actions.confirm();

    actions.suspend();

    expect(actions.snapshot().draft?.scope).toBe("mission:m-2");
    expect(actions.snapshot().confirmed).toBe(false);
  });

  it("should render all nine command stages with text and non-color symbols", () => {
    const host = document.createElement("div");
    renderCommandStages(host, ACTION_STAGES.map((stage, index) => ({
      idempotencyKey: `key-${index}`, stage,
    })));

    expect([...host.querySelectorAll("[data-command-stage]")].map(node => node.textContent))
      .toEqual(ACTION_STAGES.map(stage => expect.stringContaining(stage)));
  });

  it("should provide named focusable HTML controls for every scene entity", () => {
    const host = document.createElement("div");
    const model = new SceneSelectionModel();
    renderEntityControls(host, "Fleet readiness scene", [
      {id: "fleet:vehicle:1", type: "vehicle"},
      {id: "fleet:cell:1", type: "fleet-cell"},
    ], model);

    const buttons = [...host.querySelectorAll<HTMLButtonElement>("button")];
    buttons[1]?.click();

    expect(host.getAttribute("aria-label")).toBe("Fleet readiness scene");
    expect(buttons.map(button => button.textContent)).toEqual(["vehicle", "fleet-cell"]);
    expect(model.current?.id).toBe("fleet:cell:1");
  });

  it("should keep essential staging available in semantic fallback", () => {
    const actions = new SemanticActionController("fleet", {dispatch: vi.fn()});
    const fallback = new SemanticFallbackWorkflow(actions);
    fallback.stage({
      idempotencyKey: "fleet-1", scope: "fleet:alpha", constraints: ["epoch=9"],
      expectedVersion: "v9", approval: "approved", payload: {},
    });

    expect(fallback.inspect().draft?.scope).toBe("fleet:alpha");
  });

  it("should honor reduced motion and a persistent manual override", () => {
    const storage = new Map<string, string>();
    const motion = new MotionPreference(
      {matches: true} as MediaQueryList,
      {getItem: key => storage.get(key) ?? null, setItem: (key, value) => { storage.set(key, value); }},
    );
    expect(motion.reduced).toBe(true);

    motion.setOverride("full");
    const restored = new MotionPreference(
      {matches: true} as MediaQueryList,
      {getItem: key => storage.get(key) ?? null, setItem: (key, value) => { storage.set(key, value); }},
    );

    expect(restored.reduced).toBe(false);
  });

  it("should degrade in the declared deterministic order", () => {
    const controller = new DegradationController();
    expect(Array.from({length: 7}, () => controller.degrade())).toEqual([
      "pixel-ratio", "shadows", "particles", "model-detail",
      "update-frequency", "ambient-effects", "semantic-fallback",
    ]);
  });

  it("should rebuild a lost context once then fallback without losing the draft", () => {
    const rebuild = vi.fn(() => true);
    const fallback = vi.fn();
    const recovery = new ContextRecoveryController(rebuild, fallback);
    const draft = {scope: "vehicle:r-4"};

    recovery.contextLost(draft);
    recovery.contextLost(draft);

    expect(rebuild).toHaveBeenCalledOnce();
    expect(fallback).toHaveBeenCalledWith(draft);
  });

  it("should report performance counters without scene content or coordinates", () => {
    const sink = vi.fn();
    const telemetry = new PerformanceTelemetry(sink);
    telemetry.record({mountMs: 120, frameMs: 16, objects: 40, triangles: 900, drawCalls: 8});

    expect(sink).toHaveBeenCalledWith({
      mountMs: 120, frameMs: 16, objects: 40, triangles: 900, drawCalls: 8,
    });
  });

  it("should never fabricate intermediate authoritative facts", () => {
    const projector = new TruthfulSnapshotProjector<{position: number}>();
    const snapshot = Object.freeze({position: 10});

    expect(projector.project(snapshot, 5_000)).toBe(snapshot);
  });

  it("should leave no tracked frames, listeners, or resources after repeated cycles", () => {
    const resources = new ResourceTracker();
    for (let cycle = 0; cycle < 5; cycle++) {
      resources.trackFrame(cycle);
      resources.trackListener(() => undefined);
      resources.trackResource({dispose: vi.fn()});
      resources.disposeCycle();
    }

    expect(resources.residual).toEqual({frames: 0, listeners: 0, resources: 0});
  });
});
