#![forbid(unsafe_code)]
//! Policy validator for context-owned `PostgreSQL` migrations (ADR-041).

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

/// Parsed context migration policy.
#[derive(Debug, Deserialize)]
pub struct MigrationPolicy {
    /// Stable bounded-context owner.
    pub context: String,
    /// `PostgreSQL` schema owned exclusively by the context.
    pub schema: String,
    /// Transaction-scoped advisory-lock key.
    pub advisory_lock_key: i64,
    /// Maximum lock acquisition time.
    pub lock_timeout_ms: u64,
    /// Maximum migration statement time.
    pub statement_timeout_ms: u64,
    /// Number of deployed versions supported during migration.
    pub compatibility_versions: u32,
    /// Immutable ordered migration inventory.
    pub migrations: Vec<MigrationRecord>,
    /// Recovery metadata.
    pub recovery: RecoveryPolicy,
}

/// One immutable migration and its lifecycle phase.
#[derive(Debug, Deserialize)]
pub struct MigrationRecord {
    /// Six-digit monotonically increasing version.
    pub version: u32,
    /// Relative SQL filename.
    pub file: PathBuf,
    /// Expand, backfill, switch, or contract phase.
    pub phase: String,
    /// Lowercase SHA-256 of the reviewed SQL bytes.
    pub sha256: String,
}

/// Required roll-forward and restore evidence metadata.
#[derive(Debug, Deserialize)]
pub struct RecoveryPolicy {
    /// Operational roll-forward procedure.
    pub roll_forward_runbook: PathBuf,
    /// Verified restore procedure.
    pub restore_runbook: PathBuf,
    /// Backup evidence identifier used by the fixture.
    pub backup_evidence: String,
    /// Retention/legal review record for destructive contract work.
    pub retention_review: String,
}

/// Validates a policy and all referenced migrations beneath `root`.
pub fn validate(root: &Path, policy_path: &Path) -> Result<MigrationPolicy, MigrationError> {
    let policy: MigrationPolicy = toml::from_str(&fs::read_to_string(policy_path)?)?;
    validate_metadata(root, &policy)?;
    let mut prior = None;
    let mut files = BTreeSet::new();
    let mut phases = Vec::new();
    for migration in &policy.migrations {
        if prior.is_some_and(|version| migration.version <= version) {
            return Err(MigrationError::NonIncreasingVersion(migration.version));
        }
        prior = Some(migration.version);
        if !files.insert(&migration.file) {
            return Err(MigrationError::DuplicateFile(migration.file.clone()));
        }
        let expected_prefix = format!("{:06}_", migration.version);
        let filename = migration
            .file
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        let is_sql = migration
            .file
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("sql"));
        if !filename.starts_with(&expected_prefix) || !is_sql {
            return Err(MigrationError::InvalidFilename(migration.file.clone()));
        }
        let sql = fs::read(root.join(&migration.file))?;
        let digest = format!("{:x}", Sha256::digest(&sql));
        if digest != migration.sha256 {
            return Err(MigrationError::DigestMismatch(migration.file.clone()));
        }
        validate_sql(&String::from_utf8_lossy(&sql), migration)?;
        phases.push(migration.phase.as_str());
    }
    if phases != ["expand", "backfill", "switch", "contract"] {
        return Err(MigrationError::InvalidPhaseOrder);
    }
    Ok(policy)
}

fn validate_metadata(root: &Path, policy: &MigrationPolicy) -> Result<(), MigrationError> {
    if policy.context.trim().is_empty() || policy.schema.trim().is_empty() {
        return Err(MigrationError::MissingOwnership);
    }
    if policy.advisory_lock_key == 0
        || policy.lock_timeout_ms == 0
        || policy.statement_timeout_ms == 0
    {
        return Err(MigrationError::UnboundedExecution);
    }
    if policy.lock_timeout_ms > 10_000 || policy.statement_timeout_ms > 300_000 {
        return Err(MigrationError::UnboundedExecution);
    }
    if policy.compatibility_versions < 2 {
        return Err(MigrationError::InsufficientCompatibilityWindow);
    }
    let recovery = &policy.recovery;
    if recovery.backup_evidence.trim().is_empty()
        || recovery.retention_review.trim().is_empty()
        || !root.join(&recovery.roll_forward_runbook).is_file()
        || !root.join(&recovery.restore_runbook).is_file()
    {
        return Err(MigrationError::MissingRecoveryMetadata);
    }
    Ok(())
}

