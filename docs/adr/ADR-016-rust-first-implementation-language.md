# ADR-016: Rust-First Implementation Language

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: rust, typescript, safety, implementation

## Context

The platform combines safety-sensitive robotics, disconnected edge services, high-throughput geospatial/event processing, simulation, and a browser-based operator experience. Memory safety, predictable performance, strong type modeling, cross-compilation, and constrained-device operation are primary concerns.

## Decision

Rust is the default implementation language for domain models, application services, APIs, ingestion, event processing, edge/station software, robot gateways, simulation, safety monitors, command authorization, and infrastructure tooling. Workspace crates align with bounded contexts and enforce strict Clippy and unsafe-code policies. TypeScript is used only where Rust is not the appropriate delivery technology: browser operator consoles, design-system code, and generated client bindings. Python, R, C/C++, CUDA, and other languages are isolated behind versioned adapters only when required by adopted scientific models, ROS/vendor SDKs, hardware, or validated performance needs. Safety-critical foreign code requires an explicit review and containment boundary.

## Consequences

### Positive
- Memory-safe native performance, explicit error handling, strong domain invariants, and reusable edge/cloud code.

### Negative
- Some scientific and UI ecosystems require inter-language adapters and specialized staffing.

### Neutral
- Language choice does not replace safety evidence or operational validation.

## Links
- [ADR-002](ADR-002-bounded-context-modular-platform.md)
- [ADR-004](ADR-004-ros2-dds-for-robot-internal-middleware.md)
- [ADR-014](ADR-014-open-standards-and-dependency-evaluation.md)
