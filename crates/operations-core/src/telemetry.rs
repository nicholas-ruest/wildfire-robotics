//! OpenTelemetry-compatible propagation and bounded best-effort telemetry.

use shared_kernel::DataClassification;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
};
use thiserror::Error;

const REDACTED: &str = "[REDACTED]";

/// W3C trace context plus stable platform correlation fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropagationContext {
    /// W3C `traceparent` header.
    pub traceparent: String,
    /// Optional W3C `tracestate` header.
    pub tracestate: Option<String>,
    /// Tenant identifier.
    pub tenant_id: String,
    /// Optional incident identifier.
    pub incident_id: Option<String>,
    /// Correlation identifier.
    pub correlation_id: String,
    /// Optional causation identifier.
    pub causation_id: Option<String>,
    /// Exact release identifier.
    pub release_id: String,
}

impl PropagationContext {
    /// Validates W3C-compatible bounded propagation fields.
    pub fn validate(&self) -> Result<(), TelemetryError> {
        let parts: Vec<_> = self.traceparent.split('-').collect();
        let valid_traceparent = parts.len() == 4
            && parts[0].len() == 2
            && parts[1].len() == 32
            && parts[2].len() == 16
            && parts[3].len() == 2
            && parts[0] != "ff"
            && parts[1] != "00000000000000000000000000000000"
            && parts[2] != "0000000000000000"
            && parts.iter().all(|part| {
                part.bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            });
        if !valid_traceparent
            || self
                .tracestate
                .as_ref()
                .is_some_and(|value| value.len() > 512)
            || !bounded_identifier(&self.tenant_id)
            || !bounded_identifier(&self.correlation_id)
            || !bounded_identifier(&self.release_id)
            || self
                .incident_id
                .as_ref()
                .is_some_and(|value| !bounded_identifier(value))
            || self
                .causation_id
                .as_ref()
                .is_some_and(|value| !bounded_identifier(value))
        {
            return Err(TelemetryError::InvalidPropagationContext);
        }
        Ok(())
    }

    /// Injects W3C headers and bounded platform baggage into a text carrier.
    pub fn inject(&self, carrier: &mut BTreeMap<String, String>) -> Result<(), TelemetryError> {
        self.validate()?;
        carrier.insert("traceparent".into(), self.traceparent.clone());
        if let Some(tracestate) = &self.tracestate {
            carrier.insert("tracestate".into(), tracestate.clone());
        }
        carrier.insert("x-wildfire-tenant-id".into(), self.tenant_id.clone());
        carrier.insert(
            "x-wildfire-correlation-id".into(),
            self.correlation_id.clone(),
        );
        carrier.insert("x-wildfire-release-id".into(), self.release_id.clone());
        if let Some(incident_id) = &self.incident_id {
            carrier.insert("x-wildfire-incident-id".into(), incident_id.clone());
        }
        if let Some(causation_id) = &self.causation_id {
            carrier.insert("x-wildfire-causation-id".into(), causation_id.clone());
        }
        Ok(())
    }

    /// Extracts and validates W3C headers and platform baggage from a text carrier.
    pub fn extract(carrier: &BTreeMap<String, String>) -> Result<Self, TelemetryError> {
        let required = |key: &str| {
            carrier
                .get(key)
                .cloned()
                .ok_or(TelemetryError::InvalidPropagationContext)
        };
        let context = Self {
            traceparent: required("traceparent")?,
            tracestate: carrier.get("tracestate").cloned(),
            tenant_id: required("x-wildfire-tenant-id")?,
            incident_id: carrier.get("x-wildfire-incident-id").cloned(),
            correlation_id: required("x-wildfire-correlation-id")?,
            causation_id: carrier.get("x-wildfire-causation-id").cloned(),
            release_id: required("x-wildfire-release-id")?,
        };
        context.validate()?;
        Ok(context)
    }
}

fn bounded_identifier(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 256
}

/// Structured telemetry record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelemetryRecord {
    /// Stable event/metric name.
    pub name: String,
    /// Data classification applied before export.
    pub classification: DataClassification,
    /// Propagated context.
    pub context: PropagationContext,
    /// Bounded structured attributes.
    pub attributes: BTreeMap<String, String>,
}

