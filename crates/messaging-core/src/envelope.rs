//! Canonical validated event envelope for broker and persistence adapters.

use crate::subject::Subject;
use chrono::{DateTime, Utc};
use operations_core::telemetry::PropagationContext;
use sha2::{Digest, Sha256};
use shared_kernel::{ContentDigest, DataClassification, SemanticVersion};
use thiserror::Error;
use uuid::Uuid;

/// Maximum embedded payload; larger artifacts must use immutable object references.
pub const MAX_PAYLOAD_BYTES: usize = 1_048_576;

/// Required safety assurance references for safety-relevant events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SafetyReferences {
    /// Authority grant or decision identifier.
    pub authority_id: String,
    /// Approved operational design domain identifier.
    pub odd_id: String,
    /// Effective constraint-set identifier.
    pub constraint_id: String,
    /// Evidence artifact identifier.
    pub evidence_id: String,
    /// Clock-quality evidence identifier.
    pub clock_quality_id: String,
}

impl SafetyReferences {
    fn validate(&self) -> Result<(), EnvelopeError> {
        for value in [
            &self.authority_id,
            &self.odd_id,
            &self.constraint_id,
            &self.evidence_id,
            &self.clock_quality_id,
        ] {
            validate_text(value, 256)?;
        }
        Ok(())
    }
}

/// Construction input kept separate so no partially validated envelope exists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnvelopeInput {
    /// Stable event ID reused across transport retries.
    pub message_id: Uuid,
    /// Published contract name.
    pub event_type: String,
    /// Contract schema version.
    pub schema_version: SemanticVersion,
    /// Source occurrence time.
    pub occurred_at: DateTime<Utc>,
    /// Durable recording time.
    pub recorded_at: DateTime<Utc>,
    /// Owning producer/context name.
    pub producer: String,
    /// Exact producer release version.
    pub producer_version: SemanticVersion,
    /// Aggregate type.
    pub aggregate_type: String,
    /// Aggregate identity.
    pub aggregate_id: Uuid,
    /// Positive per-aggregate version.
    pub aggregate_version: u64,
    /// Tenant boundary copied from authenticated context.
    pub tenant_id: String,
    /// Optional region boundary.
    pub region_id: Option<String>,
    /// Optional incident scope.
    pub incident_id: Option<String>,
    /// Validated propagation context.
    pub propagation: PropagationContext,
    /// Data classification.
    pub classification: DataClassification,
    /// Canonical declared transport subject.
    pub subject: Subject,
    /// Payload content type.
    pub content_type: String,
    /// Encoded payload or immutable object reference.
    pub payload: Vec<u8>,
    /// Required only for safety-relevant facts.
    pub safety: Option<SafetyReferences>,
}

/// Fully validated immutable message envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageEnvelope {
    input: EnvelopeInput,
    payload_digest: ContentDigest,
}

impl MessageEnvelope {
    /// Validates and content-addresses an event before durable publication.
    pub fn new(input: EnvelopeInput, safety_relevant: bool) -> Result<Self, EnvelopeError> {
        for value in [&input.event_type, &input.producer, &input.aggregate_type] {
            validate_text(value, 128)?;
        }
        if input.aggregate_version == 0
            || input.payload.is_empty()
            || input.payload.len() > MAX_PAYLOAD_BYTES
            || input.content_type != "application/protobuf"
        {
            return Err(EnvelopeError::InvalidEnvelope);
        }
        input
            .propagation
            .validate()
            .map_err(|_| EnvelopeError::InvalidTrace)?;
        if input.tenant_id != input.subject.tenant()
            || input.producer != input.subject.context()
            || input.event_type != input.subject.event()
            || input.schema_version.major() != u64::from(input.subject.major())
            || input.propagation.tenant_id != input.tenant_id
            || input
                .region_id
                .as_deref()
                .is_some_and(|region| region != input.subject.region())
        {
            return Err(EnvelopeError::ScopeOrSubjectMismatch);
        }
        if safety_relevant {
            input
                .safety
                .as_ref()
                .ok_or(EnvelopeError::MissingSafetyReferences)?
                .validate()?;
        } else if input.safety.is_some() {
            return Err(EnvelopeError::UnexpectedSafetyReferences);
        }
        if input
            .incident_id
            .as_ref()
            .is_some_and(|value| validate_text(value, 256).is_err())
        {
            return Err(EnvelopeError::InvalidEnvelope);
        }
        let payload_digest =
            ContentDigest::from_sha256_bytes(Sha256::digest(&input.payload).into());
        Ok(Self {
            input,
            payload_digest,
        })
    }

    /// Exact validated metadata and payload.
    #[must_use]
    pub const fn input(&self) -> &EnvelopeInput {
        &self.input
    }
    /// Digest of exact payload bytes.
    #[must_use]
    pub const fn payload_digest(&self) -> ContentDigest {
        self.payload_digest
    }
    /// Rejects transport/header subject substitution.
    pub fn verify_transport_subject(&self, actual: &str) -> Result<(), EnvelopeError> {
        if self.input.subject.as_str() == actual {
            Ok(())
        } else {
            Err(EnvelopeError::ScopeOrSubjectMismatch)
        }
    }
    /// Rejects payload mutation at any persistence or transport boundary.
    pub fn verify_payload(&self, actual: &[u8]) -> Result<(), EnvelopeError> {
        let digest = ContentDigest::from_sha256_bytes(Sha256::digest(actual).into());
        if digest == self.payload_digest {
            Ok(())
        } else {
            Err(EnvelopeError::DigestMismatch)
        }
    }
}

fn validate_text(value: &str, maximum: usize) -> Result<(), EnvelopeError> {
    if value.trim().is_empty() || value.len() > maximum || !value.is_ascii() {
        Err(EnvelopeError::InvalidEnvelope)
    } else {
        Ok(())
    }
}

/// Stable envelope rejection reasons.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum EnvelopeError {
    /// Required metadata or payload limits are invalid.
    #[error("message envelope is invalid")]
    InvalidEnvelope,
    /// Tenant, region, producer, event, schema, or transport subject disagrees.
    #[error("message scope or subject does not match")]
    ScopeOrSubjectMismatch,
    /// Correlation/trace propagation is malformed.
    #[error("message trace context is invalid")]
    InvalidTrace,
    /// A safety fact omitted required authority/ODD/constraint/evidence/time references.
    #[error("safety-relevant message is missing assurance references")]
    MissingSafetyReferences,
    /// A non-safety event attempted to smuggle safety authority metadata.
    #[error("non-safety message contains unexpected safety references")]
    UnexpectedSafetyReferences,
    /// Payload bytes differ from the content-addressed envelope.
    #[error("message payload digest mismatch")]
    DigestMismatch,
}
