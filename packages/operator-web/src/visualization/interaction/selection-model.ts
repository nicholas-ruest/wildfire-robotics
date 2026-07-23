import type {SceneSelection} from "../contracts/workspace-scene";

export type SelectionSource = "pointer" | "keyboard" | "semantic-html";
type SelectionListener = (selection: SceneSelection | null, source: SelectionSource) => void;

export class SceneSelectionModel {
  private selection: SceneSelection | null = null;
  private readonly listeners = new Set<SelectionListener>();

  get current(): SceneSelection | null {
    return this.selection;
  }

  select(selection: SceneSelection | null, source: SelectionSource): void {
    this.selection = selection;
    this.listeners.forEach(listener => listener(selection, source));
  }

  subscribe(listener: SelectionListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }
}
