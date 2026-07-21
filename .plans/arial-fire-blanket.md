# Wildfire Robotics Aerial Fire Blanket System

## 1. Subsystem purpose

The Wildfire Robotics Aerial Fire Blanket System is an aircraft-deployed robotic system that rapidly installs a modular fire-resistant membrane over vegetation, predicted ignition zones, firebreak corridors and critical infrastructure.

The system is not intended to drop a blanket directly onto a fully developed crown fire.

Its primary purposes are to:

- prevent or reduce ignition in locations where lightning or ember landings are predicted;
- create an emergency firebreak ahead of an advancing fire;
- protect substations, communications sites, evacuation routes and communities;
- reduce the heat and ember exposure reaching protected ground;
- give ground robots additional time to remove vegetation, apply retardant and establish containment.

Long-term fire retardants decrease fire intensity and slow fire advancement, but they do not make vegetation completely fireproof.

## 2. Selected aircraft

### Full-scale aircraft: CC-177 Globemaster III

The operational system will use a CC-**177**/C-17 Globemaster **III**-class aircraft.

The CC-**177** is selected because it provides:

- a rear loading and airdrop ramp;
- strategic range;
- approximately 72,**727** kg of payload capacity;
- sufficient internal volume for the membrane deployment cradle;
- demonstrated ability to transport as many as **102** paratroopers;
- compatibility with heavy guided-airdrop operations.

Canada currently operates the CC-**177** from 8 Wing Trenton, Ontario.

The blanket must be carried **inside the aircraft**, not attached externally beneath it. An external roll would produce unacceptable aerodynamic drag, structural loading and emergency-release risks.

### Prototype aircraft: LM-100J or C-130J-30

A smaller prototype can use an LM-**100J** or C-**130J**-30 with approximately 20 to 30 robots and a 2-to-4-acre membrane.

The C-**130J**-30 has a maximum allowable payload of approximately 19,**958** kg. The LM-**100J** is the civil version of the C-**130J** and Lockheed Martin identifies aerial firefighting and delivery as potential mission applications.

## 3. Full-scale coverage

The baseline membrane contains **100** connected panels.

Each panel measures:

**25 metres × 25 metres**

Each panel covers:

****625** square metres**

The complete system covers:

**62,**500** square metres, or approximately 15.4 acres**

The panels can be arranged in two principal configurations.

### Firebreak Mode

```text 50 metres wide × 1,**250** metres long ```

The **100** panels form two parallel rows of 50 panels.

This produces:

- a 1.25-kilometre containment strip;
- approximately 15.4 acres of covered vegetation;
- two rows of 50 robots controlling opposite sides;
- approximately 25 metres between robots along each edge.

This is the preferred operational configuration because wildfires are normally fought through containment lines, fuel breaks and retardant lines rather than by attempting to cover the entire fire.

### Shield Mode

```text **250** metres × **250** metres ```

The **100** panels form a 10-by-10 square.

This configuration is used for:

- predicted lightning-strike zones;
- electrical substations;
- emergency command sites;
- neighbourhood interfaces;
- fuel-storage exclusion areas;
- evacuation-route protection.

A single deployment covers approximately 15.4 acres. Covering 1,**000** acres continuously would require approximately 65 equivalent aircraft loads, which is why the system should prioritize strategic corridors and assets rather than attempting to blanket an entire wildfire.

## 4. Preliminary payload model

The following figures are engineering targets, not validated final weights.

| Component                                           | Preliminary mass |
| --------------------------------------------------- | ---------------: |
| 100 firefighting robots at 180 kg each              |        18,000 kg |
| 100 guided parafoil, harness and tether systems     |         5,000 kg |
| 62,500 m² membrane at 0.35 kg/m²                    |        21,875 kg |
| Deployment cradle, reels and extraction system      |         8,000 kg |
| Anchors, edge retardant, sensors and communications |        10,000 kg |
| **Estimated total**                                 |    **62,875 kg** |
| CC-177 listed maximum payload                       |    **72,727 kg** |
| **Preliminary payload margin**                      |     **9,852 kg** |

The system therefore fits within the aircraft’s theoretical payload limit, but only if the membrane achieves a maximum average mass near ****350** grams per square metre**.

This material requirement is one of the project’s most important research challenges. A heavier membrane could quickly consume the remaining aircraft capacity.

## 5. Membrane construction

The fire blanket must not be one uninterrupted piece of fabric.

It must be a network of **100** independently controlled, replaceable panels connected by fire-resistant flexible joints.

Each panel requires:

