import {describe, expect, it, vi} from "vitest";
import {
  WORKSPACE_IDS,
  WorkspaceFeatureFlags,
  WorkspaceSceneCatalog,
  WorkspaceViewStateStore,
  workspaceSceneModules,
  type ActionGateway,
  type SceneServices,
} from "../src/visualization";

const expectedIds = [
  "incident", "hazard", "predictive", "mission", "fleet",
  "vehicle", "station", "logistics", "vegetation", "suppression",
  "aerial", "safety", "identity", "recovery", "commercial",
] as const;

function gateway(): ActionGateway {
  return {
    dispatch: vi.fn(async (_workspaceId, action) => ({
      idempotencyKey: (action as {idempotencyKey: string}).idempotencyKey,
      receiptId: "receipt-001",
      stage: "accepted" as const,
      physicalOutcome: "unknown" as const,
    })),
  };
}

function services(actionGateway = gateway()): SceneServices {
  return {
    renderer: {render: vi.fn(), resize: vi.fn(), dispose: vi.fn()},
    actionGateway,
    reducedMotion: false,
    invalidate: vi.fn(),
  };
}

describe("ADR-076 workspace scene catalog", () => {
  it("should register all fifteen separately owned workspace modules", () => {
    expect(WORKSPACE_IDS).toEqual(expectedIds);
    expect(workspaceSceneModules.map(module => module.owner)).toEqual(
      expectedIds.map(id => `visualization/scenes/${id}`),
    );
  });

  it.each(expectedIds)("should give %s a unique domain metaphor and scene signature", id => {
    const module = workspaceSceneModules.find(candidate => candidate.id === id)!;
    const otherSignatures = workspaceSceneModules
      .filter(candidate => candidate.id !== id)
      .map(candidate => candidate.signature);

    expect(module.metaphor.length).toBeGreaterThan(12);
    expect(otherSignatures).not.toContain(module.signature);
  });

  it.each(expectedIds)("should expose stable entities and overview controls for %s", id => {
    const module = workspaceSceneModules.find(candidate => candidate.id === id)!;
    const scene = module.create();

    expect(new Set(module.entityTypes).size).toBeGreaterThanOrEqual(3);
    expect(scene.getViewState().camera).toBe("overview");
    scene.setViewState({camera: "inspection", selectionId: `${id}:selected`});
    scene.resetView();
    expect(scene.getViewState()).toEqual({camera: "overview", selectionId: null});
  });

  it.each(expectedIds)("should provide truthful deterministic fixtures and fallback metadata for %s", id => {
    const module = workspaceSceneModules.find(candidate => candidate.id === id)!;
    const first = module.fixture();
    const second = module.fixture();
    const description = module.create().describe(first);

    expect(first).toEqual(second);
    expect(first.availableStates).toEqual(["current", "stale", "degraded", "gap", "unknown"]);
    expect(first.metadata).toMatchObject({
      units: expect.any(String),
      timestamp: expect.stringMatching(/^2026-/),
      uncertainty: expect.any(String),
      provenance: expect.any(String),
      limitations: expect.any(Array),
      simulated: true,
    });
    expect(first.dynamicSignal.value).toEqual(expect.any(Number));
    expect(description.summary).toContain(first.metadata.timestamp);
  });

  it.each(expectedIds)("should stage %s actions through the gateway and retain physical outcome truth", async id => {
    const actionGateway = gateway();
    const module = workspaceSceneModules.find(candidate => candidate.id === id)!;
    const scene = module.create();
    scene.mount(document.createElement("div"), services(actionGateway));

    const receipt = await scene.dispatch({
      kind: "stage-management-action",
      idempotencyKey: `${id}-action-1`,
      confirmed: true,
    });

    expect(actionGateway.dispatch).toHaveBeenCalledWith(id, expect.objectContaining({confirmed: true}));
    expect(receipt).toMatchObject({stage: "accepted", physicalOutcome: "unknown"});
  });

  it("should rollback an individual workspace to its semantic fallback", () => {
    const flags = new WorkspaceFeatureFlags({fleet: false});
    const catalog = new WorkspaceSceneCatalog(flags, new WorkspaceViewStateStore());

    expect(catalog.resolve("fleet").mode).toBe("fallback");
    expect(catalog.resolve("incident").mode).toBe("scene");
  });

  it("should preserve safe camera and selection state across scene replacement", () => {
    const states = new WorkspaceViewStateStore();
    const catalog = new WorkspaceSceneCatalog(new WorkspaceFeatureFlags(), states);
    const first = catalog.resolve("incident");
    if (first.mode !== "scene") throw new Error("incident scene unavailable");
    first.scene.setViewState({camera: "inspection", selectionId: "incident:division"});
    catalog.release("incident", first.scene);

    const restored = catalog.resolve("incident");

    expect(restored.mode).toBe("scene");
    if (restored.mode === "scene") {
      expect(restored.scene.getViewState()).toEqual({
        camera: "inspection",
        selectionId: "incident:division",
      });
    }
  });
});
