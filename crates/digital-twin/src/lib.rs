#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! Deterministic, evidence-producing cyber-physical digital twin.
//!
//! This package is a validation adapter. It owns no operational aggregate and
//! cannot issue commands or grant aircraft, field, or operational authority.

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// Independently versioned models integrated by the twin.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum TwinDomain {
    FireWeatherTerrain,
    Communications,
    Drones,
    GroundRobotsAndTools,
    HabitatsMicrogridsAndChargers,
    Batteries,
    PodsCarriersAndPlatoons,
    Logistics,
    MedicPods,
    Hospitals,
    AerialFireBlanket,
}

impl TwinDomain {
    pub const ALL: [Self; 11] = [
        Self::FireWeatherTerrain,
        Self::Communications,
        Self::Drones,
        Self::GroundRobotsAndTools,
        Self::HabitatsMicrogridsAndChargers,
        Self::Batteries,
        Self::PodsCarriersAndPlatoons,
        Self::Logistics,
        Self::MedicPods,
        Self::Hospitals,
        Self::AerialFireBlanket,
    ];
}

/// Required fault campaign; variants are stable evidence vocabulary.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Fault {
    Nominal,
    NetworkLoss,
    GnssLoss,
    ClockLoss,
    ThermalEvent,
    AuthorityExpiry,
    Collision,
    Intrusion,
    ToolFault,
    CarrierFailure,
    LoadFailure,
    ReleaseFailure,
    Fire,
    Smoke,
    Cold,
    CoupledAerodynamicInstability,
    Entanglement,
    CorrelatedDamage,
    TetherFailure,
    RecoveryFailure,
}

impl Fault {
    pub const ALL: [Self; 19] = [
        Self::NetworkLoss,
        Self::GnssLoss,
        Self::ClockLoss,
        Self::ThermalEvent,
        Self::AuthorityExpiry,
        Self::Collision,
        Self::Intrusion,
        Self::ToolFault,
        Self::CarrierFailure,
        Self::LoadFailure,
        Self::ReleaseFailure,
        Self::Fire,
        Self::Smoke,
        Self::Cold,
        Self::CoupledAerodynamicInstability,
        Self::Entanglement,
        Self::CorrelatedDamage,
        Self::TetherFailure,
        Self::RecoveryFailure,
    ];
}

/// Aerial blanket deployment and independent containment states.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BlanketState {
    Retained,
    Extracted,
    Released,
    FormationAcquired,
    ExpansionReefed,
    Expanded,
    TerrainAligned,
    Anchored,
    ReefedPaused,
    PanelIsolated,
    TetherBreakaway,
    GroundedMinimumRisk,
}

/// Bidirectional traceability required for every scenario.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScenarioLink {
    pub requirement_id: String,
    pub hazard_id: String,
    pub invariant_id: String,
}

impl ScenarioLink {
    pub fn new(
        requirement: impl Into<String>,
        hazard: impl Into<String>,
        invariant: impl Into<String>,
    ) -> Result<Self, TwinError> {
        let value = Self {
            requirement_id: requirement.into(),
            hazard_id: hazard.into(),
            invariant_id: invariant.into(),
        };
        if [&value.requirement_id, &value.hazard_id, &value.invariant_id]
            .iter()
            .any(|v| v.trim().is_empty())
        {
            return Err(TwinError::MissingTraceability);
        }
        Ok(value)
    }
}

/// Stable, seeded scenario definition.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    pub id: String,
    pub seed: u64,
    pub fault: Fault,
    pub link: ScenarioLink,
    pub bounded_cohort_size: u16,
    pub exclusion_zone_clear: bool,
}

