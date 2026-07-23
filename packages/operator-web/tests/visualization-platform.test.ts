import {describe, expect, it, vi} from "vitest";
import {
  DemandRenderScheduler,
  WorkspaceVisualizationPlatform,
  type ActionGateway,
  type FrameRenderer,
  type WorkspaceScene,
} from "../src/visualization";

type State = {readonly revision: number};
type Action = {readonly kind: "inspect"};

function scene(id = "incident"): WorkspaceScene<State, Action> {
  return {
    id,
    mount: vi.fn(),
    update: vi.fn(),
    resize: vi.fn(),
    setVisibility: vi.fn(),
    pick: vi.fn(() => null),
    dispatch: vi.fn(async () => ({idempotencyKey: "receipt-1", stage: "accepted" as const})),
    describe: vi.fn(() => ({
      name: "Incident terrain",
      summary: "Current incident state",
      instructions: "Use the entity list to inspect the scene.",
    })),
    dispose: vi.fn(),
  };
}

function renderer(): FrameRenderer {
  return {render: vi.fn(), resize: vi.fn(), dispose: vi.fn()};
}

function actionGateway(): ActionGateway {
  return {dispatch: vi.fn(async () => ({idempotencyKey: "receipt-1", stage: "accepted" as const}))};
}

describe("ADR-075 visualization platform", () => {
  it("should render once when invalidated and remain idle otherwise", () => {
    const callbacks: FrameRequestCallback[] = [];
    const scheduler = new DemandRenderScheduler(
      callback => (callbacks.push(callback), callbacks.length),
      vi.fn(),
    );
    const render = vi.fn();

    scheduler.invalidate(render);
    callbacks.shift()?.(10);

    expect(render).toHaveBeenCalledTimes(1);
  });

  it("should cancel scheduled work when hidden", () => {
    const cancel = vi.fn();
    const scheduler = new DemandRenderScheduler(vi.fn(() => 42), cancel);
    scheduler.invalidate(vi.fn());

    scheduler.setVisibility(false);

    expect(cancel).toHaveBeenCalledWith(42);
  });

  it("should mount only the selected scene and dispose it on replacement", () => {
    const first = scene("incident");
    const second = scene("fleet");
    const platform = new WorkspaceVisualizationPlatform({
      host: document.createElement("div"),
      renderer: renderer(),
      scheduler: new DemandRenderScheduler(vi.fn(() => 1), vi.fn()),
      actionGateway: actionGateway(),
    });

    platform.select(first, {revision: 1});
    platform.select(second, {revision: 2});

    expect(first.dispose).toHaveBeenCalledOnce();
    expect(second.mount).toHaveBeenCalledOnce();
  });

  it("should stop scene activity when the document is hidden", () => {
    const active = scene();
    const scheduler = new DemandRenderScheduler(vi.fn(() => 7), vi.fn());
    const platform = new WorkspaceVisualizationPlatform({
      host: document.createElement("div"),
      renderer: renderer(),
      scheduler,
      actionGateway: actionGateway(),
    });
    platform.select(active, {revision: 1});

    platform.setVisibility(false);

    expect(active.setVisibility).toHaveBeenCalledWith(false);
  });

  it("should expose a semantic fallback when renderer creation fails", () => {
    const host = document.createElement("div");
    const platform = WorkspaceVisualizationPlatform.create({
      host,
      createRenderer: () => {
        throw new Error("WebGL context unavailable");
      },
      scheduler: new DemandRenderScheduler(vi.fn(() => 1), vi.fn()),
      actionGateway: actionGateway(),
    });

    platform.select(scene(), {revision: 1});

    expect(host.querySelector('[role="img"]')?.textContent).toContain("Current incident state");
  });

  it("should centrally dispose renderer and active scene resources", () => {
    const active = scene();
    const frameRenderer = renderer();
    const platform = new WorkspaceVisualizationPlatform({
      host: document.createElement("div"),
      renderer: frameRenderer,
      scheduler: new DemandRenderScheduler(vi.fn(() => 1), vi.fn()),
      actionGateway: actionGateway(),
    });
    platform.select(active, {revision: 1});

    platform.dispose();

    expect(active.dispose).toHaveBeenCalledOnce();
    expect(frameRenderer.dispose).toHaveBeenCalledOnce();
  });
});
