# Wildfire Robotics — Implementation Plan (GOAP-Style Goal Decomposition & Roadmap)

**Scope of this document:** architecture and roadmap only. No code. Produced by decomposing the end-state goal ("a national fleet of AI-controlled autonomous machines that predict wildfire threats, prepare containment zones, and directly fight fires") into a goal hierarchy, then A*-style sequencing the cheapest, lowest-risk, highest-confidence actions first, deferring the least mature/highest-risk capability (autonomous direct-attack firefighting) behind extensive validation gates. Built directly on `.plans/research.md` (seed vision) and `.plans/deep-research.md` (evidence-graded findings) — the researcher's assessments override the seed doc's tooling assumptions wherever they conflict, most notably on the ruvnet stack and on Firefighters maturity.

---

## 1. Goal Decomposition

### 1.0 Root goal

> **G0: Operate a national (Canada-first) fleet of autonomous machines that reduces wildfire loss (life, property, ecosystem) at a demonstrably positive ROI, without introducing unacceptable safety, environmental, or program risk.**

G0 decomposes into five platform-layer goals and four cross-cutting capability-domain goals. Layers are *what gets built*; domains are *capabilities that cut across every layer*. Every subsystem below is tagged with its build-vs-adopt call (full table in §2) and a success criterion that is falsifiable, not aspirational.

### 1.1 Layer goals

**G1 — Wildfire Robotics Command (control platform / brain)**
- G1.1 Situational-picture ingestion: fuse GLM lightning, WWLLN, FIRMS, CWFIS/EFFIS, CFFDRS/FWI into one live map. *Success: Command displays a single fused hazard map updated at ≤15 min latency for a defined Canadian test region, cross-validated against CWFIS's own published values.*
- G1.2 Ignition-risk nowcasting layer (lightning → fire risk fusion): thin ML/rules layer on top of G1.1. *Success: nowcast model beats a persistence/climatology baseline on held-out historical ignition events in the test region.*
- G1.3 Fire-spread simulation integration (Cell2Fire tactical, WRF-Fire strategic). *Success: Cell2Fire runs reproduce a documented historical fire's observed perimeter within published Cell2Fire accuracy bounds for at least one Canadian case study.*
- G1.4 Fleet-orchestration layer (task allocation, traffic/collision deconfliction across robots — explicitly NOT provided by ROS2). *Success: orchestrator safely deconflicts ≥10 simulated concurrent agents (drones + ground units) in digital twin with zero collision violations across a defined test scenario suite.*
- G1.5 Digital-twin + human-in-the-loop change gating. *Success: no Fleet behavior change reaches physical hardware without first passing an automated digital-twin regression suite AND an explicit human sign-off step — enforced procedurally, not just documented.*
- G1.6 ROI/decision-support module (§5). *Success: Command can generate a FEMA-BCA-style benefit-cost ratio for a proposed deployment scenario using real or synthetic cost inputs.*
- G1.7 "Brain" technical foundation decision (memory/reasoning substrate). *Success: a time-boxed spike (Phase 0, see §3) produces a written go/no-go on each ruvnet component with licensing verified, OR a decision to use conventional infrastructure (Postgres+pgvector, standard sparse solvers) instead.*

**G2 — Wildfire Robotics Fleet (the robots themselves, cross-cutting chassis/perception/control)**
- G2.1 Shared perception stack (fire/smoke CV) reusable across drones, carriers, and (eventually) firefighters. *Success: a single perception model/pipeline, trained on FLAME/FLAME2/FASDD/GWFP/WIT-UAS, achieves published-benchmark-comparable detection accuracy and runs on the target edge compute hardware within its power/thermal budget.*
- G2.2 Shared middleware: ROS2/DDS as the node-level bus across all robot types. *Success: at least one drone and one ground unit interoperate over the same ROS2 domain in a joint simulation scenario.*
- G2.3 Edge-first autonomy (no cloud dependency assumption). *Success: a representative robot completes its assigned mission profile in a simulated total-connectivity-loss scenario using only onboard compute.*

