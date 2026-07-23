import {Object3D, PerspectiveCamera, Scene} from "three";
import type {
  AccessibleSceneDescription,
  ActionReceipt,
  NormalizedPointer,
  SceneSelection,
  SceneServices,
  Viewport,
} from "../../contracts/workspace-scene";
import type {
  ManagedWorkspaceScene,
  WorkspaceAction,
  WorkspaceFixture,
  WorkspaceSceneModule,
  WorkspaceViewState,
} from "../types";

const STATES = ["current", "stale", "degraded", "gap", "unknown"] as const;
const FIXED_TIME = "2026-07-23T12:00:00.000Z";

export function defineWorkspaceScene(
  definition: Omit<WorkspaceSceneModule, "owner" | "fixture" | "create">,
): WorkspaceSceneModule {
  const module: WorkspaceSceneModule = {
    ...definition,
    owner: `visualization/scenes/${definition.id}`,
    fixture: () => ({
      scenarioId: `wr-${definition.id}-demo-v1`,
      seed: stableSeed(definition.signature),
      availableStates: STATES,
      dynamicSignal: {label: definition.signal, value: stableSeed(definition.id) % 97},
      metadata: {
        units: definition.units,
        timestamp: FIXED_TIME,
        uncertainty: "95% bounded interval",
        provenance: `${definition.id}-read-model/v1`,
        limitations: ["Advisory visualization; owning context remains authoritative."],
        simulated: true,
      },
    }),
    create: initialState => new OwnedWorkspaceScene(module, initialState),
  };
  return module;
}

class OwnedWorkspaceScene implements ManagedWorkspaceScene {
  readonly id;
  private readonly threeScene = new Scene();
  private readonly camera = new PerspectiveCamera(45, 1, 0.1, 2_000);
  private services: SceneServices | null = null;
  private state: WorkspaceFixture;
  private viewState: WorkspaceViewState;

  constructor(
    private readonly module: WorkspaceSceneModule,
    initialState: WorkspaceViewState = {camera: "overview", selectionId: null},
  ) {
    this.id = module.id;
    this.state = module.fixture();
    this.viewState = initialState;
    module.entityTypes.forEach((type, index) => {
      const entity = new Object3D();
      entity.name = `${module.id}:${type}:${index + 1}`;
      entity.userData = {stableId: entity.name, type};
      this.threeScene.add(entity);
    });
    this.camera.position.set(0, 8, 12);
  }

  mount(host: HTMLElement, services: SceneServices): void {
    this.services = services;
    host.dataset.workspaceScene = this.id;
    services.invalidate();
  }

  update(state: Readonly<WorkspaceFixture>): void {
    this.state = state;
    if (this.services) this.services.renderer.render(this.threeScene, this.camera);
  }

  resize(viewport: Viewport): void {
    this.camera.aspect = viewport.width / Math.max(viewport.height, 1);
    this.camera.updateProjectionMatrix();
    this.services?.renderer.resize(viewport);
  }

  setVisibility(visible: boolean): void {
    this.threeScene.visible = visible;
  }

  pick(pointer: NormalizedPointer): SceneSelection | null {
    const index = Math.min(
      this.module.entityTypes.length - 1,
      Math.max(0, Math.floor(((pointer.x + 1) / 2) * this.module.entityTypes.length)),
    );
    const type = this.module.entityTypes[index];
    return type ? {id: `${this.id}:${type}:${index + 1}`, type} : null;
  }

  async dispatch(action: WorkspaceAction): Promise<ActionReceipt> {
    if (!this.services) throw new Error(`${this.id} scene must be mounted before dispatch`);
    if (!action.confirmed) {
      return {
        idempotencyKey: action.idempotencyKey,
        stage: "held",
        physicalOutcome: "unknown",
      };
    }
    return this.services.actionGateway.dispatch(this.id, action);
  }

  describe(state: Readonly<WorkspaceFixture>): AccessibleSceneDescription {
    return {
      name: `${this.module.metaphor} — simulated`,
      summary: `${this.module.signal}: ${state.dynamicSignal.value} ${state.metadata.units}; observation ${state.metadata.timestamp}; ${state.metadata.uncertainty}; provenance ${state.metadata.provenance}.`,
      instructions: `Reset to the stable overview, inspect ${this.module.entityTypes.join(", ")}, then ${this.module.managementWorkflow}.`,
    };
  }

  getViewState(): WorkspaceViewState {
    return {...this.viewState};
  }

  setViewState(state: WorkspaceViewState): void {
    this.viewState = {...state};
  }

  resetView(): void {
    this.viewState = {camera: "overview", selectionId: null};
    this.camera.position.set(0, 8, 12);
    this.services?.invalidate();
  }

  dispose(): void {
    this.threeScene.clear();
    this.services = null;
  }
}

function stableSeed(value: string): number {
  return [...value].reduce((seed, character) => (seed * 31 + character.charCodeAt(0)) >>> 0, 17);
}
