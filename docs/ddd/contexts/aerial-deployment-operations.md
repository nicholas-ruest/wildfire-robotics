# Aerial Deployment Operations Context

## Purpose

Plan, authorize, execute, monitor and recover experimental aircraft-deployed robot/membrane systems without owning aircraft flight safety, incident authority, suppression chemistry, vegetation treatment, fleet eligibility or repair disposition.

## Model

- **Aggregates:** BlanketConfiguration, MembraneAssembly, PayloadManifest, AerialDropMission, ReleaseAuthorization, AirborneDeployment, GroundInstallation.
- **Core invariant:** No extraction or payload release occurs unless the exact aircraft/payload/blanket/robot configuration, release corridor, drop-zone exclusion, environmental envelope, surveillance, incident authority and aircraft authority are current and independently permit it.
- **Primary workflow:** qualify configuration → plan corridor/footprint → assemble/inspect payload → authorize mission → continuously validate release → extract/establish/expand → land/install → operate temporary protection → account/recover/evaluate.

## Tactical model

| Aggregate | Lifecycle | Commands | Events |
|---|---|---|---|
| BlanketConfiguration | concept → ground-qualified → low-drop-qualified → aircraft-qualified → promoted → suspended/retired | RegisterConfiguration, AttachQualification, PromoteStage, SuspendConfiguration | BlanketConfigurationPromoted, BlanketConfigurationSuspended |
| MembraneAssembly | planned → assembling → inspected → packed → deployed → installed → recovering → recovered/sacrificed | AssemblePanels, InspectAssembly, PackAssembly, RecordDeployment, RecordInstallation, RecoverAssembly | MembranePacked, MembraneStateChanged |
| PayloadManifest | draft → reconciled → load-approved → loaded → released/retained → accounted | AddPayloadItem, ReconcileManifest, ApproveLoad, RecordLoad, RecordRelease, AccountItem | PayloadLoadApproved, PayloadAccounted |
| AerialDropMission | draft → modeled → reviewed → authorized → airborne → completed/aborted | PlanDropMission, ModelDispersion, ReviewMission, AuthorizeMission, AbortMission, CompleteMission | AerialDropMissionAuthorized, AerialDropMissionChanged |
| ReleaseAuthorization | requested → checking → armed → released/held/aborted/expired | RequestRelease, RecordAircraftDecision, RecordGroundDecision, ArmRelease, HoldRelease, CommitRelease, AbortRelease | ReleaseArmed, PayloadReleased, ReleaseAborted |
| AirborneDeployment | retained → extracted → stabilized → cohort-releasing → formation → expanding → aligning → landing → landed/isolated/jettisoned | RecordExtraction, EstablishCohort, ExtendTether, ReleaseSection, ChangeVent, IsolatePanel, JettisonSection, RecordLanding | DeploymentPhaseChanged, PanelIsolated, SectionJettisoned |
| GroundInstallation | landed → transitioning → anchoring → sealing → active → degraded → recovering → removed/temporarily-left | TransitionGroundMode, InstallAnchor, TensionPanel, SealJoint, ActivateBlanket, RepositionPanel, DeactivateBlanket, RecoverPanel | BlanketActivated, BlanketDegraded, BlanketRecovered |

Owned values include configuration/stage/evidence, aircraft adapter/configuration reference, blanket/panel/joint/vent/anchor/reel/tether/parafoil/cradle identifiers, material/packed/mass properties, payload manifest and loading station, release corridor and point-of-no-return, predicted nominal/failure dispersion, exclusion/abort/jettison/emergency-landing zones, weather/wind/turbulence/smoke, drop altitude/speed/heading, terrain/obstacles/utilities, robot/panel assignment, deployment sequence, formation/tension/stability margins, sensor quality, ground footprint, exposure/damage, component accounting and effectiveness study reference.

## Invariants

