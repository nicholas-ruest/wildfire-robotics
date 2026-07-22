# Prompt 29 deterministic digital-twin qualification

This simulation is an evidence-producing validation adapter. It does not own an
operational aggregate, issue physical commands, approve an aircraft, authorize a
field activity, or grant operational authority. A complete answer means only that
the declared simulation evidence gate is complete.

## Model composition and validity

`digital-twin/29.1` integrates version-bound models for fire/weather/terrain,
communications, drones, ground robots/tools, habitats/microgrids/chargers,
batteries, pods/carriers/platoons, logistics, medic pods, hospitals, and the
experimental aerial fire blanket. Runs use stable IDs, explicit 64-bit seeds,
integer time, deterministic ordering, and content digests. The blanket model
covers retained, extracted, released, formation-acquired, reefed expansion,
expanded, terrain-aligned, anchored, panel-isolated, tether-breakaway, and
minimum-risk states. Cohorts are bounded to 32 and every result accounts for
recovered, isolated, and unrecoverable components.

Validity is explicit per signed bundle: model version, calibrated temperature
interval, scalar calibration score, and known gaps. The reference tests declare
that flight dynamics and fire behavior still require calibration against
representative hardware and controlled field observations. Simulation and
SIL/HITL data therefore cannot substitute for aircraft, field, regulatory, or
operational evidence.

## Requirement/hazard/invariant scenario matrix

All rows are exercised by the deterministic `Fault::ALL` campaign. Each scenario
must carry non-empty requirement, hazard, and invariant identifiers; missing
links fail closed.

| Scenario | Injected fault | Required minimum-risk / compensation |
|---|---|---|
| SCN-NETWORK-LOSS | network loss | inhibit tools, land/hold, notify human |
| SCN-GNSS-LOSS | GNSS loss | inhibit tools, land/hold, notify human |
| SCN-CLOCK-LOSS | clock loss | inhibit tools, land/hold, notify human |
| SCN-THERMAL | thermal event | inhibit tools, land/hold, notify human |
| SCN-AUTHORITY | authority expiry | retain load, revoke intent |
| SCN-COLLISION | collision | inhibit tools, land/hold, notify human |
| SCN-INTRUSION | intrusion | inhibit tools, land/hold, notify human |
| SCN-TOOL | tool fault | inhibit tools, land/hold, notify human |
| SCN-CARRIER | carrier failure | retain load, revoke intent |
| SCN-LOAD | load failure | retain load, revoke intent |
| SCN-RELEASE | release failure | retain load, revoke intent |
| SCN-FIRE | fire excursion | inhibit tools, land/hold, notify human |
| SCN-SMOKE | smoke obscuration | inhibit tools, land/hold, notify human |
| SCN-COLD | cold excursion | inhibit tools, land/hold, notify human |
| SCN-AERO | coupled aerodynamic instability | reef, pause, separate cohorts |
| SCN-ENTANGLEMENT | entanglement | isolate/vent panel, account components |
| SCN-CORRELATED | correlated damage | isolate/vent panel, account components |
| SCN-TETHER | tether failure | break away into declared safe-release sector |
| SCN-RECOVERY | recovery failure | freeze workflow, account loss, escalate |

## SIL/HITL evidence contract

Both ports use `wildfire.hitl.frame.v1`: sequence, monotonic nanoseconds,
fidelity, and payload digest. The adapter rejects reordered or wrong-schema
responses. Evidence capture receives the canonical serialized response at every
tick. Bundles cover simulator/environment versions, every domain and result,
traceability, validity/gaps, and an HMAC-SHA-256 signature. Production key
custody belongs in an external signer; test keys are deliberately non-production.
