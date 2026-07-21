# ADR-029: Machine-enforced data licensing and provenance

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: implementation

## Context

Government, commercial, research, and community sources have different attribution and operational-use rights.

## Decision

Store license identifier, permitted uses, attribution, redistribution limits, geographic scope, expiry, source checksum, transformation lineage, and quality with every dataset and derivative. Policy blocks prohibited export or use. License changes trigger impact analysis and reprocessing decisions.

## Consequences

### Positive
- Prevents accidental misuse and supports defensible outputs.

### Negative
- Metadata and policy maintenance require legal and data-governance ownership.

### Neutral
- Technically accessible data is not automatically authorized data.

## Links
- [ADR index](README.md)
