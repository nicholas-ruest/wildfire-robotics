type RequestFrame = (callback: FrameRequestCallback) => number;
type CancelFrame = (handle: number) => void;

export class DemandRenderScheduler {
  private handle: number | null = null;
  private visible = true;

  constructor(
    private readonly requestFrame: RequestFrame = callback => requestAnimationFrame(callback),
    private readonly cancelFrame: CancelFrame = handle => cancelAnimationFrame(handle),
  ) {}

  invalidate(render: FrameRequestCallback): void {
    if (!this.visible || this.handle !== null) return;
    this.handle = this.requestFrame(time => {
      this.handle = null;
      if (this.visible) render(time);
    });
  }

  setVisibility(visible: boolean): void {
    this.visible = visible;
    if (!visible && this.handle !== null) {
      this.cancelFrame(this.handle);
      this.handle = null;
    }
  }

  dispose(): void {
    this.setVisibility(false);
  }
}
