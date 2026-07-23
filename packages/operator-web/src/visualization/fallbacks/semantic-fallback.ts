import type {AccessibleSceneDescription} from "../contracts/workspace-scene";

export function renderSemanticFallback(
  host: HTMLElement,
  description: AccessibleSceneDescription,
): void {
  const fallback = document.createElement("section");
  fallback.className = "workspace-scene-fallback";
  fallback.setAttribute("role", "img");
  fallback.setAttribute("aria-label", description.name);

  const summary = document.createElement("p");
  summary.textContent = description.summary;
  const instructions = document.createElement("p");
  instructions.textContent = description.instructions;
  fallback.replaceChildren(summary, instructions);
  host.replaceChildren(fallback);
}
