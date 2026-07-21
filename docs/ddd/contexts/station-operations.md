# Station Operations Context

## Purpose

Operate incident-edge compute, connectivity, energy, inventory, maintenance, and synchronization.

## Model

- **Aggregates:** Station, RobotHabitat, Microgrid, EnergyStore, ChargeSession, EdgeDeployment, MaintenanceBay.
- **Core invariant:** Only signed deployments run; reserved emergency energy cannot be consumed by routine work; sync never expands expired authority.
- **Primary workflow:** Provision -> attest -> deploy -> operate offline -> reconcile -> service assets.

## Tactical model

| Aggregate | Lifecycle | Commands | Events |
|---|---|---|---|
| Station | planned → commissioned → available → degraded/offline → decommissioned | CommissionStation, AttestStation, ChangeAvailability, DecommissionStation | StationAvailabilityChanged |
| RobotHabitat | planned → commissioned → ready → loading/deploying → degraded/isolated → decommissioned | CommissionHabitat, AdmitRobot, StagePod, EvacuateHabitat, IsolateZone | HabitatReadinessChanged |
| Microgrid | black → starting → islanded/grid-connected → constrained/emergency → shutdown | StartMicrogrid, ConnectGrid, IslandGrid, OptimizeEnergy, ShedLoad, StartGenerator, ShutdownMicrogrid | MicrogridModeChanged, EnergyShortfallPredicted |
| EdgeDeployment | staged → verified → active → degraded/rolling-back/superseded | StageDeployment, VerifyDeployment, ActivateDeployment, RollbackDeployment | EdgeDeploymentActivated, EdgeDeploymentRolledBack |
| EnergyStore | unknown → available → reserved → critical/unavailable | RecordEnergy, ReserveEnergy, ReleaseEnergy, DeclareCritical | EnergyCritical, EnergyReservationChanged |
| ChargeSession | requested → admitted → prechecking → charging → complete/aborted/quarantined | RequestCharge, AdmitCharge, StartCharge, ChangeChargeRate, CompleteCharge, AbortCharge | ChargeSessionChanged, BatteryAnomalyDetected |
| MaintenanceBay | available → reserved → servicing → blocked | ReserveBay, StartService, CompleteService, BlockBay | MaintenanceBayStateChanged |

Owned values include station/site identity, habitat/dock/pod/zone, services and capacity, deployment/configuration digests, attestation, connectivity, storage, clock/GNSS quality, PV/weather forecast, grid/generator/fuel state, bus/feeder/charger limits, stationary/robot/carrier energy state and reserves, charge deadline/rate/curve, thermal/fire separation, inventory, maintenance resources, sync cursor/conflicts, and environmental envelope.

## Invariants

- `SO-INV-001`: Only promoted, signed, compatible artifacts and policy/data bundles activate after local verification.
- `SO-INV-002`: Emergency reserve energy and command/audit capacity cannot be consumed by lower-criticality workload.
- `SO-INV-003`: Reconciliation preserves authority precedence, provenance, tombstones, causal history, and stricter constraints.
- `SO-INV-004`: Expired authority, revoked trust, or superseded safety policy cannot be resurrected by offline state.
- `SO-INV-005`: Disk, power, thermal, clock, or connectivity thresholds deterministically shed work by criticality and declare degradation.
- `SO-INV-006`: RVM collaboration partitions and RuPixel indexes are signed, resource-bounded, rebuildable/non-authoritative workloads and are shed before command, safety, identity, and audit services.
- `SO-INV-007`: Habitat readiness requires safe structure/environment, communications, energy reserve, functional isolation/fire controls, compatible docks/chargers, maintenance capacity and verified evacuation/deployment paths.
- `SO-INV-008`: Local energy dispatch protects emergency departure/return, safety, heating/thermal conditioning, identity, command, audit and communications before optional charging/compute loads.
- `SO-INV-009`: A charge begins only after battery/charger/vehicle identity, compatibility, isolation, temperature, BMS authority, zone capacity and fenced schedule pass; BMS or protection trip always overrides optimization.
- `SO-INV-010`: State-of-charge/health/power values carry source/time/method/uncertainty; stale or conflicting estimates reduce eligibility and cannot be averaged into false certainty.

## Ports and read models

Ports cover power/BMS, networking, GNSS/time, storage, deployment runtime, inventory/CMMS, synchronization, physical access, and environment sensors. Read models expose readiness, energy forecast, deployment health, sync lag/conflicts, storage pressure, inventory, and maintenance schedule.

## Boundary and failure policy

Owns edge deployment synchronization under the [process-manager rules](../process-managers.md). Backhaul/power loss, partial sync, disk pressure, thermal excursion, clock uncertainty, or failed upgrade sheds noncritical load, retains the command/audit/safety path, and rolls back or isolates only when compatibility permits.

## Implementation acceptance

Domain invariants must be executable and property-tested; API/event contracts require compatibility tests; persistence requires migration/rollback and concurrency tests; adapters require fault-injection and replay tests; operational promotion requires the applicable evidence in the [production readiness standard](../../operations/production-readiness.md).
