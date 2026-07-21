//! Opaque identity, scope, concurrency, and message-tracing primitives.
//!
//! These types implement the technical identity portions of ADR-023 and the
//! tactical-model standard. Marker types prevent IDs from different namespaces
//! from being interchanged by otherwise well-typed code.

use core::{fmt, marker::PhantomData};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;
use uuid::Uuid;

/// Identifies an ID namespace without storing the marker at runtime.
pub trait IdentifierKind: Copy + fmt::Debug + Eq + Send + Sync + 'static {
    /// Stable namespace name used in diagnostics.
    const NAMESPACE: &'static str;
}

macro_rules! identifier_kinds {
    ($($kind:ident => $namespace:literal),+ $(,)?) => {$ (
        #[doc = concat!("Marker for the `", $namespace, "` identifier namespace.")]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub enum $kind {}

        impl IdentifierKind for $kind {
            const NAMESPACE: &'static str = $namespace;
        }
    )+ };
}

identifier_kinds! {
    Tenant => "tenant",
    Incident => "incident",
    Mission => "mission",
    Vehicle => "vehicle",
    Principal => "principal",
    Correlation => "correlation",
    Causation => "causation",
}

/// Globally unique opaque identifier belonging to namespace `K`.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Identifier<K: IdentifierKind> {
    value: Uuid,
    kind: PhantomData<K>,
}

impl<K: IdentifierKind> Identifier<K> {
    /// Generates a new random UUID identifier.
    #[must_use]
    pub fn new() -> Self {
        Self::from_uuid(Uuid::new_v4())
    }

    /// Creates an identifier from an already validated UUID.
    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self {
            value,
            kind: PhantomData,
        }
    }

    /// Parses the canonical or hyphenated UUID text representation.
    pub fn parse(value: &str) -> Result<Self, IdentityError> {
        Uuid::parse_str(value)
            .map(Self::from_uuid)
            .map_err(|_| IdentityError::InvalidIdentifier {
                namespace: K::NAMESPACE,
            })
    }

    /// Returns the underlying UUID for persistence and adapter mapping.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.value
    }
}

impl<K: IdentifierKind> Default for Identifier<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: IdentifierKind> fmt::Debug for Identifier<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple(K::NAMESPACE).field(&self.value).finish()
    }
}

impl<K: IdentifierKind> fmt::Display for Identifier<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<K: IdentifierKind> Serialize for Identifier<K> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(serializer)
    }
}

impl<'de, K: IdentifierKind> Deserialize<'de> for Identifier<K> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Uuid::deserialize(deserializer).map(Self::from_uuid)
    }
}

/// Tenant identifier.
pub type TenantId = Identifier<Tenant>;
/// Incident identifier.
pub type IncidentId = Identifier<Incident>;
/// Mission identifier.
pub type MissionId = Identifier<Mission>;
/// Vehicle identifier.
pub type VehicleId = Identifier<Vehicle>;
/// Human or workload principal identifier.
pub type PrincipalId = Identifier<Principal>;
/// End-to-end operation trace identifier.
pub type CorrelationId = Identifier<Correlation>;
/// Identifier of the message or action that directly caused another.
pub type CausationId = Identifier<Causation>;

/// Tenant authority boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TenantScope {
    tenant_id: TenantId,
}

impl TenantScope {
    /// Creates a tenant scope.
    #[must_use]
    pub const fn new(tenant_id: TenantId) -> Self {
        Self { tenant_id }
    }
    /// Returns the tenant boundary.
    #[must_use]
    pub const fn tenant_id(self) -> TenantId {
        self.tenant_id
    }
}

/// Incident authority boundary, always anchored to a tenant.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct IncidentScope {
    tenant_id: TenantId,
    incident_id: IncidentId,
}

impl IncidentScope {
    /// Creates an incident scope within a tenant.
    #[must_use]
    pub const fn new(tenant_id: TenantId, incident_id: IncidentId) -> Self {
        Self {
            tenant_id,
            incident_id,
        }
    }
    /// Returns the containing tenant.
    #[must_use]
    pub const fn tenant_id(self) -> TenantId {
        self.tenant_id
    }
    /// Returns the incident boundary.
    #[must_use]
    pub const fn incident_id(self) -> IncidentId {
        self.incident_id
    }
    /// Returns the containing tenant scope.
    #[must_use]
    pub const fn tenant_scope(self) -> TenantScope {
        TenantScope::new(self.tenant_id)
    }
}

/// Optimistic concurrency version. Version zero represents new state.
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct AggregateVersion(u64);

impl AggregateVersion {
    /// Initial version before the first committed change.
    pub const INITIAL: Self = Self(0);
    /// Restores a persisted version.
    #[must_use]
    pub const fn from_u64(value: u64) -> Self {
        Self(value)
    }
    /// Returns its wire-neutral integer representation.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
    /// Advances the version, detecting exhaustion instead of wrapping.
    pub const fn checked_next(self) -> Result<Self, IdentityError> {
        match self.0.checked_add(1) {
            Some(value) => Ok(Self(value)),
            None => Err(IdentityError::VersionExhausted),
        }
    }
}

/// Monotonically increasing lease/leadership token. Zero is never valid.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "u64", into = "u64")]
pub struct FencingToken(u64);

