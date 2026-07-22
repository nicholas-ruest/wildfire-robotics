export class GeneratedAerialClient {
    transport;
    constructor(transport) {
        this.transport = transport;
    }
    view(incident, kind, resourceId, signal) {
        return this.transport.request(`/v1/aerial/incidents/${encodeURIComponent(incident)}/views/${kind}/${encodeURIComponent(resourceId)}`, { method: "GET", ...(signal ? { signal } : {}) });
    }
    command(value, idempotencyKey, signal) {
        return this.transport.request("/v1/aerial/commands", { method: "POST", headers: { "Content-Type": "application/json", "Idempotency-Key": idempotencyKey }, body: JSON.stringify(value), ...(signal ? { signal } : {}) });
    }
}