impl Scenario {
    pub fn new(
        id: impl Into<String>,
        seed: u64,
        fault: Fault,
        link: ScenarioLink,
    ) -> Result<Self, TwinError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(TwinError::InvalidScenario);
        }
        Ok(Self {
            id,
            seed,
            fault,
            link,
            bounded_cohort_size: 8,
            exclusion_zone_clear: true,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecoveryAccounting {
    pub expected: u16,
    pub recovered: u16,
    pub isolated: u16,
    pub unrecoverable: u16,
}
impl RecoveryAccounting {
    #[must_use]
    pub fn all_components_accounted(&self) -> bool {
        self.expected == self.recovered + self.isolated + self.unrecoverable
    }
}

/// Deterministic scenario observation retained as evidence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScenarioResult {
    pub scenario_id: String,
    pub seed: u64,
    pub fault: Fault,
    pub trace: ScenarioLink,
    pub visited_blanket_states: Vec<BlanketState>,
    pub final_blanket_state: BlanketState,
    pub minimum_risk_reached: bool,
    pub compensation: String,
    pub dispersion_envelope_millimetres: u32,
    pub tick_count: u32,
    pub recovery: RecoveryAccounting,
    pub digest: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Fidelity {
    SoftwareInLoop,
    HardwareInLoop,
}

/// Versioned frame boundary shared by SIL and HITL adapters.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HardwareFrame {
    pub schema: String,
    pub sequence: u32,
    pub monotonic_nanos: u64,
    pub fidelity: Fidelity,
    pub payload_digest: [u8; 32],
}

pub trait HardwarePort {
    fn exchange(&mut self, frame: HardwareFrame) -> Result<HardwareFrame, String>;
}
pub trait EvidenceCapturePort {
    fn capture(&mut self, bytes: &[u8]) -> Result<(), String>;
}

/// Quantified applicability, calibration confidence, and known limitations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValidityClaim {
    pub model_version: String,
    pub temperature_celsius: [f32; 2],
    pub calibration_score: f32,
    pub gaps: Vec<String>,
}

