export type DataState = "current" | "stale" | "degraded" | "gap" | "unknown" | "simulated";
export interface DataEvidence {
  readonly state: DataState;
  readonly freshness: string;
  readonly provenance: string;
  readonly uncertainty: string;
  readonly limitation: string;
}
export type UiNamedState = "loading" | "error" | "offline" | "permission";

export function renderDataSemantics(
  host: HTMLElement,
  input: DataEvidence | {readonly kind: UiNamedState},
): void {
  if ("kind" in input) {
    const region = document.createElement("section");
    region.dataset.uiState = input.kind;
    region.setAttribute("role", "status");
    region.setAttribute("aria-live", input.kind === "error" ? "assertive" : "polite");
    region.textContent = STATE_LABELS[input.kind];
    host.replaceChildren(region);
    return;
  }
  const dl = document.createElement("dl");
  dl.dataset.component = "data-evidence";
  appendFact(dl, "Data state", input.state, "data-state-badge");
  appendFact(dl, "Freshness", input.freshness, "freshness");
  appendFact(dl, "Provenance", input.provenance, "provenance");
  appendFact(dl, "Uncertainty", input.uncertainty, "uncertainty");
  appendFact(dl, "Limitation", input.limitation, "limitation");
  host.replaceChildren(dl);
}

export function renderCommandFeedback(
  host: HTMLElement,
  feedback: {readonly kind: "critical" | "receipt" | "outcome"; readonly message: string},
): void {
  const region = document.createElement("section");
  region.dataset.feedback = feedback.kind === "critical" ? "durable-critical" : feedback.kind;
  region.setAttribute("role", feedback.kind === "critical" ? "alert" : "status");
  region.textContent = feedback.message;
  host.append(region);
}

export const COMPONENT_CONTRACTS = [
  "AppShell", "WorkspaceNavigation", "SituationHeader", "AttentionQueue",
  "DataStateBadge", "Freshness", "Provenance", "Uncertainty", "Limitation",
  "Metric", "EntityTable", "FilterBar", "Timeline", "Legend", "DetailsPanel",
  "EmptyState", "LoadingState", "ErrorState", "OfflineState", "PermissionState",
  "ActionForm", "Confirmation", "ApprovalState", "CommandReceipt", "OutcomeState",
] as const;

const STATE_LABELS: Record<UiNamedState, string> = {
  loading: "Loading authorized data",
  error: "Data could not be loaded",
  offline: "Offline — last-known information remains inspectable",
  permission: "Permission is required for this information",
};

function appendFact(host: HTMLElement, label: string, value: string, component: string): void {
  const wrapper = document.createElement("div");
  const term = document.createElement("dt");
  term.textContent = label;
  const detail = document.createElement("dd");
  detail.dataset.component = component;
  detail.textContent = value;
  wrapper.append(term, detail);
  host.append(wrapper);
}
