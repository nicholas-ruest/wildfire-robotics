import {Scene, PerspectiveCamera} from "three";
import {describe, expect, it, vi} from "vitest";
import type {FrameRenderer} from "../src/visualization";
import {renderOperatorShell} from "../src/shell";
import {contexts, type OperatorSnapshot} from "../src/models";

const snapshot: OperatorSnapshot = {
  tenant: "demo-tenant",
  region: "test",
  generatedAt: "2026-07-23T12:00:00Z",
  models: contexts.map(context => ({
    context, title: context, summary: "summary", state: "current",
    uncertainty: "bounded",
    provenance: {source: "test", observedAt: "2026-07-23T12:00:00Z", receivedAt: "2026-07-23T12:00:00Z", lineage: context},
  })),
  commands: [],
};

function renderer(): FrameRenderer & {element: HTMLCanvasElement} {
  return {
    element: document.createElement("canvas"),
    render: vi.fn((_scene: Scene, _camera: PerspectiveCamera) => undefined),
    resize: vi.fn(),
    dispose: vi.fn(),
  };
}

describe("production visualization runtime", () => {
  it("should mount only the active scene and preserve safe view state across tab switches", () => {
    const root = document.createElement("div");
    const frameRenderer = renderer();
    const shell = renderOperatorShell(root, snapshot, {createRenderer: () => frameRenderer});

    expect(root.querySelector(".visualization-runtime-host canvas")).toBe(frameRenderer.element);
    expect(root.querySelector<HTMLElement>(".visualization-runtime-host")?.dataset.workspaceScene).toBe("incident");
    shell.visualization.setViewState({camera: "inspection", selectionId: "incident:division:1"});

    root.querySelector<HTMLButtonElement>("#tab-fleet")?.click();
    expect(root.querySelector<HTMLElement>(".visualization-runtime-host")?.dataset.workspaceScene).toBe("fleet");
    root.querySelector<HTMLButtonElement>("#tab-incident")?.click();

    expect(shell.visualization.getViewState()).toEqual({
      camera: "inspection", selectionId: "incident:division:1",
    });
    expect(root.querySelectorAll(".visualization-runtime-host canvas")).toHaveLength(1);
    shell.dispose();
    expect(frameRenderer.dispose).toHaveBeenCalledOnce();
  });

  it("should show the semantic fallback when WebGL renderer creation fails", () => {
    const root = document.createElement("div");
    const shell = renderOperatorShell(root, snapshot, {
      createRenderer: () => { throw new Error("WebGL unavailable"); },
    });

    expect(root.querySelector(".visualization-runtime-host [role=img]")?.textContent)
      .toContain("observation 2026-07-23");
    shell.dispose();
  });

  it("should pause the active scene while the document is hidden", () => {
    const root = document.createElement("div");
    const shell = renderOperatorShell(root, snapshot, {createRenderer: renderer});
    const host = root.querySelector<HTMLElement>(".visualization-runtime-host")!;

    shell.visualization.setVisibility(false);
    expect(host.dataset.sceneVisibility).toBe("hidden");
    shell.visualization.setVisibility(true);
    expect(host.dataset.sceneVisibility).toBe("visible");
    shell.dispose();
  });

  it("should clear demo background work when the shell is disposed", () => {
    vi.useFakeTimers();
    const root = document.createElement("div");
    const shell = renderOperatorShell(root, snapshot, {createRenderer: renderer});

    expect(vi.getTimerCount()).toBeGreaterThan(0);
    shell.dispose();

    expect(vi.getTimerCount()).toBe(0);
    vi.useRealTimers();
  });
});