- reflective upper surface;
- fire-resistant structural fabric;
- thermally insulated underside;
- reinforced corner and edge attachment points;
- rip-stop boundaries;
- controlled air vents;
- closable vent flaps;
- temperature and ember sensors;
- sacrificial overload connectors;
- attachment points for ground anchors;
- identification tag and panel health telemetry.

The material should be designed primarily to:

- block embers;
- reflect radiant heat;
- isolate vegetation from oxygen and burning debris;
- retain water or fire retardant where appropriate;
- resist short-duration flame contact;
- prevent a local tear from destroying the entire structure.

It must not be described as completely fireproof. Existing wildfire fire shelters can lose protective effectiveness during direct flame contact, demonstrating how difficult sustained flame exposure is for lightweight materials.

## 6. Aircraft packing system

The blanket is carried inside the CC-**177** on a specialized extraction cradle.

Because a 50-metre-wide membrane cannot be stored as a simple cylindrical roll, it is:

1. folded laterally into narrow layers;
2. rolled or accordion-packed longitudinally;
3. divided into controlled deployment sections;
4. connected to **100** numbered robot tether lines;
5. secured inside a stabilizing deployment pod.

Each robot is assigned:

- a robot identifier;
- a panel identifier;
- a deployment sequence;
- an airborne formation position;
- a landing coordinate;
- an anchoring location;
- a post-landing firefighting assignment.

The robots do not jump and then attempt to catch the blanket freely.

Every robot is attached to its assigned membrane node before deployment using a controlled-length tether and powered reel.

## 7. Deployment sequence

### Phase 1: Mission planning

The Wildfire Robotics platform receives:

- lightning probability;
- weather forecasts;
- wind speed and direction;
- fuel moisture;
- vegetation classification;
- terrain and slope;
- active fire perimeter;
- predicted fire-spread direction;
- roads, communities and critical infrastructure;
- airspace restrictions;
- aircraft and robot availability.

The deployment planner determines whether the mission requires Firebreak Mode or Shield Mode.

A human incident commander must approve the final deployment area.

### Phase 2: Aircraft approach

The CC-**177** approaches the release corridor from an approved direction calculated using wind, terrain, smoke and aircraft constraints.

The blanket remains packed inside the aircraft.

The system verifies:

- all **100** robots are operational;
- parafoils are armed;
- tethers are connected;
- panel sensors are functioning;
- drop-zone coordinates are current;
- no unauthorized aircraft or people are inside the deployment area.

### Phase 3: Payload extraction

An extraction parachute pulls the deployment cradle through the rear cargo ramp.

The cradle deploys a stabilizing drogue so that the packed membrane does not tumble uncontrollably.

The system begins releasing robots in sequenced groups rather than releasing all **100** simultaneously.

### Phase 4: Robot parafoil deployment

Each robot opens its own steerable parafoil.

The robots use:

- **GNSS**;
- inertial navigation;
- terrain-relative vision;
- peer-to-peer positioning;
- wind estimation;
- collision avoidance.

**GPS**-guided autonomous parafoil systems already exist through the Joint Precision Airdrop System. Current systems autonomously steer parafoils toward designated landing areas, providing a technical foundation for the concept, although a **100**-robot shared membrane has not been demonstrated.

### Phase 5: Formation establishment

The robots fly toward their assigned positions.

In Firebreak Mode, they form two parallel lines.

```text Robot line A ●——●——●——●——●——●

      Membrane

●——●——●——●——●——● Robot line B ```

The tethers initially remain short so the membrane stays compact.

Once the robots establish safe separation, powered reels gradually extend the tethers.

### Phase 6: Controlled blanket deployment

The deployment cradle releases one membrane section at a time.

The membrane must not suddenly open into a solid 15-acre parachute.

Instead:

- panel vents remain open;
- folds release progressively;
- robots actively control membrane tension;
- overloaded sections disconnect safely;
- robots compensate for gusts;
- the system pauses deployment when formation stability deteriorates.

Progressive deployment is essential because flexible parachute-like surfaces can generate extremely high opening loads. Reefing and staged inflation are established methods for reducing parachute opening shock.

### Phase 7: Terrain alignment

Before landing, the fleet maps:

- trees;
- rocks;
- utility poles;
- buildings;
- water;
- slope changes;
- active flames;
- people and vehicles.

Individual panels can be:

- raised;
- lowered;
- disconnected;
- repositioned;
- omitted around obstacles.

The membrane therefore conforms to the environment as a network rather than behaving like a single rigid blanket.

