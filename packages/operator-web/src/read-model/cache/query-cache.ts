import type {ReadEnvelope} from "../contracts/read-envelope";

export interface QueryKeyParts {
  readonly tenant: string;
  readonly incident: string;
  readonly context: string;
  readonly resource: string;
  readonly filters: Readonly<Record<string, string>>;
  readonly schemaVersion: string;
  readonly authorizationScope: string;
}

export function createQueryKey(parts: QueryKeyParts): string {
  const filters = Object.entries(parts.filters).sort(([a], [b]) => a.localeCompare(b))
    .map(([key, value]) => `${key}=${value}`).join("&");
  return [
    parts.tenant, parts.incident, parts.context, parts.resource, filters,
    parts.schemaVersion, parts.authorizationScope,
  ].join("|");
}

interface CacheMetrics {
  hits: number;
  retries: number;
  invalidations: number;
  coalesced: number;
  outOfOrderRejected: number;
  cancellations: number;
}

export class QueryCache {
  private readonly values = new Map<string, ReadEnvelope<unknown>>();
  private readonly inflight = new Map<string, Promise<ReadEnvelope<unknown>>>();
  private readonly controllers = new Map<string, AbortController>();
  private readonly coalescedValues = new Map<string, unknown>();
  private readonly counters: CacheMetrics = {
    hits: 0, retries: 0, invalidations: 0, coalesced: 0,
    outOfOrderRejected: 0, cancellations: 0,
  };

  constructor(private readonly options: {maxReadRetries: number; jitter: () => number}) {}

  read<T>(
    key: string,
    reader: (signal: AbortSignal) => Promise<ReadEnvelope<T>>,
  ): Promise<ReadEnvelope<T>> {
    const pending = this.inflight.get(key);
    if (pending) {
      this.counters.hits++;
      return pending as Promise<ReadEnvelope<T>>;
    }
    const controller = new AbortController();
    this.controllers.set(key, controller);
    const request = this.runRead(reader, controller.signal)
      .then(envelope => {
        this.accept(key, envelope);
        return envelope;
      })
      .finally(() => {
        this.inflight.delete(key);
        this.controllers.delete(key);
      });
    this.inflight.set(key, request);
    return request;
  }

  async accept<T>(key: string, envelope: ReadEnvelope<T>): Promise<void> {
    const previous = this.values.get(key);
    if (previous && compareEnvelope(envelope, previous) < 0) {
      this.counters.outOfOrderRejected++;
      return;
    }
    this.values.set(key, envelope);
  }

  peek<T>(key: string): ReadEnvelope<T> | undefined {
    return this.values.get(key) as ReadEnvelope<T> | undefined;
  }

  cancel(key: string): void {
    const controller = this.controllers.get(key);
    if (!controller) return;
    controller.abort();
    this.counters.cancellations++;
  }

  invalidate(key: string): void {
    this.cancel(key);
    this.values.delete(key);
    this.coalescedValues.delete(key);
    this.counters.invalidations++;
  }

  coalesce<T>(key: string, value: T): void {
    if (this.coalescedValues.has(key)) this.counters.coalesced++;
    this.coalescedValues.set(key, value);
  }

  metrics(): Readonly<CacheMetrics> {
    return {...this.counters};
  }

  private async runRead<T>(
    reader: (signal: AbortSignal) => Promise<ReadEnvelope<T>>,
    signal: AbortSignal,
  ): Promise<ReadEnvelope<T>> {
    let attempt = 0;
    for (;;) {
      try {
        return await reader(signal);
      } catch (error) {
        if (signal.aborted || attempt >= this.options.maxReadRetries) throw error;
        attempt++;
        this.counters.retries++;
        const delay = Math.max(0, this.options.jitter());
        if (delay) await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
  }
}

function compareEnvelope(next: ReadEnvelope<unknown>, previous: ReadEnvelope<unknown>): number {
  const nextVersion = Number(next.asOfVersion);
  const previousVersion = Number(previous.asOfVersion);
  if (Number.isFinite(nextVersion) && Number.isFinite(previousVersion) && nextVersion !== previousVersion) {
    return nextVersion - previousVersion;
  }
  return Date.parse(next.sourceTime ?? "") - Date.parse(previous.sourceTime ?? "");
}
