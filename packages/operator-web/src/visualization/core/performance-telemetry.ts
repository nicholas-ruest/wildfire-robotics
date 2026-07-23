export interface ScenePerformanceMetrics {
  readonly mountMs: number;
  readonly frameMs: number;
  readonly objects: number;
  readonly triangles: number;
  readonly drawCalls: number;
}

export class PerformanceTelemetry {
  constructor(private readonly sink: (metrics: ScenePerformanceMetrics) => void) {}
  record(metrics: ScenePerformanceMetrics): void {
    this.sink({...metrics});
  }
}

export class ResourceTracker {
  private readonly frames = new Set<number>();
  private readonly listeners = new Set<() => void>();
  private readonly resources = new Set<{dispose(): void}>();

  trackFrame(handle: number): void { this.frames.add(handle); }
  trackListener(remove: () => void): void { this.listeners.add(remove); }
  trackResource(resource: {dispose(): void}): void { this.resources.add(resource); }

  disposeCycle(): void {
    this.listeners.forEach(remove => remove());
    this.resources.forEach(resource => resource.dispose());
    this.frames.clear();
    this.listeners.clear();
    this.resources.clear();
  }

  get residual(): {frames: number; listeners: number; resources: number} {
    return {frames: this.frames.size, listeners: this.listeners.size, resources: this.resources.size};
  }
}
