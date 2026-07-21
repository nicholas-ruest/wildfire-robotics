# ADR-052: Regulatory compliance and evidence matrix

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: compliance, governance

## Context

Applicable obligations vary by jurisdiction, aircraft, radio, vehicle, suppressant, environment, worker role, data, contract, and operational design domain.

## Decision

Maintain a jurisdiction- and product-specific compliance matrix mapping obligation, applicability rationale, accountable owner, control, evidence, authority, renewal/expiry, and release/operation gate. Legal and regulatory specialists approve interpretations; software must not encode guessed law. Track aviation permissions, spectrum/radio certification, environmental and suppressant approvals, occupational safety, privacy/residency, accessibility, cybersecurity, procurement, export controls, records, insurance, and incident reporting. Changes trigger impact analysis and may narrow deployment or ODD.

## Consequences

### Positive
- Prevents generic compliance claims and makes approvals traceable.
### Negative
- Regional expansion requires ongoing specialist review and evidence renewal.
### Neutral
- The matrix records applicability; it does not itself grant permission.

## Links
- [ADR-013](ADR-013-portable-cloud-platform-and-canadian-data-residency.md)
- [ADR-045](ADR-045-assurance-case-hazard-analysis-and-independent-verification.md)
