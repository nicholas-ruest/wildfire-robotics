# Cross-Context Process Managers

These workflows implement ADR-043. Their durable state belongs to the named owner; every transition records correlation, authority, expected versions, deadline, attempts, and evidence.

The normative timeout, compensation, escalation, and observable-containment contract for every process below is machine-readable in `contracts/release-registry.toml` and enforced by `contract-check`. A deadline expiring is an input to the durable state machine, not permission to assume success: the owner executes the named compensation idempotently, emits the containment fact, and escalates to the named independent authority. Recovery resumes from recorded state and fenced versions; it never rewrites completed physical or ledger facts.

## Incident activation

- **Owner:** Incident Command.
- **Flow:** open incident → establish operational period → validate authority → publish restrictions → confirm policy distribution → permit assignments.
- **Compensation:** revoke unconsumed assignments; restrictions remain until explicitly superseded.
- **Escalation:** ambiguous authority, policy-distribution gap, or expired period blocks issuance.

## Vehicle enrollment and capability promotion

- **Owner:** Fleet Operations; Identity owns device trust and Safety owns promotion.
- **Flow:** enroll identity → attest hardware/software → validate maintenance/calibration → execute required tests → approve evidence → publish capability.
- **Compensation:** revoke trust or capability, ground vehicle, preserve evidence.
- **Escalation:** inconsistent attestation, vulnerable version, missing evidence, or failed test.

## Mission authorization and dispatch

- **Owner:** Mission Control.
- **Flow:** accept assignment → snapshot constraints/hazard/fleet state → plan → reserve resources → deconflict → obtain approval → acquire lease → dispatch intents → monitor → close.
- **Compensation:** release unused reservations, revoke reachable intents, command vehicle-specific minimum-risk condition, notify Incident Command.
- **Escalation:** conflict, stale input, lost lease, unknown acknowledgement, or authority expiry.

## Edge deployment synchronization

- **Owner:** Station Operations.
- **Flow:** stage signed deployment/policy/data → verify compatibility/capacity → activate → attest health → reconcile local events and audit.
- **Compensation:** roll back compatible software, retain stricter policy, quarantine conflicts, never reactivate expired authority.
- **Escalation:** partial state, corrupt log, clock uncertainty, insufficient emergency energy, or schema incompatibility.

## Hazard ingestion and forecast publication

- **Owner:** Predictive Planning after Hazard Intelligence publishes an immutable input snapshot.
- **Flow:** ingest/validate → construct picture → freeze licensed input manifest → select promoted model → run → quality/calibration gates → publish advisory.
- **Compensation:** quarantine invalid observations; retract/supersede advisory with explicit reason and affected consumers.
- **Escalation:** license conflict, material disagreement, model-domain violation, drift, or missing provenance.

## Logistics delivery

- **Owner:** Logistics.
- **Flow:** validate demand/source → reserve source/capacity/vehicle → authorize route → dispatch → custody checkpoints → deliver → reconcile quantity and usage.
- **Compensation:** release reservations, safe stop or authorized reroute, return load where safe.
- **Escalation:** contamination, depleted source, route/ODD breach, custody mismatch, or stranded vehicle risk.

## Suppression operation

- **Owner:** Suppression Operations; Safety owns constraints and Incident Command owns authority.
- **Flow:** approve target and agent → issue signed envelope → verify two-person arming and independent stop → operate under continuous monitoring → disarm → reconcile dose/effect/evidence.
- **Compensation:** immediate inhibit/disarm, withdraw/secure agent where possible, report occurrence; physical application is never rolled back.
- **Escalation:** person/aircraft intrusion, envelope breach, lost supervision, actuator anomaly, weather/visibility excursion, or uncertain dose.

## Release promotion

- **Owner:** Safety Assurance.
- **Flow:** identify release/configuration → assemble traceability and test evidence → security/privacy/operations review → independent safety review → approve stage and ODD → sign promotion → controlled rollout → monitor.
- **Compensation:** halt rollout, revoke promotion, rollback compatible artifacts, ground affected capability, notify operators/customers as required.
- **Escalation:** evidence gap, near miss, vulnerability, SLO/error-budget breach, model drift, or incompatible fleet state.

