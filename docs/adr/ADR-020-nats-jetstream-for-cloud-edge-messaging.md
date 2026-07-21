# ADR-020: NATS JetStream for cloud-edge messaging

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: implementation

## Context

The control plane needs low-latency commands, durable events, request/reply, backpressure, and disconnected station synchronization without a heavyweight broker at every edge.

## Decision

Adopt NATS with JetStream for domain events, command transport, and station leaf-node topology. Use replicated streams in production, explicit acknowledgements, bounded retention, subject authorization, and context-owned stream policies. Benchmark before national scale and retain broker abstraction for replacement.

## Consequences

### Positive
- Single operational fabric supports cloud, edge, request/reply, and durable replay.

### Negative
- JetStream operational limits and ordering semantics require careful stream design and testing.

### Neutral
- Exactly-once business effects are achieved by idempotency, never assumed from transport.

## Links
- [ADR index](README.md)
