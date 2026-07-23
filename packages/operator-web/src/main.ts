/// <reference types="vite/client" />
import "./live-fire-map.css";
import "./experience-visuals.css";
import "./workspace-experiences.css";
import "./visibility.css";
import "./operational-panels.css";
import "./workspace-maps.css";
import "./static-lightning.css";
import "./mass-swarm.css";
import "./fleet-map.css";
import "./vehicle-map.css";
import "./vegetation-map.css";
import "./styles.css";
import "./workspace-diagrams.css";
import { OperatorApiAdapter } from "./client";
import type { OperatorSnapshot } from "./models";
import { renderOperatorShell } from "./shell";
const root = document.querySelector<HTMLElement>("#app");
if (!root) throw new Error("Missing application root");
const bffAccessToken = (() => {
  let token: string | null = null;
  return async () => {
    if (token) return token;
    const response = await fetch("/auth/operator-token", {
      credentials: "same-origin",
      cache: "no-store",
      headers: { Accept: "application/json" },
    });
    if (!response.ok) throw new Error("Operator authentication is required");
    const body = (await response.json()) as { accessToken?: unknown };
    if (typeof body.accessToken !== "string" || body.accessToken.length < 16)
      throw new Error("Invalid operator authentication response");
    token = body.accessToken;
    return token;
  };
})();
root.innerHTML =
  '<main id="workspace" tabindex="-1"><h1>Wildfire operations</h1><p role="status">Loading authorized operational views…</p></main>';
const demoSnapshot = (): OperatorSnapshot => {
  const now = new Date().toISOString();
  const contexts = [
    "incident",
    "mission",
    "fleet",
    "station",
    "logistics",
    "hazard",
    "safety",
    "recovery",
  ] as const;
  const summaries = [
    "Operational period active · objectives: 4",
    "Recon mission staged · vehicles: 12",
    "Available: 86 · grounded: 3",
    "Edge synchronized · reserve power: 78%",
    "Water reserved: 42,000 L · routes: 6",
    "Fire perimeter current · confidence: 91%",
    "Open occurrences: 1 · constraints current",
    "Cases active: 7 · quarantine: 2",
  ];
  return {
    tenant: "demo-tenant",
    region: "north-sector",
    generatedAt: now,
    models: contexts.map((context, index) => ({
      context,
      title: `${context} operational picture`,
      summary: summaries[index] ?? "No summary available",
      state: index === 4 ? "degraded" : index === 7 ? "stale" : "current",
      uncertainty:
        index === 2
          ? "Battery readiness ±3%"
          : "Within declared operational tolerance",
      provenance: {
        source: "development scenario fixture",
        observedAt: now,
        receivedAt: now,
        lineage: `demo-${context}-v1`,
      },
      ...(index === 4
        ? {
            limitation:
              "One supply route is unavailable; reservations remain authoritative.",
          }
        : {}),
    })),
    commands: [
      {
        id: "CMD-DEMO-001",
        stage: "acknowledged",
        detail: "Recon dispatch acknowledged by vehicle gateway",
        updatedAt: now,
      },
      {
        id: "CMD-DEMO-002",
        stage: "outcome-confirmed",
        detail: "Station safety inspection physically verified",
        updatedAt: now,
      },
    ],
  };
};
if (import.meta.env.VITE_DEMO_MODE === "true")
  renderOperatorShell(root, demoSnapshot());
else {
  const client = new OperatorApiAdapter({
    baseUrl: import.meta.env.VITE_API_BASE_URL ?? location.origin,
    accessToken: bffAccessToken,
    tenant: document.documentElement.dataset.tenant ?? "unknown",
    region: document.documentElement.dataset.region ?? "unknown",
    incident: document.documentElement.dataset.incident ?? "active",
    authorizationScope:
      document.documentElement.dataset.authorizationScope ?? "operator",
  });
  try {
    renderOperatorShell(root, await client.snapshot());
  } catch (error) {
    const message =
      error instanceof Error ? error.message : "Unknown gateway failure";
    root.innerHTML = `<main id="workspace" tabindex="-1"><h1>Operational views unavailable</h1><p role="alert"></p><p>No stale or cross-tenant data was substituted.</p></main>`;
    root.querySelector("[role=alert]")!.textContent = message;
  }
}