## Tenant onboarding and offboarding

- **Owner:** Commercial Operations.
- **Flow:** approve contract/data region → create isolation/key scopes → establish identities/admins → configure entitlements/support → validate isolation → activate. Offboarding freezes new optional work, exports authorized records, revokes access, applies retention/deletion/legal hold, and proves completion.
- **Compensation:** disable incomplete tenant and destroy unneeded provisional resources.
- **Escalation:** residency conflict, failed isolation test, unpaid state during an active incident, legal hold, or disputed data ownership.

## Usage rating and invoicing

- **Owner:** Commercial Operations.
- **Flow:** receive deduplicated usage → close metering window → reconcile gaps → rate against effective contract → review exceptions → finalize invoice → post financial result.
- **Compensation:** adjustment or credit entries; finalized ledger facts are not rewritten.
- **Escalation:** missing usage, price ambiguity, tax failure, dispute, or cross-tenant contamination.

## Lightning prediction and reconnaissance learning

- **Owner:** Predictive Planning; Hazard Intelligence owns observations/media and Mission Control owns reconnaissance authority.
- **Flow:** freeze authoritative lightning/weather/fuel input → run baseline and promoted ML → publish calibrated priority areas → value-of-information planner requests drone coverage → allocate bounded ruv-drone cohort/relay → capture calibrated imagery/telemetry → ingest and RuPixel-index media → verify/label outcomes including negatives/censoring → align prediction/outcome → evaluate drift/utility → build immutable candidate dataset → train and shadow-test candidate → approve or reject promotion.
- **Compensation:** supersede erroneous products/indexes, withdraw affected recommendation, quarantine suspect labels/models, retain original evidence and notify consumers by lineage.
- **Escalation:** source disagreement/outage, poor coverage, georegistration/calibration failure, sampling bias, model drift, safety/airspace constraint, or insufficient evidence.

## Fleet-cell formation and adaptive collaboration

- **Owner:** Fleet Operations; Mission Control owns operational allocation.
- **Flow:** observe partition load/geography/capability/connectivity → propose cell and epoch → fence prior membership → form bounded cohort → optionally initialize RVM partitions/communication graph → execute cooperative tasks → collect signed outcome/safety facts → evaluate time-decayed relationships → shadow candidate cohort/policy → promote advisory profile → split/merge when thresholds require.
- **Compensation:** revoke advisory profile, restore conventional allocator, repartition with a new epoch, quarantine poisoned evidence, and preserve mission leases/fencing.
- **Escalation:** hot partition, split-brain membership, stale epoch, graph poisoning, RVM failure, unsafe emergent behavior, or scale/SLO breach.

## Vegetation-treatment program

- **Owner:** Vegetation Management.
- **Flow:** ingest survey/fuel state → create and approve prescription → partition treatment units/work packages → reserve robots/drones/tools/energy/biomass logistics → authorize missions → execute with exclusions/tool stop → verify actual work/media → reconcile material/cost → schedule and publish effectiveness assessments.
- **Compensation:** release reservations, inhibit tools, mark missed/rework geometry, secure/return biomass, and preserve planned-vs-actual evidence.
- **Escalation:** authority/prescription change, high fire danger, exclusion intrusion, utility/wildlife discovery, tool/localization fault, or effectiveness below threshold.

## Supply planning and replenishment

- **Owner:** Logistics.
- **Flow:** consume incident/scenario demand → snapshot stock/custody/maintenance/energy/supplier/route state → forecast distributions → optimize staging/replenishment/substitution → approve plan → reserve and order → deliver/handoff/consume → reconcile → replan on variance.
- **Compensation:** release reservations, cancel uncommitted orders, reroute/return safely, issue inventory adjustments with custody evidence.
- **Escalation:** critical shortage, incompatible substitution, supplier/route failure, contaminated source, custody discrepancy, or infeasible service level.

## Habitat energy and fleet charging

