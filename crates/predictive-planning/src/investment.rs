//! Reproducible techno-economic scenarios. Outputs are advisory decision support.
use crate::PlanningError;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::collections::BTreeSet;

const MAX_FACTS: usize = 100_000;
const MAX_RUNS: u32 = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CostCategory {
    Capital,
    Energy,
    Logistics,
    Maintenance,
    Downtime,
    Staffing,
    Training,
    Communications,
    Compute,
    Outcome,
}
impl CostCategory {
    fn capital(self) -> bool {
        self == Self::Capital
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Strategy {
    HumanOnly,
    RobotAssisted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationalCostFact {
    id: String,
    tenant: String,
    region: String,
    start_year: u16,
    category: CostCategory,
    strategy: Strategy,
    mode: i64,
    low: i64,
    high: i64,
    content_digest: String,
}
impl OperationalCostFact {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &str,
        tenant: &str,
        region: &str,
        start_year: u16,
        category: CostCategory,
        strategy: Strategy,
        mode: i64,
        low: i64,
        high: i64,
    ) -> Result<Self, PlanningError> {
        if [id, tenant, region].contains(&"") || low < 0 || low > mode || mode > high {
            return Err(PlanningError::InvalidScenario("invalid operational fact"));
        }
        let content_digest = hash_json(&(
            id, tenant, region, start_year, category, strategy, mode, low, high,
        ))?;
        Ok(Self {
            id: id.into(),
            tenant: tenant.into(),
            region: region.into(),
            start_year,
            category,
            strategy,
            mode,
            low,
            high,
            content_digest,
        })
    }
    pub fn content_digest(&self) -> &str {
        &self.content_digest
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovedCausalMethod {
    pub method_id: String,
    pub approval_id: String,
    pub evidence_digest: String,
}
impl ApprovedCausalMethod {
    pub fn new(method: &str, approval: &str, digest: &str) -> Result<Self, PlanningError> {
        if method.is_empty()
            || approval.is_empty()
            || digest.len() != 64
            || !digest.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(PlanningError::InvalidScenario("invalid causal evidence"));
        }
        Ok(Self {
            method_id: method.into(),
            approval_id: approval.into(),
            evidence_digest: digest.to_ascii_lowercase(),
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counterfactual {
    pub version: u64,
    pub human_only: bool,
    pub approved_causal_method: Option<ApprovedCausalMethod>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestmentState {
    Draft,
    Calibrated,
    Simulated,
    Reviewed,
    Published,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvestmentScenario {
    id: String,
    version: u64,
    tenant: String,
    region: String,
    currency: String,
    base_year: i32,
    horizon: u16,
    discount_bps: u16,
    seed: u64,
    runs: u32,
    counterfactual: Counterfactual,
    facts: Vec<OperationalCostFact>,
    model_version: String,
    rng_version: String,
    content_digest: String,
    state: InvestmentState,
    result_digest: Option<String>,
    protected_units: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    pub p05: i64,
    pub median: i64,
    pub p95: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TcoBreakdown {
    pub capital: i64,
    pub recurring: i64,
    pub total: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sensitivity {
    pub fact_id: String,
    pub npv_swing: i64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IrrStatus {
    Unique { percent: f64 },
    UndefinedNoSignChange,
    MultipleRoots,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaybackStatus {
    Year(u16),
    Never,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoundingPolicy {
    NearestMinorUnitHalfAwayFromZero,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BreakEven {
    pub status: PaybackStatus,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitCosts {
    pub robot_cost_minor_units: i64,
    pub human_cost_minor_units: i64,
    pub denominator_units: u64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvestmentResult {
    pub scenario_id: String,
    pub scenario_version: u64,
    pub seed: u64,
    pub runs: u32,
    pub currency: String,
    pub base_year: i32,
    pub counterfactual_version: u64,
    pub npv: Range,
    pub irr: IrrStatus,
    pub payback: PaybackStatus,
    pub break_even: BreakEven,
    pub robot_tco: TcoBreakdown,
    pub human_tco: TcoBreakdown,
    pub sensitivity: Vec<Sensitivity>,
    pub causal_claim: bool,
    pub limitations: Vec<String>,
    pub scenario_digest: String,
    pub result_digest: String,
    pub rounding: RoundingPolicy,
    pub unit_costs: Option<UnitCosts>,
}

impl InvestmentScenario {
    #[allow(clippy::too_many_arguments)]
    pub fn define(
        id: &str,
        version: u64,
        tenant: &str,
        region: &str,
        currency: &str,
        base_year: i32,
        horizon: u16,
        discount_bps: u16,
        seed: u64,
        runs: u32,
        counterfactual: Counterfactual,
        facts: Vec<OperationalCostFact>,
    ) -> Result<Self, PlanningError> {
        if [id, tenant, region, currency].contains(&"")
            || version == 0
            || horizon == 0
            || horizon > 100
            || discount_bps > 10_000
            || seed == 0
            || !(2..=MAX_RUNS).contains(&runs)
            || facts.is_empty()
            || facts.len() > MAX_FACTS
            || counterfactual.version == 0
            || !counterfactual.human_only
        {
            return Err(PlanningError::InvalidScenario(
                "invalid investment scenario",
            ));
        }
        let mut ids = BTreeSet::new();
        if facts.iter().any(|f| {
            f.tenant != tenant || f.region != region || f.start_year > horizon || !ids.insert(&f.id)
        }) {
            return Err(PlanningError::InvalidScenario(
                "fact scope, horizon, or identity mismatch",
            ));
        }
        let model_version = "roi-model-v1".to_owned();
        let rng_version = "splitmix64-triangular-v1".to_owned();
        let content_digest = hash_json(&(
            id,
            version,
            tenant,
            region,
            currency,
            base_year,
            horizon,
            discount_bps,
            seed,
            runs,
            &counterfactual,
            &facts,
            &model_version,
            &rng_version,
        ))?;
        Ok(Self {
            id: id.into(),
            version,
            tenant: tenant.into(),
            region: region.into(),
            currency: currency.into(),
            base_year,
            horizon,
            discount_bps,
            seed,
            runs,
            counterfactual,
            facts,
            model_version,
            rng_version,
            content_digest,
            state: InvestmentState::Draft,
            result_digest: None,
            protected_units: None,
        })
    }
    pub fn content_digest(&self) -> &str {
        &self.content_digest
    }
    pub fn state(&self) -> InvestmentState {
        self.state
    }
    pub fn with_protected_units(mut self, units: u64) -> Result<Self, PlanningError> {
        if units == 0 {
            return Err(PlanningError::InvalidScenario("units must be positive"));
        }
        self.protected_units = Some(units);
        Ok(self)
    }
    pub fn calibrate(&mut self) -> Result<(), PlanningError> {
        if self.state != InvestmentState::Draft {
            return Err(PlanningError::InvalidTransition);
        }
        self.state = InvestmentState::Calibrated;
        Ok(())
    }
    pub fn mark_simulated(&mut self, result: &InvestmentResult) -> Result<(), PlanningError> {
        if self.state != InvestmentState::Calibrated
            || result.scenario_digest != self.content_digest
        {
            return Err(PlanningError::InvalidTransition);
        }
        self.result_digest = Some(result.result_digest.clone());
        self.state = InvestmentState::Simulated;
        Ok(())
    }
    pub fn review(&mut self) -> Result<(), PlanningError> {
        if self.state != InvestmentState::Simulated {
            return Err(PlanningError::InvalidTransition);
        }
        self.state = InvestmentState::Reviewed;
        Ok(())
    }
    pub fn publish(&mut self) -> Result<&str, PlanningError> {
        if self.state != InvestmentState::Reviewed {
            return Err(PlanningError::InvalidTransition);
        }
        self.state = InvestmentState::Published;
        self.result_digest
            .as_deref()
            .ok_or(PlanningError::InvalidScenario("missing result digest"))
    }
    pub fn run(&self) -> Result<InvestmentResult, PlanningError> {
        if self.state != InvestmentState::Calibrated {
            return Err(PlanningError::InvalidTransition);
        }
        let mut rng = SplitMix64(self.seed);
        let mut npvs = Vec::with_capacity(self.runs as usize);
        let mut representative = Vec::new();
        for iteration in 0..self.runs {
            let flows = self.cash_flows(|f| sample(f, &mut rng))?;
            if iteration == 0 {
                representative.clone_from(&flows);
            }
            npvs.push(npv(&flows, self.discount_bps)?);
        }
        npvs.sort_unstable();
        let range = Range {
            p05: quantile(&npvs, 5),
            median: quantile(&npvs, 50),
            p95: quantile(&npvs, 95),
        };
        let robot = self.tco(Strategy::RobotAssisted)?;
        let human = self.tco(Strategy::HumanOnly)?;
        let mut sensitivity = self
            .facts
            .iter()
            .map(|f| {
                Ok(Sensitivity {
                    fact_id: f.id.clone(),
                    npv_swing: self.fact_swing(f)?,
                })
            })
            .collect::<Result<Vec<_>, PlanningError>>()?;
        sensitivity.sort_by(|a, b| {
            b.npv_swing
                .cmp(&a.npv_swing)
                .then(a.fact_id.cmp(&b.fact_id))
        });
        let payback = payback(&representative)?;
        let unit_costs = self.protected_units.map(|units| UnitCosts {
            robot_cost_minor_units: robot.total,
            human_cost_minor_units: human.total,
            denominator_units: units,
        });
        let mut result=InvestmentResult{scenario_id:self.id.clone(),scenario_version:self.version,seed:self.seed,runs:self.runs,currency:self.currency.clone(),base_year:self.base_year,counterfactual_version:self.counterfactual.version,npv:range,irr:irr(&representative)?,payback:payback.clone(),break_even:BreakEven{status:payback},robot_tco:robot,human_tco:human,sensitivity,causal_claim:self.counterfactual.approved_causal_method.is_some(),limitations:vec!["Scenario comparison is not causal attribution unless an approved identification method is recorded.".into(),"Ranges reflect declared fact uncertainty and exclude unknown unknowns.".into()],scenario_digest:self.content_digest.clone(),result_digest:String::new(),rounding:RoundingPolicy::NearestMinorUnitHalfAwayFromZero,unit_costs};
        result.result_digest = hash_json(&result)?;
        Ok(result)
    }
    fn cash_flows(
        &self,
        mut value: impl FnMut(&OperationalCostFact) -> Result<i64, PlanningError>,
    ) -> Result<Vec<i64>, PlanningError> {
        let mut flows = vec![0i64; usize::from(self.horizon) + 1];
        for f in &self.facts {
            let amount = value(f)?;
            let sign: i64 = if f.strategy == Strategy::HumanOnly {
                1
            } else {
                -1
            };
            if f.category.capital() {
                flows[usize::from(f.start_year)] = flows[usize::from(f.start_year)]
                    .checked_add(
                        sign.checked_mul(amount)
                            .ok_or(PlanningError::InvalidScenario("cash flow overflow"))?,
                    )
                    .ok_or(PlanningError::InvalidScenario("cash flow overflow"))?;
            } else {
                for year in f.start_year..=self.horizon {
                    flows[usize::from(year)] = flows[usize::from(year)]
                        .checked_add(
                            sign.checked_mul(amount)
                                .ok_or(PlanningError::InvalidScenario("cash flow overflow"))?,
                        )
                        .ok_or(PlanningError::InvalidScenario("cash flow overflow"))?;
                }
            }
        }
        Ok(flows)
    }
    fn tco(&self, strategy: Strategy) -> Result<TcoBreakdown, PlanningError> {
        let capital = self
            .facts
            .iter()
            .filter(|f| f.strategy == strategy && f.category.capital())
            .map(|f| f.mode)
            .try_fold(0i64, |a, v| {
                a.checked_add(v)
                    .ok_or(PlanningError::InvalidScenario("TCO overflow"))
            })?;
        let recurring = self
            .facts
            .iter()
            .filter(|f| f.strategy == strategy && !f.category.capital())
            .try_fold(0i64, |a, f| {
                f.mode
                    .checked_mul(i64::from(self.horizon - f.start_year + 1))
                    .and_then(|v| a.checked_add(v))
                    .ok_or(PlanningError::InvalidScenario("TCO overflow"))
            })?;
        Ok(TcoBreakdown {
            capital,
            recurring,
            total: capital
                .checked_add(recurring)
                .ok_or(PlanningError::InvalidScenario("TCO overflow"))?,
        })
    }
    fn fact_swing(&self, f: &OperationalCostFact) -> Result<i64, PlanningError> {
        let years = if f.category.capital() {
            1
        } else {
            i64::from(self.horizon - f.start_year + 1)
        };
        f.high
            .checked_sub(f.low)
            .and_then(|v| v.checked_mul(years))
            .ok_or(PlanningError::InvalidScenario("sensitivity overflow"))
    }
}
fn sample(f: &OperationalCostFact, rng: &mut SplitMix64) -> Result<i64, PlanningError> {
    if f.low == f.high {
        return Ok(f.low);
    }
    let span = f.high.cast_unsigned() - f.low.cast_unsigned() + 1;
    let offset_a = i64::try_from(rng.next() % span)
        .map_err(|_| PlanningError::InvalidScenario("sample conversion overflow"))?;
    let offset_b = i64::try_from(rng.next() % span)
        .map_err(|_| PlanningError::InvalidScenario("sample conversion overflow"))?;
    let a = f
        .low
        .checked_add(offset_a)
        .ok_or(PlanningError::InvalidScenario("sample overflow"))?;
    let b = f
        .low
        .checked_add(offset_b)
        .ok_or(PlanningError::InvalidScenario("sample overflow"))?;
    Ok(i64::midpoint(i64::midpoint(a, b), f.mode))
}
fn quantile(v: &[i64], pct: usize) -> i64 {
    v[(v.len() - 1) * pct / 100]
}
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss
)]
fn npv(flows: &[i64], discount_bps: u16) -> Result<i64, PlanningError> {
    flows
        .iter()
        .enumerate()
        .map(|(year, value)| {
            let discounted =
                *value as f64 / (1.0 + f64::from(discount_bps) / 10_000.0).powi(year as i32);
            let rounded = discounted.round();
            if !rounded.is_finite()
                || rounded < i64::MIN as f64
                || rounded >= 9_223_372_036_854_775_808.0
            {
                return Err(PlanningError::InvalidScenario(
                    "non-finite or out-of-range NPV component",
                ));
            }
            Ok(rounded as i64)
        })
        .try_fold(0i64, |a, v| {
            a.checked_add(v?)
                .ok_or(PlanningError::InvalidScenario("NPV overflow"))
        })
}
fn payback(flows: &[i64]) -> Result<PaybackStatus, PlanningError> {
    let mut cumulative = 0i64;
    for (year, value) in flows.iter().enumerate() {
        cumulative = cumulative
            .checked_add(*value)
            .ok_or(PlanningError::InvalidScenario("payback overflow"))?;
        if cumulative >= 0 && year > 0 {
            return u16::try_from(year)
                .map(PaybackStatus::Year)
                .map_err(|_| PlanningError::InvalidScenario("payback year overflow"));
        }
    }
    Ok(PaybackStatus::Never)
}
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss
)]
fn irr(flows: &[i64]) -> Result<IrrStatus, PlanningError> {
    if !flows.iter().any(|x| *x < 0) || !flows.iter().any(|x| *x > 0) {
        return Ok(IrrStatus::UndefinedNoSignChange);
    }
    let changes = flows
        .windows(2)
        .filter(|w| (w[0] < 0 && w[1] > 0) || (w[0] > 0 && w[1] < 0))
        .count();
    if changes > 1 {
        return Ok(IrrStatus::MultipleRoots);
    }
    let (mut lo, mut hi) = (-0.99, 10.0);
    for _ in 0..100 {
        let mid = f64::midpoint(lo, hi);
        let value = flows
            .iter()
            .enumerate()
            .map(|(y, v)| *v as f64 / (1.0_f64 + mid).powi(y as i32))
            .sum::<f64>();
        if value > 0.0 { lo = mid } else { hi = mid }
    }
    let percent = f64::midpoint(lo, hi) * 100.0;
    if !percent.is_finite() {
        return Err(PlanningError::InvalidScenario("non-finite IRR"));
    }
    Ok(IrrStatus::Unique { percent })
}
fn hash_json(value: &impl Serialize) -> Result<String, PlanningError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|_| PlanningError::InvalidScenario("digest serialization"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
struct SplitMix64(u64);
impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}