/// Redaction policy allowing known attributes and masking sensitive values.
#[derive(Clone, Debug)]
pub struct RedactionPolicy {
    /// Exportable attribute keys.
    pub allowed_keys: BTreeSet<String>,
    /// Allowed keys whose values must be masked.
    pub sensitive_keys: BTreeSet<String>,
    /// Maximum UTF-8 byte length per value.
    pub maximum_value_bytes: usize,
    /// Maximum number of attributes retained per record.
    pub maximum_attribute_count: usize,
    /// Maximum UTF-8 byte length of an allowed key.
    pub maximum_key_bytes: usize,
}

impl RedactionPolicy {
    /// Drops unknown keys, masks sensitive values, and truncates at UTF-8 boundaries.
    #[must_use]
    pub fn apply(&self, attributes: &BTreeMap<String, String>) -> BTreeMap<String, String> {
        attributes
            .iter()
            .filter_map(|(key, value)| {
                if !self.allowed_keys.contains(key) || key.len() > self.maximum_key_bytes {
                    return None;
                }
                let value = if self.sensitive_keys.contains(key) {
                    REDACTED.to_owned()
                } else {
                    truncate_utf8(value, self.maximum_value_bytes)
                };
                Some((key.clone(), value))
            })
            .take(self.maximum_attribute_count)
            .collect()
    }
}

fn truncate_utf8(value: &str, maximum_bytes: usize) -> String {
    if value.len() <= maximum_bytes {
        return value.to_owned();
    }
    let mut boundary = maximum_bytes.min(value.len());
    while !value.is_char_boundary(boundary) {
        boundary = boundary.saturating_sub(1);
    }
    value[..boundary].to_owned()
}

/// Per-key cardinality limiter for bounded label values.
#[derive(Debug)]
pub struct CardinalityLimiter {
    maximum_distinct_values: usize,
    observed: BTreeMap<String, BTreeSet<String>>,
}

impl CardinalityLimiter {
    /// Creates a limiter with a positive per-key bound.
    pub fn new(maximum_distinct_values: usize) -> Result<Self, TelemetryError> {
        if maximum_distinct_values == 0 {
            return Err(TelemetryError::InvalidCardinalityLimit);
        }
        Ok(Self {
            maximum_distinct_values,
            observed: BTreeMap::new(),
        })
    }
    /// Admits known/new values until the bound, then rejects unseen values.
    pub fn admit(&mut self, key: &str, value: &str) -> bool {
        let values = self.observed.entry(key.to_owned()).or_default();
        values.contains(value)
            || (values.len() < self.maximum_distinct_values && values.insert(value.to_owned()))
    }
}

/// Deterministic head-sampling policy expressed in millionths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SamplingPolicy(u32);

impl SamplingPolicy {
    /// Creates a rate from 0 through 1,000,000 millionths.
    pub const fn from_millionths(value: u32) -> Result<Self, TelemetryError> {
        if value <= 1_000_000 {
            Ok(Self(value))
        } else {
            Err(TelemetryError::InvalidSamplingRate)
        }
    }
    /// Deterministically samples on the trace identifier.
    #[must_use]
    pub fn includes(self, trace_id: &str) -> bool {
        let hash = trace_id.bytes().fold(2_166_136_261_u32, |hash, byte| {
            hash.wrapping_mul(16_777_619) ^ u32::from(byte)
        });
        hash % 1_000_000 < self.0
    }
}

/// Nonblocking telemetry adapter. Implementations must never wait for capacity.
pub trait NonblockingTelemetrySink {
    /// Attempts immediate enqueue and reports saturation/unavailability.
    fn try_emit(&self, record: &TelemetryRecord) -> Result<(), TelemetryError>;
}

/// Best-effort emitter that converts telemetry failure into bounded drop accounting.
pub struct TelemetryEmitter<'a> {
    sink: &'a dyn NonblockingTelemetrySink,
    redaction: RedactionPolicy,
    cardinality: Mutex<CardinalityLimiter>,
    dropped: AtomicU64,
}

