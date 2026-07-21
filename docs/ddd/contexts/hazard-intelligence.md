# Hazard Intelligence Context

## Purpose

Normalize authoritative hazard observations into a provenance-aware common hazard picture.

## Model

- **Aggregates:** Source, IngestionRun, ObservationSet, HazardPicture.
- **Core invariant:** Observations are immutable; units/CRS/source/license/event-time are mandatory; quarantined data cannot enter an operational picture.
- **Primary workflow:** Source adapter -> validate -> normalize -> quality score -> persist/outbox -> project picture.

## Tactical model

| Aggregate | Lifecycle | Commands | Events |
|---|---|---|---|
| Source | candidate → active ↔ suspended → retired | RegisterSource, ApproveTerms, ActivateSource, SuspendSource, RotateCredential, RetireSource | SourceRegistered, SourceActivated, SourceSuspended, SourceRetired |
| IngestionRun | created → fetching → validating → accepted/quarantined/failed | StartIngestion, RecordFetch, ValidateBatch, AcceptBatch, QuarantineBatch, FailRun | IngestionStarted, ObservationAccepted, BatchQuarantined, IngestionFailed |
| ObservationSet | open → sealed → superseded | AppendObservation, SealSet, SupersedeObservation | ObservationSetSealed, ObservationSuperseded |
| HazardPicture | building → published → stale/superseded | BuildPicture, PublishPicture, MarkStale, SupersedePicture | HazardPictureUpdated, DataQualityDegraded |
| VisualEvidenceSet | ingesting → indexed → verified → superseded | RegisterMedia, ExtractKeyframes, IndexVisuals, VerifyObservation, SupersedeIndex | VisualEvidenceIndexed, VisualObservationVerified |

Owned values include provider/source identifiers, license policy, spatial/temporal coverage, CRS, quantity/unit, event and ingestion time, quality flags, uncertainty, lineage, content digest, and correction reference. Lightning observations preserve network/source, detection method, polarity/current where licensed, location/time uncertainty, coverage and latency. Drone media preserves pose/footprint, sensor/calibration and quality; RuPixel embeddings/index versions remain rebuildable projections. Source payloads remain immutable evidence; normalized observations never overwrite source claims.

## Invariants

- `HI-INV-001`: An operational observation has source, license, event time, ingestion time, geometry/CRS, quantity/unit, quality, lineage, and digest.
- `HI-INV-002`: Duplicate provider identity plus source version/content digest produces one semantic observation.
- `HI-INV-003`: Quarantined, unlicensed, invalid-unit, invalid-geometry, or out-of-policy data cannot enter an operational picture.
- `HI-INV-004`: Corrections append a superseding observation and preserve the original and affected products.
- `HI-INV-005`: Every picture declares constituent snapshot, valid time, freshness, uncertainty, gaps, and build algorithm version.
- `HI-INV-006`: A visual similarity result cannot become a lightning/fire observation until geospatial-temporal alignment and a declared verification method produce confidence and provenance.

## Ports and read models

Provider adapters are anti-corruption ports with quota, retry, checksum, licensing, and replay behavior. RuPixel is an anti-corruption visual-index port under ADR-057; raw media/object manifests remain authoritative. Repositories exist per aggregate; bulk source and picture artifacts use ADR-019. Read models expose source health, lightning coverage/gaps, reconnaissance priorities, quarantine queue, freshness map, visual similarity candidates, and provenance graph. Direct provider/RuPixel types never cross the context boundary.

## Boundary and failure policy

Consumes and publishes only the contracts in the [integration registry](../integration-contracts.md). Provider outage, late/corrected data, duplicates, coordinate/unit errors, license withdrawal, and material source disagreement retain last-known data with visible staleness/gaps and never silently substitute. License withdrawal stops new use and triggers lineage-based impact handling under policy.

## Implementation acceptance

Domain invariants must be executable and property-tested; API/event contracts require compatibility tests; persistence requires migration/rollback and concurrency tests; adapters require fault-injection and replay tests; operational promotion requires the applicable evidence in the [production readiness standard](../../operations/production-readiness.md).
