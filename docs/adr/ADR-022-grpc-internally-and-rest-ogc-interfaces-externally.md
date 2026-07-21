# ADR-022: gRPC internally and REST OGC interfaces externally

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: implementation

## Context

Internal services need efficient typed RPC while public agencies and geospatial tools require accessible standards.

## Decision

Use gRPC over mutually authenticated HTTP/2 for synchronous internal APIs. Expose versioned REST/JSON and applicable OGC API interfaces through an edge gateway for customers and geospatial interoperability. Never expose internal gRPC services directly to the public internet.

## Consequences

### Positive
- Strong internal contracts plus broad external compatibility.

### Negative
- Dual representations require mapping and contract testing.

### Neutral
- Events remain preferred for facts; RPC is used for queries and bounded commands.

## Links
- [ADR index](README.md)
