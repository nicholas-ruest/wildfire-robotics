# ADR-061: Robotics techno-economic and ROI model

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: economics, roi, analytics

## Context

Investment decisions require a reproducible comparison between human-only, robot-assisted, and alternative fleet strategies across acquisition, maintenance, operations, performance, safety, and avoided losses.

## Decision

Implement a versioned scenario-based techno-economic model using actual fleet, maintenance, energy, logistics, staffing-hours, training, communications, compute, downtime, failure, utilization, response-time, productivity, incident, and outcome facts. Compare a credible counterfactual human-capital baseline with robot-assisted alternatives over lifecycle horizons. Report discounted cash flow, NPV, IRR, payback, total cost of ownership, cost per protected hectare/mission/hour, availability, marginal capacity, avoided exposure/injury where methodologically supportable, and outcome/avoided-loss ranges. Separate one-time capital from recurring maintenance/replacement/operations; robots are never represented as a one-time cost. Preserve assumptions, currency/base year, discount rate, uncertainty distributions, sensitivity, confidence, data provenance, and scenario version. Use Monte Carlo and break-even analysis; never present causal savings without an approved identification method.

## Consequences

### Positive
- Makes human-versus-robot investment claims reproducible, updateable, and uncertainty-aware.
### Negative
- Sparse counterfactuals and extreme-fire variance can dominate results.
### Neutral
- ROI is decision support, not an operational command or safety justification.

## Links
- [ADR-015](ADR-015-commercial-multi-tenant-product-boundary.md)
- [ADR-031](ADR-031-model-registry-with-immutable-releases-and-approval-stages.md)
