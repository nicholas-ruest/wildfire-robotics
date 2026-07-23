export interface Viewport {
  readonly width: number;
  readonly height: number;
  readonly devicePixelRatio: number;
}

export interface NormalizedPointer {
  readonly x: number;
  readonly y: number;
}

export interface SceneSelection {
  readonly id: string;
  readonly type: string;
}

export type ActionStage =
  | "accepted"
  | "rejected"
  | "acknowledged"
  | "executing"
  | "outcome-confirmed"
  | "outcome-unknown"
  | "held"
  | "revoked"
  | "failed";

export interface ActionReceipt {
  readonly idempotencyKey: string;
  readonly stage: ActionStage;
  readonly receiptId?: string;
  readonly physicalOutcome?: "confirmed" | "unknown";
}

export interface AccessibleSceneDescription {
  readonly name: string;
  readonly summary: string;
  readonly instructions: string;
}

export interface ActionGateway {
  dispatch<TAction>(workspaceId: string, action: TAction): Promise<ActionReceipt>;
}

export interface FrameRenderer {
  render(scene: Scene, camera: Camera): void;
  resize(viewport: Viewport): void;
  dispose(): void;
}

export interface SceneServices {
  readonly renderer: FrameRenderer;
  readonly actionGateway: ActionGateway;
  readonly reducedMotion: boolean;
  invalidate(): void;
}

export interface WorkspaceScene<TState, TAction> {
  readonly id: string;
  mount(host: HTMLElement, services: SceneServices): void;
  update(state: Readonly<TState>, deltaMs: number): void;
  resize(viewport: Viewport): void;
  setVisibility(visible: boolean): void;
  pick(pointer: NormalizedPointer): SceneSelection | null;
  dispatch(action: TAction): Promise<ActionReceipt>;
  describe(state: Readonly<TState>): AccessibleSceneDescription;
  dispose(): void;
}
import type {Camera, Scene} from "three";