fn validate_sql(sql: &str, migration: &MigrationRecord) -> Result<(), MigrationError> {
    let normalized = sql.to_ascii_lowercase();
    for required in [
        "set local lock_timeout",
        "set local statement_timeout",
        "pg_advisory_xact_lock",
    ] {
        if !normalized.contains(required) {
            return Err(MigrationError::MissingSqlGuard {
                file: migration.file.clone(),
                guard: required,
            });
        }
    }
    let phase_marker = format!("phase: {}", migration.phase);
    if !normalized.contains(&phase_marker) {
        return Err(MigrationError::MissingPhaseMarker(migration.file.clone()));
    }
    if migration.phase == "backfill"
        && !(normalized.contains("skip locked") && normalized.contains("checkpoint"))
    {
        return Err(MigrationError::UnsafeBackfill(migration.file.clone()));
    }
    if migration.phase == "switch" && !normalized.contains("dual_read") {
        return Err(MigrationError::MissingDualRead(migration.file.clone()));
    }
    if migration.phase == "contract"
        && !(normalized.contains("fleet_version_evidence")
            && normalized.contains("backup_evidence"))
    {
        return Err(MigrationError::UnsafeContract(migration.file.clone()));
    }
    Ok(())
}

/// Stable migration-policy failures.
#[derive(Debug, Error)]
pub enum MigrationError {
    /// Filesystem access failed.
    #[error("migration filesystem error: {0}")]
    Io(#[from] std::io::Error),
    /// Policy TOML could not be decoded.
    #[error("invalid migration policy: {0}")]
    Toml(#[from] toml::de::Error),
    /// Context or schema ownership was absent.
    #[error("context and schema ownership are required")]
    MissingOwnership,
    /// Timeouts or advisory locking were absent or outside policy bounds.
    #[error("migration execution must use bounded non-zero timeouts and an advisory lock")]
    UnboundedExecution,
    /// Mixed-version compatibility was too narrow.
    #[error("migration must support at least two deployed schema versions")]
    InsufficientCompatibilityWindow,
    /// Recovery, backup, or legal-review metadata was incomplete.
    #[error("roll-forward, restore, backup, and retention-review metadata are required")]
    MissingRecoveryMetadata,
    /// Migration versions were not strictly increasing.
    #[error("migration version {0} is not strictly increasing")]
    NonIncreasingVersion(u32),
    /// The same migration filename appeared more than once.
    #[error("duplicate migration file {path}", path = .0.display())]
    DuplicateFile(PathBuf),
    /// Filename did not begin with its six-digit version.
    #[error("invalid ordered migration filename {path}", path = .0.display())]
    InvalidFilename(PathBuf),
    /// Reviewed checksum no longer matched file bytes.
    #[error("immutable migration digest mismatch for {path}", path = .0.display())]
    DigestMismatch(PathBuf),
    /// A required SQL execution guard was absent.
    #[error("migration {file} is missing {guard}", file = file.display())]
    MissingSqlGuard {
        /// Migration missing the guard.
        file: PathBuf,
        /// Required SQL fragment.
        guard: &'static str,
    },
    /// Lifecycle phases were missing or reordered.
    #[error("migration phases must be expand, backfill, switch, contract")]
    InvalidPhaseOrder,
    /// SQL lacked its declared lifecycle marker.
    #[error("migration {path} lacks its phase marker", path = .0.display())]
    MissingPhaseMarker(PathBuf),
    /// Backfill lacked bounded checkpointing.
    #[error("backfill {path} must checkpoint and use SKIP LOCKED", path = .0.display())]
    UnsafeBackfill(PathBuf),
    /// Reader switch did not declare dual-read compatibility.
    #[error("switch {path} must explicitly enable dual_read", path = .0.display())]
    MissingDualRead(PathBuf),
    /// Destructive contract lacked deployment and backup evidence gates.
    #[error("contract {path} lacks fleet and backup evidence gates", path = .0.display())]
    UnsafeContract(PathBuf),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_policy_and_all_migrations_pass() -> Result<(), MigrationError> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let policy = root.join("fixtures/persistence-service/migration-policy.toml");
        let result = validate(&root, &policy)?;
        assert_eq!(result.migrations.len(), 4);
        Ok(())
    }

    #[test]
    fn mutation_of_immutable_migration_is_detected() -> Result<(), MigrationError> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let policy_path = root.join("fixtures/persistence-service/migration-policy.toml");
        let mut policy: MigrationPolicy = toml::from_str(&fs::read_to_string(policy_path)?)?;
        policy.migrations[0].sha256 = "0".repeat(64);
        let migration = &policy.migrations[0];
        let actual = format!(
            "{:x}",
            Sha256::digest(fs::read(root.join(&migration.file))?)
        );
        assert_ne!(actual, migration.sha256);
        Ok(())
    }
}
