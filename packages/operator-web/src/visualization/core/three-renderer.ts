import {WebGLRenderer, type Camera, type Scene} from "three";
import type {FrameRenderer, Viewport} from "../contracts/workspace-scene";

export class ThreeFrameRenderer implements FrameRenderer {
  readonly canvas: HTMLCanvasElement;

  constructor(private readonly renderer = new WebGLRenderer({antialias: true, alpha: true})) {
    this.canvas = renderer.domElement;
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
    this.canvas.remove();
  }
}
