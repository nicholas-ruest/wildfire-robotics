# ADR-050: Commercial entitlements, metering, and billing

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: commercial, billing

## Context

A viable product needs explainable packaging, usage measurement, invoicing, credits, and revenue controls without entering the safety path.

## Decision

Version product catalog, contract terms, entitlements, prices, taxes, currency rules, billing periods, and effective dates. Consume immutable, deduplicated operational usage facts into a metering ledger; preserve raw-to-rated lineage and support replay, reconciliation, adjustment, dispute, credit, and invoice finalization. Entitlement is checked before optional work begins, but an active authorized safety operation continues through billing failure or commercial state change and is reconciled afterward. Financial access uses separation of duties and audit.

## Consequences

### Positive
- Supports defensible invoices, product tiers, and measured unit economics.
### Negative
- Tax, currency, contract variation, and disputes require specialist integration.
### Neutral
- The metering ledger is not the operational audit ledger.

## Links
- [ADR-015](ADR-015-commercial-multi-tenant-product-boundary.md)
- [ADR-024](ADR-024-transactional-outbox-and-consumer-inbox.md)
