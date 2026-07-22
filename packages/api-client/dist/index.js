export class ApiProblem extends Error {
    problem;
    constructor(problem) {
        super(`${problem.status} ${problem.title}`);
        this.problem = problem;
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
};
export class WildfireApiClient {
    #baseUrl;
    #accessToken;
    #fetch;
    constructor(options) {
        this.#baseUrl = options.baseUrl.replace(/\/$/, "");
        this.#accessToken = options.accessToken;
        this.#fetch = options.fetch ?? globalThis.fetch;
    }
    list(kind, options) {
        const query = new URLSearchParams();
        if (options.limit !== undefined)
            query.set("limit", String(options.limit));
        if (options.cursor !== undefined)
            query.set("cursor", options.cursor);
        return this.#request(`${READ_PATHS[kind]}?${query}`, {
            deadline: options.deadline,
            signal: options.signal ?? null,
        });
    }
    dispatchMission(missionId, body, idempotencyKey, options) {
        return this.#request(`/v1/missions/${encodeURIComponent(missionId)}/dispatch`, {
            method: "POST",
            deadline: options.deadline,
            signal: options.signal ?? null,
            headers: { "Idempotency-Key": idempotencyKey },
            body: JSON.stringify(body),
        });
    }
    commandOutcome(commandId, options) {
        return this.#request(`/v1/commands/${encodeURIComponent(commandId)}`, {
            deadline: options.deadline,
            signal: options.signal ?? null,
        });
    }
    async #request(path, init) {
        const token = await this.#accessToken();
        const headers = new Headers(init.headers);
        headers.set("Authorization", `Bearer ${token}`);
        headers.set("Accept", "application/json");
        headers.set("X-Request-Deadline", init.deadline.toISOString());
        if (init.body !== undefined)
            headers.set("Content-Type", "application/json");
        const response = await this.#fetch(`${this.#baseUrl}${path}`, { ...init, headers });
        const payload = await response.json();
        if (!response.ok)
            throw new ApiProblem(payload);
        return payload;
    }
}
