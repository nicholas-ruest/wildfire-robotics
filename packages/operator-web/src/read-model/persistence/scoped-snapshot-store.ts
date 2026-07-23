interface PersistedSnapshot {
  readonly tenant: string;
  readonly incident: string;
  readonly key: string;
  readonly value: unknown;
  readonly expiresAt: number;
  readonly classification: "approved-read" | "harmless-view" | "secret";
}

export class ScopedSnapshotStore {
  private readonly records = new Map<string, PersistedSnapshot>();
  constructor(private readonly clock: {now(): number} = Date) {}

  save(snapshot: PersistedSnapshot): void {
    if (snapshot.classification === "secret") throw new Error("sensitive browser persistence is prohibited");
    this.records.set(scopedKey(snapshot.tenant, snapshot.incident, snapshot.key), structuredClone(snapshot));
  }

  load<T>(tenant: string, incident: string, key: string): T | null {
    const record = this.records.get(scopedKey(tenant, incident, key));
    if (!record) return null;
    if (record.expiresAt <= this.clock.now()) {
      this.records.delete(scopedKey(tenant, incident, key));
      return null;
    }
    return structuredClone(record.value) as T;
  }

  clearScope(tenant: string, incident: string): void {
    const prefix = `${tenant}|${incident}|`;
    [...this.records.keys()].filter(key => key.startsWith(prefix)).forEach(key => this.records.delete(key));
  }
}

function scopedKey(tenant: string, incident: string, key: string): string {
  return `${tenant}|${incident}|${key}`;
}
