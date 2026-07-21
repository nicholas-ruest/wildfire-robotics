# Domain-Driven Design Context Map

## Strategic domains

| Context | Classification | Owns | Does not own |
|---|---|---|---|
| Hazard Intelligence | Core | Normalized observations, provenance, quality, hazard picture | Mission decisions or vehicle commands |
| Predictive Planning | Core | Nowcasts, spread scenarios, uncertainty, planning recommendations | Source observations or command authority |
| Incident Command | Core | Incidents, operational periods, objectives, authority, assignments, restrictions | Low-level vehicle control |
| Mission Control | Core | Mission plans, policy validation, allocation, leases, deconfliction, execution state | Robot-local control loops |
| Fleet Operations | Core | Assets, capabilities, health, telemetry summaries, configuration eligibility | Vendor protocol details |
| Vehicle Integration | Supporting | ROS/MAVLink adapters, command gateway, telemetry normalization | Fleet policy or incident authority |
| Station Operations | Supporting | Robot habitats, microgrids, charging, station availability, edge sync, energy, local inventory and maintenance facilities | Regional supply portfolio or vehicle design authority |
| Logistics | Core | Supply plans, resources/custody, transport pods/carriers, mobilization waves, routes, deliveries, water sources and relay cycles | Vehicle maintenance records or charger electrical safety |
| Suppression Operations | Core/R&D | Suppression envelope, target, agent, dose, teleoperation and actuation outcome | Authorization policy or generic navigation |
| Safety Assurance | Core | Hazards, constraints, ODD, evidence, promotion decisions, incidents/near misses | Mission optimization |
| Identity and Access | Generic | Principals, devices, credentials, roles, policies, approvals | Incident objectives |
| Commercial Operations | Supporting | Tenants, contracts, entitlements, metering, support, regional economics | Active safety authority |
| Vegetation Management | Core | Preventive fuel-treatment prescriptions, treatment units, robot work, biomass disposition, effectiveness evidence | Active-fire suppression or incident authority |
| Robot Care and Recovery | Supporting/Core-enabling | Maintenance policy/work, medic recovery, damage/quarantine, hospital repair/recertification and retirement | Fleet eligibility, habitat energy, incident rescue priority or supply ownership |
| Aerial Deployment Operations | Core/R&D | Blanket configuration/assembly, payload manifest, release handshake, coupled airborne deployment, ground installation and component accounting | Aircraft flight/release veto, incident authority, suppression chemistry, vegetation work, fleet eligibility or repair disposition |

## Upstream/downstream relationships

```text
External hazard providers
  -> Hazard Intelligence
      -> Predictive Planning
          -> Incident Command
              -> Mission Control
                  -> Fleet Operations -> Vehicle Integration -> physical vehicles
                  -> Logistics --------^                 |
                  -> Suppression Operations <------------+
                  -> Vegetation Management -> Fleet Operations / Vehicle Integration
                  -> Robot Care and Recovery -> Fleet Operations / Station Operations / Logistics
                  -> Aerial Deployment Operations -> Vehicle Integration / Suppression / Vegetation

Identity and Access -> all command/administration boundaries
Safety Assurance    -> constrains Incident Command, Mission Control, Vehicle Integration,
                       Logistics, Suppression Operations, and every release promotion
Station Operations  <-> Mission Control / Fleet Operations / Logistics (edge synchronization)
Commercial Operations <- operational usage events (never a command dependency)
Vegetation Management <- hazard/fuel intelligence; publishes treated-fuel and effectiveness observations
Robot Care and Recovery <- fleet/vehicle fault and exposure facts; publishes service, quarantine, recertification and retirement facts
Aerial Deployment Operations <- incident/safety/aircraft authority and prediction; publishes release/deployment/installation/accounting facts
```

## Integration contracts

- **Hazard Intelligence → Predictive Planning:** published language. Versioned `ObservationAccepted`, `HazardPictureUpdated`, and `DataQualityDegraded` events with provenance and uncertainty.
- **Predictive Planning → Incident Command:** customer/supplier. Recommendations are immutable, versioned advisory products; Incident Command decides whether to use them.
- **Incident Command → Mission Control:** customer/supplier. An `OperationalAssignment` conveys authority, objective, constraints, validity, and approvers—not actuator instructions.
- **Mission Control → Vehicle Integration:** conformist at the capability contract only. Commands are short-lived, idempotent, policy-approved intents; the adapter maps them to vendor protocols.
- **Fleet Operations ↔ Vehicle Integration:** anti-corruption layer. Vendor/ROS/autopilot types never enter Fleet aggregates.
- **Safety Assurance → operational contexts:** policy/constraint service plus signed evidence. If unavailable, cached, unexpired local constraints apply; expansion of authority fails closed.
- **Commercial Operations:** open-host service consuming metering events. It has no synchronous dependency in mission execution.
- **Hazard Intelligence → Vegetation Management:** published language. Fuel/terrain/hazard products are immutable, freshness-labelled planning inputs.
- **Vegetation Management → Hazard Intelligence/Predictive Planning:** published treatment geometry, actual method, imagery, and effectiveness become provenance-aware observations, never silent map edits.
- **Mission Control ↔ Vegetation Management:** work packages become missions only after prescription, authority, tool, ODD, exclusion, and safety validation.
- **Fleet Operations ↔ Robot Care:** Fleet publishes configuration/health/eligibility and consumes evidence-backed maintenance, recertification and retirement outcomes; Robot Care cannot directly make a robot operationally eligible.
- **Station/Logistics ↔ Robot Care:** Station supplies maintenance/quarantine/hospital capacity; Logistics owns medic/repair transport custody, parts, consumables, salvage and hazardous disposition.
- **Incident/Mission/Safety → Aerial Deployment:** incident objective, robot allocation/lease and promoted exact configuration/ODD constrain the mission; none is sufficient alone to release payload.
- **Aircraft adapter ↔ Aerial Deployment:** aircraft authority owns loading/release veto and flight state; the context supplies a configuration/manifest/corridor handshake without commanding aircraft flight systems.
- **Aerial Deployment → Suppression/Vegetation:** an installed blanket publishes bounded panel/footprint/health facts; suppressant and vegetation commands remain with their owning contexts.
- **Aerial Deployment → Logistics/Robot Care:** serialized accounting and exposure/damage facts drive recovery, quarantine, repair, reuse, recycling or approved sacrificial disposition.

## Consistency boundaries

- Strong consistency is local to one aggregate and its outbox.
- Cross-context workflows are idempotent sagas with explicit timeout, compensation, and operator escalation.
- Command authorization, mission lease acquisition, and safety-envelope version checks use optimistic concurrency and fail closed.
- Telemetry and read models are eventually consistent and always display freshness.

## Ownership rules

Each context owns its schema, migrations, public contracts, tests, SLOs, dashboards, threat model, runbooks, and on-call rotation. Direct database access across contexts is prohibited. Shared libraries may contain technical primitives and contract-generated types, never shared domain aggregates.

Rust workspace crates are the authoritative implementation of these contexts. TypeScript is restricted to operator-console and generated client code under ADR-016.
