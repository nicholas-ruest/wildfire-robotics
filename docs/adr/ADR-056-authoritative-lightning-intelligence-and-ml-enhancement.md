# ADR-056: Authoritative lightning intelligence and ML enhancement

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: lightning, machine-learning, hazard

## Context

Lightning detection and fire-risk prediction must start from authoritative pre-existing observations and established physical/operational products, then improve locally without replacing evidence with opaque guesses.

## Decision

Ingest licensed authoritative lightning detections, weather/NWP, fuels, terrain, drought/fire-weather indices, satellite/hotspot products, historical ignitions, and field reports through provider adapters. Preserve each source claim, uncertainty, latency, coverage, correction, and license. Establish versioned operational baselines before ML. Train calibrated models to estimate holdover ignition probability, location/time uncertainty, expected detection value, spread/impact risk, and reconnaissance priority. Compare candidate ML against baseline using spatial-temporal leakage controls, rare-event metrics, calibration, geography/season/fire-year holdouts, and prospective shadow evaluation. ML produces advisory ranked search areas with uncertainty and abstention; it never fabricates a strike or grants robot/drone authority. Outcomes continuously update datasets, but production models change only through governed retraining and promotion.

## Consequences

### Positive
- Combines trusted intelligence with measurable, continuously improving wildfire-specific prediction.
### Negative
- Labels are delayed/censored, fire years shift, and provider changes create drift and bias.
### Neutral
- Better ranking may improve response without proving every ignition prediction correct.

## Links
- [ADR-007](ADR-007-adopt-authoritative-hazard-data-and-models.md)
- [ADR-031](ADR-031-model-registry-with-immutable-releases-and-approval-stages.md)
