//! Station and deployment aggregates plus priority/resource policy.
#![allow(missing_docs)]
use chrono::{DateTime, Utc};
use shared_kernel::EntityId;
use std::collections::BTreeSet;
use thiserror::Error;
pub type Digest = [u8; 32];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorkloadClass {
    OptionalIndexing,
    OptionalMl,
    Telemetry,
    Maps,
    Mission,
    Identity,
    Safety,
    Command,
    Audit,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceBudget {
    pub cpu_millis: u32,
    pub memory_mib: u32,
    pub disk_mib: u64,
}
impl ResourceBudget {
    #[must_use]
    pub fn contains(&self, other: &Self) -> bool {
        other.cpu_millis <= self.cpu_millis
            && other.memory_mib <= self.memory_mib
            && other.disk_mib <= self.disk_mib
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourcePressure {
    pub disk_used_bps: u16,
    pub memory_used_bps: u16,
    pub cpu_used_bps: u16,
    pub power_available_bps: u16,
    pub thermal_margin_bps: u16,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StationState {
    Planned,
    Commissioned,
    Available,
    Degraded,
    Offline,
    Decommissioned,
}

#[derive(Clone, Debug)]
pub struct Station {
    id: EntityId,
    state: StationState,
    capacity: ResourceBudget,
    emergency_energy_wh: u64,
    available_energy_wh: u64,
    enabled: BTreeSet<WorkloadClass>,
    version: u64,
}
impl Station {
    pub fn commission(
        id: EntityId,
        capacity: ResourceBudget,
        emergency_energy_wh: u64,
        available_energy_wh: u64,
    ) -> Result<Self, StationError> {
        if capacity.cpu_millis == 0
            || capacity.memory_mib == 0
            || capacity.disk_mib == 0
            || emergency_energy_wh == 0
            || available_energy_wh < emergency_energy_wh
        {
            return Err(StationError::UnsafeCapacity);
        }
        Ok(Self {
            id,
            state: StationState::Commissioned,
            capacity,
            emergency_energy_wh,
            available_energy_wh,
            enabled: BTreeSet::new(),
            version: 1,
        })
    }
    pub fn attest(&mut self) -> Result<(), StationError> {
        if self.state != StationState::Commissioned {
            return Err(StationError::InvalidTransition);
        }
        self.state = StationState::Available;
        self.enabled = BTreeSet::from([
            WorkloadClass::Audit,
            WorkloadClass::Command,
            WorkloadClass::Safety,
            WorkloadClass::Identity,
            WorkloadClass::Mission,
            WorkloadClass::Maps,
            WorkloadClass::Telemetry,
            WorkloadClass::OptionalMl,
            WorkloadClass::OptionalIndexing,
        ]);
        self.bump()
    }
    pub fn apply_pressure(
        &mut self,
        p: ResourcePressure,
    ) -> Result<BTreeSet<WorkloadClass>, StationError> {
        if [
            p.disk_used_bps,
            p.memory_used_bps,
            p.cpu_used_bps,
            p.power_available_bps,
            p.thermal_margin_bps,
        ]
        .into_iter()
        .any(|v| v > 10_000)
        {
            return Err(StationError::InvalidPressure);
        }
        let mut shed = BTreeSet::new();
        let severity = p
            .disk_used_bps
            .max(p.memory_used_bps)
            .max(p.cpu_used_bps)
            .max(10_000 - p.power_available_bps)
            .max(10_000 - p.thermal_margin_bps);
        for class in if severity >= 9_500 {
            vec![
                WorkloadClass::OptionalIndexing,
                WorkloadClass::OptionalMl,
                WorkloadClass::Telemetry,
                WorkloadClass::Maps,
                WorkloadClass::Mission,
            ]
        } else if severity >= 8_500 {
            vec![
                WorkloadClass::OptionalIndexing,
                WorkloadClass::OptionalMl,
                WorkloadClass::Telemetry,
            ]
        } else if severity >= 7_500 {
            vec![WorkloadClass::OptionalIndexing, WorkloadClass::OptionalMl]
        } else {
            vec![]
        } {
            if self.enabled.remove(&class) {
                shed.insert(class);
            }
        }
        if !shed.is_empty() {
            self.state = StationState::Degraded;
            self.bump()?;
        }
        debug_assert!(
            self.enabled.contains(&WorkloadClass::Audit)
                && self.enabled.contains(&WorkloadClass::Command)
                && self.enabled.contains(&WorkloadClass::Safety)
                && self.enabled.contains(&WorkloadClass::Identity)
        );
        Ok(shed)
    }
    pub fn reserve_routine_energy(&mut self, amount_wh: u64) -> Result<(), StationError> {
        if self.available_energy_wh.saturating_sub(amount_wh) < self.emergency_energy_wh {
            return Err(StationError::EmergencyReserve);
        }
        self.available_energy_wh -= amount_wh;
        self.bump()
    }
    fn bump(&mut self) -> Result<(), StationError> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or(StationError::VersionExhausted)?;
        Ok(())
    }
    #[must_use]
    pub fn enabled(&self, class: WorkloadClass) -> bool {
        self.enabled.contains(&class)
    }
    #[must_use]
    pub fn capacity(&self) -> ResourceBudget {
        self.capacity
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeploymentState {
    Staged,
    Verified,
    Active,
    Degraded,
    RollingBack,
    Superseded,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeploymentManifest {
    pub deployment_id: EntityId,
    pub release_digest: Digest,
    pub configuration_digest: Digest,
    pub policy_digest: Digest,
    pub identity_bundle_digest: Digest,
    pub map_digest: Digest,
    pub schema_version: u32,
    pub required: ResourceBudget,
    pub minimum_runtime_version: u32,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub signature: String,
}
impl DeploymentManifest {
    pub fn validate_shape(&self, now: DateTime<Utc>) -> Result<(), StationError> {
        if [
            self.release_digest,
            self.configuration_digest,
            self.policy_digest,
            self.identity_bundle_digest,
            self.map_digest,
        ]
        .contains(&[0; 32])
            || self.schema_version == 0
            || self.minimum_runtime_version == 0
            || self.created_at > now
            || now >= self.expires_at
            || self.signature.len() < 16
        {
            return Err(StationError::InvalidManifest);
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct EdgeDeployment {
    manifest: DeploymentManifest,
    state: DeploymentState,
    verified_at: Option<DateTime<Utc>>,
    recovery_checkpoint: Option<Digest>,
    previous_release: Option<Digest>,
    version: u64,
}
impl EdgeDeployment {
    #[must_use]
    pub fn stage(manifest: DeploymentManifest, previous_release: Option<Digest>) -> Self {
        Self {
            manifest,
            state: DeploymentState::Staged,
            verified_at: None,
            recovery_checkpoint: None,
            previous_release,
            version: 1,
        }
    }
    pub fn mark_verified(
        &mut self,
        at: DateTime<Utc>,
        checkpoint: Digest,
    ) -> Result<(), StationError> {
        if self.state != DeploymentState::Staged || checkpoint == [0; 32] {
            return Err(StationError::InvalidTransition);
        }
        self.verified_at = Some(at);
        self.recovery_checkpoint = Some(checkpoint);
        self.state = DeploymentState::Verified;
        self.bump()
    }
    pub fn activate(&mut self) -> Result<(), StationError> {
        if self.state != DeploymentState::Verified {
            return Err(StationError::InvalidTransition);
        }
        self.state = DeploymentState::Active;
        self.bump()
    }
    pub fn begin_rollback(&mut self) -> Result<Digest, StationError> {
        if !matches!(
            self.state,
            DeploymentState::Active | DeploymentState::Degraded
        ) || self.previous_release.is_none()
            || self.recovery_checkpoint.is_none()
        {
            return Err(StationError::RollbackUnavailable);
        }
        self.state = DeploymentState::RollingBack;
        self.bump()?;
        self.previous_release
            .ok_or(StationError::RollbackUnavailable)
    }
    pub fn complete_rollback(&mut self) -> Result<(), StationError> {
        if self.state != DeploymentState::RollingBack {
            return Err(StationError::InvalidTransition);
        }
        self.state = DeploymentState::Superseded;
        self.bump()
    }
    fn bump(&mut self) -> Result<(), StationError> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or(StationError::VersionExhausted)?;
        Ok(())
    }
    #[must_use]
    pub fn manifest(&self) -> &DeploymentManifest {
        &self.manifest
    }
    #[must_use]
    pub fn state(&self) -> DeploymentState {
        self.state
    }
    #[must_use]
    pub const fn recovery_checkpoint(&self) -> Option<Digest> {
        self.recovery_checkpoint
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum StationError {
    #[error("station capacity or energy is unsafe")]
    UnsafeCapacity,
    #[error("invalid station transition")]
    InvalidTransition,
    #[error("resource pressure input is invalid")]
    InvalidPressure,
    #[error("emergency reserve would be consumed")]
    EmergencyReserve,
    #[error("deployment manifest is invalid or expired")]
    InvalidManifest,
    #[error("deployment signature is invalid")]
    InvalidSignature,
    #[error("deployment is incompatible with the local runtime")]
    IncompatibleDeployment,
    #[error("rollback or checkpoint is unavailable")]
    RollbackUnavailable,
    #[error("durable local log is corrupt or exhausted")]
    CorruptLog,
    #[error("clock uncertainty is unsafe")]
    ClockUncertain,
    #[error("reconciliation is partial, contradictory, or ambiguous")]
    ReconciliationConflict,
    #[error("aggregate version exhausted")]
    VersionExhausted,
}
