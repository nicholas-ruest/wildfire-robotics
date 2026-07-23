import {aerialScene} from "./aerial";
import {commercialScene} from "./commercial";
import {fleetScene} from "./fleet";
import {hazardScene} from "./hazard";
import {identityScene} from "./identity";
import {incidentScene} from "./incident";
import {logisticsScene} from "./logistics";
import {missionScene} from "./mission";
import {predictiveScene} from "./predictive";
import {recoveryScene} from "./recovery";
import {safetyScene} from "./safety";
import {stationScene} from "./station";
import {suppressionScene} from "./suppression";
import type {ManagedWorkspaceScene, WorkspaceId, WorkspaceSceneModule} from "./types";
import {vehicleScene} from "./vehicle";
import {vegetationScene} from "./vegetation";

export const workspaceSceneModules: readonly WorkspaceSceneModule[] = [
  incidentScene, hazardScene, predictiveScene, missionScene, fleetScene,
  vehicleScene, stationScene, logisticsScene, vegetationScene, suppressionScene,
  aerialScene, safetyScene, identityScene, recoveryScene, commercialScene,
];

export class WorkspaceFeatureFlags {
  private readonly flags: Partial<Record<WorkspaceId, boolean>>;

  constructor(flags: Partial<Record<WorkspaceId, boolean>> = {}) {
    this.flags = {...flags};
  }

  isEnabled(id: WorkspaceId): boolean {
    return this.flags[id] ?? true;
  }
}

export class WorkspaceViewStateStore {
  private readonly states = new Map<WorkspaceId, ReturnType<ManagedWorkspaceScene["getViewState"]>>();

  load(id: WorkspaceId): ReturnType<ManagedWorkspaceScene["getViewState"]> | undefined {
    const state = this.states.get(id);
    return state ? {...state} : undefined;
  }

  save(id: WorkspaceId, scene: ManagedWorkspaceScene): void {
    this.states.set(id, scene.getViewState());
  }
}

type Resolution =
  | {readonly mode: "scene"; readonly scene: ManagedWorkspaceScene}
  | {readonly mode: "fallback"; readonly description: ReturnType<ManagedWorkspaceScene["describe"]>};

export class WorkspaceSceneCatalog {
  constructor(
    private readonly flags: WorkspaceFeatureFlags,
    private readonly states: WorkspaceViewStateStore,
  ) {}

  resolve(id: WorkspaceId): Resolution {
    const module = workspaceSceneModules.find(candidate => candidate.id === id);
    if (!module) throw new Error(`Unknown workspace: ${id}`);
    const scene = module.create(this.states.load(id));
    if (this.flags.isEnabled(id)) return {mode: "scene", scene};
    return {mode: "fallback", description: scene.describe(module.fixture())};
  }

  release(id: WorkspaceId, scene: ManagedWorkspaceScene): void {
    this.states.save(id, scene);
    scene.dispose();
  }
}
