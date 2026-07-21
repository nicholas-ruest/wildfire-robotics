# ADR-068: Robot care, recovery, and maintenance network

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: maintenance, recovery, robots

## Context

Firefighting and vegetation robots will suffer heat, smoke, water, suppressant, impact, tool, battery, sensor, mobility, communications, and structural damage. Disabled robots cannot be abandoned, returned to ordinary charging racks, or moved without knowing their energy and contamination hazards. Millions of robots also require continuous preventive care at distributed habitats.

## Decision

Create a Robot Care and Recovery bounded context and a tiered service network. Small autonomous medic pods patrol assigned habitat/incident sectors, assess and stabilize disabled robots, isolate energy/tool hazards, recover them with compatible lift/tow/cradle equipment, and transfer custody to habitat triage or regional robot hospitals. Medic pods never enter an unapproved fire/terrain/air-quality ODD and do not compromise a person rescue priority. Each large habitat has maintenance robots that execute scheduled and condition-based inspection, cleaning, lubrication, consumable replacement, connector/charger service, calibration checks, modular line-replaceable-unit exchange, firmware/configuration attestation, and test cycles. Robot hospitals provide quarantine, decontamination, battery isolation, diagnosis, repair, calibration, burn-in, recertification, remanufacture, salvage, and controlled retirement. Heat-exposed, swollen, leaking, electrically unsafe, contaminated, structurally unstable, or unknown-state units use fire-separated quarantine transport and storage. Every action preserves asset/battery/part identity, custody, fault evidence, configuration, technician/robot identity, parts, test result, disposition, and return-to-service approval.

## Consequences

### Positive
- Increases readiness, recovers expensive assets, contains damaged-energy hazards, and creates field-failure learning data.
### Negative
- Recovery tooling, quarantine capacity, spare modules, decontamination and hospital throughput add fleet and logistics demand.
### Neutral
- “Medic” and “hospital” are operational names; human life safety always has priority.

## Links
- [ADR-047](ADR-047-vehicle-firmware-ota-and-fleet-compatibility.md)
- [ADR-060](ADR-060-wildfire-supply-chain-and-resource-digital-thread.md)
- [ADR-064](ADR-064-million-battery-energy-and-charging-control.md)
