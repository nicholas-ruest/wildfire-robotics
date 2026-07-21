# Production Readiness Standard

This checklist is a release control, not guidance. An evidence link and accountable approver are required for every applicable item.

## Product and domain

- Requirements and safety constraints are traceable to executable tests.
- Operator workflows are validated with incident-command personnel.
- Data/model outputs expose provenance, freshness, confidence, and limitations.
- Accessibility, localization, audit export, retention, and customer isolation are verified.

## Engineering

- Reproducible builds, pinned toolchains, reviewed migrations, compatibility policy, and rollback exist.
- Boundary inputs use schemas, limits, authentication, validation, and idempotency.
- Distributed workflows tolerate duplication, reordering, delay, partition, clock skew, and restart.
- Performance, endurance, capacity, and cost tests meet approved budgets.

## Safety and security

- Hazard analysis and threat model are current; mitigations have tests and owners.
- Local fail-safe, emergency stop, geofence, actuation envelope, and command expiry are independently tested.
- SBOM and signed provenance exist; critical findings and release-blocking vulnerabilities are zero.
- Privileged access, key rotation, incident response, forensics, and disclosure processes are exercised.

## Reliability and operations

- Metrics, logs, traces, model/data quality signals, SLOs, alerts, and runbooks are deployed.
- Backup restoration and regional/station recovery have passed exercises.
- Rollout, canary, feature isolation, rollback, support escalation, and status communication are rehearsed.
- On-call ownership, spare parts, field service, calibration, and preventative maintenance are funded.

## Cyber-physical promotion

- Simulation precedes HITL; HITL precedes controlled field trials; field trials precede operational deployment.
- Test authorization defines personnel, environmental envelope, exclusion zone, abort criteria, and emergency services.
- Near misses block promotion pending review. Human authority and emergency stop cannot be disabled by configuration.
- Operational deployment is limited to the approved domain; excursions force safe state and operator intervention.

## Million-asset and adaptive-system promotion

- The approved workload model demonstrates 1,000,000 registered and concurrently connected assets, declared active mixes, 1 Hz normalized summaries, 10× reconnect burst, hot-region skew, partition split/merge, regional isolation, rolling upgrade, recovery, and cost under ADR-053.
- No fleet-wide consensus, scan, lock, synchronous scheduler, or single-region dependency exists; stale epochs and split brain are fenced under fault injection.
- ruv-drone is pinned and isolated behind conformance contracts; bounded cohort, relay, collision, energy, link-loss, autopilot, simulation and HITL evidence passes before each promoted capability.
- RVM is optional and replaceable; capability/proof/witness, graph poisoning, partition failure, checkpoint recovery, timing/resource bounds, and conventional fallback are verified. RVM/coherence output remains advisory.
- RuPixel indexes are reproducible from immutable media manifests; retrieval quality, latency, domain shift, false similarity, privacy, index corruption and fallback are measured. Retrieval never becomes an observation without verification.
- Lightning ML beats or complements the approved authoritative baseline on held-out geography/time/fire years, calibration, rare-event metrics and prospective shadow utility; source outage, drift, abstention and rollback are exercised.
- The prediction-observation loop records unobserved/censored areas, sampling policy, interventions, negative outcomes and label confidence; no online learning bypasses model/policy promotion.
- Supply-chain plans reconcile inventory/custody/compatibility, maintenance, energy, lead-time uncertainty, transport and substitutions under disruption tests.
- ROI outputs reproduce from immutable facts and versioned assumptions, separate capital from recurring costs, propagate uncertainty and expose human-only counterfactual and sensitivity.
- Vegetation robots prove prescription/geofence/exclusion/tool-stop behavior and planned-versus-actual/effectiveness evidence in simulation, HITL and controlled operations.
- Each habitat passes site-specific monthly energy/resource, low-solar, cold-soak, snow/smoke, outage, fuel-delay, fire-zone, thermal-runaway, islanding, black-start, emergency-reserve, evacuation and restoration tests; annual-average solar is not accepted as capacity evidence.
- One-million battery twins and partitioned charging schedules pass identity/custody, BMS compatibility, stale/conflicting estimate, simultaneous recall, charging surge, charger/feeder failure, thermal isolation, degradation and quarantine tests without global scheduling dependency.
- Standard pods pass measured mass/centre-of-gravity, securement, structural, energy isolation, ingress, thermal/fire, data/charge interface, loading/unloading, emergency access and intermodal transfer tests.
- Hybrid-electric carriers and bounded platoons pass traction/reserve-energy, range-extender, braking/steering, load shift, cold/smoke/weather, obstacle, V2X/leader/cloud loss, route ODD, safe stop, manual recovery and destination-admission tests. Vehicle PV is not credited for propulsion range.
- A 100,000-robot mobilization exercise/simulation uses measured pod, carrier, road/bridge/ferry/rail/barge, loading, energy, staging, unloading, inspection and assignment constraints and meets the approved useful-arrival window without double reservation or destination gridlock.
- Maintenance robots pass procedure/tool/module compatibility, energy/tool isolation, measurement, serialized-part custody, calibration, test, ambiguity escalation and failed-service recovery for every promoted autonomous task.
- Medic pods pass disabled-robot localization, scene/route ODD, stored-energy/tool stabilization, lift/tow/cradle compatibility, active-fire retreat, communications loss, medic failure, custody transfer, hazardous-load separation and safe destination-diversion tests.
- Robot hospitals pass correlated-damage surge, fire-separated battery/robot quarantine, decontamination, diagnosis, parts provenance, repair, calibration, burn-in, recertification, repeat-fault, salvage, data sanitization, trust revocation and hazardous-disposition exercises.
