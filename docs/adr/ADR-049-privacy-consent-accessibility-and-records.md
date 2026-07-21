# ADR-049: Privacy, consent, accessibility, and records

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: privacy, product, compliance

## Context

Imagery, location, personnel, support, and operational records may contain personal or sensitive information and operator interfaces are safety relevant.

## Decision

Apply privacy-by-design: documented purpose and lawful authority, minimization, collection notice/consent where applicable, access/correction/export/deletion workflows, privacy impact assessment, processor inventory, breach handling, and verifiable retention. Detect and protect people, residences, critical infrastructure, and sensitive locations in imagery and exports. Operator and customer experiences meet the contractually selected current accessibility standard and are tested with assistive technology. Records classification distinguishes operational evidence, public records, commercial records, and personal information.

## Consequences

### Positive
- Enables lawful, inclusive, and contract-ready product operation.
### Negative
- Redaction and rights handling complicate immutable evidence and derived data.
### Neutral
- Legal hold and safety obligations may lawfully limit deletion.

## Links
- [ADR-028](ADR-028-data-classification-retention-and-deletion-policy.md)
- [ADR-036](ADR-036-tenant-isolation-and-dedicated-deployment-tiers.md)
