import {WebGLRenderer, type Camera, type Scene} from "three";
import type {FrameRenderer, Viewport} from "../contracts/workspace-scene";

export class ThreeFrameRenderer implements FrameRenderer {
  readonly element: HTMLCanvasElement;
  private readonly renderer: WebGLRenderer;

  constructor(renderer?: WebGLRenderer) {
    if (!renderer && typeof WebGLRenderingContext === "undefined") {
      throw new Error("WebGL is unavailable");
    }
    this.renderer = renderer ?? new WebGLRenderer({antialias: true, alpha: true});
    this.element = this.renderer.domElement;
    this.element.className = "workspace-scene-canvas";
  }

  render(scene: Scene, camera: Camera): void {
    this.renderer.render(scene, camera);
  }

  resize(viewport: Viewport): void {
    this.renderer.setPixelRatio(Math.min(viewport.devicePixelRatio, 2));
    this.renderer.setSize(viewport.width, viewport.height, false);
  }

  dispose(): void {
    this.renderer.dispose();
    this.renderer.forceContextLoss();
    this.element.remove();
  }
}
