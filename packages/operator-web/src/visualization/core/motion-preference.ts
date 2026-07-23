export type MotionOverride = "reduced" | "full" | "system";
export type MotionStorage = Pick<Storage, "getItem" | "setItem">;
const STORAGE_KEY = "operator.motion-preference";

export class MotionPreference {
  private override: MotionOverride;

  constructor(
    private readonly media: MediaQueryList,
    private readonly storage: MotionStorage,
  ) {
    const stored = storage.getItem(STORAGE_KEY);
    this.override = stored === "reduced" || stored === "full" ? stored : "system";
  }

  get reduced(): boolean {
    return this.override === "reduced" || (this.override === "system" && this.media.matches);
  }

  setOverride(value: MotionOverride): void {
    this.override = value;
    this.storage.setItem(STORAGE_KEY, value);
  }
}
