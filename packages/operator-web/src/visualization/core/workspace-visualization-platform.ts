import type {
  ActionGateway,
  FrameRenderer,
  SceneServices,
  WorkspaceScene,
} from "../contracts/workspace-scene";
import {renderSemanticFallback} from "../fallbacks/semantic-fallback";
import {DemandRenderScheduler} from "./demand-render-scheduler";
import {ThreeFrameRenderer} from "./three-renderer";

interface PlatformOptions {
  readonly host: HTMLElement;
  readonly renderer: FrameRenderer;
  readonly scheduler: DemandRenderScheduler;
  readonly actionGateway: ActionGateway;
  readonly reducedMotion?: boolean;
}

interface CreateOptions extends Omit<PlatformOptions, "renderer"> {
  readonly createRenderer?: () => FrameRenderer;
}

interface ActiveScene {
  readonly dispose: () => void;
  readonly describe: (state: unknown) => ReturnType<WorkspaceScene<never, never>["describe"]>;
  readonly setVisibility: (visible: boolean) => void;
  readonly update: (state: unknown, deltaMs: number) => void;
  state: unknown;
}

export class WorkspaceVisualizationPlatform {
  private active: ActiveScene | null = null;
  private fallback = false;
  private visible = true;
  private lastFrameTime: number | null = null;

  constructor(private readonly options: PlatformOptions) {}

  static create(options: CreateOptions): WorkspaceVisualizationPlatform {
    try {
      const renderer = (options.createRenderer ?? (() => new ThreeFrameRenderer()))();
      return new WorkspaceVisualizationPlatform({...options, renderer});
    } catch {
      const renderer: FrameRenderer = {render() {}, resize() {}, dispose() {}};
      const platform = new WorkspaceVisualizationPlatform({...options, renderer});
      platform.fallback = true;
      return platform;
    }
  }

  select<TState, TAction>(scene: WorkspaceScene<TState, TAction>, state: Readonly<TState>): void {
    this.releaseActive();
    this.active = {
      dispose: () => scene.dispose(),
      describe: next => scene.describe(next as Readonly<TState>),
      setVisibility: visible => scene.setVisibility(visible),
      update: (next, deltaMs) => scene.update(next as Readonly<TState>, deltaMs),
      state,
    };
    if (this.fallback) {
      renderSemanticFallback(this.options.host, scene.describe(state));
      return;
    }
    const services: SceneServices = {
      renderer: this.options.renderer,
      actionGateway: this.options.actionGateway,
      reducedMotion: this.options.reducedMotion ?? false,
      invalidate: () => this.invalidate(),
    };
    scene.mount(this.options.host, services);
    if (this.options.renderer.element && !this.options.renderer.element.isConnected) {
      this.options.host.prepend(this.options.renderer.element);
    }
    scene.setVisibility(this.visible);
    this.invalidate();
  }

  update<TState>(state: Readonly<TState>): void {
    if (!this.active) return;
    this.active.state = state;
    if (this.fallback) {
      renderSemanticFallback(this.options.host, this.active.describe(state));
    } else {
      this.invalidate();
    }
  }

  setVisibility(visible: boolean): void {
    this.visible = visible;
    this.options.scheduler.setVisibility(visible);
    this.active?.setVisibility(visible);
    if (visible) this.invalidate();
  }

  dispose(): void {
    this.releaseActive();
    this.options.scheduler.dispose();
    this.options.renderer.dispose();
    this.options.host.replaceChildren();
  }

  private invalidate(): void {
    this.options.scheduler.invalidate(time => {
      const active = this.active;
      if (!active) return;
      const deltaMs = this.lastFrameTime === null ? 0 : Math.max(0, time - this.lastFrameTime);
      this.lastFrameTime = time;
      active.update(active.state, deltaMs);
    });
  }

  private releaseActive(): void {
    this.active?.dispose();
    this.active = null;
    this.lastFrameTime = null;
  }
}
