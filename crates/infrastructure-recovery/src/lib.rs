#![forbid(unsafe_code)]
#![allow(missing_docs, clippy::must_use_candidate)]
//! Provider-neutral backup and disaster-recovery evidence tooling.
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RecoveryError {
    #[error("invalid immutable backup manifest")]
    InvalidManifest,
    #[error("restore order violation")]
    OrderViolation,
    #[error("recovery evidence mismatch")]
    EvidenceMismatch,
    #[error("RPO or RTO objective missed")]
    ObjectiveMissed,
    #[error("key service unavailable")]
    KeyUnavailable,
    #[error("invalid recovery transition")]
    InvalidTransition,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ResourceKind {
    KeyMaterial,
    Database,
    ObjectStore,
    Broker,
    Station,
    CloudRegion,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupObject {
    pub resource: ResourceKind,
    pub object_id: String,
    pub checksum: [u8; 32],
    pub bytes: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupManifest {
    pub id: String,
    pub tenant: String,
    pub source_region: String,
    pub backup_region: String,
    pub created_ms: u64,
    pub recovery_point_ms: u64,
    pub key_id: String,
    pub objects: Vec<BackupObject>,
    pub authority_high_watermark: u64,
    pub command_high_watermark: u64,
    pub digest: [u8; 32],
}
impl BackupManifest {
    #[allow(clippy::too_many_arguments)]
    pub fn seal(
        id: &str,
        tenant: &str,
        source_region: &str,
        backup_region: &str,
        created_ms: u64,
        recovery_point_ms: u64,
        key_id: &str,
        mut objects: Vec<BackupObject>,
        authority_high_watermark: u64,
        command_high_watermark: u64,
    ) -> Result<Self, RecoveryError> {
        if [id, tenant, source_region, backup_region, key_id].contains(&"")
            || !source_region.starts_with("ca-")
            || !backup_region.starts_with("ca-")
            || created_ms < recovery_point_ms
            || objects.is_empty()
            || objects
                .iter()
                .any(|o| o.object_id.is_empty() || o.checksum == [0; 32] || o.bytes == 0)
        {
            return Err(RecoveryError::InvalidManifest);
        }
        objects.sort_by(|a, b| {
            (a.resource, a.object_id.as_str()).cmp(&(b.resource, b.object_id.as_str()))
        });
        if objects
            .windows(2)
            .any(|w| w[0].resource == w[1].resource && w[0].object_id == w[1].object_id)
        {
            return Err(RecoveryError::InvalidManifest);
        }
        let digest = manifest_digest(
            id,
            tenant,
            source_region,
            backup_region,
            created_ms,
            recovery_point_ms,
            key_id,
            &objects,
            authority_high_watermark,
            command_high_watermark,
        );
        Ok(Self {
            id: id.into(),
            tenant: tenant.into(),
            source_region: source_region.into(),
            backup_region: backup_region.into(),
            created_ms,
            recovery_point_ms,
            key_id: key_id.into(),
            objects,
            authority_high_watermark,
            command_high_watermark,
            digest,
        })
    }
    pub fn verify(&self) -> bool {
        self.digest
            == manifest_digest(
                &self.id,
                &self.tenant,
                &self.source_region,
                &self.backup_region,
                self.created_ms,
                self.recovery_point_ms,
                &self.key_id,
                &self.objects,
                self.authority_high_watermark,
                self.command_high_watermark,
            )
    }
}
#[allow(clippy::too_many_arguments)]
fn manifest_digest(
    id: &str,
    tenant: &str,
    source: &str,
    backup: &str,
    created: u64,
    point: u64,
    key: &str,
    objects: &[BackupObject],
    authority: u64,
    commands: u64,
) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"wildfire-immutable-backup-v1");
    for s in [id, tenant, source, backup, key] {
        h.update((s.len() as u64).to_be_bytes());
        h.update(s);
    }
    h.update(created.to_be_bytes());
    h.update(point.to_be_bytes());
    h.update(authority.to_be_bytes());
    h.update(commands.to_be_bytes());
    for o in objects {
        h.update([o.resource as u8]);
        h.update((o.object_id.len() as u64).to_be_bytes());
        h.update(&o.object_id);
        h.update(o.checksum);
        h.update(o.bytes.to_be_bytes());
    }
    h.finalize().into()
}

pub trait KeyRecoveryPort {
    fn verify_key_available(&self, key_id: &str) -> Result<(), RecoveryError>;
    fn verify_manifest_signature(&self, digest: [u8; 32]) -> Result<(), RecoveryError>;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryAction {
    RestoreNetworkAndTrustedTime,
    RestoreKeyAccess,
    RestoreIdentityPkiAndRevocation,
    FailoverRegion,
    RestoreDatabasePitr,
    RestoreObjectEvidence,
    RestoreAuditAndEvidence,
    RestoreBrokerQuarantined,
    ReplayFactsIdempotently,
    RestoreProjectionsApisAndGateway,
    ReconcileStation,
    RevalidateAuthority,
    ReconcileCommandOutcomes,
    EnableCommandTraffic,
}
impl RecoveryAction {
    fn rank(self) -> u8 {
        match self {
            Self::RestoreNetworkAndTrustedTime => 0,
            Self::RestoreKeyAccess => 1,
            Self::RestoreIdentityPkiAndRevocation => 2,
            Self::FailoverRegion => 3,
            Self::RestoreDatabasePitr => 4,
            Self::RestoreObjectEvidence => 5,
            Self::RestoreAuditAndEvidence => 6,
            Self::RestoreBrokerQuarantined => 7,
            Self::ReplayFactsIdempotently => 8,
            Self::RestoreProjectionsApisAndGateway => 9,
            Self::ReconcileStation => 10,
            Self::RevalidateAuthority => 11,
            Self::ReconcileCommandOutcomes => 12,
            Self::EnableCommandTraffic => 13,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryPlan {
    pub id: String,
    pub manifest_digest: [u8; 32],
    pub actions: Vec<RecoveryAction>,
    pub target_rpo_ms: u64,
    pub target_rto_ms: u64,
    pub disaster_declaration_ref: String,
    pub recovery_approval_ref: String,
    pub approval_signature: [u8; 32],
    pub recovery_epoch: u64,
    pub fencing_token: [u8; 32],
}
impl RecoveryPlan {
    #[allow(clippy::similar_names, clippy::too_many_arguments)]
    pub fn define(
        id: &str,
        manifest: &BackupManifest,
        actions: Vec<RecoveryAction>,
        target_rpo_ms: u64,
        target_rto_ms: u64,
        disaster_declaration_ref: &str,
        recovery_approval_ref: &str,
        approval_signature: [u8; 32],
        recovery_epoch: u64,
        fencing_token: [u8; 32],
    ) -> Result<Self, RecoveryError> {
        if id.is_empty()
            || !manifest.verify()
            || target_rto_ms == 0
            || disaster_declaration_ref.is_empty()
            || recovery_approval_ref.is_empty()
            || approval_signature == [0; 32]
            || recovery_epoch <= manifest.authority_high_watermark
            || fencing_token == [0; 32]
            || actions.is_empty()
            || !actions.windows(2).all(|w| w[0].rank() < w[1].rank())
            || !actions.contains(&RecoveryAction::RevalidateAuthority)
            || !actions.contains(&RecoveryAction::ReconcileCommandOutcomes)
            || !actions.contains(&RecoveryAction::EnableCommandTraffic)
        {
            return Err(RecoveryError::OrderViolation);
        }
        Ok(Self {
            id: id.into(),
            manifest_digest: manifest.digest,
            actions,
            target_rpo_ms,
            target_rto_ms,
            disaster_declaration_ref: disaster_declaration_ref.into(),
            recovery_approval_ref: recovery_approval_ref.into(),
            approval_signature,
            recovery_epoch,
            fencing_token,
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExerciseState {
    Declared,
    Running,
    Failed,
    Restored,
    Verified,
    RolledBack,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepEvidence {
    pub action: RecoveryAction,
    pub started_ms: u64,
    pub completed_ms: u64,
    pub artifact_digest: [u8; 32],
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct RecoveryEvidence {
    pub plan_id: String,
    pub manifest_digest: [u8; 32],
    pub state: ExerciseState,
    pub next_step: usize,
    pub steps: Vec<StepEvidence>,
    pub declared_ms: u64,
    pub latest_source_fact_ms: u64,
    pub authority_restored: bool,
    pub commands_replayed: bool,
    pub broker_quarantined: bool,
    pub facts_replayed_idempotently: bool,
    pub command_traffic_enabled: bool,
    pub authority_revalidation: Option<AuthorityRevalidation>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityRevalidation {
    pub evidence_ref: String,
    pub authority_epoch: u64,
    pub valid_until_ms: u64,
    pub signature: [u8; 32],
}
impl RecoveryEvidence {
    pub fn start(plan: &RecoveryPlan, declared_ms: u64, latest_source_fact_ms: u64) -> Self {
        Self {
            plan_id: plan.id.clone(),
            manifest_digest: plan.manifest_digest,
            state: ExerciseState::Declared,
            next_step: 0,
            steps: Vec::new(),
            declared_ms,
            latest_source_fact_ms,
            authority_restored: false,
            commands_replayed: false,
            broker_quarantined: false,
            facts_replayed_idempotently: false,
            command_traffic_enabled: false,
            authority_revalidation: None,
        }
    }
    pub fn resume(
        &mut self,
        plan: &RecoveryPlan,
        manifest: &BackupManifest,
        keys: &impl KeyRecoveryPort,
    ) -> Result<(), RecoveryError> {
        if self.plan_id != plan.id
            || self.manifest_digest != plan.manifest_digest
            || plan.manifest_digest != manifest.digest
            || !manifest.verify()
        {
            return Err(RecoveryError::EvidenceMismatch);
        }
        keys.verify_key_available(&manifest.key_id)?;
        keys.verify_manifest_signature(manifest.digest)?;
        if !matches!(
            self.state,
            ExerciseState::Declared | ExerciseState::Running | ExerciseState::Failed
        ) {
            return Err(RecoveryError::InvalidTransition);
        }
        self.state = ExerciseState::Running;
        Ok(())
    }
    pub fn record_step(
        &mut self,
        plan: &RecoveryPlan,
        step: StepEvidence,
    ) -> Result<(), RecoveryError> {
        if self.state != ExerciseState::Running
            || plan.actions.get(self.next_step) != Some(&step.action)
            || step.completed_ms < step.started_ms
            || step.artifact_digest == [0; 32]
        {
            self.state = ExerciseState::Failed;
            return Err(RecoveryError::OrderViolation);
        }
        if step.action == RecoveryAction::RestoreBrokerQuarantined {
            self.broker_quarantined = true;
        }
        if step.action == RecoveryAction::ReplayFactsIdempotently {
            if !self.broker_quarantined {
                return Err(RecoveryError::OrderViolation);
            }
            self.facts_replayed_idempotently = true;
        }
        if step.action == RecoveryAction::RevalidateAuthority
            && self.authority_revalidation.as_ref().is_none_or(|e| {
                e.authority_epoch != plan.recovery_epoch || e.valid_until_ms < step.completed_ms
            })
        {
            self.state = ExerciseState::Failed;
            return Err(RecoveryError::EvidenceMismatch);
        }
        if step.action == RecoveryAction::ReconcileCommandOutcomes {
            self.commands_replayed = false;
        }
        if step.action == RecoveryAction::EnableCommandTraffic {
            if !self.broker_quarantined
                || !self.facts_replayed_idempotently
                || self.authority_revalidation.is_none()
            {
                self.state = ExerciseState::Failed;
                return Err(RecoveryError::EvidenceMismatch);
            }
            self.command_traffic_enabled = true;
        }
        self.steps.push(step);
        self.next_step += 1;
        if self.next_step == plan.actions.len() {
            self.state = ExerciseState::Restored;
        }
        Ok(())
    }
    pub fn attach_authority_revalidation(
        &mut self,
        plan: &RecoveryPlan,
        evidence: AuthorityRevalidation,
    ) -> Result<(), RecoveryError> {
        if evidence.evidence_ref.is_empty()
            || evidence.signature == [0; 32]
            || evidence.authority_epoch != plan.recovery_epoch
            || evidence.authority_epoch <= plan.recovery_epoch.saturating_sub(1)
        {
            return Err(RecoveryError::EvidenceMismatch);
        }
        self.authority_revalidation = Some(evidence);
        self.authority_restored = false;
        Ok(())
    }
    pub fn verify_objectives(
        &mut self,
        plan: &RecoveryPlan,
        manifest: &BackupManifest,
        verified_ms: u64,
    ) -> Result<(u64, u64), RecoveryError> {
        if self.state != ExerciseState::Restored
            || self.authority_restored
            || self.commands_replayed
            || !self.command_traffic_enabled
            || self.authority_revalidation.as_ref().is_none_or(|e| {
                e.authority_epoch != plan.recovery_epoch || e.valid_until_ms < verified_ms
            })
        {
            return Err(RecoveryError::InvalidTransition);
        }
        let rpo = self
            .latest_source_fact_ms
            .saturating_sub(manifest.recovery_point_ms);
        let rto = verified_ms.saturating_sub(self.declared_ms);
        if rpo > plan.target_rpo_ms || rto > plan.target_rto_ms {
            self.state = ExerciseState::Failed;
            return Err(RecoveryError::ObjectiveMissed);
        }
        self.state = ExerciseState::Verified;
        Ok((rpo, rto))
    }
    pub fn rollback(&mut self) -> Result<(), RecoveryError> {
        if !matches!(
            self.state,
            ExerciseState::Running | ExerciseState::Failed | ExerciseState::Restored
        ) {
            return Err(RecoveryError::InvalidTransition);
        }
        self.state = ExerciseState::RolledBack;
        self.authority_restored = false;
        self.commands_replayed = false;
        self.command_traffic_enabled = false;
        Ok(())
    }
}
pub fn required_actions(resources: &BTreeSet<ResourceKind>) -> Vec<RecoveryAction> {
    let mut a = vec![
        RecoveryAction::RestoreNetworkAndTrustedTime,
        RecoveryAction::RestoreKeyAccess,
        RecoveryAction::RestoreIdentityPkiAndRevocation,
    ];
    if resources.contains(&ResourceKind::CloudRegion) {
        a.push(RecoveryAction::FailoverRegion);
    }
    if resources.contains(&ResourceKind::Database) {
        a.push(RecoveryAction::RestoreDatabasePitr);
    }
    if resources.contains(&ResourceKind::ObjectStore) {
        a.push(RecoveryAction::RestoreObjectEvidence);
    }
    a.push(RecoveryAction::RestoreAuditAndEvidence);
    if resources.contains(&ResourceKind::Broker) {
        a.push(RecoveryAction::RestoreBrokerQuarantined);
        a.push(RecoveryAction::ReplayFactsIdempotently);
    }
    a.push(RecoveryAction::RestoreProjectionsApisAndGateway);
    if resources.contains(&ResourceKind::Station) {
        a.push(RecoveryAction::ReconcileStation);
    }
    a.extend([
        RecoveryAction::RevalidateAuthority,
        RecoveryAction::ReconcileCommandOutcomes,
        RecoveryAction::EnableCommandTraffic,
    ]);
    a
}