**G3 — Wildfire Robotics Stations (remote bases)**
- G3.1 Station siting model driven by G1.6 ROI outputs and logistics coverage (§5). *Success: a station-placement recommendation is produced from real geographic/risk input data, not manually chosen.*
- G3.2 Edge-compute/communications hub at each station (candidate fit for `rvm`'s Appliance hardware profile, pending G1.7 validation). *Success: station hub relays Fleet telemetry to Command with defined latency/availability targets under intermittent connectivity.*
- G3.3 Charging/refuel and maintenance turnaround for fleet units based at the station. *Success: documented turnaround-time target met in a field or simulated pilot.*

**G4 — Wildfire Robotics Carriers (autonomous ground transport/logistics)**
- G4.1 Fixed-route/forwarder-style logistics automation (per AORO/ATREF precedent — NOT general off-road autonomy). *Success: a Carrier autonomously completes a repeated point-to-point supply/equipment transport route in forest terrain in field trial, matching the AORO "autonomous log forwarding" precedent's task shape.*
- G4.2 Non-GPS-dependent localization for under-canopy operation. *Success: Carrier maintains position estimate within defined error bounds in a forest-canopy test site where GPS is deliberately degraded/denied.*
- G4.3 Adapted off-road navigation stack (Autoware + `autoware.off-road`, Nav2 adapted). *Success: Carrier navigates a defined off-road test course without human intervention, with documented failure-mode handling (stop-and-call-human, not silent failure).*

**G5 — Wildfire Robotics Firefighters (direct-attack machines)**
- G5.1 Perception-only pilot (fire/smoke detection + human/hazard detection near active fire, no actuation). *Success: unit correctly identifies fire-line position and unsafe zones in live-fire training-ground trials, matching human-observer ground truth.*
- G5.2 Mobility/actuation platform informed by structural-firefighting precedent (Thermite RS3/Colossus design patterns) adapted for wildland terrain. *Success: platform survives defined heat/particulate/vibration exposure profile in bench testing.*
- G5.3 Remote-operated wildland suppression (human-in-the-loop, not autonomous) as an intermediate milestone before attempting autonomy. *Success: remote operator can direct suppression action using Command's fused situational picture in a controlled burn or training exercise.*
- G5.4 Autonomous direct-attack suppression — **the core unsolved R&D goal of the entire program.** *Success criterion deliberately left open pending Phase 3/4 findings; no existing system anywhere (open or commercial) achieves this, per research §3. Do not commit a date to this goal before G5.1–G5.3 are field-proven.*

### 1.2 Cross-cutting capability-domain goals

**GA — Robotic Intelligence** (spans G1, G2, G5): shared perception + shared reasoning/orchestration substrate + simulation-first validation discipline. Depends on G1.7 (brain decision) and G1.5 (digital twin) as prerequisites for anything touching physical hardware.

**GB — Drone Operations** (primarily G2 + G1.4): PX4/ArduPilot autopilot adoption, Aerostack2 + Swarm-SLAM swarm-coordination adaptation, perimeter-mapping mission profile. Independent of Carriers/Firefighters — can proceed in Phase 1 as the first physical-hardware workstream.

**GC — Logistics** (primarily G4 + G3): forwarder-style Carrier automation, station placement/turnaround, fleet allocation feeding back into G1.6 ROI loop. Depends on forestry-precedent-informed scoping (narrow, not general off-road autonomy).

**GD — ROI Calculation** (primarily G1.6): FEMA BCA Toolkit adoption, CBO/insurance-premium evidence as supporting data, explicit Canadian-equivalent gap to be closed (Phase 0 action item). Feeds back into G3.1 (station siting) and G1.4 (fleet allocation) as a decision input, not just a reporting output.

---

## 2. Build-vs-Adopt Table (refined for planning use)

Legend: **Adopt** = use as-is with integration work only. **Adapt** = real starting point exists but needs material engineering/field-hardening. **Build** = no viable open/commercial substitute exists.

| # | Subsystem | Call | Primary component(s) | Owning goal(s) | Confidence |
|---|---|---|---|---|---|
| 1 | Lightning detection | Adopt | GOES GLM, WWLLN, (budget-permitting) CLDN/NLDN/ENTLN, Blitzortung | G1.1 | High |
| 2 | Wildfire hotspot detection | Adopt | NASA FIRMS, CWFIS, Copernicus EFFIS | G1.1 | High |
| 3 | Fire-danger rating | Adopt | `cffdrs` (Canadian FWI System) | G1.1 | High |
| 4 | Fire-spread simulation (tactical) | Adopt | Cell2Fire | G1.3 | High |
| 5 | Fire-spread simulation (strategic) | Adopt (heavier) | WRF-Fire | G1.3 | Medium-High |
| 6 | Ignition-risk nowcasting | Build (thin layer) | Fuses #1+#3 outputs | G1.2 | Medium (no canonical tool exists, but well-scoped) |
| 7 | Fire/smoke CV perception | Adopt (datasets), build (models tuned to edge hardware) | FLAME/FLAME2, FASDD, GWFP, WIT-UAS | G2.1, G5.1 | High |
| 8 | Drone autopilot | Adopt | PX4 (ROS2-native, primary), ArduPilot (broader vehicle types incl. ground/marine Carriers) | GB | High |
| 9 | Drone GCS/SDK | Adopt | QGroundControl, MAVSDK, MAVLink | GB | High |
| 10 | Drone swarm coordination | Adapt | Aerostack2 + Swarm-SLAM | GB | Medium |
| 11 | Ground off-road autonomy | Adapt | Autoware + `autoware.off-road`; Nav2 adapted, not adopted as-is | G4.3 | Medium-Low (field honestly says this is immature) |
| 12 | Ground logistics task scoping | Adapt (precedent, not software) | AORO/ATREF/FPInnovations forwarder-automation pattern | G4.1 | Medium (decades-long precedent, but forwarding-only) |
| 13 | Under-canopy localization | Build | No off-the-shelf GPS-alternative stack identified in research; must integrate/tune (e.g., LiDAR-inertial odometry, UWB, visual-inertial) | G4.2 | Low-Medium — flagged research gap |
| 14 | Autonomous wildland suppression actuation | Build | No open or commercial precedent anywhere; informed only by structural-firefighting mobility/actuation design (Thermite RS3, Colossus) | G5.4 | Very Low — multi-year R&D, not a near-term deliverable |
| 15 | Fleet-level orchestration/traffic management | Build (on top of ROS2) | Industry-unanimous: ROS2 alone does not provide this | G1.4 | Medium — well-precedented need, custom implementation |
| 16 | Digital twin + sim-first validation | Adopt pattern, build implementation | Autoware.off-road/CARLA precedent; Unity ROS-TCP-Connector-class latency precedent | G1.5 | Medium-High |
| 17 | ROI / cost-benefit methodology | Adopt | FEMA BCA Toolkit (Wildfire, Green Infra, Ecosystem Services modules); CBO + insurance-premium studies as supporting evidence | G1.6 | High (US); **open gap for Canada — see §6** |
| 18 | Command "brain" (memory/reasoning substrate) | **Evaluate selectively — no wholesale adoption** | RuVector, rvm merit closer technical evaluation; `agentic-robotics` ROS2 bridge unbuilt, do not rely on it; verify licensing on `rvm`/`daa`; do not adopt `ruv-FANN`'s swarm/forecasting claims or `sublinear-time-solver` without independent benchmarking against real workloads | G1.7 | Low confidence in the stack as named; the *evaluation process* itself is high confidence |
| 19 | Edge/cloud split architecture | Adopt pattern, build implementation | Edge-intelligence + digital-twin-sync three-layer pattern; favor onboard autonomy over cloud dependence given wildfire-site connectivity risk | GA, G2.3 | Medium-High |

**Key deviation from the seed document:** the seed doc treats the ruvnet stack (RuVector, ruv-FANN, rvm, daa, sublinear-time-solver, agentic-robotics) as the presumptive architecture for Command's brain. The research does not support this as a foregone conclusion — `agentic-robotics`, the one repo actually branded for robotics, is assessed as the weakest and most internally inconsistent of the six (mismatched GitHub metadata, unbuilt ROS2 bridge, zero releases). This plan therefore inserts an explicit, time-boxed evaluation spike (Phase 0, item P0.5) before any commitment, and treats "use conventional infrastructure instead" as an equally live outcome of that spike.

---

## 3. Phased Roadmap

Phases are sequenced by an A*-style cost/risk ordering: cheapest-to-validate, highest-confidence capabilities first (detection/data integration), physical hardware introduced only after simulation validation, and the single highest-risk capability (autonomous direct-attack firefighting, G5.4) deliberately pushed to the last phase, gated behind everything else. Each phase has explicit **entry dependencies** and a **go/no-go gate** — the program should not advance to the next phase until its gate is met.

### Phase 0 — Foundations, Evaluation, Simulation Baseline (no physical robots)
**Goals touched:** G1.1–G1.3, G1.6, G1.7 (spike only), GD
**Dependencies:** none (this is the starting phase)
**Work:**
- Stand up Command's data-fusion layer: ingest GLM, WWLLN, FIRMS, CWFIS, EFFIS, `cffdrs` FWI outputs into one hazard picture (G1.1).
- Integrate Cell2Fire (tactical) and evaluate WRF-Fire (strategic) against at least one historical Canadian fire case study (G1.3).
- **P0.5 — ruvnet stack evaluation spike (time-boxed, e.g. 2–4 weeks):** verify licensing on `rvm` and `daa`; check current build status of `agentic-robotics`'s ROS2 bridge; independently benchmark RuVector and `sublinear-time-solver` against this project's actual candidate workloads vs. conventional alternatives (Postgres+pgvector, standard sparse solvers). Produce a written go/no-go per component (G1.7).
- Stand up the FEMA BCA Toolkit (Wildfire/Green Infra/Ecosystem Services modules) against synthetic or available cost data; begin the Canadian-equivalent gap search (§6) (G1.6, GD).
- Establish the digital-twin/simulation environment (ROS2 + a simulator such as CARLA/Gazebo/Unity) as the mandatory validation substrate for every subsequent phase (G1.5 foundation).
**Go/no-go gate to Phase 1:** fused hazard map validated against CWFIS's own published data for a test region; ruvnet spike produces a documented decision (adopt/adapt/reject per component) with licensing verified; simulation environment operational and able to run at least one synthetic robot mission end-to-end.

### Phase 1 — Single-Domain Physical Pilots: Perception + Drones (no ground robots, no suppression)
**Goals touched:** G1.2, G2.1, G2.2, GB
**Dependencies:** Phase 0 gate met (fused hazard picture + simulation substrate operational)
**Work:**
- Build the ignition-risk nowcasting layer (G1.2) on top of Phase 0's fused data.
- Train/validate fire-smoke CV perception models on FLAME/FLAME2/FASDD/GWFP/WIT-UAS, deployed on target edge compute hardware (G2.1).
- Stand up PX4 + ROS2/XRCE-DDS (or ArduPilot+MAVROS where a vehicle type requires it) as the shared autopilot/middleware foundation (G2.2, GB).
- Single-drone field pilot: perception + autopilot integration, no swarm yet, over a real or controlled test area. Validate first in digital twin, then physically (G1.5 discipline applied from day one).
**Go/no-go gate to Phase 2:** single drone reliably detects/reports fire-relevant CV signals matching published-benchmark accuracy in field trial; ROS2 domain proven interoperable between at least the drone and the simulation/Command link; nowcasting model beats baseline on held-out data.

### Phase 2 — Command Platform + Multi-Drone Fleet Coordination
**Goals touched:** G1.4, G1.5 (full), GA, GB (swarm)
**Dependencies:** Phase 1 gate met
**Work:**
- Build the fleet-orchestration layer on top of ROS2 (G1.4) — this is explicitly custom work per research, not a ROS2 feature.
- Integrate Aerostack2 + Swarm-SLAM for coordinated multi-drone perimeter mapping (GB).
- Formalize the digital-twin + human-in-the-loop gating process as a procedural requirement, not just a design pattern (G1.5).
- Multi-drone field pilot: coordinated perimeter-mapping mission with real-time Command visualization.
**Go/no-go gate to Phase 3:** ≥3 drones complete a coordinated mapping mission with zero safety-relevant incidents; fleet-orchestration layer demonstrates conflict-free task allocation across a defined scenario suite in digital twin before the physical trial; human-in-the-loop gate procedurally enforced (no bypass path exists) for at least one behavior change during the pilot.

### Phase 3 — Ground Carrier Logistics (Stations + Carriers, no ground suppression)
**Goals touched:** G3, G4, GC
**Dependencies:** Phase 2 gate met (Command platform and orchestration layer must exist before Carriers report into it); independently, G4.2 (under-canopy localization) work can start in Phase 1/2 in parallel as a research track since it does not depend on drone fleet maturity.
**Work:**
- Scope Carrier automation narrowly as fixed-route/forwarder-style logistics (G4.1), per AORO/ATREF/FPInnovations precedent — explicitly not general off-road autonomy.
- Solve/validate non-GPS-dependent localization under forest canopy (G4.2) — treat as a first-class deliverable, not a later hardening pass, per ATREF's own findings.
- Adapt Autoware + `autoware.off-road` (Nav2 adapted) for the target terrain and vehicle (G4.3).
- Stand up at least one Station as an edge-compute/communications hub with defined telemetry latency/availability targets under intermittent connectivity (G3.2).
- Field pilot: one Carrier completes a repeated supply-transport route between a Station and a defined delivery point in forest terrain, with defined stop-and-call-human failure handling.
**Go/no-go gate to Phase 4:** Carrier completes the fixed-route pilot without GPS-loss-induced failure (localization backup proven), with all failure modes resulting in safe stop rather than silent failure; Station telemetry relay meets its defined targets under simulated connectivity loss; ROI module (G1.6) ingests real logistics-cost data from this pilot to validate the ROI loop before scaling.

### Phase 4 — Direct-Attack Firefighting: Perception and Remote-Operated Milestones Only
**Goals touched:** G5.1, G5.2, G5.3
**Dependencies:** Phase 2 gate (Command/perception maturity) and Phase 3 gate (mobility/localization-under-canopy maturity, since Firefighters inherit the same terrain problem as Carriers) both met.
**Work:**
- G5.1: deploy the shared perception stack (from G2.1) in live-fire training-ground trials to validate fire-line/hazard detection near active fire, with NO actuation attached yet.
- G5.2: adapt structural-firefighting mobility/actuation design patterns (Thermite RS3/Colossus: tracked mobility, heat-resistant construction, water/foam cannon integration) to a wildland-capable platform; bench-test heat/particulate/vibration survivability.
- G5.3: integrate remote human-operated suppression control, fed by Command's fused situational picture — this is the *first* time the program puts a suppression-capable machine near live fire, and it is explicitly human-operated, not autonomous.
**Go/no-go gate to Phase 5:** perception unit correctly identifies fire-line/hazard zones matching human-observer ground truth across multiple live-fire training exercises; mobility platform survives its defined environmental exposure profile; remote-operated suppression is demonstrated successfully and safely in a controlled burn or training exercise with no safety incidents. **Explicitly: autonomous suppression (G5.4) is not attempted in this phase and has no committed date.**

### Phase 5 — Autonomous Direct-Attack Firefighting (long-horizon R&D track) + National Scale-Out
**Goals touched:** G5.4, national scale-out of G1–G4
**Dependencies:** Phase 4 gate met; this phase should be treated as an ongoing, separately-funded, separately-timelined R&D program running in parallel with (not blocking) national scale-out of the already-validated Command/Fleet/Stations/Carriers capabilities.
**Work (two parallel tracks):**
- **Track A — Autonomous suppression R&D:** incrementally reduce human-operator involvement from G5.3's remote-operated baseline, validated at every step in simulation first, then in live-fire training exercises, with an explicit, conservative definition of "autonomous" (e.g., human-supervised autonomy with abort authority, before any fully unsupervised claim). No fixed completion date should be committed publicly; treat as multi-year.
- **Track B — National scale-out:** replicate the validated Command/Fleet(drones)/Stations/Carriers stack across additional Canadian regions, using G1.6's ROI module and G3.1's station-siting model to prioritize rollout order; formalize partnerships (§6) at national scale.
**Go/no-go gate:** none terminal — Track A has its own internal milestone gates (each increment of autonomy re-validated in sim + controlled trial before the next); Track B gates on ROI module showing positive BCR (≥1.0 per FEMA methodology) for each new region before rollout.

---

## 4. Risk Register

### 4.1 Technical risks

| Risk | Affected goal(s) | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| Autonomous suppression (G5.4) has no precedent anywhere — schedule/scope risk is open-ended | G5.4 | High | Severe | Sequenced last (Phase 5), run as separately-funded R&D track, no public date commitments, staged human-supervised-autonomy increments |
| ruvnet "brain" stack treated as locked-in without validation | G1.7 | Medium (mitigated by plan) | High if unmitigated | Phase 0 mandatory evaluation spike; explicit "reject and use conventional infra" as a valid outcome |
| `rvm`/`daa` licensing unverified | G1.7 | Medium | Medium-High (legal/supply-chain) | Verify actual LICENSE files in Phase 0 spike before any dependency commitment |
| GPS unreliable under forest canopy | G4.2, G5 (terrain-shared) | High (confirmed by ATREF) | High | Treat under-canopy localization as first-class Phase 3 deliverable, not later hardening |
| Off-road autonomy less mature than on-road (Nav2/Autoware require adaptation, no rigorously-evaluated complete open stack exists) | G4.3 | High | Medium-High | Scope Phase 3 narrowly (fixed-route forwarder pattern), set stakeholder expectations explicitly, budget more integration time than an on-road-AD-style estimate would suggest |
| ROS2 does not solve fleet-level orchestration | G1.4 | Certain (confirmed by industry precedent) | Medium (known, budgeted) | Explicit custom fleet-orchestration layer scoped in Phase 2, not assumed free from ROS2 |
| Connectivity cannot be assumed at incident sites | G1.5, G2.3, G3.2 | High | Medium-High | Edge-first/onboard-autonomy-first architecture; digital twin syncs periodically, not continuously; test explicitly under simulated total connectivity loss (G2.3 success criterion) |
| Wildfire-environment hardware harshness (heat, smoke/particulate, vibration) untested for any candidate platform in this domain | G2, G5.2 | Medium-High | High | Bench environmental testing required before every field trial (explicit in Phase 4 gate) |
| Swarm coordination frameworks (Aerostack2, Swarm-SLAM) not wildfire-proven | GB | Medium | Medium | Treat as adapt not adopt; budget integration/field-hardening time in Phase 2, don't assume turnkey |

### 4.2 Safety risks

| Risk | Affected goal(s) | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| Physical harm to humans from autonomous machines operating near fire | G5, G4 (Carriers sharing terrain with responders) | Low-Medium if gated properly | Severe | Mandatory digital-twin + human-in-the-loop gate before any physical deployment (G1.5), enforced procedurally; G5 sequenced through perception-only → remote-operated → supervised-autonomous increments, never skipping a stage |
| Silent failure modes (e.g., localization loss, actuator fault) near active fire | G4, G5 | Medium | Severe | Explicit "stop-and-call-human" failure handling required as a pass/fail success criterion (G4.1, Phase 3 gate) |
| Environmental harm from suppression agents (foam/chemical) at scale | G5.2, G5.4 | Low-Medium | Medium-High | Environmental-impact assessment folded into G5.2 bench-testing and Phase 4 gate; align with ecosystem-services framing already present in the FEMA BCA toolkit (G1.6) |
| Overclaiming autonomy readiness to stakeholders/funders creates pressure to skip validation gates | G5.4 program-wide | Medium | High | This document's explicit refusal to commit a date to G5.4; gates are described as blocking, not advisory |

### 4.3 Program risks

| Risk | Affected goal(s) | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| Funding tied to unrealistic "autonomous firefighting robot" timelines | Whole program, esp. G5 | Medium-High | Severe | Present Phase 4/5 explicitly as long-horizon R&D in all funding narratives; lead funding asks with the high-confidence Phase 0–3 capabilities |
| No Canadian-federal BCA equivalent to FEMA's toolkit identified | G1.6, GD, national framing | High (confirmed gap) | Medium | Dedicated follow-up research task (owned by researcher, see §6) before Phase 0's ROI module is finalized for the Canadian program specifically |
| Insurance-premium-reduction ROI framing risks distorting policy incentives (per research's California example) | G1.6, funding narrative | Medium | Medium | Use insurance-premium studies as supporting evidence only, never as the primary ROI claim; explicitly name the market-distortion caveat in any funding document |
| Required government/agency partnerships not secured in time | G1.1 (CWFIS access), G5 (live-fire training grounds), national scale-out | Medium | High | Partnership list and prerequisites made explicit in §6; begin outreach in Phase 0, not later |
| Dependency on a small/single-maintainer OSS ecosystem (ruvnet repos) for critical infrastructure | G1.7 | Medium | Medium | Phase 0 spike explicitly considers "reject and use conventional infra" as equally valid; avoid architecture lock-in before validation |

---

## 5. ROI / Logistics Design

### 5.1 ROI calculation approach (Command's G1.6 module)

Adopt FEMA's BCA methodology as the calculation engine, not a from-scratch model:
- **Inputs (cost side):** Fleet capex (drones, carriers, firefighting units), Station capex/opex, Command platform build/run cost, maintenance/turnaround cost from G3.3, training/certification cost.
- **Inputs (benefit side):** avoided suppression cost (using CBO's quantified prevention-vs-suppression framework: ~$65M prevention + $42M suppression vs. $236M suppression-only over a 50-year horizon, scaled to the deployment's actual scope), avoided property/ecosystem loss (via FEMA's Ecosystem Services and Green Infrastructure modules), and — as *supporting*, not primary, evidence — avoided insurance-premium cost (with the market-distortion caveat explicitly stated alongside any figure).
- **Methodology:** apply the cost-plus-net-value-change (C+NVC) framework as the theoretical justification for the investment level, and the Value-of-Information (VOI) framework to quantify the specific value of Command's detection/nowcasting layer (G1.1/G1.2) in reducing suppression-decision uncertainty — this gives the program a principled way to value the "boring" data-fusion work, not just the dramatic robot hardware.
- **Output:** a benefit-cost ratio (BCR) per proposed deployment scenario (e.g., "add a Station + 3 Carriers + 5 drones to Region X"), following FEMA's BCR ≥ 1.0 funding-qualification threshold as the internal go/no-go bar for scale-out decisions in Phase 5's Track B.
- **Canadian gap:** FEMA's toolkit is a US federal instrument. No Canadian-federal equivalent was identified in research. Before this module is used to justify the "Wildfire Robotics Canada" national program to Canadian federal/provincial funders, a dedicated follow-up (owned by researcher; see §6) must confirm whether an equivalent exists (e.g., via Public Safety Canada, NRCan, or provincial emergency-management bodies) or whether FEMA's methodology must be adapted/justified as a cross-jurisdictional best-practice import.

### 5.2 Logistics feeding the ROI loop

- **Station placement (G3.1)** is not a manual siting decision — it is an output of the ROI module: candidate station locations are scored by (a) fused hazard picture coverage from G1.1, (b) response-time/route feasibility for Carriers (G4) and drones (G2) from that station, and (c) the marginal BCR improvement of adding coverage there, versus its capex/opex cost.
- **Fleet allocation** (how many drones/carriers/firefighting units per Station) is likewise an ROI-module output, not a fixed ratio — driven by regional hazard density (G1.1/G1.2 outputs) and the avoided-loss estimates the BCA methodology produces for that region.
- **Carrier routing (G4.1)** feeds real operational cost/reliability data back into the ROI module (actual transport cost, actual failure rates from Phase 3 field pilots) — closing the loop so later phases' ROI projections are grounded in measured data, not just modeled assumptions from Phase 0.
- This creates a genuine feedback loop: Command's ROI module informs where to put Stations and how to allocate Fleet units → Carriers/drones operating from those Stations generate real cost/performance data → that data recalibrates the ROI module for the next region's siting decision (Phase 5, Track B).

---

## 6. Dependencies & External Partnerships

### 6.1 Data/API access prerequisites
- **CWFIS** (Canadian Forest Service/NRCan): data is licensed, not sold — confirm licensing/attribution terms before Phase 0 integration (G1.1). Directly relevant given the Canadian national-program framing.
- **NASA FIRMS, GOES GLM, Copernicus EFFIS:** free/official access; confirm any usage-volume or attribution requirements for a production (not research) integration.
- **WWLLN:** research/university-operated; confirm terms for continuous operational (non-research) use.
- **Regional commercial lightning networks (CLDN/NLDN/ENTLN/GLD360):** budget-dependent licensing decision — evaluate cost/precision tradeoff against WWLLN/Blitzortung in Phase 0 before committing spend.

### 6.2 Government/agency partnerships
- **Canadian wildfire agencies** (provincial fire services, Canadian Forest Service/NRCan): needed for CWFIS access terms, live-fire training-ground access (Phase 4 gate depends on this), and eventual field-deployment authorization.
- **ECCC (Environment and Climate Change Canada)** and/or **Public Safety Canada**: relevant to the Canadian-BCA-equivalent gap (§5.1) and to national-scale program legitimacy/funding.
- **Transport Canada / airspace regulators:** required before any multi-drone field pilot (Phase 1–2) — drone swarm operations near active or simulated wildfire incidents will require airspace authorization, not assumed as a software-only concern.
- **FEMA (US) as a methodology reference, not a funder:** useful for BCA Toolkit methodology access/consultation even though the program is Canada-first.

### 6.3 Hardware/industry partnerships
- **Forestry-robotics precedent holders** (Sweden's AORO lab, Canada's ATREF/FPInnovations, Finland's Rakkatec/RCM Harveri): potential technical-advisory or licensing partnerships given they hold the only real-world precedent for forest-terrain ground autonomy (G4).
- **Structural-firefighting robot vendors** (Howe & Howe/Thermite RS3, Shark Robotics/Colossus): potential partnership or component-licensing path for actuation/mobility hardware precedent feeding G5.2, even though neither vendor offers autonomy or wildland capability directly.
- **Drone airframe/autopilot ecosystem** (PX4/ArduPilot open-source communities, and companies building on `aerial-autonomy-stack`): community engagement rather than formal partnership, but worth tracking given the September/October 2025 arXiv paper on heterogeneous UAV swarm architecture as a field the program should stay current with.

### 6.4 Immediate researcher follow-ups required before later phases
1. Confirm whether a Canadian-federal or provincial equivalent to FEMA's BCA Toolkit exists (blocks finalizing G1.6 for the Canadian program specifically — needed before Phase 0 exit).
2. Confirm current build status of `agentic-robotics`'s ROS2 bridge and verify `rvm`/`daa` licensing directly against their LICENSE files (part of the Phase 0 spike, P0.5).
3. Investigate concrete non-GPS localization approaches proven in forest-canopy conditions (beyond ATREF's high-level finding that GPS alone is insufficient) — needed before Phase 3 detailed design of G4.2.

---

## Summary of Sequencing Logic (GOAP rationale)

The ordering above is an A*-style path through the goal-state space where each phase's "cost" is its validation risk and each phase's entry condition is the prior phase's proven (not assumed) capability:

Phase 0 (data/simulation, near-zero physical risk) → Phase 1 (single drone, physical but contained) → Phase 2 (multi-drone coordination + the Command orchestration layer that everything else depends on) → Phase 3 (ground logistics, narrow-scoped per real-world forestry precedent, in parallel research-track on localization) → Phase 4 (firefighting perception + remote-operated only — the first time a suppression-capable machine goes near live fire, done under human control) → Phase 5 (the two long-horizon tracks: autonomous suppression R&D, kept separately funded and un-dated, running alongside national scale-out of everything already validated). This ordering directly reflects the research's core finding: the components with the strongest existing precedent (detection, ROI, drone autopilot) are adopted early and cheaply, while the component with zero precedent anywhere (autonomous direct-attack firefighting) is isolated to its own track so it cannot become a scheduling dependency for the rest of the program.
