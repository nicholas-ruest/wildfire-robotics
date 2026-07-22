//! Versioned operator contracts, safety read models, and owning-service ports (ADR-019/073).
use crate::{DomainError, OperationScope};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const AERIAL_OPERATOR_API_VERSION: &str = "v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewKind {
    QualificationMatrix,
    AssemblyManifest,
    LoadApproval,
    CorridorMap,
    ExclusionMap,
    DispersionMap,
    ReleaseChecklist,
    DualDecisions,
    DeploymentPhase,
    CohortStability,
    TetherTension,
    PanelHealth,
    Footprint,
    DegradedZones,
    UnaccountedComponents,
    Disposition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyProvenance {
    pub source_time: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub uncertainty_bps: u16,
    pub configuration_digest: String,
    pub evidence_digest: String,
    pub authority: String,
}
impl SafetyProvenance {
    pub fn validate(&self, now: DateTime<Utc>) -> Result<(), DomainError> {
        let digest = |v: &str| {
            v.len() == 71
                && v.starts_with("sha256:")
                && v[7..]
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        };
        if self.source_time > now
            || now >= self.expires_at
            || self.expires_at <= self.source_time
            || self.uncertainty_bps > 10_000
            || !digest(&self.configuration_digest)
            || !digest(&self.evidence_digest)
            || self.authority.trim().is_empty()
        {
            return Err(DomainError::UnsafeOperatorView);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyView<T> {
    pub api_version: String,
    pub scope: OperationScope,
    pub resource_id: String,
    pub kind: ViewKind,
    pub aggregate_version: u64,
    pub provenance: SafetyProvenance,
    pub value: T,
}
impl<T> SafetyView<T> {
    pub fn authorize(
        &self,
        auth: &OperatorAuthorization,
        now: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        auth.authorize(&self.scope, OperatorPermission::Read)?;
        if self.api_version != AERIAL_OPERATOR_API_VERSION || self.resource_id.trim().is_empty() {
            return Err(DomainError::UnsafeOperatorView);
        }
        self.provenance.validate(now)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeoView {
    pub crs: String,
    /// `GeoJSON` coordinates stored as signed millionths to retain deterministic equality.
    pub coordinates_e6: Vec<(i32, i32)>,
}
impl GeoView {
    pub fn new(crs: &str, coordinates_e6: Vec<(i32, i32)>) -> Result<Self, DomainError> {
        if crs != "urn:ogc:def:crs:OGC::CRS84"
            || coordinates_e6.len() < 2
            || coordinates_e6.iter().any(|(lon, lat)| {
                !(-180_000_000..=180_000_000).contains(lon)
                    || !(-90_000_000..=90_000_000).contains(lat)
            })
        {
            return Err(DomainError::InvalidMapReference);
        }
        Ok(Self {
            crs: crs.into(),
            coordinates_e6,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OperatorPermission {
    Read,
    Command,
    IrreversibleCommand,
    AuditReplay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorAuthorization {
    pub principal: String,
    pub tenant: String,
    pub incidents: BTreeSet<String>,
    pub permissions: BTreeSet<OperatorPermission>,
}
impl OperatorAuthorization {
    pub fn authorize(
        &self,
        scope: &OperationScope,
        permission: OperatorPermission,
    ) -> Result<(), DomainError> {
        if self.principal.trim().is_empty()
            || self.tenant != scope.tenant
            || !self.incidents.contains(&scope.incident)
            || !self.permissions.contains(&permission)
        {
            return Err(DomainError::OperatorForbidden);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityLane {
    Aircraft,
    Incident,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionOutcome {
    Requested,
    Accepted,
    Executed,
    Observed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorCommand {
    pub id: String,
    pub scope: OperationScope,
    pub resource_id: String,
    pub action: String,
    pub authority_lane: AuthorityLane,
    pub expected_version: u64,
    pub irreversible: bool,
}
impl OperatorCommand {
    #[must_use]
    pub fn digest(&self) -> String {
        let canonical = format!(
            "{}|{}|{}|{}|{}|{}|{:?}|{}|{}",
            self.id,
            self.scope.tenant,
            self.scope.region,
            self.scope.incident,
            self.resource_id,
            self.action,
            self.authority_lane,
            self.expected_version,
            self.irreversible
        );
        format!("sha256:{:x}", Sha256::digest(canonical.as_bytes()))
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplicitConfirmation {
    pub command_digest: String,
    pub statement: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditRecord {
    pub sequence: u64,
    pub command_id: String,
    pub command_digest: String,
    pub principal: String,
    pub scope: OperationScope,
    pub authority_lane: AuthorityLane,
    pub outcome: ActionOutcome,
    pub occurred_at: DateTime<Utc>,
    pub previous_digest: String,
    pub digest: String,
}
impl AuditRecord {
    #[must_use]
    pub fn verify(&self) -> bool {
        let raw = format!(
            "{}|{}|{}|{}|{}|{}|{}|{:?}|{:?}|{}|{}",
            self.sequence,
            self.command_id,
            self.command_digest,
            self.principal,
            self.scope.tenant,
            self.scope.region,
            self.scope.incident,
            self.authority_lane,
            self.outcome,
            self.occurred_at.to_rfc3339(),
            self.previous_digest
        );
        self.digest == format!("sha256:{:x}", Sha256::digest(raw.as_bytes()))
    }
}

pub trait ImmutableAuditPort {
    fn head(&self, scope: &OperationScope) -> Result<Option<AuditRecord>, DomainError>;
    fn append(&mut self, record: AuditRecord) -> Result<(), DomainError>;
    fn replay(&self, scope: &OperationScope) -> Result<Vec<AuditRecord>, DomainError>;
}
pub trait BulkArtifactPort {
    fn put_immutable(
        &mut self,
        scope: &OperationScope,
        digest: &str,
        bytes: &[u8],
    ) -> Result<String, DomainError>;
}

/// Persists a content-addressed artifact through its owning platform boundary.
pub fn persist_bulk_artifact<B: BulkArtifactPort>(
    port: &mut B,
    scope: &OperationScope,
    bytes: &[u8],
) -> Result<(String, String), DomainError> {
    if bytes.is_empty() {
        return Err(DomainError::OperatorPersistenceFailed);
    }
    let digest = format!("sha256:{:x}", Sha256::digest(bytes));
    let uri = port
        .put_immutable(scope, &digest, bytes)
        .map_err(|_| DomainError::OperatorPersistenceFailed)?;
    if uri.trim().is_empty() {
        return Err(DomainError::OperatorPersistenceFailed);
    }
    Ok((digest, uri))
}

pub fn record_command<A: ImmutableAuditPort>(
    audit: &mut A,
    auth: &OperatorAuthorization,
    command: &OperatorCommand,
    actual_version: u64,
    confirmation: Option<&ExplicitConfirmation>,
    outcome: ActionOutcome,
    at: DateTime<Utc>,
) -> Result<AuditRecord, DomainError> {
    let permission = if command.irreversible {
        OperatorPermission::IrreversibleCommand
    } else {
        OperatorPermission::Command
    };
    auth.authorize(&command.scope, permission)?;
    if actual_version != command.expected_version {
        return Err(DomainError::VersionConflict);
    }
    if command.irreversible {
        let Some(value) = confirmation else {
            return Err(DomainError::ConfirmationRequired);
        };
        if value.command_digest != command.digest()
            || value.statement != "CONFIRM IRREVERSIBLE ACTION"
        {
            return Err(DomainError::ConfirmationRequired);
        }
    }
    let previous = audit
        .head(&command.scope)?
        .map_or_else(|| "genesis".into(), |r| r.digest);
    let sequence = audit.replay(&command.scope)?.len() as u64 + 1;
    let command_digest = command.digest();
    let raw = format!(
        "{}|{}|{}|{}|{}|{}|{}|{:?}|{:?}|{}|{}",
        sequence,
        command.id,
        command_digest,
        auth.principal,
        command.scope.tenant,
        command.scope.region,
        command.scope.incident,
        command.authority_lane,
        outcome,
        at.to_rfc3339(),
        previous
    );
    let record = AuditRecord {
        sequence,
        command_id: command.id.clone(),
        command_digest,
        principal: auth.principal.clone(),
        scope: command.scope.clone(),
        authority_lane: command.authority_lane,
        outcome,
        occurred_at: at,
        previous_digest: previous,
        digest: format!("sha256:{:x}", Sha256::digest(raw.as_bytes())),
    };
    audit
        .append(record.clone())
        .map_err(|_| DomainError::OperatorPersistenceFailed)?;
    Ok(record)
}

#[must_use]
pub fn verify_audit_replay(records: &[AuditRecord]) -> bool {
    let scope = records.first().map(|record| &record.scope);
    records.iter().enumerate().all(|(index, record)| {
        record.sequence == index as u64 + 1
            && scope == Some(&record.scope)
            && record.verify()
            && (index == 0 && record.previous_digest == "genesis"
                || index > 0 && record.previous_digest == records[index - 1].digest)
    })
}
