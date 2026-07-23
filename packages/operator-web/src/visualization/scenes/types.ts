import type {WorkspaceScene} from "../contracts/workspace-scene";

export const WORKSPACE_IDS = [
  "incident", "hazard", "predictive", "mission", "fleet",
  "vehicle", "station", "logistics", "vegetation", "suppression",
  "aerial", "safety", "identity", "recovery", "commercial",
] as const;

export type WorkspaceId = typeof WORKSPACE_IDS[number];
export type WorkspaceDataState = "current" | "stale" | "degraded" | "gap" | "unknown";

export interface WorkspaceFixture {
  readonly scenarioId: string;
  readonly seed: number;
  readonly availableStates: readonly WorkspaceDataState[];
  readonly dynamicSignal: {readonly label: string; readonly value: number};
  readonly metadata: {
    readonly units: string;
    readonly timestamp: string;
    readonly uncertainty: string;
    readonly provenance: string;
    readonly limitations: readonly string[];
    readonly simulated: true;
  };
}

export interface WorkspaceViewState {
  readonly camera: "overview" | "inspection";
  readonly selectionId: string | null;
}

export interface WorkspaceAction {
  readonly kind: "stage-management-action";
  readonly idempotencyKey: string;
  readonly confirmed: boolean;
}

export interface ManagedWorkspaceScene
  extends WorkspaceScene<WorkspaceFixture, WorkspaceAction> {
  getViewState(): WorkspaceViewState;
  setViewState(state: WorkspaceViewState): void;
  resetView(): void;
}

export interface WorkspaceSceneModule {
  readonly id: WorkspaceId;
  readonly owner: `visualization/scenes/${WorkspaceId}`;
  readonly metaphor: string;
  readonly signature: string;
  readonly entityTypes: readonly [string, string, string, ...string[]];
  readonly signal: string;
  readonly units: string;
  readonly managementWorkflow: string;
  fixture(): WorkspaceFixture;
  create(initialState?: WorkspaceViewState): ManagedWorkspaceScene;
}
