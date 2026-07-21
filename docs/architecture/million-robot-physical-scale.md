# Million-Robot Physical Scale and Viability Model

This document converts the million-robot requirement into measurable physical and computing constraints. Values below are reference scenarios, not hidden product assumptions. Every implementation must replace them with measured robot, route, site and incident profiles and retain the equations.

## Governing principle

The viable system is a distributed network of habitats, pods, carriers, fleet cells and regional control planes. It is not one warehouse, one battery, one carrier, one convoy, one message bus, or one scheduler. Scale is achieved by bounded replication and locality.

## Required sizing inputs

| Symbol | Input | Unit |
|---|---|---|
| `N` | robot count by class/readiness state | robots |
| `m_r`, `m_f` | robot and fixture/pod-share mass | kg/robot |
| `e_b`, `p_c` | usable onboard energy and charge power | kWh, kW/robot |
| `d`, `e_d` | daily duty fraction and energy per active day | fraction, kWh/day |
| `q_p`, `m_p` | robots and gross mass per pod | robots, tonnes |
| `m_c`, `v_c` | legal carrier payload and average trip velocity | tonnes, km/h |
| `s_h` | robots per habitat | robots/site |
| `y_{pv,m}` | monthly site PV yield | kWh/kWp/day |
| `a` | required no-generation critical autonomy | days |
| `L` | distribution/charging/thermal reserve loss factor | fraction |
| `T` | required mobilization arrival window | hours |

## First-order equations

- Fleet onboard energy: `E_fleet = Σ(N_class × e_b,class)`.
- Daily habitat energy: `E_day = N × (d × e_d + standby + thermal + maintenance) / (1-L)`.
- Site PV: `P_pv = E_day,site / y_pv,design-month`; use the selected design month and measured snow/availability derates, not annual average.
- Critical stationary storage: `E_store ≥ a × E_critical / usable_depth_of_discharge`, plus power and cold-temperature constraints.
- Generator/fallback energy: covers the declared low-solar/outage scenario after load shedding and must include fuel delivery lead-time uncertainty.
- Gross mobilized mass: `M = N_move × (m_r + m_f)`.
- Minimum payload trips: `Trips ≥ M / m_c`, then increase for volume, axle, route, pod, charging and turnaround constraints.
- Required average useful arrival rate: `R = N_move / T`; every load, road, energy, unload, staging and assignment stage must sustain `R` or it is the bottleneck.

## Reference mass reality check

At an exceptionally small combined robot-and-fixture mass of 300 kg, moving 100,000 robots means 30,000 tonnes before carrier structure, energy, tools, spares and support equipment. At a 20-tonne usable road payload, the theoretical floor is 1,500 payload trips; volume and route constraints increase it. At 1,000 kg per robot/fixture, the floor is 100,000 tonnes and 5,000 such trips. Therefore one 100,000-robot road vehicle is rejected. A viable requirement is one coordinated mobilization wave distributed across many pods, carriers and intermodal routes.

## Reference energy reality check

If one million robots average 20 kWh onboard, fleet nameplate energy is 20 GWh. If 10% perform a 20 kWh duty cycle per day, traction/tool replenishment alone is 2 GWh/day before standby, heating, charging and station losses. At an illustrative design yield of 2 kWh/kWp/day, that component alone requires roughly 1 GWp of PV; a northern winter design month can be materially worse and some sites may approach negligible solar availability. Consequently:

- habitats are distributed near workload to reduce mobilization energy;
- site PV is sized from official monthly resource data and measured snow/shading;
- batteries shift energy but do not solve seasonal deficit;
- grid, hydro, wind and dispatchable fuel generation complement solar by site;
- readiness classes limit simultaneous full charge while emergency reserves remain protected;
- transportable generation/storage and energy logistics are part of incident supply planning.

Natural Resources Canada publishes approximately 2 km solar-resource/PV-potential data and warns that its estimates are first-order lifetime averages that do not include monthly snow and other variations. Site design therefore requires bankable onsite resource, load and reliability analysis rather than a national average.