### Phase 8: Landing and anchoring

The robots land first along the membrane perimeter.

After landing, each robot:

1. transitions from parafoil mode to ground mode;
2. reels in or releases excess tether;
3. drives to its exact anchor position;
4. installs ground anchors;
5. tensions its assigned panel;
6. confirms contact with the ground;
7. closes the panel’s airborne vents;
8. seals gaps with adjacent panels.

The parafoils detach and are either recovered automatically or repurposed as additional edge barriers.

### Phase 9: Active firefighting

Once the membrane is installed, the robots do more than hold it down.

They:

- remove vegetation around the membrane perimeter;
- apply water, gel or approved retardant along exposed edges;
- extinguish embers landing outside the blanket;
- monitor temperatures above and below every panel;
- reposition panels where the fire changes direction;
- repair or replace damaged sections;
- maintain communications relays;
- identify approaching fire breakthroughs;
- create secondary containment lines.

The blanket slows ignition and fire spread while the robots construct a more durable firebreak.

### Phase 10: Recovery or sacrificial operation

After the threat passes, the system decides whether to:

- roll and recover the membrane;
- transport reusable panels to another fire line;
- abandon damaged sacrificial panels;
- recycle damaged material;
- leave selected panels temporarily protecting infrastructure.

Every panel and robot records its deployment, exposure, damage and recovery status.

## 8. Control architecture

The subsystem should connect to the larger Wildfire Robotics platform through five control layers.

### Strategic Prediction Layer

Determines where fire is likely to start or travel.

### Aerial Mission Planner

Calculates:

- aircraft route;
- deployment orientation;
- drop corridor;
- panel configuration;
- robot landing locations;
- alternate and abort zones.

### Airborne Swarm Controller

Coordinates:

- parafoil navigation;
- robot spacing;
- tether length;
- membrane tension;
- collision avoidance;
- deployment timing.

### Ground Firefighting Controller

Assigns:

- anchoring;
- vegetation removal;
- edge suppression;
- sensor monitoring;
- panel repair;
- fireline expansion.

### Safety and Authorization Layer

Requires human approval for:

- aircraft launch;
- final deployment coordinates;
- payload release;
- operation near communities;
- use of chemical retardants;
- entry into controlled airspace.

No autonomous component may override aircraft safety systems or incident-command authority.

## 9. Abort and failure behaviour

The system must assume that individual robots, parafoils and panels will fail.

It requires:

- automatic separation when tethers cross;
- breakaway connectors that prevent one robot from pulling down the formation;
- independent panel isolation;
- reserve parafoils for the deployment cradle;
- safe panel jettison zones;
- automatic cancellation if wind exceeds the certified envelope;
- automatic diversion if people enter the landing zone;
- robot-controlled emergency landing coordinates;
- membrane vent reopening if the sheet begins acting like an uncontrolled sail;
- multiple communications paths;
- local robot control if the central network connection is lost.

A single robot or panel failure must not produce total system failure.

## 10. Development stages

### Stage 1: Ground demonstration

- Four robots
- Four 5 m × 5 m panels
- No aircraft
- Test tensioning, anchoring, venting and panel connection

### Stage 2: Drone-drop demonstration

- Four to ten robots
- Up to 50 m × 50 m
- Low-altitude controlled test range
- Test parafoils, tethers and staged unfolding

### Stage 3: C-130J/LM-100J prototype

- 20 to 30 robots
- Approximately 2 to 4 acres
- Test operational aircraft extraction and coordinated landing

### Stage 4: CC-177 partial-scale test

- 50 robots
- Approximately 7 acres
- Validate large-formation control and membrane loading

### Stage 5: Full operational demonstration

- **100** robots
- **100** panels
- 15.4 acres
- 1.25-kilometre Firebreak Mode or **250**-metre Shield Mode

## 11. Core project statement

The Wildfire Robotics Aerial Fire Blanket System uses a CC-**177** Globemaster **III** to deploy **100** autonomous firefighting robots and a **100**-panel, 15-acre fire-resistant membrane.

Each robot descends under an individually guided parafoil while remaining pre-connected to an assigned membrane panel. The swarm progressively unfolds, positions and lowers the membrane, then lands, anchors the panels and transitions into active ground firefighting.

The system is deployed ahead of an advancing wildfire or over a predicted ignition area. It is designed to block embers, reduce radiant heating, isolate vegetation, support retardant application and rapidly establish a robotic containment line.

The first operational configuration creates a 50-metre-wide, 1.25-kilometre-long emergency firebreak from a single aircraft deployment.