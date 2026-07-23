import type {UiWorkspaceId} from "../navigation/workspaces";

export function renderAppShell(host: HTMLElement, state: {readonly workspace: UiWorkspaceId}): void {
  const levels = [
    ["global-situation", "Global situation"],
    ["workspace-attention", "Requires attention"],
    ["primary-task", `${state.workspace} primary task`],
    ["selection-action", "Selection, evidence, and permitted actions"],
  ] as const;
  host.replaceChildren(...levels.map(([level, label]) => {
    const region = document.createElement(level === "primary-task" ? "main" : "section");
    region.dataset.hierarchy = level;
    region.setAttribute("aria-label", label);
    return region;
  }));
}
