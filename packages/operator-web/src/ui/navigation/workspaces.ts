export const WORKSPACE_GROUPS = [
  {label: "Command", workspaces: [
    ["incident", "Incident Command"], ["hazard", "Hazard Intelligence"],
    ["predictive", "Predictive Planning"], ["mission", "Mission Control"],
  ]},
  {label: "Operations", workspaces: [
    ["fleet", "Fleet Operations"], ["vehicle", "Vehicle Integration"],
    ["station", "Station Operations"], ["logistics", "Logistics"],
    ["vegetation", "Vegetation"], ["suppression", "Suppression"],
    ["aerial", "Aerial Deployment"],
  ]},
  {label: "Assurance", workspaces: [
    ["safety", "Safety Assurance"], ["identity", "Identity & Access"],
    ["recovery", "Robot Care"],
  ]},
  {label: "Business", workspaces: [["commercial", "Commercial Operations"]]},
] as const;

export type UiWorkspaceId = typeof WORKSPACE_GROUPS[number]["workspaces"][number][0];

export function renderWorkspaceNavigation(host: HTMLElement, selected: UiWorkspaceId): void {
  host.setAttribute("aria-label", "Operator workspaces");
  const groups = WORKSPACE_GROUPS.map(group => {
    const section = document.createElement("section");
    section.dataset.navigationGroup = group.label;
    const heading = document.createElement("h2");
    heading.textContent = group.label;
    const list = document.createElement("div");
    group.workspaces.forEach(([id, label]) => {
      const link = document.createElement("a");
      link.href = `/operator?workspace=${id}`;
      link.dataset.workspace = id;
      link.textContent = label;
      if (id === selected) link.setAttribute("aria-current", "page");
      list.append(link);
    });
    section.append(heading, list);
    return section;
  });
  host.replaceChildren(...groups);
}