- **Owner:** Station Operations; Fleet owns battery/vehicle eligibility and Logistics owns fuel/energy replenishment.
- **Flow:** forecast solar/grid/wind/generator/fuel and critical/mission loads → reserve emergency/thermal/safety energy → admit compatible batteries/robots/carriers by zone → optimize bounded charging schedule → perform BMS/protection prechecks → charge under local limits → publish summaries/anomalies → update fleet eligibility and maintenance → replan continuously.
- **Compensation:** reduce/stop charge, isolate charger/pack/fire zone, preserve emergency reserve, start fallback generation, move optional deadlines or request energy resupply.
- **Escalation:** thermal/isolation anomaly, conflicting BMS estimate, PV/grid/generator failure, fuel delay, correlated mission recall, fire-zone loss or forecast reserve breach.

## Mass robot mobilization

- **Owner:** Logistics; Incident Command declares demand, Fleet supplies eligible assets, Station owns origin/destination admission, and Mission Control assigns arrived capability.
- **Flow:** translate objective to capability/time demand → query hierarchical habitat capacity → reserve robots/batteries/tools/pods/carriers → solve time-expanded multimodal flow → reserve load/route/energy/staging/unload slots → precondition and attest robots/loads → seal/transfer pod custody → release bounded independent carrier waves → monitor/replan → admit/unload/inspect/energize → declare useful arrival → assign missions → execute reverse logistics.
- **Compensation:** hold/resequence waves, release reservations, divert to prepared hub, return pods, isolate failed loads/carriers, restore custody and destination admission without duplicating dispatch.
- **Escalation:** overload/centre-of-gravity/securement failure, road/bridge/ferry/rail closure, weather/ODD excursion, charging/fuel shortage, convoy/V2X loss, destination saturation or insufficient recovery capacity.

## Robot maintenance and field recovery

- **Owner:** Robot Care and Recovery; Fleet owns eligibility, Incident/Mission owns field authority, Station owns safe zones/bays, and Logistics owns transport/parts custody.
- **Flow:** monitor duty/exposure/condition → predict or schedule maintenance → assign compatible maintenance robot/bay/procedure/parts → isolate/service/test → publish evidence → update eligibility. On failure: request recovery → assess scene/hazards → reserve medic pod and safe destination → authorize route/ODD → stabilize energy/tools → lift/tow/cradle and transfer custody → triage → quarantine/decontaminate when required → diagnose/repair/calibrate/burn-in → independently recertify or retire/salvage → update fleet/identity/inventory and learning evidence.
- **Compensation:** abort or retreat medic to minimum-risk state, isolate robot in place, divert to compatible quarantine/hospital, reverse uncommitted parts reservations, reopen failed work and preserve every replaced part/fault state.
- **Escalation:** human rescue priority, active fire/exclusion, unknown stored energy, thermal runaway, contamination, unstable structure, incompatible lift/tow, medic/communications failure, no quarantine capacity, repeat fault, failed burn-in or disputed custody.

## Aerial fire blanket qualification and deployment

- **Owner:** Aerial Deployment Operations; aircraft authority owns aircraft/loading/release veto, Incident Command owns objective/area, Safety owns configuration/ODD promotion, Mission owns robot allocation/leases, and Logistics owns custody/recovery.
- **Flow:** qualify material/components → qualify ground multi-panel system → qualify low-drop/subscale extraction → register exact aircraft/payload configuration → reconcile/inspect/pack manifest → model nominal/failure dispersion and ground installation → approve mission/ODD → load and verify → continuously validate aircraft plus ground release decisions and exclusion surveillance → commit irreversible release → extract/stabilize → release bounded cohorts → progressively reef/vent/expand → align/land → transition/anchor/seal → activate temporary protection → hand off suppression/vegetation work → monitor/degrade/isolate → recover/account/evaluate.
- **Compensation:** before release retain/hold/abort/unload; after extraction pause/vent/isolate/divert/emergency-land or jettison only into pre-authorized sectors; on ground inhibit anchoring/repositioning, isolate panels, publish gaps, recover/quarantine components and construct alternate containment.
- **Escalation:** aircraft/load mismatch, stale dual authorization, unmodeled footprint, excessive wind/turbulence/smoke/tension/instability, entanglement/collision, navigation/time/communications loss, intrusion, unavailable emergency zones, anchor/utility uncertainty, thermal breach, unaccounted component or environmental contamination.
