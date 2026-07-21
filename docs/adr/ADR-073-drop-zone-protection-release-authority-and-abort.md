# ADR-073: Drop-zone protection, release authority, and abort

- **Status**: proposed
- **Date**: 2026-07-21
- **Deciders**:
- **Tags**: safety, airspace, release

## Context

A heavy multi-object airdrop has a large evolving hazard footprint. Stale fire, wind, airspace, terrain, people, vehicle, infrastructure, and communications information can make an otherwise planned release unsafe.

## Decision

Model a release corridor, predicted dispersion/failed-component footprint, exclusion volume, jettison sectors, robot emergency landing zones, ground evacuation boundary, temporal validity and surveillance confidence as signed versioned constraints. Payload release requires two-key authorization: aircraft-authority release plus current incident/safety authorization; either can veto, and no automation can compel release. Perform continuous pre-release checks for aircraft/payload configuration, robot/panel/parafoil/tether health, mass/loading, weather/wind/turbulence/smoke, airspace, terrain, fire position, people/vehicles/aircraft, navigation/time quality, communications and ground readiness. Any stale/indeterminate/violated mandatory condition aborts or holds. After point-of-no-return, local controllers execute the pre-authorized least-harm contingency without expanding target area. The system broadcasts and records exclusion/abort state through independent paths and supports post-drop unexploded/energized-equipment-style accounting for every robot, parafoil, tether, panel, anchor and chemical payload.

## Consequences

### Positive
- Makes release a separately governed irreversible action with continuously validated public/crew safety.
### Negative
- Surveillance confidence, dispersion prediction and rapid-changing fire/airspace conditions may cause frequent aborts.
### Neutral
- Mission approval does not equal payload-release authorization.

## Links
- [ADR-001](ADR-001-safety-led-human-command-authority.md)
- [ADR-023](ADR-023-canonical-authenticated-command-envelope.md)
