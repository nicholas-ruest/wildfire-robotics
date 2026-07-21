# ADR-057: RuPixel visual evidence and outcome retrieval

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: imagery, visual-search, machine-learning

## Context

Drone imagery and video must be searchable, comparable with predicted conditions, and reusable for model evaluation without treating semantic similarity as physical truth.

## Decision

Adopt `ruvnet/rupixel` behind a Hazard Intelligence visual-index port for keyframe gating, visual/text embeddings, and approximate-nearest-neighbor retrieval over authorized imagery. Raw media and calibrated metadata remain immutable truth artifacts in object storage; RuPixel indexes are rebuildable read models. Every frame/keyframe carries capture time, pose/footprint, altitude, sensor/calibration, vehicle, mission, weather, classification, license, digest, model/version, and quality. Wildfire-specific detection, georegistration, change analysis, and prediction/outcome scoring are separate validated pipelines. Retrieval results are candidates with similarity and provenance, never observations without verification. Pin and benchmark the dependency and preserve a portable embedding/index contract.

## Consequences

### Positive
- Enables semantic visual discovery and efficient feedback-dataset construction.
### Negative
- Domain shift, false similarity, indexing cost, privacy, and approximate search require controls.
### Neutral
- RuPixel accelerates retrieval; it does not itself validate lightning or fire predictions.

## Links
- [ADR-019](ADR-019-immutable-object-storage-for-bulk-artifacts.md)
- [ADR-056](ADR-056-authoritative-lightning-intelligence-and-ml-enhancement.md)
- [RuPixel](https://github.com/ruvnet/rupixel)
