# ADR-053: Million-asset hierarchical fleet architecture

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: scale, fleet, distributed-systems

## Context

The target platform must inventory, observe, plan for, and coordinate up to one million heterogeneous firefighting, vegetation-management, logistics, station, and aerial assets without global consensus or per-asset cloud chatter becoming a bottleneck.

## Decision

Use a hierarchical cell architecture: vehicle → local cohort → incident sector → station/region → global control plane. Authority, planning, telemetry, membership, and failure containment are partitioned by tenant, region, incident, sector, geography, capability, and time. Consensus is restricted to small bounded cohorts that require it; no million-member Raft group, global lock, global scheduler, or fleet-wide synchronous transaction is permitted. Global services maintain asynchronous summaries and placement indexes, while incident/edge cells execute current missions during upstream isolation. Stable asset IDs and rendezvous/consistent hashing assign partitions; cells split/merge with epochs and fencing. Capacity is proven at 1M registered, declared concurrently connected and active subsets, traffic mix, burst, failover, and cost—not inferred from component benchmarks.

## Consequences

### Positive
- Enables horizontal scale, locality, graceful degradation, and bounded blast radius.
### Negative
- Cross-cell coordination, repartitioning, summary lag, and hot incidents require explicit engineering.
### Neutral
- One million registered assets does not imply one million simultaneously actuating robots.

## Links
- [ADR-005](ADR-005-event-driven-fleet-control-plane.md)
- [ADR-026](ADR-026-tiered-telemetry-and-bounded-backpressure.md)
