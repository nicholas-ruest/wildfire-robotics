# ADR-032: OCI-isolated scientific model execution

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: implementation

## Context

Cell2Fire, WRF-Fire, CFFDRS, Python or R models, and GPU workloads have incompatible runtimes and may not be memory-safe Rust.

## Decision

Run non-Rust scientific components in pinned, signed OCI images behind Rust ports with resource limits, read-only roots, network policy, deterministic inputs, timeouts, output validation, provenance capture, and sandboxing. Promote images through the same evidence pipeline as services.

## Consequences

### Positive
- Allows best scientific tools without contaminating safety-critical processes.

### Negative
- Container startup, data transfer, GPU scheduling, and reproducibility remain operational concerns.

### Neutral
- Foreign runtimes are replaceable adapters, not the domain authority.

## Links
- [ADR index](README.md)
