export const DEGRADATION_ORDER = [
  "pixel-ratio", "shadows", "particles", "model-detail",
  "update-frequency", "ambient-effects", "semantic-fallback",
] as const;
export type DegradationStep = typeof DEGRADATION_ORDER[number];

export class DegradationController {
  private index = 0;
  degrade(): DegradationStep {
    const step = DEGRADATION_ORDER[Math.min(this.index, DEGRADATION_ORDER.length - 1)]!;
    this.index++;
    return step;
  }
}

export class ContextRecoveryController<TDraft> {
  private rebuilt = false;
  constructor(
    private readonly rebuild: () => boolean,
    private readonly fallback: (draft: TDraft) => void,
  ) {}
  contextLost(draft: TDraft): void {
    if (!this.rebuilt) {
      this.rebuilt = true;
      if (this.rebuild()) return;
    }
    this.fallback(draft);
  }
}