## Habitat topology

| Layer | Indicative function | Scale behavior |
|---|---|---|
| Dock/pod | isolate, charge, thermal condition, attest one robot/battery | local hard safety; no cloud dependency |
| Local habitat | house tens/hundreds, PV/storage/generator, first-line maintenance | one edge energy/fleet cell |
| Sector depot | aggregate pods, tools, spares and carriers; heavy service | coordinates local habitats |
| Regional hub | intermodal transfer, major maintenance, inventory, model/data distribution | regional control and disaster recovery |
| Global plane | product configuration, aggregate capacity, learning and portfolio analytics | summaries only; no direct charge loop |

The site count is derived from geography and response objectives, not only division: `Sites ≥ max(N/s_h, area coverage requirement, travel-time coverage requirement, correlated-hazard separation requirement)`.

Every habitat has routine maintenance-robot coverage and a declared medic-pod response SLO. Sector/regional capacity planning includes disabled-robot retrieval, fire-separated quarantine, decontamination, hospital bays, diagnostic/calibration/burn-in equipment, spares and retirement flow. Medic coverage is solved like emergency service coverage—by travel/scene/recovery time and correlated incident damage—not merely by average work orders.

## Information-scale viability

- One million stable asset/battery/pod identities use sharded authoritative twins.
- High-rate BMS, motion, camera and tool data terminates locally; only exceptions, events and bounded summaries flow upstream.
- At a one-message-per-second normalized fleet summary, the platform must sustain 1 million messages/second plus reconnect bursts, but this load is partitioned by cell/region and retained by tier.
- Charging schedules are solved locally under a regional energy budget; no global optimization blocks a charger.
- Global views are eventually consistent materializations with declared freshness.
- Command/authority paths address cells and exact targets, use leases/fencing/idempotency and never broadcast uncontrolled actuation.
- A digital twin does not mirror every sensor sample; it stores current authoritative operational state plus immutable evidence references.

## Mobilization architecture

1. Incident Command declares objective, geography, urgency and constraints.
2. Mission/Logistics computes required capability over time rather than raw robot count.
3. Fleet queries regional/cell capacity summaries and reserves eligible robots.
4. Logistics selects compatible pods, carriers and intermodal legs and solves the capacitated flow plan.
5. Stations precondition batteries/tools, isolate transport energy, attest load and stage pods.
6. Carrier platoons depart in bounded independently recoverable waves.
7. Destination stations admit, unload, inspect, energize and assign robots at their sustainable rate.
8. Reverse logistics recovers batteries, failed robots, pods, biomass/waste and carriers.

## Acceptance tests

- One million connected asset and battery twins under normal, hot-region and reconnect-burst profiles.
- Loss of cloud/region/habitat/controller without unsafe charge, thermal event, duplicate dispatch or lost custody.
- Coordinated charge surge, cold soak, PV collapse, generator failure, fuel delay and fire-zone isolation.
- Cell/site split/merge and fleet reassignment without stale fencing or double reservation.
- 100,000-robot mobilization simulation using measured pod/carrier/road/rail/barge/load/unload/energy constraints and declared arrival SLA.
- Autonomous-carrier platoon loss, road closure, load shift, brake/power fault, V2X loss and manual recovery.
- Destination congestion proves admission control; plans cannot count a robot operational until unloaded, checked, energized, connected and mission-eligible.
- Correlated heat/smoke/water/impact damage produces realistic medic queues, hazardous-load segregation, quarantine/hospital saturation, spares consumption, repair yield and retirement flow without allowing damaged robots into ordinary charging/storage.
- Capacity and cost results include p50/p95/p99 latency, throughput, queueing, energy, utilization, bottleneck, recovery and cost per ready/deployed robot.

## Viability decision

The concept is technically viable as a distributed infrastructure program. It is not viable as a monolithic solar warehouse or single 100,000-robot carrier. The architecture must pass the parametric energy, mass, route, charging, staging, information and destination-throughput gates above before any fleet-size claim is accepted.
