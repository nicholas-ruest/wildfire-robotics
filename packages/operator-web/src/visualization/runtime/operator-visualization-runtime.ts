import type {FrameRenderer} from "../contracts/workspace-scene";
import {DemandRenderScheduler} from "../core/demand-render-scheduler";
import {WorkspaceVisualizationPlatform} from "../core/workspace-visualization-platform";
import {
  WorkspaceFeatureFlags,
  WorkspaceSceneCatalog,
  WorkspaceViewStateStore,
  workspaceSceneModules,
} from "../scenes/catalog";
import type {ManagedWorkspaceScene, WorkspaceId, WorkspaceViewState} from "../scenes/types";

export interface OperatorVisualizationRuntime {
  readonly currentWorkspace: WorkspaceId;
  getViewState(): WorkspaceViewState;
  setViewState(state: WorkspaceViewState): void;
  setVisibility(visible: boolean): void;
  dispose(): void;
}

export function mountOperatorVisualizationRuntime(
  root: HTMLElement,
  options: {readonly createRenderer?: () => FrameRenderer} = {},
): OperatorVisualizationRuntime {
  const host = document.createElement("section");
  host.className = "visualization-runtime-host";
  host.setAttribute("aria-label", "Active workspace visualization");
  root.querySelector("#workspace")?.prepend(host);

  const states = new WorkspaceViewStateStore();
  const catalog = new WorkspaceSceneCatalog(new WorkspaceFeatureFlags(), states);
  const scheduler = new DemandRenderScheduler();
  const platform = WorkspaceVisualizationPlatform.create({
    host,
    scheduler,
    actionGateway: {
      dispatch: async () => {
        throw new Error("Operational command submission is not configured in the visualization runtime");
      },
    },
    ...(options.createRenderer ? {createRenderer: options.createRenderer} : {}),
  });
  let currentId: WorkspaceId = selectedWorkspace(root);
  let currentScene: ManagedWorkspaceScene | null = null;

  const select = (id: WorkspaceId) => {
    if (currentScene) states.save(currentId, currentScene);
    const resolution = catalog.resolve(id);
    currentId = id;
    if (resolution.mode === "fallback") {
      currentScene = null;
      return;
    }
    currentScene = resolution.scene;
    platform.select(currentScene, workspaceSceneModules.find(module => module.id === id)!.fixture());
  };
  select(currentId);

  const tabListeners = [...root.querySelectorAll<HTMLButtonElement>("[role=tab]")].map(tab => {
    const listener = () => select(tab.id.replace("tab-", "") as WorkspaceId);
    tab.addEventListener("click", listener);
    return () => tab.removeEventListener("click", listener);
  });
  const onVisibility = () => runtime.setVisibility(!document.hidden);
  document.addEventListener("visibilitychange", onVisibility);

  const runtime: OperatorVisualizationRuntime = {
    get currentWorkspace() { return currentId; },
    getViewState: () => currentScene?.getViewState() ?? states.load(currentId) ?? {camera: "overview", selectionId: null},
    setViewState: state => currentScene?.setViewState(state),
    setVisibility: visible => {
      host.dataset.sceneVisibility = visible ? "visible" : "hidden";
      platform.setVisibility(visible);
    },
    dispose: () => {
      if (currentScene) states.save(currentId, currentScene);
      tabListeners.forEach(remove => remove());
      document.removeEventListener("visibilitychange", onVisibility);
      platform.dispose();
      host.remove();
      currentScene = null;
    },
  };
  runtime.setVisibility(!document.hidden);
  return runtime;
}

function selectedWorkspace(root: HTMLElement): WorkspaceId {
  const id = root.querySelector<HTMLElement>('[role=tab][aria-selected="true"]')?.id.replace("tab-", "");
  return (workspaceSceneModules.some(module => module.id === id) ? id : "incident") as WorkspaceId;
}
