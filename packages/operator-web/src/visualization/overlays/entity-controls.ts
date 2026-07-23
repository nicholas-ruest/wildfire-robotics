import type {SceneSelection} from "../contracts/workspace-scene";
import {SceneSelectionModel} from "../interaction/selection-model";

export function renderEntityControls(
  host: HTMLElement,
  accessibleName: string,
  entities: readonly SceneSelection[],
  selection: SceneSelectionModel,
): void {
  host.setAttribute("role", "group");
  host.setAttribute("aria-label", accessibleName);
  const controls = entities.map(entity => {
    const button = document.createElement("button");
    button.type = "button";
    button.dataset.entityId = entity.id;
    button.textContent = entity.type;
    button.setAttribute("aria-pressed", String(selection.current?.id === entity.id));
    button.addEventListener("click", () => selection.select(entity, "semantic-html"));
    return button;
  });
  selection.subscribe(current => {
    controls.forEach(button => {
      button.setAttribute("aria-pressed", String(button.dataset.entityId === current?.id));
    });
  });
  host.replaceChildren(...controls);
}
