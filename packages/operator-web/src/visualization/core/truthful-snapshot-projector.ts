export class TruthfulSnapshotProjector<TSnapshot extends object> {
  project(snapshot: Readonly<TSnapshot>, _deltaMs: number): Readonly<TSnapshot> {
    return snapshot;
  }
}
