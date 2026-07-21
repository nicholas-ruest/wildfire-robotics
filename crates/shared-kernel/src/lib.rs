#![forbid(unsafe_code)]
//! Stable technical value objects shared across bounded contexts.
//!
//! This crate intentionally contains no business aggregates, framework types,
//! persistence concerns, network clients, clocks, randomness sources, or vendor
//! APIs. Boundary adapters map transport representations into these values.

pub mod error;
pub mod geo;
pub mod identity;
pub mod metadata;
pub mod numeric;
pub mod time;

pub use error::{ErrorCategory, ErrorCode, InvalidErrorCode, RetryClassification};
pub use geo::{
    Altitude, AltitudeReference, CoordinateReferenceSystem, GeoPoint, GeoPolygon, GeometryError,
};
pub use identity::{
    AggregateVersion, CausationId, CorrelationId, EntityId, FencingToken, Identifier,
    IdentifierKind, IncidentId, IncidentScope, MessageTrace, MissionId, PrincipalId, TenantId,
    TenantScope, VehicleId,
};
pub use metadata::{
    ArtifactReference, ContentDigest, DataClassification, EvidenceReference, MetadataError,
    SemanticVersion,
};
pub use numeric::{
    Confidence, Dimension, Finite, Freshness, NumericError, Probability, Quantity, Unit,
};
pub use time::{
    ClockEpoch, ClockQuality, ClockSource, MonotonicDeadline, MonotonicInstant, TimeError,
    TimeWindow, UtcInstant,
};