impl FencingToken {
    /// Validates a token restored from an external boundary.
    pub const fn new(value: u64) -> Result<Self, IdentityError> {
        if value == 0 {
            Err(IdentityError::InvalidFencingToken)
        } else {
            Ok(Self(value))
        }
    }
    /// Returns its integer representation.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
    /// Issues the next token, detecting exhaustion.
    pub const fn checked_next(self) -> Result<Self, IdentityError> {
        match self.0.checked_add(1) {
            Some(value) => Ok(Self(value)),
            None => Err(IdentityError::FencingTokenExhausted),
        }
    }
}

impl TryFrom<u64> for FencingToken {
    type Error = IdentityError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<FencingToken> for u64 {
    fn from(value: FencingToken) -> Self {
        value.get()
    }
}

/// Trace context carried across command and event boundaries.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct MessageTrace {
    correlation_id: CorrelationId,
    causation_id: Option<CausationId>,
}

impl MessageTrace {
    /// Starts a root operation without a causal predecessor.
    #[must_use]
    pub const fn root(correlation_id: CorrelationId) -> Self {
        Self {
            correlation_id,
            causation_id: None,
        }
    }
    /// Continues an operation from a direct causal predecessor.
    #[must_use]
    pub const fn caused_by(correlation_id: CorrelationId, causation_id: CausationId) -> Self {
        Self {
            correlation_id,
            causation_id: Some(causation_id),
        }
    }
    /// Returns the end-to-end correlation identifier.
    #[must_use]
    pub const fn correlation_id(self) -> CorrelationId {
        self.correlation_id
    }
    /// Returns the direct cause, if this is not a root operation.
    #[must_use]
    pub const fn causation_id(self) -> Option<CausationId> {
        self.causation_id
    }
}

/// Compatibility identifier for pre-Prompt-02 callers.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct EntityId(String);

impl EntityId {
    /// Creates a UUID-backed compatibility identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    /// Validates the legacy textual boundary.
    pub fn parse(value: impl Into<String>) -> Result<Self, IdentityError> {
        let value = value.into();
        if value.trim().is_empty() || value.len() > 128 {
            Err(IdentityError::InvalidLegacyIdentifier)
        } else {
            Ok(Self(value))
        }
    }
    /// Returns the stable textual representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<String> for EntityId {
    type Error = IdentityError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl From<EntityId> for String {
    fn from(value: EntityId) -> Self {
        value.0
    }
}

/// Identity and concurrency validation failures.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum IdentityError {
    /// A namespace-specific identifier was not a UUID.
    #[error("invalid {namespace} identifier")]
    InvalidIdentifier {
        /// Stable namespace of the rejected identifier.
        namespace: &'static str,
    },
    /// Legacy textual ID is blank or longer than 128 bytes.
    #[error("identifier must contain 1 to 128 bytes")]
    InvalidLegacyIdentifier,
    /// Aggregate version cannot advance beyond `u64::MAX`.
    #[error("aggregate version exhausted")]
    VersionExhausted,
    /// Fencing token zero is reserved as invalid.
    #[error("fencing token must be greater than zero")]
    InvalidFencingToken,
    /// Fencing token cannot advance beyond `u64::MAX`.
    #[error("fencing token exhausted")]
    FencingTokenExhausted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_round_trips_without_its_serialization_format() -> Result<(), IdentityError> {
        let raw = Uuid::from_u128(42);
        let id = TenantId::parse(&raw.to_string())?;
        assert_eq!(id.as_uuid(), raw);
        assert_eq!(id.to_string(), raw.to_string());
        Ok(())
    }

    #[test]
    fn scope_retains_both_authority_boundaries() {
        let tenant = TenantId::from_uuid(Uuid::from_u128(1));
        let incident = IncidentId::from_uuid(Uuid::from_u128(2));
        let scope = IncidentScope::new(tenant, incident);
        assert_eq!(scope.tenant_scope(), TenantScope::new(tenant));
        assert_eq!(scope.incident_id(), incident);
    }

    #[test]
    fn aggregate_version_never_wraps() {
        assert_eq!(
            AggregateVersion::INITIAL.checked_next(),
            Ok(AggregateVersion::from_u64(1))
        );
        assert_eq!(
            AggregateVersion::from_u64(u64::MAX).checked_next(),
            Err(IdentityError::VersionExhausted)
        );
    }

    #[test]
    fn fencing_tokens_are_nonzero_and_strictly_advance() -> Result<(), IdentityError> {
        assert_eq!(
            FencingToken::new(0),
            Err(IdentityError::InvalidFencingToken)
        );
        let current = FencingToken::new(7)?;
        assert!(current.checked_next()? > current);
        assert_eq!(
            FencingToken::new(u64::MAX)?.checked_next(),
            Err(IdentityError::FencingTokenExhausted)
        );
        Ok(())
    }

    #[test]
    fn trace_distinguishes_root_from_caused_message() {
        let correlation = CorrelationId::from_uuid(Uuid::from_u128(3));
        let cause = CausationId::from_uuid(Uuid::from_u128(4));
        assert_eq!(MessageTrace::root(correlation).causation_id(), None);
        assert_eq!(
            MessageTrace::caused_by(correlation, cause).causation_id(),
            Some(cause)
        );
    }

    #[test]
    fn invalid_identifier_reports_its_namespace() {
        assert_eq!(
            VehicleId::parse("not-a-uuid"),
            Err(IdentityError::InvalidIdentifier {
                namespace: "vehicle"
            })
        );
    }
}
