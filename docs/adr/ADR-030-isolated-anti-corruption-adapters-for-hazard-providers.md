# ADR-030: Isolated anti-corruption adapters for hazard providers

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: implementation

## Context

External hazard APIs change independently, use inconsistent units and schemas, and can return late or corrupt data.

## Decision

Give each provider a Rust adapter implementing a canonical observation port. Adapters perform authentication, rate limiting, schema, unit and CRS validation, provenance, retries with jitter, circuit breaking, quarantine, fixtures, and replay. Provider types never enter the domain model.

## Consequences

### Positive
- Contains external churn and makes feeds independently testable.

### Negative
- Many adapters duplicate operational plumbing and require continuous maintenance.

### Neutral
- Fallback sources are explicit and confidence-weighted, never silent replacements.

## Links
- [ADR index](README.md)
