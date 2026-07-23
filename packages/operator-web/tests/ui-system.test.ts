import {describe, expect, it, vi} from "vitest";
import {
  OPERATOR_TOKENS,
  OperatorRouteCodec,
  SafeRouter,
  WORKSPACE_GROUPS,
  renderAppShell,
  renderCommandFeedback,
  renderDataSemantics,
  renderWorkspaceNavigation,
} from "../src/ui";

describe("ADR-078 task-oriented UI system", () => {
  it("should expose fifteen text-labeled destinations in four stable groups", () => {
    const host = document.createElement("nav");
    renderWorkspaceNavigation(host, "fleet");

    expect(WORKSPACE_GROUPS.map(group => group.label)).toEqual([
      "Command", "Operations", "Assurance", "Business",
    ]);
    expect(host.querySelectorAll("[data-workspace]")).toHaveLength(15);
    expect([...host.querySelectorAll("[data-workspace]")].every(node => Boolean(node.textContent?.trim()))).toBe(true);
  });

  it("should render global situation before attention, task, and selection/action context", () => {
    const host = document.createElement("div");
    renderAppShell(host, {workspace: "incident"});

    expect([...host.querySelectorAll("[data-hierarchy]")].map(node => node.getAttribute("data-hierarchy")))
      .toEqual(["global-situation", "workspace-attention", "primary-task", "selection-action"]);
  });

  it("should render semantic data and durable command states with full evidence", () => {
    const host = document.createElement("div");
    renderDataSemantics(host, {
      state: "stale", freshness: "18 min", provenance: "hazard/v4",
      uncertainty: "±4 ha", limitation: "Smoke obscures sector 7",
    });
    renderCommandFeedback(host, {kind: "critical", message: "Physical outcome unknown"});

    expect(host.textContent).toContain("stale");
    expect(host.textContent).toContain("18 min");
    expect(host.textContent).toContain("hazard/v4");
    expect(host.textContent).toContain("±4 ha");
    expect(host.textContent).toContain("Smoke obscures sector 7");
    expect(host.querySelector("[data-feedback='durable-critical']")).not.toBeNull();
  });

  it("should render loading, error, offline, and permission states as named regions", () => {
    const host = document.createElement("div");
    for (const kind of ["loading", "error", "offline", "permission"] as const) {
      renderDataSemantics(host, {kind});
      expect(host.querySelector(`[data-ui-state="${kind}"]`)?.getAttribute("role")).toBe("status");
    }
  });

  it("should round-trip safe route state while excluding secrets and command state", () => {
    const codec = new OperatorRouteCodec();
    const url = codec.serialize({
      workspace: "fleet", view: "capacity", filters: ["eligible", "cell:alpha"],
      timeRange: "PT4H", selection: "fleet:alpha",
      token: "secret", confirmation: true, draft: {flow: 4},
    });
    const restored = codec.parse(url);

    expect(restored).toEqual({
      workspace: "fleet", view: "capacity", filters: ["eligible", "cell:alpha"],
      timeRange: "PT4H", selection: "fleet:alpha",
    });
    expect(url).not.toMatch(/secret|token|confirmation|draft/);
  });

  it("should restore route state on browser popstate", () => {
    const codec = new OperatorRouteCodec();
    const listener = vi.fn();
    const router = new SafeRouter(window, codec);
    router.subscribe(listener);
    history.replaceState({}, "", codec.serialize({workspace: "safety", view: "occurrences"}));

    window.dispatchEvent(new PopStateEvent("popstate"));

    expect(listener).toHaveBeenCalledWith(expect.objectContaining({workspace: "safety"}));
    router.dispose();
  });

  it("should version tokens and separate operational semantic families", () => {
    expect(OPERATOR_TOKENS.version).toBe("1.0.0");
    expect(Object.keys(OPERATOR_TOKENS.semantic)).toEqual([
      "dataQuality", "health", "severity", "authorization", "progress",
    ]);
    expect(OPERATOR_TOKENS.density).toEqual(expect.objectContaining({
      comfortable: expect.any(Object), compact: expect.any(Object),
    }));
    expect(OPERATOR_TOKENS.themes).toHaveProperty("highContrast");
    expect(OPERATOR_TOKENS.capabilities).toEqual(["wide", "bounded", "narrow"]);
  });
});
