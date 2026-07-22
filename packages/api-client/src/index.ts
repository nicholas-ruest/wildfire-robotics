/** Generated-style bindings for wildfire-api-v1.yaml. Do not add domain policy here. */
export type DataState = "fresh" | "stale" | "gap" | "degraded" | "unknown";
export type CommandState =
  | "rejected"
  | "accepted"
  | "queued"
  | "dispatched"
  | "acknowledged"
  | "executed"
  | "verified"
  | "expired"
  | "inhibited"
  | "indeterminate";

export interface Freshness {
  state: DataState;
  observedAt?: string | null;
  validAt?: string | null;
  recordedAt: string;
  expiresAt?: string | null;
  ageMillis?: number;
  gaps?: readonly string[];
}
export interface Provenance {
  producer: string;
  sourceVersion: string;
  digest: string;
  limitations?: readonly string[];
}
export interface Uncertainty {
  description: string;
  confidence?: number | null;
  lower?: number | null;
  upper?: number | null;
  unit?: string | null;
}
export interface ReadModel {
  id: string;
  kind: string;
  freshness: Freshness;
  provenance: Provenance;
  uncertainty: Uncertainty;
  summary: Readonly<Record<string, string | number | boolean | null>>;
}
export interface ReadModelPage {
  items: readonly ReadModel[];
  nextCursor?: string | null;
}
export interface DispatchRequest {
  assignmentId: string;
  planDigest: string;
  authorityVersion: number;
  expectedMissionVersion: number;
}
export interface CommandOutcome {
  commandId: string;
  state: CommandState;
  authoritative: boolean;
  updatedAt: string;
  correlationId: string;
  reason?: string | null;
}
export interface ProblemDetails {
  type: string;
  title: string;
  status: number;
  detail?: string;
  correlationId: string;
}
export interface RequestOptions {
  signal?: AbortSignal;
  deadline: Date;
}
export interface ListOptions extends RequestOptions {
  limit?: number;
  cursor?: string;
}
export interface ClientOptions {
  baseUrl: string;
  accessToken: () => string | Promise<string>;
  fetch?: typeof globalThis.fetch;
}

export class ApiProblem extends Error {
  public constructor(public readonly problem: ProblemDetails) {
    super(`${problem.status} ${problem.title}`);
    this.name = "ApiProblem";
  }
}

const READ_PATHS = {
  incidents: "/v1/incidents",
  missions: "/v1/missions",
  fleetAssets: "/v1/fleet/assets",
  stations: "/v1/stations",
  deliveries: "/v1/logistics/deliveries",
  hazards: "/v1/hazards",
  safetyOccurrences: "/v1/safety/occurrences",
  recoveryCases: "/v1/recovery/cases",
} as const;

export class WildfireApiClient {
  readonly #baseUrl: string;
  readonly #accessToken: ClientOptions["accessToken"];
  readonly #fetch: typeof globalThis.fetch;

  public constructor(options: ClientOptions) {
    this.#baseUrl = options.baseUrl.replace(/\/$/, "");
    this.#accessToken = options.accessToken;
    this.#fetch = options.fetch ?? globalThis.fetch;
  }

  public list(kind: keyof typeof READ_PATHS, options: ListOptions): Promise<ReadModelPage> {
    const query = new URLSearchParams();
    if (options.limit !== undefined) query.set("limit", String(options.limit));
    if (options.cursor !== undefined) query.set("cursor", options.cursor);
    return this.#request<ReadModelPage>(`${READ_PATHS[kind]}?${query}`, {
      deadline: options.deadline,
      signal: options.signal ?? null,
    });
  }

  public dispatchMission(
    missionId: string,
    body: DispatchRequest,
    idempotencyKey: string,
    options: RequestOptions,
  ): Promise<CommandOutcome> {
    return this.#request<CommandOutcome>(
      `/v1/missions/${encodeURIComponent(missionId)}/dispatch`,
      {
        method: "POST",
        deadline: options.deadline,
        signal: options.signal ?? null,
        headers: { "Idempotency-Key": idempotencyKey },
        body: JSON.stringify(body),
      },
    );
  }

  public commandOutcome(commandId: string, options: RequestOptions): Promise<CommandOutcome> {
    return this.#request<CommandOutcome>(`/v1/commands/${encodeURIComponent(commandId)}`, {
      deadline: options.deadline,
      signal: options.signal ?? null,
    });
  }

  async #request<T>(
    path: string,
    init: RequestInit & { deadline: Date },
  ): Promise<T> {
    const token = await this.#accessToken();
    const headers = new Headers(init.headers);
    headers.set("Authorization", `Bearer ${token}`);
    headers.set("Accept", "application/json");
    headers.set("X-Request-Deadline", init.deadline.toISOString());
    if (init.body !== undefined) headers.set("Content-Type", "application/json");
    const response = await this.#fetch(`${this.#baseUrl}${path}`, { ...init, headers });
    const payload: unknown = await response.json();
    if (!response.ok) throw new ApiProblem(payload as ProblemDetails);
    return payload as T;
  }
}
export * from "./aerial.generated.js";