impl ValidityClaim {
    pub fn new(
        model_version: impl Into<String>,
        temperature_celsius: [f32; 2],
        calibration_score: f32,
        gaps: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, TwinError> {
        let value = Self {
            model_version: model_version.into(),
            temperature_celsius,
            calibration_score,
            gaps: gaps.into_iter().map(Into::into).collect(),
        };
        if value.model_version.trim().is_empty()
            || !temperature_celsius.iter().all(|value| value.is_finite())
            || !value.calibration_score.is_finite()
            || !(0.0..=1.0).contains(&value.calibration_score)
            || temperature_celsius[0] >= temperature_celsius[1]
        {
            return Err(TwinError::InvalidValidity);
        }
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PromotionAnswer {
    SimulationEvidenceComplete,
    Incomplete { gaps: Vec<String> },
}

/// Signed, replayable campaign evidence. The signature covers every preceding field.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceBundle {
    pub schema: String,
    pub simulator_version: String,
    pub environment_digest: [u8; 32],
    pub domains: Vec<TwinDomain>,
    pub results: Vec<ScenarioResult>,
    pub validity: ValidityClaim,
    pub signature: Vec<u8>,
}

impl EvidenceBundle {
    fn unsigned_bytes(&self) -> Option<Vec<u8>> {
        let mut copy = self.clone();
        copy.signature.clear();
        serde_json::to_vec(&copy).ok()
    }
    #[must_use]
    pub fn verify(&self, key: &[u8]) -> bool {
        let (Ok(mut mac), Some(bytes)) = (HmacSha256::new_from_slice(key), self.unsigned_bytes())
        else {
            return false;
        };
        mac.update(&bytes);
        mac.verify_slice(&self.signature).is_ok()
    }
    #[must_use]
    pub const fn grants_operational_authority(&self) -> bool {
        false
    }
    #[must_use]
    pub fn promotion_answer(&self, verification_key: &[u8]) -> PromotionAnswer {
        let covered: BTreeSet<_> = self.results.iter().map(|r| r.fault).collect();
        let mut gaps = Vec::new();
        if !self.verify(verification_key) {
            gaps.push("evidence signature is invalid".into());
        }
        let domains: BTreeSet<_> = self.domains.iter().copied().collect();
        if TwinDomain::ALL
            .iter()
            .any(|domain| !domains.contains(domain))
        {
            gaps.push("required twin domain is missing".into());
        }
        for fault in Fault::ALL {
            if !covered.contains(&fault) {
                gaps.push(format!("missing scenario for {fault:?}"));
            }
        }
        if self.validity.calibration_score < 0.8 {
            gaps.push("calibration score below 0.80".into());
        }
        gaps.extend(
            self.validity
                .gaps
                .iter()
                .map(|gap| format!("model validity gap: {gap}")),
        );
        if self
            .results
            .iter()
            .any(|r| !r.minimum_risk_reached || !r.recovery.all_components_accounted())
        {
            gaps.push("minimum-risk or recovery invariant failed".into());
        }
        if gaps.is_empty() {
            PromotionAnswer::SimulationEvidenceComplete
        } else {
            PromotionAnswer::Incomplete { gaps }
        }
    }
}

/// Pure deterministic orchestrator; domain models are identified and version-bound.
#[derive(Clone, Debug)]
pub struct DeterministicTwin {
    version: String,
    domains: Vec<TwinDomain>,
}

impl DeterministicTwin {
    #[must_use]
    pub fn standard() -> Self {
        Self {
            version: "digital-twin/29.1".into(),
            domains: TwinDomain::ALL.to_vec(),
        }
    }
    #[must_use]
    pub fn domains(&self) -> &[TwinDomain] {
        &self.domains
    }

    pub fn run(&self, scenario: &Scenario) -> Result<ScenarioResult, TwinError> {
        if scenario.bounded_cohort_size == 0 || scenario.bounded_cohort_size > 32 {
            return Err(TwinError::UnboundedCohort);
        }
        let (state, compensation) = response_for(scenario.fault, scenario.exclusion_zone_clear);
        let mut rng = scenario.seed ^ stable_u64(scenario.id.as_bytes());
        rng = xorshift(rng);
        let dispersion = 2_000 + u32::try_from(rng % 8_000).map_err(|_| TwinError::NumericRange)?;
        let trace = if scenario.fault == Fault::Nominal {
            vec![
                BlanketState::Retained,
                BlanketState::Extracted,
                BlanketState::Released,
                BlanketState::FormationAcquired,
                BlanketState::ExpansionReefed,
                BlanketState::Expanded,
                BlanketState::TerrainAligned,
                BlanketState::Anchored,
            ]
        } else {
            vec![
                BlanketState::Retained,
                BlanketState::Extracted,
                BlanketState::Released,
                BlanketState::FormationAcquired,
                BlanketState::ExpansionReefed,
                state,
            ]
        };
        let recovery = recovery_for(scenario.bounded_cohort_size, scenario.fault);
        let tick_count = u32::try_from(trace.len()).map_err(|_| TwinError::NumericRange)?;
        let mut result = ScenarioResult {
            scenario_id: scenario.id.clone(),
            seed: scenario.seed,
            fault: scenario.fault,
            trace: scenario.link.clone(),
            visited_blanket_states: trace,
            final_blanket_state: state,
            minimum_risk_reached: scenario.fault != Fault::Nominal,
            compensation: compensation.into(),
            dispersion_envelope_millimetres: dispersion,
            tick_count,
            recovery,
            digest: [0; 32],
        };
        let bytes = serde_json::to_vec(&result).map_err(|_| TwinError::Serialization)?;
        result.digest = Sha256::digest(bytes).into();
        Ok(result)
    }

    pub fn run_with_port(
        &self,
        scenario: &Scenario,
        fidelity: Fidelity,
        hardware: &mut dyn HardwarePort,
        capture: &mut dyn EvidenceCapturePort,
    ) -> Result<ScenarioResult, TwinError> {
        let result = self.run(scenario)?;
        for sequence in 0..result.tick_count {
            let frame = HardwareFrame {
                schema: "wildfire.hitl.frame.v1".into(),
                sequence,
                monotonic_nanos: u64::from(sequence) * 20_000_000,
                fidelity,
                payload_digest: result.digest,
            };
            let returned = hardware.exchange(frame.clone()).map_err(TwinError::Port)?;
            if returned != frame {
                return Err(TwinError::InvalidHardwareFrame);
            }
            let bytes = serde_json::to_vec(&returned).map_err(|_| TwinError::Serialization)?;
            capture.capture(&bytes).map_err(TwinError::Port)?;
        }
        Ok(result)
    }

    pub fn campaign(
        &self,
        scenarios: &[Scenario],
        validity: ValidityClaim,
        key: &[u8],
    ) -> Result<EvidenceBundle, TwinError> {
        if scenarios.is_empty() || key.len() < 16 {
            return Err(TwinError::InvalidEvidence);
        }
        let mut results = scenarios
            .iter()
            .map(|s| self.run(s))
            .collect::<Result<Vec<_>, _>>()?;
        results.sort_by(|a, b| a.scenario_id.cmp(&b.scenario_id));
        let mut environment = Sha256::new();
        environment.update(self.version.as_bytes());
        for d in &self.domains {
            environment.update(format!("{d:?}"));
        }
        let mut bundle = EvidenceBundle {
            schema: "wildfire.digital-twin.evidence.v1".into(),
            simulator_version: self.version.clone(),
            environment_digest: environment.finalize().into(),
            domains: self.domains.clone(),
            results,
            validity,
            signature: Vec::new(),
        };
        let bytes = bundle.unsigned_bytes().ok_or(TwinError::Serialization)?;
        let mut mac = HmacSha256::new_from_slice(key).map_err(|_| TwinError::InvalidEvidence)?;
        mac.update(&bytes);
        bundle.signature = mac.finalize().into_bytes().to_vec();
        Ok(bundle)
    }
}

fn response_for(fault: Fault, exclusion_clear: bool) -> (BlanketState, &'static str) {
    if !exclusion_clear {
        return (
            BlanketState::Retained,
            "inhibit release; clear exclusion zone",
        );
    }
    match fault {
        Fault::Nominal => (
            BlanketState::Anchored,
            "deployment complete; recovery accounting active",
        ),
        Fault::CarrierFailure
        | Fault::LoadFailure
        | Fault::ReleaseFailure
        | Fault::AuthorityExpiry => (
            BlanketState::Retained,
            "retain load; revoke intent; notify command",
        ),
        Fault::CoupledAerodynamicInstability => (
            BlanketState::ReefedPaused,
            "pause expansion; reef and separate cohorts",
        ),
        Fault::Entanglement | Fault::CorrelatedDamage => (
            BlanketState::PanelIsolated,
            "isolate panel; vent section; account components",
        ),
        Fault::TetherFailure => (
            BlanketState::TetherBreakaway,
            "break away into safe-release sector",
        ),
        Fault::RecoveryFailure => (
            BlanketState::TerrainAligned,
            "freeze recovery workflow; escalate unaccounted components",
        ),
        _ => (
            BlanketState::GroundedMinimumRisk,
            "inhibit tools; land or hold; request human review",
        ),
    }
}

fn recovery_for(count: u16, fault: Fault) -> RecoveryAccounting {
    match fault {
        Fault::Entanglement | Fault::CorrelatedDamage => RecoveryAccounting {
            expected: count,
            recovered: count.saturating_sub(1),
            isolated: 1,
            unrecoverable: 0,
        },
        Fault::RecoveryFailure => RecoveryAccounting {
            expected: count,
            recovered: count.saturating_sub(1),
            isolated: 0,
            unrecoverable: 1,
        },
        _ => RecoveryAccounting {
            expected: count,
            recovered: count,
            isolated: 0,
            unrecoverable: 0,
        },
    }
}

fn stable_u64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |v, b| {
        (v ^ u64::from(*b)).wrapping_mul(0x100_0000_01b3)
    })
}
fn xorshift(mut value: u64) -> u64 {
    value ^= value << 13;
    value ^= value >> 7;
    value ^ (value << 17)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum TwinError {
    #[error("scenario traceability is incomplete")]
    MissingTraceability,
    #[error("scenario is invalid")]
    InvalidScenario,
    #[error("cohort is outside the bounded size")]
    UnboundedCohort,
    #[error("validity claim is invalid")]
    InvalidValidity,
    #[error("evidence input is invalid")]
    InvalidEvidence,
    #[error("serialization failed")]
    Serialization,
    #[error("hardware frame is invalid")]
    InvalidHardwareFrame,
    #[error("hardware/evidence port failed: {0}")]
    Port(String),
    #[error("numeric range conversion failed")]
    NumericRange,
}