impl<'a> TelemetryEmitter<'a> {
    /// Wraps a nonblocking sink.
    pub fn new(
        sink: &'a dyn NonblockingTelemetrySink,
        redaction: RedactionPolicy,
        maximum_distinct_values: usize,
    ) -> Result<Self, TelemetryError> {
        Ok(Self {
            sink,
            redaction,
            cardinality: Mutex::new(CardinalityLimiter::new(maximum_distinct_values)?),
            dropped: AtomicU64::new(0),
        })
    }
    /// Emits without propagating failure into the caller's safety behavior.
    pub fn emit_best_effort(&self, record: &TelemetryRecord) {
        if !bounded_identifier(&record.name) || record.context.validate().is_err() {
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return;
        }
        let mut sanitized = record.clone();
        sanitized.attributes = self.redaction.apply(&record.attributes);
        let Ok(mut limiter) = self.cardinality.lock() else {
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return;
        };
        sanitized
            .attributes
            .retain(|key, value| limiter.admit(key, value));
        drop(limiter);
        if self.sink.try_emit(&sanitized).is_err() {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
    /// Number of locally observed dropped records.
    #[must_use]
    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }
}

/// Stable telemetry validation and delivery failures.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum TelemetryError {
    /// Trace or correlation propagation was malformed or oversized.
    #[error("propagation context is invalid")]
    InvalidPropagationContext,
    /// Cardinality limit must be positive.
    #[error("cardinality limit must be positive")]
    InvalidCardinalityLimit,
    /// Sampling rate exceeded one million millionths.
    #[error("sampling rate must be between zero and one million millionths")]
    InvalidSamplingRate,
    /// Sink had no immediate capacity.
    #[error("telemetry sink is saturated")]
    Saturated,
    /// Sink dependency was unavailable.
    #[error("telemetry sink is unavailable")]
    Unavailable,
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Saturated;
    impl NonblockingTelemetrySink for Saturated {
        fn try_emit(&self, _: &TelemetryRecord) -> Result<(), TelemetryError> {
            Err(TelemetryError::Saturated)
        }
    }
    fn context() -> PropagationContext {
        PropagationContext {
            traceparent: "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01".into(),
            tracestate: None,
            tenant_id: "tenant".into(),
            incident_id: None,
            correlation_id: "correlation".into(),
            causation_id: None,
            release_id: "release".into(),
        }
    }

    #[test]
    fn propagation_redaction_and_cardinality_are_bounded() -> Result<(), TelemetryError> {
        context().validate()?;
        let mut carrier = BTreeMap::new();
        context().inject(&mut carrier)?;
        assert_eq!(PropagationContext::extract(&carrier)?, context());
        let policy = RedactionPolicy {
            allowed_keys: BTreeSet::from(["safe".into(), "secret".into()]),
            sensitive_keys: BTreeSet::from(["secret".into()]),
            maximum_value_bytes: 3,
            maximum_attribute_count: 2,
            maximum_key_bytes: 32,
        };
        let result = policy.apply(&BTreeMap::from([
            ("safe".into(), "abcdef".into()),
            ("secret".into(), "token".into()),
            ("unknown".into(), "x".into()),
        ]));
        assert_eq!(result.get("safe").map(String::as_str), Some("abc"));
        assert_eq!(result.get("secret").map(String::as_str), Some(REDACTED));
        assert!(!result.contains_key("unknown"));
        let mut limiter = CardinalityLimiter::new(1)?;
        assert!(limiter.admit("vehicle", "one"));
        assert!(!limiter.admit("vehicle", "two"));
        Ok(())
    }

    #[test]
    fn telemetry_failure_never_propagates_to_safety_caller() -> Result<(), TelemetryError> {
        let record = TelemetryRecord {
            name: "safety.stop".into(),
            classification: DataClassification::Restricted,
            context: context(),
            attributes: BTreeMap::new(),
        };
        let emitter = TelemetryEmitter::new(
            &Saturated,
            RedactionPolicy {
                allowed_keys: BTreeSet::new(),
                sensitive_keys: BTreeSet::new(),
                maximum_value_bytes: 64,
                maximum_attribute_count: 8,
                maximum_key_bytes: 64,
            },
            8,
        )?;
        emitter.emit_best_effort(&record);
        assert_eq!(emitter.dropped(), 1);
        Ok(())
    }
}
