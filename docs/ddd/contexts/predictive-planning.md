# Predictive Planning Context

## Purpose

Produce reproducible, calibrated advisory forecasts and scenarios.

## Model

- **Aggregates:** ModelRelease, ForecastRun, SpreadScenario, Recommendation.
- **Core invariant:** Only approved model releases run operationally; inputs and seeds are immutable; recommendations expose confidence and never authorize action.
- **Primary workflow:** Snapshot inputs -> execute model adapter -> validate/calibrate -> publish versioned advisory -> monitor drift.

## Tactical model

| Aggregate | Lifecycle | Commands | Events |
|---|---|---|---|
| ModelRelease | registered → validating → approved → promoted → suspended/retired | RegisterModel, AttachEvidence, ApproveModel, PromoteModel, SuspendModel | ModelRegistered, ModelPromoted, ModelSuspended |
| ForecastRun | requested → queued → running → validating → published/failed/rejected | RequestForecast, StartRun, RecordOutput, ValidateRun, PublishForecast, FailRun | ForecastStarted, ForecastPublished, ForecastRejected |
| SpreadScenario | draft → running → complete → compared/expired | DefineScenario, RunScenario, CompareScenario, ExpireScenario | ScenarioCompleted, ScenarioCompared |
| Recommendation | draft → published → expired/superseded/withdrawn | CreateRecommendation, PublishRecommendation, WithdrawRecommendation | RecommendationPublished, RecommendationWithdrawn |
| EvaluationStudy | designed → frozen → running → reviewed → accepted/rejected | DesignEvaluation, FreezeCohort, RunEvaluation, ReviewEvaluation | ModelEvaluationCompleted |
| InvestmentScenario | draft → calibrated → simulated → reviewed → published/superseded | DefineInvestmentScenario, CalibrateCosts, RunMonteCarlo, PublishInvestmentCase | InvestmentCasePublished |

Owned values include model/input release, immutable run manifest, seed, container digest, parameters with units, ODD, horizon, calibration/evaluation cohort, leakage controls, baseline/champion, metrics, confidence/uncertainty, limitations, artifact digest, compute usage, expiry, and sampling/intervention policy. Investment scenarios additionally own counterfactual strategy, cost/outcome sources, base currency/year, horizon, discount rate, uncertainty distributions, NPV/IRR/payback/TCO and sensitivity.

## Invariants

- `PP-INV-001`: Operational runs use a promoted immutable model release inside its validated ODD.
- `PP-INV-002`: Inputs, parameters, seed, runtime image, code and dependency digests fully identify a reproducible run.
- `PP-INV-003`: Publication requires schema, physical-plausibility, calibration, completeness, licensing, and domain-validity gates.
- `PP-INV-004`: Advice always exposes uncertainty, freshness, limitations, expiry, and alternatives; it never grants authority.
- `PP-INV-005`: Drift, invalid assumptions, or superseded inputs affect and, when material, withdraw dependent outputs by lineage.
- `PP-INV-006`: Lightning/ignition models are evaluated against a versioned authoritative baseline using incident/geography/time/fire-year isolation, rare-event discrimination, calibration, and prospective shadow evidence.
- `PP-INV-007`: Training data records reconnaissance selection, unobserved/censored regions, interventions and negative outcomes; production behavior changes only through promoted immutable releases.
- `PP-INV-008`: ROI output separates capital and recurring costs, identifies its human-only counterfactual, propagates uncertainty, and cannot claim causal benefit without an approved method.

## Ports and read models

Model runners execute through OCI sandbox ports with bounded CPU/GPU/memory/time/network and immutable artifacts. Calibration, baseline, registry, object-store, and compute-scheduler ports are replaceable. Read models expose run status, forecast catalog, model scorecards, drift, calibration, and cost; no read projection authorizes action.

## Boundary and failure policy

Consumes immutable pictures and publishes advisory products through the [integration registry](../integration-contracts.md). Timeout, drift, unavailable compute, invalid domain, numerical instability, and model disagreement produce unavailable/degraded advice; the prior product remains visible only with its original expiry and supersession state.

## Implementation acceptance

Domain invariants must be executable and property-tested; API/event contracts require compatibility tests; persistence requires migration/rollback and concurrency tests; adapters require fault-injection and replay tests; operational promotion requires the applicable evidence in the [production readiness standard](../../operations/production-readiness.md).
