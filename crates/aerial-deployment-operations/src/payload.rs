//! Payload/loadmaster boundary. All limits originate in configuration-controlled evidence.
use crate::{AircraftConfigurationId, ComponentId, DomainError, EvidenceRef, PayloadManifestId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

fn finite_nonnegative(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Mass {
    nominal_kg: f64,
    uncertainty_kg: f64,
}
impl Mass {
    pub fn new(nominal_kg: f64, uncertainty_kg: f64) -> Result<Self, DomainError> {
        if !finite_nonnegative(nominal_kg)
            || !finite_nonnegative(uncertainty_kg)
            || uncertainty_kg > nominal_kg
        {
            return Err(DomainError::InvalidQuantity);
        }
        Ok(Self {
            nominal_kg,
            uncertainty_kg,
        })
    }
    #[must_use]
    pub const fn nominal_kg(self) -> f64 {
        self.nominal_kg
    }
    #[must_use]
    pub const fn uncertainty_kg(self) -> f64 {
        self.uncertainty_kg
    }
    #[must_use]
    pub fn upper_kg(self) -> f64 {
        self.nominal_kg + self.uncertainty_kg
    }
    fn lower_kg(self) -> f64 {
        self.nominal_kg - self.uncertainty_kg
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position3 {
    pub longitudinal_m: f64,
    pub lateral_m: f64,
    pub vertical_m: f64,
}
impl Position3 {
    pub fn new(longitudinal_m: f64, lateral_m: f64, vertical_m: f64) -> Result<Self, DomainError> {
        if ![longitudinal_m, lateral_m, vertical_m]
            .into_iter()
            .all(f64::is_finite)
        {
            return Err(DomainError::InvalidQuantity);
        }
        Ok(Self {
            longitudinal_m,
            lateral_m,
            vertical_m,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PackedGeometry {
    pub length_m: f64,
    pub width_m: f64,
    pub height_m: f64,
}
impl PackedGeometry {
    pub fn new(length_m: f64, width_m: f64, height_m: f64) -> Result<Self, DomainError> {
        if ![length_m, width_m, height_m]
            .into_iter()
            .all(|v| v.is_finite() && v > 0.0)
        {
            return Err(DomainError::InvalidQuantity);
        }
        Ok(Self {
            length_m,
            width_m,
            height_m,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentLoad {
    pub component: ComponentId,
    serial: String,
    pub mass: Mass,
    pub position: Position3,
}
impl ComponentLoad {
    pub fn new(
        component: ComponentId,
        serial: &str,
        mass: Mass,
        position: Position3,
    ) -> Result<Self, DomainError> {
        let serial = serial.trim();
        if serial.is_empty() || serial.len() > 128 {
            return Err(DomainError::InvalidComponent);
        }
        Ok(Self {
            component,
            serial: serial.to_owned(),
            mass,
            position,
        })
    }
    #[must_use]
    pub fn serial(&self) -> &str {
        &self.serial
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoadKind {
    Floor {
        station: String,
        maximum_kg_per_m2: f64,
    },
    Ramp {
        station: String,
        maximum_kg: f64,
    },
    RollerPoint {
        station: String,
        maximum_kg: f64,
    },
    RollerDistributed {
        station: String,
        maximum_kg_per_m: f64,
    },
    Opening {
        interface: String,
        maximum_kn: f64,
    },
}
impl LoadKind {
    fn valid(&self) -> bool {
        match self {
            Self::Floor {
                station,
                maximum_kg_per_m2,
            } => !station.trim().is_empty() && finite_nonnegative(*maximum_kg_per_m2),
            Self::Ramp {
                station,
                maximum_kg,
            }
            | Self::RollerPoint {
                station,
                maximum_kg,
            } => !station.trim().is_empty() && finite_nonnegative(*maximum_kg),
            Self::RollerDistributed {
                station,
                maximum_kg_per_m,
            } => !station.trim().is_empty() && finite_nonnegative(*maximum_kg_per_m),
            Self::Opening {
                interface,
                maximum_kn,
            } => !interface.trim().is_empty() && finite_nonnegative(*maximum_kn),
        }
    }
    fn identity_and_limit(&self) -> (&str, u8, f64) {
        match self {
            Self::Floor {
                station,
                maximum_kg_per_m2,
            } => (station, 0, *maximum_kg_per_m2),
            Self::Ramp {
                station,
                maximum_kg,
            } => (station, 1, *maximum_kg),
            Self::RollerPoint {
                station,
                maximum_kg,
            } => (station, 2, *maximum_kg),
            Self::RollerDistributed {
                station,
                maximum_kg_per_m,
            } => (station, 3, *maximum_kg_per_m),
            Self::Opening {
                interface,
                maximum_kn,
            } => (interface, 4, *maximum_kn),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MechanicalInterface {
    pub restraint_revision: String,
    pub extraction_revision: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceLimits {
    pub electrical_revision: String,
    pub maximum_voltage_v: f64,
    pub maximum_current_a: f64,
    pub data_revision: String,
    pub maximum_data_rate_mbps: f64,
    pub environmental_revision: String,
    pub minimum_temperature_c: f64,
    pub maximum_temperature_c: f64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HazardousContent {
    pub proper_shipping_name: String,
    pub quantity_milligrams: u64,
    pub evidence: EvidenceRef,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MassBalance {
    pub total_mass: Mass,
    pub longitudinal_moment_kg_m: f64,
    pub lateral_moment_kg_m: f64,
    pub vertical_moment_kg_m: f64,
    pub longitudinal_cg_m: f64,
    pub longitudinal_cg_interval_m: (f64, f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReconciliationState {
    Planned,
    Inspected,
    Loaded,
    Retained,
    Released,
    Recovered,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayloadManifest {
    id: PayloadManifestId,
    components: Vec<ComponentLoad>,
    pub packed_geometry: Option<PackedGeometry>,
    pub imposed_loads: Vec<LoadKind>,
    pub mechanical_interface: Option<MechanicalInterface>,
    pub service_limits: Option<ServiceLimits>,
    pub hazardous_contents: Vec<HazardousContent>,
    pub loading_stations: Vec<String>,
    state: ReconciliationState,
}
impl PayloadManifest {
    pub fn draft(
        id: PayloadManifestId,
        components: Vec<ComponentLoad>,
    ) -> Result<Self, DomainError> {
        let serials: BTreeSet<_> = components.iter().map(|c| c.serial.as_str()).collect();
        if components.is_empty() || serials.len() != components.len() {
            return Err(DomainError::InvalidComponent);
        }
        Ok(Self {
            id,
            components,
            packed_geometry: None,
            imposed_loads: vec![],
            mechanical_interface: None,
            service_limits: None,
            hazardous_contents: vec![],
            loading_stations: vec![],
            state: ReconciliationState::Planned,
        })
    }
    #[must_use]
    pub fn id(&self) -> &PayloadManifestId {
        &self.id
    }
    #[must_use]
    pub const fn state(&self) -> ReconciliationState {
        self.state
    }
    pub fn set_loading_plan(
        &mut self,
        geometry: PackedGeometry,
        loads: Vec<LoadKind>,
        stations: Vec<String>,
        mechanical: MechanicalInterface,
        services: ServiceLimits,
        hazards: Vec<HazardousContent>,
    ) -> Result<(), DomainError> {
        let unique_stations: BTreeSet<_> = stations.iter().map(|s| s.trim()).collect();
        let services_valid = finite_nonnegative(services.maximum_voltage_v)
            && finite_nonnegative(services.maximum_current_a)
            && finite_nonnegative(services.maximum_data_rate_mbps)
            && services.minimum_temperature_c.is_finite()
            && services.maximum_temperature_c.is_finite()
            && services.minimum_temperature_c <= services.maximum_temperature_c;
        if loads.iter().any(|v| !v.valid())
            || stations.is_empty()
            || stations.iter().any(|s| s.trim().is_empty())
            || unique_stations.len() != stations.len()
            || mechanical.restraint_revision.trim().is_empty()
            || mechanical.extraction_revision.trim().is_empty()
            || services.electrical_revision.trim().is_empty()
            || services.data_revision.trim().is_empty()
            || services.environmental_revision.trim().is_empty()
            || !services_valid
            || hazards.iter().any(|hazard| {
                hazard.proper_shipping_name.trim().is_empty() || hazard.quantity_milligrams == 0
            })
        {
            return Err(DomainError::InvalidConfiguration);
        }
        self.packed_geometry = Some(geometry);
        self.imposed_loads = loads;
        self.loading_stations = stations;
        self.mechanical_interface = Some(mechanical);
        self.service_limits = Some(services);
        self.hazardous_contents = hazards;
        Ok(())
    }
    pub fn mass_balance(&self) -> Result<MassBalance, DomainError> {
        let nominal: f64 = self.components.iter().map(|c| c.mass.nominal_kg).sum();
        let uncertainty: f64 = self.components.iter().map(|c| c.mass.uncertainty_kg).sum();
        if nominal <= 0.0 {
            return Err(DomainError::InvalidQuantity);
        }
        let moment = |axis: fn(&Position3) -> f64| {
            self.components
                .iter()
                .map(|c| c.mass.nominal_kg * axis(&c.position))
                .sum::<f64>()
        };
        let x_moment = moment(|p| p.longitudinal_m);
        if nominal <= uncertainty {
            return Err(DomainError::InvalidQuantity);
        }
        let (min_num, max_num) = self.components.iter().fold((0.0, 0.0), |(low, high), c| {
            let a = c.mass.lower_kg() * c.position.longitudinal_m;
            let b = c.mass.upper_kg() * c.position.longitudinal_m;
            (low + a.min(b), high + a.max(b))
        });
        let candidates = [
            min_num / (nominal - uncertainty),
            min_num / (nominal + uncertainty),
            max_num / (nominal - uncertainty),
            max_num / (nominal + uncertainty),
        ];
        let lower = candidates.into_iter().fold(f64::INFINITY, f64::min);
        let upper = candidates.into_iter().fold(f64::NEG_INFINITY, f64::max);
        Ok(MassBalance {
            total_mass: Mass::new(nominal, uncertainty)?,
            longitudinal_moment_kg_m: x_moment,
            lateral_moment_kg_m: moment(|p| p.lateral_m),
            vertical_moment_kg_m: moment(|p| p.vertical_m),
            longitudinal_cg_m: x_moment / nominal,
            longitudinal_cg_interval_m: (lower, upper),
        })
    }
    pub fn validate_for(
        &self,
        envelope: &LoadEnvelope,
        requested: AircraftInterfaceVersion,
        at: DateTime<Utc>,
    ) -> Result<(), DomainError> {
        if requested != envelope.interface_version {
            return Err(DomainError::InterfaceVersionMismatch);
        }
        if !envelope.evidence.is_current_at(at)
            || self
                .hazardous_contents
                .iter()
                .any(|hazard| !hazard.evidence.is_current_at(at))
        {
            return Err(DomainError::AircraftEvidenceStale);
        }
        let geometry = self
            .packed_geometry
            .ok_or(DomainError::IncompleteLoadingPlan)?;
        let constraints = envelope
            .constraints
            .as_ref()
            .ok_or(DomainError::IncompleteLoadingPlan)?;
        if self.imposed_loads.is_empty()
            || self.loading_stations.is_empty()
            || self.mechanical_interface.is_none()
            || self.service_limits.is_none()
        {
            return Err(DomainError::IncompleteLoadingPlan);
        }
        let balance = self.mass_balance()?;
        if balance.total_mass.upper_kg() > envelope.maximum_payload.nominal_kg() {
            return Err(DomainError::LoadEnvelopeExceeded);
        }
        for actual in &self.imposed_loads {
            let (name, kind, value) = actual.identity_and_limit();
            let approved = envelope
                .limits
                .iter()
                .find(|l| {
                    let (n, k, _) = l.identity_and_limit();
                    n == name && k == kind
                })
                .map(|l| l.identity_and_limit().2);
            if approved.is_none_or(|limit| value > limit) {
                return Err(DomainError::LoadEnvelopeExceeded);
            }
        }
        if geometry.length_m > constraints.maximum_geometry.length_m
            || geometry.width_m > constraints.maximum_geometry.width_m
            || geometry.height_m > constraints.maximum_geometry.height_m
            || balance.longitudinal_cg_interval_m.0 < constraints.longitudinal_cg_range_m.0
            || balance.longitudinal_cg_interval_m.1 > constraints.longitudinal_cg_range_m.1
            || balance.longitudinal_moment_kg_m.abs()
                > constraints.maximum_absolute_moments_kg_m.longitudinal_m
            || balance.lateral_moment_kg_m.abs()
                > constraints.maximum_absolute_moments_kg_m.lateral_m
            || balance.vertical_moment_kg_m.abs()
                > constraints.maximum_absolute_moments_kg_m.vertical_m
            || self.mechanical_interface.as_ref() != Some(&constraints.mechanical_interface)
            || !service_within(self.service_limits.as_ref(), &constraints.service_limits)
            || self
                .loading_stations
                .iter()
                .any(|station| !constraints.loading_stations.contains(station))
            || self.hazardous_contents.iter().any(|hazard| {
                !constraints
                    .permitted_hazardous_contents
                    .contains(&hazard.proper_shipping_name)
            })
        {
            return Err(DomainError::LoadEnvelopeExceeded);
        }
        Ok(())
    }
    pub fn reconcile(
        &mut self,
        next: ReconciliationState,
        observed_serials: &[&str],
    ) -> Result<(), DomainError> {
        let expected: BTreeSet<_> = self.components.iter().map(ComponentLoad::serial).collect();
        let observed: BTreeSet<_> = observed_serials.iter().copied().collect();
        if expected != observed || observed.len() != observed_serials.len() {
            return Err(DomainError::ManifestSubstitution);
        }
        let valid = matches!(
            (self.state, next),
            (ReconciliationState::Planned, ReconciliationState::Inspected)
                | (ReconciliationState::Inspected, ReconciliationState::Loaded)
                | (
                    ReconciliationState::Loaded,
                    ReconciliationState::Retained | ReconciliationState::Released
                )
                | (
                    ReconciliationState::Retained | ReconciliationState::Released,
                    ReconciliationState::Recovered
                )
        );
        if !valid {
            return Err(DomainError::InvalidReconciliation);
        }
        self.state = next;
        Ok(())
    }
}

fn service_within(actual: Option<&ServiceLimits>, approved: &ServiceLimits) -> bool {
    actual.is_some_and(|actual| {
        actual.electrical_revision == approved.electrical_revision
            && actual.data_revision == approved.data_revision
            && actual.environmental_revision == approved.environmental_revision
            && actual.maximum_voltage_v <= approved.maximum_voltage_v
            && actual.maximum_current_a <= approved.maximum_current_a
            && actual.maximum_data_rate_mbps <= approved.maximum_data_rate_mbps
            && actual.minimum_temperature_c >= approved.minimum_temperature_c
            && actual.maximum_temperature_c <= approved.maximum_temperature_c
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AircraftInterfaceVersion {
    pub major: u16,
    pub minor: u16,
}
impl AircraftInterfaceVersion {
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AircraftIdentity {
    pub aircraft_type: String,
    pub tail: String,
    pub configuration: AircraftConfigurationId,
}
impl AircraftIdentity {
    pub fn new(
        aircraft_type: &str,
        tail: &str,
        configuration: AircraftConfigurationId,
    ) -> Result<Self, DomainError> {
        if aircraft_type.trim().is_empty() || tail.trim().is_empty() {
            return Err(DomainError::InvalidConfiguration);
        }
        Ok(Self {
            aircraft_type: aircraft_type.trim().into(),
            tail: tail.trim().into(),
            configuration,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadEnvelope {
    pub aircraft: AircraftIdentity,
    pub interface_version: AircraftInterfaceVersion,
    evidence: EvidenceRef,
    maximum_payload: Mass,
    limits: Vec<LoadKind>,
    constraints: Option<AircraftLoadingConstraints>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AircraftLoadingConstraints {
    pub maximum_geometry: PackedGeometry,
    pub longitudinal_cg_range_m: (f64, f64),
    pub maximum_absolute_moments_kg_m: Position3,
    pub mechanical_interface: MechanicalInterface,
    pub service_limits: ServiceLimits,
    pub loading_stations: BTreeSet<String>,
    pub permitted_hazardous_contents: BTreeSet<String>,
}
impl LoadEnvelope {
    pub fn new(
        aircraft: AircraftIdentity,
        interface_version: AircraftInterfaceVersion,
        evidence: EvidenceRef,
        maximum_payload: Mass,
        limits: Vec<LoadKind>,
    ) -> Result<Self, DomainError> {
        if interface_version.major == 0 || limits.iter().any(|l| !l.valid()) {
            return Err(DomainError::InvalidConfiguration);
        }
        if maximum_payload.uncertainty_kg().abs() > f64::EPSILON {
            return Err(DomainError::InvalidConfiguration);
        }
        Ok(Self {
            aircraft,
            interface_version,
            evidence,
            maximum_payload,
            limits,
            constraints: None,
        })
    }
    pub fn with_constraints(
        mut self,
        constraints: AircraftLoadingConstraints,
    ) -> Result<Self, DomainError> {
        let cg = constraints.longitudinal_cg_range_m;
        let moments = constraints.maximum_absolute_moments_kg_m;
        if !cg.0.is_finite()
            || !cg.1.is_finite()
            || cg.0 > cg.1
            || !finite_nonnegative(moments.longitudinal_m)
            || !finite_nonnegative(moments.lateral_m)
            || !finite_nonnegative(moments.vertical_m)
            || constraints.loading_stations.is_empty()
        {
            return Err(DomainError::InvalidConfiguration);
        }
        self.constraints = Some(constraints);
        Ok(self)
    }
    #[must_use]
    pub fn evidence(&self) -> &EvidenceRef {
        &self.evidence
    }
}

pub trait AerialPayloadInterface: Send + Sync {
    fn interface_version(&self) -> AircraftInterfaceVersion;
    fn approved_envelope(
        &self,
        aircraft: &AircraftIdentity,
        at: DateTime<Utc>,
    ) -> Result<LoadEnvelope, DomainError>;
}

#[derive(Debug, Clone)]
pub struct EvidenceBackedAircraftAdapter {
    envelope: LoadEnvelope,
}
impl EvidenceBackedAircraftAdapter {
    #[must_use]
    pub const fn new(envelope: LoadEnvelope) -> Self {
        Self { envelope }
    }
}
impl AerialPayloadInterface for EvidenceBackedAircraftAdapter {
    fn interface_version(&self) -> AircraftInterfaceVersion {
        self.envelope.interface_version
    }
    fn approved_envelope(
        &self,
        aircraft: &AircraftIdentity,
        at: DateTime<Utc>,
    ) -> Result<LoadEnvelope, DomainError> {
        if aircraft != &self.envelope.aircraft {
            return Err(DomainError::EvidenceMismatch);
        }
        if !self.envelope.evidence.is_current_at(at) {
            return Err(DomainError::AircraftEvidenceStale);
        }
        Ok(self.envelope.clone())
    }
}

/// Non-authoritative aircraft classes available exclusively to tests/fixture builds.
#[cfg(any(test, feature = "test-fixtures"))]
pub mod test_fixtures {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SimulatedAircraftClass {
        Cc177C17Class,
        C130jLm100jClass,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixture_aircraft_classes_are_explicitly_simulated() {
        assert_ne!(
            test_fixtures::SimulatedAircraftClass::Cc177C17Class,
            test_fixtures::SimulatedAircraftClass::C130jLm100jClass
        );
    }
    #[test]
    fn rejects_dimensionally_invalid_values() {
        assert_eq!(Mass::new(1.0, 2.0), Err(DomainError::InvalidQuantity));
        assert!(PackedGeometry::new(1.0, 0.0, 1.0).is_err());
    }
}
