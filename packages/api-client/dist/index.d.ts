/** Generated-style bindings for wildfire-api-v1.yaml. Do not add domain policy here. */
export type DataState = "fresh" | "stale" | "gap" | "degraded" | "unknown";
export type CommandState = "rejected" | "accepted" | "queued" | "dispatched" | "acknowledged" | "executed" | "verified" | "expired" | "inhibited" | "indeterminate";
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
export declare class ApiProblem extends Error {
    readonly problem: ProblemDetails;
    constructor(problem: ProblemDetails);
}
declare const READ_PATHS: {
    readonly incidents: "/v1/incidents";
    readonly missions: "/v1/missions";
    readonly fleetAssets: "/v1/fleet/assets";
    readonly stations: "/v1/stations";
    readonly deliveries: "/v1/logistics/deliveries";
    readonly hazards: "/v1/hazards";
    readonly safetyOccurrences: "/v1/safety/occurrences";
    readonly recoveryCases: "/v1/recovery/cases";
};
export declare class WildfireApiClient {
    #private;
    constructor(options: ClientOptions);
    list(kind: keyof typeof READ_PATHS, options: ListOptions): Promise<ReadModelPage>;
    dispatchMission(missionId: string, body: DispatchRequest, idempotencyKey: string, options: RequestOptions): Promise<CommandOutcome>;
    commandOutcome(commandId: string, options: RequestOptions): Promise<CommandOutcome>;
}
export {};