- `AD-INV-001`: A promoted configuration binds exact material/panel/joint/tether/reel/parafoil/cradle/robot versions, geometry, mass properties, ODD, qualification stage and evidence; substitution invalidates promotion until assessed.
- `AD-INV-002`: The reconciled payload mass, volume, centre-of-gravity, floor/ramp/roller loads, restraint/extraction interfaces and hazardous contents fit an aircraft-specific approved loading/release envelope with measured margin.
- `AD-INV-003`: Mission planning includes nominal and component/system-failure footprints, release corridor, exclusions, jettison sectors, emergency landing zones, alternate/abort plan, destination ground capacity and continuously expiring source data.
- `AD-INV-004`: Payload release requires distinct current aircraft and incident/safety decisions bound to the same configuration, manifest, corridor, conditions and command digest; either veto or any stale/indeterminate mandatory input prevents release.
- `AD-INV-005`: Every deployment phase transition satisfies declared formation separation, tether routing/tension, panel/vent/reefing, wind/stability, navigation/time, communications, drop-zone clearance and remaining contingency margins.
- `AD-INV-006`: One robot/panel/parafoil/tether failure cannot command another unsafe transition; local isolation/breakaway/vent/retain/jettison actions are bounded to pre-authorized safe sectors and are fully audited.
- `AD-INV-007`: Ground anchoring/tensioning requires landed safe robot state and current terrain, ownership, utilities, people/wildlife, slope, wind/uplift, fuel/fire and anchor compatibility; uncertainty inhibits the affected zone.
- `AD-INV-008`: Active installation continuously exposes panel contact, top/bottom temperature, ember, tension, wind/uplift, tear, vent, gap and sensor freshness; breach triggers bounded vent/isolate/reposition/suppression/escalation rather than an effectiveness claim.
- `AD-INV-009`: Installation cannot authorize suppressant application, vegetation removal, aircraft flight, robot capability or incident-area expansion; those remain explicit commands from their owning contexts.
- `AD-INV-010`: Mission closure accounts for every robot, panel, parafoil, tether, reel, anchor, cradle section and chemical payload and routes damage/contamination through custody, quarantine, repair/reuse/recycle or approved sacrificial disposition.
- `AD-INV-011`: Effectiveness states protected area/time, exposure, counterfactual/baseline, uncertainty and limitations; deployed area or apparent survival alone is not proof of fire prevention.

## Ports and read models

Ports cover aircraft/loadmaster integration, payload/extraction/cradle controls, wind/weather/fire/airspace, dispersion/aeroelastic/thermal/terrain simulation, drop-zone surveillance, robot/parafoil/reel/tether control, ruv-drone bounded coordination, membrane sensing, incident/release approval, fleet capability, mission allocation, suppression/vegetation commands, logistics custody and Robot Care disposition. Read models expose qualification/configuration matrix, assembly/manifest reconciliation, corridor/exclusion map, release checklist/freshness, deployment phase/stability/tension, per-panel health/footprint, unaccounted components, recovery/disposition and effectiveness evidence.

## Boundary and failure policy

Aircraft crew/authority owns aircraft operation and release veto. Incident Command owns operational objective/area. Mission Control owns robot allocation and mission leases. Safety Assurance owns promotion/ODD/constraints. Suppression and Vegetation own physical work after installation. Aerial Deployment owns the blanket-specific payload, release handshake, coupled airborne deployment and ground installation lifecycle. Any mismatch, stale input, unmodeled footprint, excessive wind/tension/instability, intrusion, lost surveillance, navigation/time uncertainty, aircraft/payload fault or exhausted safe contingency causes retain, hold, abort, pause, vent, isolate, divert, emergency land or pre-authorized jettison according to the last reachable safe state.

## Implementation acceptance

Every invariant requires property/state-machine tests. Promotion progresses through coupon/material, panel/joint/tether/reel, ground multi-panel, low-drop, subscale extraction, SIL, HITL, instrumented range, aircraft ground/extraction, partial-scale flight and full-system evidence. Tests cover load/CG, extraction shock, reefing/venting, coupled aeroelastic dynamics, entanglement/collision, navigation/time/communications loss, wind/turbulence/smoke, people/aircraft intrusion, failed robot/panel/parafoil/tether, emergency landing/jettison footprint, landing/anchoring/utilities, thermal/ember exposure, fire spread, contamination, accounting and recovery. Passing one stage authorizes only that exact configuration and next bounded test.
