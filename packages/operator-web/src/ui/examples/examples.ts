import {renderDataSemantics} from "../components/semantic-components";
import {renderWorkspaceNavigation} from "../navigation/workspaces";

export function renderUiExamples(host: HTMLElement): void {
  const navigation = document.createElement("nav");
  renderWorkspaceNavigation(navigation, "incident");
  const evidence = document.createElement("section");
  renderDataSemantics(evidence, {
    state: "stale", freshness: "12 min", provenance: "example/v1",
    uncertainty: "±5%", limitation: "Example data only",
  });
  host.replaceChildren(navigation, evidence);
}
