//! Asset aggregates implementing FO-INV-001 through FO-INV-005 and FO-INV-008.
#![allow(missing_docs)]
use chrono::{DateTime, Utc};
use shared_kernel::EntityId;
use std::collections::BTreeSet;
use thiserror::Error;
pub type Digest = [u8; 32];
fn text(v: impl Into<String>) -> Result<String, FleetError> {
    let v = v.into();
    if v.trim().is_empty() || v.len() > 256 {
        Err(FleetError::InvalidField)
    } else {
        Ok(v)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigurationState {
    Draft,
    Validated,
    Approved,
    Installed,
    Superseded,
}
#[derive(Clone, Debug)]
pub struct Configuration {
    id: EntityId,
    digest: Digest,
    hardware_bom: Digest,
    software_bom: Digest,
    compatibility_matrix_digest: Digest,
    matrix_signature: String,
    state: ConfigurationState,
    version: u64,
}
impl Configuration {
    pub fn register(
        id: EntityId,
        digest: Digest,
        hardware_bom: Digest,
        software_bom: Digest,
    ) -> Result<Self, FleetError> {
        if [digest, hardware_bom, software_bom].contains(&[0; 32]) {
            return Err(FleetError::InvalidDigest);
        }
        Ok(Self {
            id,
            digest,
            hardware_bom,
            software_bom,
            compatibility_matrix_digest: [0; 32],
            matrix_signature: String::new(),
            state: ConfigurationState::Draft,
            version: 1,
        })
    }
    pub fn validate(
        &mut self,
        matrix_digest: Digest,
        signature: impl Into<String>,
    ) -> Result<(), FleetError> {
        let signature = signature.into();
        if self.state != ConfigurationState::Draft
            || matrix_digest == [0; 32]
            || signature.len() < 16
        {
            return Err(FleetError::IncompatibleConfiguration);
        }
        self.compatibility_matrix_digest = matrix_digest;
        self.matrix_signature = signature;
        self.state = ConfigurationState::Validated;
        self.bump()
    }
    pub fn approve(&mut self) -> Result<(), FleetError> {
        self.transition(ConfigurationState::Validated, ConfigurationState::Approved)
    }
    pub fn attest_installation(&mut self, observed_digest: Digest) -> Result<(), FleetError> {
        if self.state != ConfigurationState::Approved || observed_digest != self.digest {
            return Err(FleetError::AttestationMismatch);
        }
        self.state = ConfigurationState::Installed;
        self.bump()
    }
    fn transition(
        &mut self,
        f: ConfigurationState,
        t: ConfigurationState,
    ) -> Result<(), FleetError> {
        if self.state != f {
            return Err(FleetError::InvalidTransition);
        }
        self.state = t;
        self.bump()
    }
    fn bump(&mut self) -> Result<(), FleetError> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or(FleetError::VersionExhausted)?;
        Ok(())
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }
    #[must_use]
    pub const fn bills_of_material(&self) -> (Digest, Digest) {
        (self.hardware_bom, self.software_bom)
    }
    #[must_use]
    pub fn installed(&self) -> bool {
        self.state == ConfigurationState::Installed
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityState {
    Claimed,
    Testing,
    Attested,
    Suspended,
    Expired,
}
#[derive(Clone, Debug)]
pub struct CapabilityRecord {
    id: EntityId,
    vehicle_id: EntityId,
    name: String,
    odd_digest: Digest,
    configuration_digest: Digest,
    evidence: BTreeSet<Digest>,
    expires_at: DateTime<Utc>,
    state: CapabilityState,
    version: u64,
}
impl CapabilityRecord {
    pub fn claim(
        id: EntityId,
        vehicle_id: EntityId,
        name: impl Into<String>,
        odd_digest: Digest,
        configuration_digest: Digest,
        expires_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<Self, FleetError> {
        if odd_digest == [0; 32] || configuration_digest == [0; 32] || expires_at <= now {
            return Err(FleetError::InvalidCapability);
        }
        Ok(Self {
            id,
            vehicle_id,
            name: text(name)?,
            odd_digest,
            configuration_digest,
            evidence: BTreeSet::new(),
            expires_at,
            state: CapabilityState::Claimed,
            version: 1,
        })
    }
    pub fn attach_evidence(&mut self, digest: Digest) -> Result<(), FleetError> {
        if !matches!(
            self.state,
            CapabilityState::Claimed | CapabilityState::Testing
        ) || digest == [0; 32]
        {
            return Err(FleetError::InvalidCapability);
        }
        self.evidence.insert(digest);
        self.state = CapabilityState::Testing;
        self.bump()
    }
    pub fn attest(
        &mut self,
        configuration: &Configuration,
        now: DateTime<Utc>,
    ) -> Result<(), FleetError> {
        if self.state != CapabilityState::Testing
            || self.evidence.is_empty()
            || self.configuration_digest != configuration.digest()
            || !configuration.installed()
            || now >= self.expires_at
        {
            return Err(FleetError::InvalidCapability);
        }
        self.state = CapabilityState::Attested;
        self.bump()
    }
    pub fn suspend(&mut self) -> Result<(), FleetError> {
        if !matches!(
            self.state,
            CapabilityState::Attested | CapabilityState::Testing
        ) {
            return Err(FleetError::InvalidTransition);
        }
        self.state = CapabilityState::Suspended;
        self.bump()
    }
    fn bump(&mut self) -> Result<(), FleetError> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or(FleetError::VersionExhausted)?;
        Ok(())
    }
    #[must_use]
    pub fn eligible(
        &self,
        vehicle: &EntityId,
        configuration: Digest,
        odd: Digest,
        now: DateTime<Utc>,
    ) -> bool {
        self.vehicle_id == *vehicle
            && self.configuration_digest == configuration
            && self.odd_digest == odd
            && self.state == CapabilityState::Attested
            && now < self.expires_at
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HealthState {
    Collecting,
    Assessed,
    Stale,
    Superseded,
}
#[derive(Clone, Debug)]
pub struct HealthAssessment {
    id: EntityId,
    vehicle_id: EntityId,
    source: String,
    quality_bps: u16,
    assessed_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    blocking_faults: BTreeSet<String>,
    state: HealthState,
}
impl HealthAssessment {
    pub fn assess(
        id: EntityId,
        vehicle_id: EntityId,
        source: impl Into<String>,
        quality_bps: u16,
        assessed_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
        blocking_faults: impl IntoIterator<Item = String>,
    ) -> Result<Self, FleetError> {
        if quality_bps > 10_000 || expires_at <= assessed_at {
            return Err(FleetError::InvalidHealth);
        }
        Ok(Self {
            id,
            vehicle_id,
            source: text(source)?,
            quality_bps,
            assessed_at,
            expires_at,
            blocking_faults: blocking_faults.into_iter().collect(),
            state: HealthState::Assessed,
        })
    }
    #[must_use]
    pub fn healthy_at(
        &self,
        vehicle: &EntityId,
        minimum_quality_bps: u16,
        now: DateTime<Utc>,
    ) -> bool {
        self.vehicle_id == *vehicle
            && self.state == HealthState::Assessed
            && self.quality_bps >= minimum_quality_bps
            && self.blocking_faults.is_empty()
            && now >= self.assessed_at
            && now < self.expires_at
    }
    pub fn mark_stale(&mut self) {
        self.state = HealthState::Stale;
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatteryState {
    Registered,
    Available,
    Assigned,
    Charging,
    InUse,
    Quarantined,
    Servicing,
    Retired,
}
#[derive(Clone, Debug)]
pub struct EnergyAssessment {
    pub usable_wh: u64,
    pub uncertainty_wh: u64,
    pub continuous_power_w: u64,
    pub temperature_millicelsius: i64,
    pub assessed_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
#[derive(Clone, Debug)]
pub struct BatteryAsset {
    id: EntityId,
    chemistry: String,
    form_factor: String,
    compatible_configurations: BTreeSet<Digest>,
    state: BatteryState,
    assessment: Option<EnergyAssessment>,
    version: u64,
}
impl BatteryAsset {
    pub fn register(
        id: EntityId,
        chemistry: impl Into<String>,
        form_factor: impl Into<String>,
        compatible: impl IntoIterator<Item = Digest>,
    ) -> Result<Self, FleetError> {
        let compatible = compatible.into_iter().collect::<BTreeSet<_>>();
        if compatible.is_empty() || compatible.contains(&[0; 32]) {
            return Err(FleetError::IncompatibleBattery);
        }
        Ok(Self {
            id,
            chemistry: text(chemistry)?,
            form_factor: text(form_factor)?,
            compatible_configurations: compatible,
            state: BatteryState::Registered,
            assessment: None,
            version: 1,
        })
    }
    pub fn make_available(&mut self, assessment: EnergyAssessment) -> Result<(), FleetError> {
        if matches!(
            self.state,
            BatteryState::Quarantined | BatteryState::Retired
        ) || assessment.expires_at <= assessment.assessed_at
            || assessment.uncertainty_wh > assessment.usable_wh
        {
            return Err(FleetError::InvalidEnergy);
        }
        self.assessment = Some(assessment);
        self.state = BatteryState::Available;
        self.bump()
    }
    #[must_use]
    pub fn eligible(
        &self,
        configuration: Digest,
        required_departure_wh: u64,
        minimum_risk_reserve_wh: u64,
        required_power_w: u64,
        maximum_temperature_millicelsius: i64,
        now: DateTime<Utc>,
    ) -> bool {
        self.state == BatteryState::Available
            && self.compatible_configurations.contains(&configuration)
            && self.assessment.as_ref().is_some_and(|a| {
                now >= a.assessed_at
                    && now < a.expires_at
                    && a.usable_wh.saturating_sub(a.uncertainty_wh)
                        >= required_departure_wh.saturating_add(minimum_risk_reserve_wh)
                    && a.continuous_power_w >= required_power_w
                    && a.temperature_millicelsius <= maximum_temperature_millicelsius
            })
    }
    pub fn quarantine(&mut self) -> Result<(), FleetError> {
        if self.state == BatteryState::Retired {
            return Err(FleetError::InvalidTransition);
        }
        self.state = BatteryState::Quarantined;
        self.bump()
    }
    fn bump(&mut self) -> Result<(), FleetError> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or(FleetError::VersionExhausted)?;
        Ok(())
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub fn chemistry(&self) -> &str {
        &self.chemistry
    }
    #[must_use]
    pub fn form_factor(&self) -> &str {
        &self.form_factor
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VehicleState {
    Candidate,
    Enrolled,
    Active,
    Grounded,
    Retired,
}
#[derive(Clone, Debug)]
pub struct Vehicle {
    id: EntityId,
    physical_serial: String,
    device_identity: String,
    tenant: String,
    owner: String,
    configuration_digest: Digest,
    attestation_digest: Digest,
    trust_expires_at: DateTime<Utc>,
    maintenance_due_at: DateTime<Utc>,
    calibration_due_at: DateTime<Utc>,
    state: VehicleState,
    grounding: Option<String>,
    version: u64,
}
impl Vehicle {
    #[allow(clippy::too_many_arguments)]
    pub fn enroll(
        id: EntityId,
        physical_serial: impl Into<String>,
        device_identity: impl Into<String>,
        tenant: impl Into<String>,
        owner: impl Into<String>,
        configuration_digest: Digest,
        attestation_digest: Digest,
        trust_expires_at: DateTime<Utc>,
        maintenance_due_at: DateTime<Utc>,
        calibration_due_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Result<Self, FleetError> {
        if configuration_digest == [0; 32]
            || attestation_digest == [0; 32]
            || [trust_expires_at, maintenance_due_at, calibration_due_at]
                .into_iter()
                .any(|v| v <= now)
        {
            return Err(FleetError::AttestationMismatch);
        }
        Ok(Self {
            id,
            physical_serial: text(physical_serial)?,
            device_identity: text(device_identity)?,
            tenant: text(tenant)?,
            owner: text(owner)?,
            configuration_digest,
            attestation_digest,
            trust_expires_at,
            maintenance_due_at,
            calibration_due_at,
            state: VehicleState::Enrolled,
            grounding: None,
            version: 1,
        })
    }
    pub fn activate(&mut self, configuration: &Configuration) -> Result<(), FleetError> {
        if self.state != VehicleState::Enrolled
            || configuration.digest() != self.configuration_digest
            || !configuration.installed()
        {
            return Err(FleetError::IncompatibleConfiguration);
        }
        self.state = VehicleState::Active;
        self.bump()
    }
    pub fn ground(&mut self, reason: impl Into<String>) -> Result<(), FleetError> {
        if self.state == VehicleState::Retired {
            return Err(FleetError::InvalidTransition);
        }
        self.grounding = Some(text(reason)?);
        self.state = VehicleState::Grounded;
        self.bump()
    }
    pub fn clear_grounding(
        &mut self,
        resolved_cause: &str,
        evidence: Digest,
        authorized: bool,
    ) -> Result<(), FleetError> {
        if self.state != VehicleState::Grounded
            || self.grounding.as_deref() != Some(resolved_cause)
            || evidence == [0; 32]
            || !authorized
        {
            return Err(FleetError::InvalidClearance);
        }
        self.grounding = None;
        self.state = VehicleState::Active;
        self.bump()
    }
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn allocatable(
        &self,
        configuration: &Configuration,
        capability: &CapabilityRecord,
        health: &HealthAssessment,
        battery: &BatteryAsset,
        odd: Digest,
        energy: &MissionEnergyRequirement,
        now: DateTime<Utc>,
    ) -> bool {
        self.state == VehicleState::Active
            && configuration.installed()
            && configuration.digest() == self.configuration_digest
            && self.attestation_digest != [0; 32]
            && now < self.trust_expires_at
            && now < self.maintenance_due_at
            && now < self.calibration_due_at
            && capability.eligible(&self.id, self.configuration_digest, odd, now)
            && health.healthy_at(&self.id, energy.minimum_health_quality_bps, now)
            && battery.eligible(
                self.configuration_digest,
                energy.departure_wh,
                energy.minimum_risk_reserve_wh,
                energy.required_power_w,
                energy.maximum_temperature_millicelsius,
                now,
            )
    }
    fn bump(&mut self) -> Result<(), FleetError> {
        self.version = self
            .version
            .checked_add(1)
            .ok_or(FleetError::VersionExhausted)?;
        Ok(())
    }
    #[must_use]
    pub fn id(&self) -> &EntityId {
        &self.id
    }
    #[must_use]
    pub fn identity_binding(&self) -> (&str, &str, &str, &str) {
        (
            &self.physical_serial,
            &self.device_identity,
            &self.tenant,
            &self.owner,
        )
    }
    #[must_use]
    pub fn version(&self) -> u64 {
        self.version
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MissionEnergyRequirement {
    pub departure_wh: u64,
    pub minimum_risk_reserve_wh: u64,
    pub required_power_w: u64,
    pub maximum_temperature_millicelsius: i64,
    pub minimum_health_quality_bps: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FleetEvent {
    VehicleRegistered { vehicle_id: EntityId },
    VehicleGrounded { vehicle_id: EntityId },
    VehicleCleared { vehicle_id: EntityId },
    BatteryEligibilityChanged { battery_id: EntityId },
    CapabilityAttested { capability_id: EntityId },
    FleetCellChanged { cell_id: EntityId, epoch: u64 },
}
pub trait Repository<A> {
    type Error;
    fn load(&self, id: &EntityId) -> Result<Option<A>, Self::Error>;
    fn save(
        &self,
        aggregate: &A,
        expected_version: u64,
        events: &[FleetEvent],
    ) -> Result<(), Self::Error>;
}
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum FleetError {
    #[error("invalid governed field")]
    InvalidField,
    #[error("invalid content digest")]
    InvalidDigest,
    #[error("invalid aggregate transition")]
    InvalidTransition,
    #[error("configuration is incompatible or unsigned")]
    IncompatibleConfiguration,
    #[error("attestation does not match configuration")]
    AttestationMismatch,
    #[error("capability evidence is invalid")]
    InvalidCapability,
    #[error("health assessment is invalid")]
    InvalidHealth,
    #[error("battery is incompatible")]
    IncompatibleBattery,
    #[error("energy assessment or uncertainty is invalid")]
    InvalidEnergy,
    #[error("grounding clearance is invalid")]
    InvalidClearance,
    #[error("aggregate version exhausted")]
    VersionExhausted,
    #[error("stale fleet cell epoch or fencing token")]
    StaleEpoch,
    #[error("fleet cell capacity exceeded")]
    CellCapacity,
    #[error("membership is ambiguous")]
    AmbiguousMembership,
}
