export class ReconnectReconciler<TSnapshot> {
  constructor(
    private readonly changesSince: (cursor: string) => Promise<{cursor: string; snapshot: TSnapshot}>,
    private readonly fetchSnapshot: () => Promise<{cursor: string; snapshot: TSnapshot}>,
  ) {}

  async reconnect(input: {readonly cursor: string | null; readonly continuityProven: boolean}) {
    if (input.cursor && input.continuityProven) {
      return {mode: "delta" as const, animateMissed: false, ...await this.changesSince(input.cursor)};
    }
    return {mode: "snapshot" as const, animateMissed: false, ...await this.fetchSnapshot()};
  }
}

export async function settlePartialReads<T extends Record<string, Promise<unknown>>>(
  reads: T,
): Promise<{[K in keyof T]: PromiseSettledResult<Awaited<T[K]>>}> {
  const entries = Object.entries(reads);
  const settled = await Promise.allSettled(entries.map(([, promise]) => promise));
  return Object.fromEntries(entries.map(([key], index) => [key, settled[index]])) as {
    [K in keyof T]: PromiseSettledResult<Awaited<T[K]>>
  };
}
