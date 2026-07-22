import assert from "node:assert/strict";
import test from "node:test";
import { ApiProblem, WildfireApiClient } from "./index.js";
test("read request carries bearer token and deadline", async () => {
    let captured;
    const fetch = async (input, init) => {
        captured = new Request(input, init);
        return Response.json({ items: [] });
    };
    const client = new WildfireApiClient({ baseUrl: "https://api.test/", accessToken: () => "token", fetch });
    const deadline = new Date("2026-07-22T12:00:00Z");
    assert.deepEqual(await client.list("incidents", { deadline, limit: 10 }), { items: [] });
    assert.equal(captured?.url, "https://api.test/v1/incidents?limit=10");
    assert.equal(captured?.headers.get("authorization"), "Bearer token");
    assert.equal(captured?.headers.get("x-request-deadline"), deadline.toISOString());
});
test("dispatch encodes path and supplies idempotency without claiming execution", async () => {
    let captured;
    const outcome = { commandId: "c1", state: "accepted", authoritative: false, updatedAt: "2026-07-22T12:00:00Z", correlationId: "r1" };
    const fetch = async (input, init) => {
        captured = new Request(input, init);
        return Response.json(outcome, { status: 202 });
    };
    const client = new WildfireApiClient({ baseUrl: "https://api.test", accessToken: async () => "token", fetch });
    const result = await client.dispatchMission("mission/a", { assignmentId: "a1", planDigest: "a".repeat(64), authorityVersion: 2, expectedMissionVersion: 3 }, "idem-000000000001", { deadline: new Date("2026-07-22T12:00:01Z") });
    assert.equal(captured?.url, "https://api.test/v1/missions/mission%2Fa/dispatch");
    assert.equal(captured?.headers.get("idempotency-key"), "idem-000000000001");
    assert.equal(result.authoritative, false);
    assert.equal(result.state, "accepted");
});
test("problem response becomes a typed API error", async () => {
    const problem = { type: "https://api.test/problems/forbidden", title: "Forbidden", status: 403, correlationId: "r2" };
    const client = new WildfireApiClient({ baseUrl: "https://api.test", accessToken: () => "token", fetch: async () => Response.json(problem, { status: 403 }) });
    await assert.rejects(client.commandOutcome("c1", { deadline: new Date("2026-07-22T12:00:00Z") }), (error) => error instanceof ApiProblem && error.problem.status === 403);
});
