# ADR-070: Aircraft-independent payload and certified release envelope

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: aircraft, payload, airdrop

## Context

The concept names CC-177/C-17-class aircraft for full scale and LM-100J/C-130J-30-class aircraft for prototypes. Published maximum payload figures do not establish usable airdrop payload, centre of gravity, floor/ramp loads, extraction compatibility, opening loads, release altitude/speed, emergency procedures, or aircraft availability/authorization.

## Decision

Define an aircraft-independent `AerialPayloadInterface` and certify a separate immutable mission configuration for each aircraft/tail/configuration. The interface covers geometric envelope, packed mass/volume, mass properties/centre-of-gravity, floor/ramp/roller loads, restraint/extraction interfaces, electrical/data/environment limits, parachute/drogue/reefing systems, hazardous materials, cargo-bay crew separation, jettison/retention behavior, telemetry, and recovery. A release envelope binds aircraft configuration, payload revision, loading plan, route/drop corridor, altitude/airspeed/attitude, wind/turbulence/smoke/icing, extraction sequence, abort/divert/jettison zones, crew roles, inspection, and evidence. Maximum brochure payload is never treated as allowable airdrop mass. CC-177 and C-130J/LM-100J remain candidates until approved engineering, loadmaster, ground, extraction and flight-test evidence exists. Aircraft systems and crew commands always override blanket automation.

## Consequences

### Positive
- Prevents aircraft assumptions from contaminating the domain and enables staged carrier substitution.
### Negative
- Every aircraft/payload revision requires configuration control and expensive integration evidence.
### Neutral
- Aircraft selection is an adapter/configuration decision, not blanket-domain ownership.

## Links
- [ADR-014](ADR-014-open-standards-and-dependency-evaluation.md)
- [ADR-047](ADR-047-vehicle-firmware-ota-and-fleet-compatibility.md)
