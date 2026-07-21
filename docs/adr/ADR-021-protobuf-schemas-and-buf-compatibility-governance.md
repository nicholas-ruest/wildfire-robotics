# ADR-021: Protobuf schemas and Buf compatibility governance

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: implementation

## Context

Rust, TypeScript, ROS gateways, simulators, and external partners need language-neutral contracts with enforceable compatibility.

## Decision

Use Protobuf for internal commands, events, and gRPC APIs. Govern schemas with Buf linting, breaking-change checks, reserved fields, semantic package versions, generated Rust/TypeScript bindings, and golden serialization tests. JSON remains an explicit external representation where needed.

## Consequences

### Positive
- Compact contracts, strong generated types, and automated compatibility checks.

### Negative
- Schema evolution and optionality require discipline; Protobuf is less human-friendly than JSON.

### Neutral
- Domain types do not directly depend on generated transport types.

## Links
- [ADR index](README.md)
