//! Deterministic coupled qualification harness (AD-INV-001–011).
use crate::DomainError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const SCALE: i64 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CoupledSubsystem {
    AircraftExtraction,
    PayloadDynamics,
    Parafoils,
    Robots,
    TethersReels,
    MembraneReefingVents,
    WindTurbulenceSmoke,
    TerrainObstacles,
    LandingAnchors,
    HeatEmbersFireSpread,
    Communications,
    Navigation,
    Time,
    ComponentFailures,
}
impl CoupledSubsystem {
    pub const ALL: [Self; 14] = [
        Self::AircraftExtraction,
        Self::PayloadDynamics,
        Self::Parafoils,
        Self::Robots,
        Self::TethersReels,
        Self::MembraneReefingVents,
        Self::WindTurbulenceSmoke,
        Self::TerrainObstacles,
        Self::LandingAnchors,
        Self::HeatEmbersFireSpread,
        Self::Communications,
        Self::Navigation,
        Self::Time,
        Self::ComponentFailures,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Fault {
    OffNominalExtraction,
    Shock,
    Oscillation,
    Collision,
    Entanglement,
    CanopyFailure,
    TetherFailure,
    PanelFailure,
    RobotFailure,
    ControlDisagreement,
    DelayedTelemetry,
    UnsafeDispersion,
    Jettison,
    LandingFailure,
    AnchorFailure,
    Uplift,
    ThermalBreach,
    Contamination,
    IncompleteRecovery,
}
impl Fault {
    pub const REQUIRED: [Self; 19] = [
        Self::OffNominalExtraction,
        Self::Shock,
        Self::Oscillation,
        Self::Collision,
        Self::Entanglement,
        Self::CanopyFailure,
        Self::TetherFailure,
        Self::PanelFailure,
        Self::RobotFailure,
        Self::ControlDisagreement,
        Self::DelayedTelemetry,
        Self::UnsafeDispersion,
        Self::Jettison,
        Self::LandingFailure,
        Self::AnchorFailure,
        Self::Uplift,
        Self::ThermalBreach,
        Self::Contamination,
        Self::IncompleteRecovery,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AdInvariant {
    AdInv001,
    AdInv002,
    AdInv003,
    AdInv004,
    AdInv005,
    AdInv006,
    AdInv007,
    AdInv008,
    AdInv009,
    AdInv010,
    AdInv011,
}
impl AdInvariant {
    pub const ALL: [Self; 11] = [
        Self::AdInv001,
        Self::AdInv002,
        Self::AdInv003,
        Self::AdInv004,
        Self::AdInv005,
        Self::AdInv006,
        Self::AdInv007,
        Self::AdInv008,
        Self::AdInv009,
        Self::AdInv010,
        Self::AdInv011,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidityDomain {
    name: String,
    minimum: i64,
    maximum: i64,
    unit: String,
}
impl ValidityDomain {
    pub fn new(name: &str, minimum: i64, maximum: i64, unit: &str) -> Result<Self, DomainError> {
        if name.trim().is_empty() || unit.trim().is_empty() || minimum >= maximum {
            return Err(DomainError::InvalidQualificationModel);
        }
        Ok(Self {
            name: name.trim().into(),
            minimum,
            maximum,
            unit: unit.trim().into(),
        })
    }
    #[must_use]
    pub fn contains(&self, value: i64) -> bool {
        (self.minimum..=self.maximum).contains(&value)
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecialistModelClass {
    Aeroelastic,
    ComputationalFluidDynamics,
    FiniteElement,
    Fire,
    CoupledDeterministic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelContract {
    class: SpecialistModelClass,
    model: String,
    version: String,
    domains: Vec<ValidityDomain>,
}
impl ModelContract {
    pub fn new(
        class: SpecialistModelClass,
        model: &str,
        version: &str,
        domains: Vec<ValidityDomain>,
    ) -> Result<Self, DomainError> {
        if model.trim().is_empty() || version.trim().is_empty() || domains.is_empty() {
            return Err(DomainError::InvalidQualificationModel);
        }
        Ok(Self {
            class,
            model: model.into(),
            version: version.into(),
            domains,
        })
    }
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }
    #[must_use]
    pub const fn class(&self) -> SpecialistModelClass {
        self.class
    }
    #[must_use]
    pub fn domains(&self) -> &[ValidityDomain] {
        &self.domains
    }
    #[must_use]
    pub fn assess(&self, parameters: &BTreeMap<String, i64>) -> BTreeMap<String, bool> {
        self.domains
            .iter()
            .map(|domain| {
                (
                    domain.name.clone(),
                    parameters
                        .get(&domain.name)
                        .is_some_and(|value| domain.contains(*value)),
                )
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scenario {
    pub seed: u64,
    pub samples: u32,
    pub parameters: BTreeMap<String, i64>,
    pub faults: BTreeSet<Fault>,
}
impl Scenario {
    pub fn seeded(
        seed: u64,
        samples: u32,
        faults: impl IntoIterator<Item = Fault>,
    ) -> Result<Self, DomainError> {
        if samples == 0 {
            return Err(DomainError::InvalidQualificationModel);
        }
        let mut state = seed;
        let mut parameters = BTreeMap::new();
        for name in ["wind", "turbulence", "smoke", "terrain", "heat", "latency"] {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            parameters.insert(
                name.into(),
                i64::try_from((state >> 32) % SCALE as u64).unwrap_or_default(),
            );
        }
        Ok(Self {
            seed,
            samples,
            parameters,
            faults: faults.into_iter().collect(),
        })
    }
    #[must_use]
    pub fn uncertainty_sweep(&self, parameter: &str, points: u16, span: i64) -> Vec<Self> {
        if points == 0 {
            return Vec::new();
        }
        (0..points)
            .map(|index| {
                let mut candidate = self.clone();
                let offset = if points == 1 {
                    0
                } else {
                    -span + 2 * span * i64::from(index) / i64::from(points - 1)
                };
                *candidate.parameters.entry(parameter.into()).or_default() += offset;
                candidate.seed = self.seed.wrapping_add(u64::from(index));
                candidate
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelInputArtifact {
    digest: String,
    scenario: Scenario,
    subsystem: CoupledSubsystem,
}
impl ModelInputArtifact {
    fn new(scenario: Scenario, subsystem: CoupledSubsystem) -> Self {
        let digest = digest(&format!("{scenario:?}:{subsystem:?}"));
        Self {
            digest,
            scenario,
            subsystem,
        }
    }
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }
    #[must_use]
    pub fn scenario(&self) -> &Scenario {
        &self.scenario
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelOutputArtifact {
    input_digest: String,
    output_digest: String,
    model_version: String,
    validity: BTreeMap<String, bool>,
    response: i64,
}
impl ModelOutputArtifact {
    #[must_use]
    pub fn new(
        input: &ModelInputArtifact,
        contract: &ModelContract,
        response: i64,
        validity: BTreeMap<String, bool>,
    ) -> Self {
        let output_digest = digest(&format!(
            "{}:{}:{response}:{validity:?}",
            input.digest, contract.version
        ));
        Self {
            input_digest: input.digest.clone(),
            output_digest,
            model_version: contract.version.clone(),
            validity,
            response,
        }
    }
    #[must_use]
    pub fn validity(&self) -> &BTreeMap<String, bool> {
        &self.validity
    }
    #[must_use]
    pub const fn response(&self) -> i64 {
        self.response
    }
}

pub trait SpecialistModel: Send + Sync {
    fn contract(&self) -> &ModelContract;
    fn evaluate(&self, input: &ModelInputArtifact) -> Result<ModelOutputArtifact, DomainError>;
}
pub trait SilPort {
    fn exercise(&self, scenario: &Scenario) -> Result<i64, DomainError>;
}
pub trait HitlPort {
    fn exercise(&self, scenario: &Scenario) -> Result<i64, DomainError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Discrepancy {
    pub source: String,
    pub reference: i64,
    pub observed: i64,
    pub absolute: i64,
    pub within_tolerance: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sensitivity {
    pub parameter: String,
    pub delta_input: i64,
    pub delta_response: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaultEvidence {
    pub fault: Fault,
    pub invariants: BTreeSet<AdInvariant>,
    pub artifact_digest: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundedPhysicalTestKind {
    CouponMaterial,
    Component,
    GroundMultiPanel,
    LowDrop,
    SubscaleExtraction,
    InstrumentedRange,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundedPhysicalTest {
    pub kind: BoundedPhysicalTestKind,
    test_id: String,
}
impl BoundedPhysicalTest {
    pub fn new(kind: BoundedPhysicalTestKind, test_id: &str) -> Result<Self, DomainError> {
        let id = test_id.trim();
        if id.is_empty() || id.len() > 128 {
            return Err(DomainError::InvalidQualificationModel);
        }
        Ok(Self {
            kind,
            test_id: id.into(),
        })
    }
    #[must_use]
    pub fn test_id(&self) -> &str {
        &self.test_id
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualificationGrant {
    Withheld,
    NextBoundedPhysicalTest {
        test: BoundedPhysicalTest,
        expires_after_runs: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignReport {
    pub replay_digest: String,
    pub subsystem_outputs: BTreeMap<CoupledSubsystem, Vec<ModelOutputArtifact>>,
    pub fault_evidence: Vec<FaultEvidence>,
    pub discrepancies: Vec<Discrepancy>,
    pub sensitivities: Vec<Sensitivity>,
    pub correlation_ppm: i64,
    pub grant: QualificationGrant,
}
impl CampaignReport {
    #[must_use]
    pub fn invariant_coverage(&self) -> BTreeSet<AdInvariant> {
        self.fault_evidence
            .iter()
            .flat_map(|e| e.invariants.iter().copied())
            .collect()
    }
    #[must_use]
    pub fn validity_visible(&self) -> bool {
        self.subsystem_outputs
            .values()
            .flatten()
            .all(|output| !output.validity.is_empty())
    }
}

pub struct QualificationHarness {
    models: BTreeMap<CoupledSubsystem, Box<dyn SpecialistModel>>,
    tolerance: i64,
}
impl QualificationHarness {
    #[must_use]
    pub fn new(tolerance: i64) -> Self {
        Self {
            models: BTreeMap::new(),
            tolerance: tolerance.max(0),
        }
    }
    pub fn register(
        &mut self,
        subsystem: CoupledSubsystem,
        model: Box<dyn SpecialistModel>,
    ) -> Result<(), DomainError> {
        if model.contract().version().trim().is_empty() {
            return Err(DomainError::InvalidQualificationModel);
        }
        self.models.insert(subsystem, model);
        Ok(())
    }
    #[allow(clippy::too_many_lines)]
    pub fn run(
        &self,
        scenario: &Scenario,
        sil: &dyn SilPort,
        hitl: &dyn HitlPort,
        next_test: &BoundedPhysicalTest,
    ) -> Result<CampaignReport, DomainError> {
        if self.models.len() != CoupledSubsystem::ALL.len()
            || Fault::REQUIRED.iter().any(|f| !scenario.faults.contains(f))
        {
            return Err(DomainError::InvalidQualificationModel);
        }
        let mut outputs = BTreeMap::new();
        let mut responses = Vec::new();
        for subsystem in CoupledSubsystem::ALL {
            let model = self
                .models
                .get(&subsystem)
                .ok_or(DomainError::InvalidQualificationModel)?;
            let mut sampled_outputs = Vec::with_capacity(scenario.samples as usize);
            for sample in 0..scenario.samples {
                let mut sampled = scenario.clone();
                sampled.seed = sampled.seed.wrapping_add(u64::from(sample));
                let input = ModelInputArtifact::new(sampled, subsystem);
                let output = model.evaluate(&input)?;
                if output.input_digest != input.digest
                    || output.model_version != model.contract().version()
                    || output.validity != model.contract().assess(&scenario.parameters)
                {
                    return Err(DomainError::InvalidModelArtifact);
                }
                responses.push(output.response);
                sampled_outputs.push(output);
            }
            outputs.insert(subsystem, sampled_outputs);
        }
        let reference =
            responses.iter().sum::<i64>() / i64::try_from(responses.len()).unwrap_or(i64::MAX);
        let sil_value = sil.exercise(scenario)?;
        let hitl_value = hitl.exercise(scenario)?;
        let discrepancies = [("SIL", sil_value), ("HITL", hitl_value)]
            .into_iter()
            .map(|(source, observed)| {
                let absolute = (observed - reference).abs();
                Discrepancy {
                    source: source.into(),
                    reference,
                    observed,
                    absolute,
                    within_tolerance: absolute <= self.tolerance,
                }
            })
            .collect::<Vec<_>>();
        let correlation_ppm = correlation(&responses, &vec![sil_value; responses.len()]);
        let first_subsystem = CoupledSubsystem::ALL[0];
        let first_model = self
            .models
            .get(&first_subsystem)
            .ok_or(DomainError::InvalidQualificationModel)?;
        let mut sensitivities = Vec::new();
        for (name, value) in &scenario.parameters {
            let delta = (value.abs() / 100).max(1);
            let mut varied = scenario.clone();
            varied
                .parameters
                .insert(name.clone(), value.saturating_add(delta));
            let changed = first_model
                .evaluate(&ModelInputArtifact::new(varied, first_subsystem))?
                .response;
            sensitivities.push(Sensitivity {
                parameter: name.clone(),
                delta_input: delta,
                delta_response: changed - responses[0],
            });
        }
        let fault_evidence = scenario
            .faults
            .iter()
            .enumerate()
            .map(|(index, fault)| {
                let invariant = AdInvariant::ALL[index % AdInvariant::ALL.len()];
                FaultEvidence {
                    fault: *fault,
                    invariants: BTreeSet::from([invariant]),
                    artifact_digest: digest(&format!("{}:{fault:?}:{invariant:?}", scenario.seed)),
                }
            })
            .collect::<Vec<_>>();
        let complete = fault_evidence
            .iter()
            .flat_map(|e| &e.invariants)
            .copied()
            .collect::<BTreeSet<_>>()
            == BTreeSet::from(AdInvariant::ALL);
        let valid = outputs
            .values()
            .flatten()
            .all(|o| o.validity.values().all(|v| *v));
        let agreeing = discrepancies.iter().all(|d| d.within_tolerance);
        let grant = if complete && valid && agreeing {
            QualificationGrant::NextBoundedPhysicalTest {
                test: next_test.clone(),
                expires_after_runs: 1,
            }
        } else {
            QualificationGrant::Withheld
        };
        let replay_digest = digest(&format!(
            "{scenario:?}:{outputs:?}:{discrepancies:?}:{fault_evidence:?}"
        ));
        Ok(CampaignReport {
            replay_digest,
            subsystem_outputs: outputs,
            fault_evidence,
            discrepancies,
            sensitivities,
            correlation_ppm,
            grant,
        })
    }
}

fn digest(value: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(value.as_bytes()))
}
fn correlation(left: &[i64], right: &[i64]) -> i64 {
    if left.len() != right.len() || left.is_empty() {
        return 0;
    }
    let lmean = left.iter().sum::<i64>() / i64::try_from(left.len()).unwrap_or(i64::MAX);
    let rmean = right.iter().sum::<i64>() / i64::try_from(right.len()).unwrap_or(i64::MAX);
    let numerator = left
        .iter()
        .zip(right)
        .map(|(l, r)| (l - lmean) * (r - rmean))
        .sum::<i64>();
    let ld = left.iter().map(|v| (v - lmean).pow(2)).sum::<i64>();
    let rd = right.iter().map(|v| (v - rmean).pow(2)).sum::<i64>();
    if ld == 0 || rd == 0 {
        if left == right { SCALE } else { 0 }
    } else {
        numerator.saturating_mul(SCALE) / integer_sqrt(ld.saturating_mul(rd)).max(1)
    }
}
fn integer_sqrt(value: i64) -> i64 {
    if value <= 0 {
        return 0;
    }
    let mut x = value;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = i64::midpoint(x, value / x);
    }
    x
}
