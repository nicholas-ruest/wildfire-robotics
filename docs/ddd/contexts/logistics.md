# Logistics Context

## Purpose

Plan and execute fixed-route supply, equipment, and water-relay movements.

## Model

- **Aggregates:** LogisticsMission, Route, Delivery, ResourceItem, SupplyPlan, TransportPod, Carrier, MobilizationWave, WaterSource, RelayCycle.
- **Core invariant:** Routes stay within approved ODD; water-source status and capacity are fresh; failure cannot strand a vehicle in an unsafe state.
- **Primary workflow:** Validate demand/source -> plan route -> reserve assets -> dispatch -> confirm custody/delivery -> reconcile.

## Tactical model

| Aggregate | Lifecycle | Commands | Events |
|---|---|---|---|
| LogisticsMission | draft → validated → authorized → dispatched → delivered/aborted/failed | PlanDelivery, ValidateDelivery, DispatchDelivery, AbortDelivery, ConfirmDelivery | DeliveryStateChanged |
| Route | proposed → validated → active → blocked/expired | ProposeRoute, ValidateRoute, ActivateRoute, BlockRoute | RouteValidated, RouteBlocked |
| Delivery | prepared → in-custody → in-transit → handed-over/rejected/lost | PrepareLoad, TransferCustody, RecordCheckpoint, ConfirmHandover, RejectHandover | CustodyTransferred, DeliveryCompleted |
| WaterSource | candidate → verified → available → restricted/depleted/contaminated | RegisterSource, VerifySource, ReserveQuantity, RecordQuality, RestrictSource | WaterSourceChanged |
| RelayCycle | planned → reserved → active → complete/interrupted | PlanRelay, ReserveCycle, AdvanceRelay, InterruptRelay | RelayCycleChanged |
| ResourceItem | planned → ordered → received → stocked → reserved → issued → consumed/returned/disposed | RegisterItem, ReceiveItem, ReserveItem, TransferItem, ConsumeItem, ReturnItem, DisposeItem | ResourceStateChanged |
| SupplyPlan | draft → optimized → approved → executing → replanning → closed | ForecastDemand, OptimizeSupply, ApproveSupplyPlan, ReplanSupply, CloseSupplyPlan | SupplyPlanApproved, SupplyShortagePredicted |
| TransportPod | available → loading → sealed → in-transit → staged → unloading → returned/servicing | ReservePod, LoadPod, VerifyLoad, SealPod, TransferPod, UnloadPod | PodLoaded, PodCustodyChanged |
| Carrier | available → reserved → loading → ready → in-transit → recovering/servicing | ReserveCarrier, VerifyCarrierLoad, DispatchCarrier, RecordCheckpoint, RecoverCarrier | CarrierStateChanged |
| MobilizationWave | draft → capacity-checked → authorized → releasing → in-transit → arriving → complete/aborted | PlanMobilization, CheckCapacity, AuthorizeWave, ReleaseWave, ReplanWave, CompleteWave | MobilizationWaveChanged |

Owned values include demand/load and unit, item/batch/serial, pod/rack identity and interfaces, robot manifest, mass/volume/centre-of-gravity/securement, carrier/axle/load limits, compatibility/substitution, hazardous-material and energy-isolation class, custody, condition/expiry, maintenance/calibration, source/supplier, lead-time distribution, stock/service level, source quality/capacity/freshness, route/bridge/ferry/rail/barge geometry/capacity/ODD/risk, vehicle/resource reservations, charging/refueling, loading/unloading/staging slots, mobilization arrival window, checkpoints, handoff evidence, delivered/consumed variance, reverse flow and exception reason.

## Invariants

- `LO-INV-001`: Load, vehicle, route, source, operator, weather, and communications remain inside their approved ODD and restrictions.
- `LO-INV-002`: Source reservation cannot exceed verified usable quantity and expires to prevent phantom capacity.
- `LO-INV-003`: Every custody transition identifies giver, receiver, quantity/condition, place/time, evidence, and discrepancies.
- `LO-INV-004`: Contaminated, restricted, stale, or depleted sources cannot supply delivery or suppression.
- `LO-INV-005`: Route blockage/localization loss cannot strand a vehicle; only pre-authorized safe stop/reroute is autonomous.
- `LO-INV-006`: Available-to-promise inventory excludes expired, incompatible, quarantined, unserviceable, already reserved, or location-infeasible resources.
- `LO-INV-007`: Supply optimization reconciles demand, stock, lead-time uncertainty, energy, maintenance, transport, substitution, and incident priority; it cannot allocate the same serial/batch quantity twice.
- `LO-INV-008`: Every consumable, battery/fuel/agent, critical part, and maintained asset preserves source-to-use custody and actual consumption/return/disposition.
- `LO-INV-009`: A sealed pod manifest reconciles every robot/battery/tool, measured gross/axle/centre-of-gravity and securement, energy isolation, compatibility and custody before movement.
- `LO-INV-010`: Carrier dispatch requires legal/route/load, traction/reserve energy, braking/steering/tires, automated-driving ODD, communications fallback, recovery, and destination admission capacity.
- `LO-INV-011`: A mobilization wave cannot reserve the same robot/pod/carrier/corridor/charger/staging slot twice or release faster than any downstream load, route, energy, unload, inspection or assignment bottleneck.
- `LO-INV-012`: Arrival counts as useful capacity only after pod custody, unloading, robot inspection, energy, connectivity, eligibility and destination mission admission succeed.

## Ports and read models

Ports cover demand forecasting, time-expanded flow optimization, supplier/procurement feeds, pod/load sensing, carrier/convoy automation, road/bridge/ferry/rail/barge capacity, routing/maps, traffic/closure/weather, source testing, inventory, maintenance, charging/fueling, fleet reservation, custody scan, mission authority, and origin/destination station admission. Read models expose multi-echelon stock, habitat/pod/carrier readiness, loading/unloading queues, wave/corridor ETA, useful-arrival rate, demand/capacity, shortage/bottleneck risk, lead-time uncertainty, route risk, source freshness, custody chain, relay status, utilization and delivery exceptions.

## Boundary and failure policy

Owns the delivery process manager described in [process managers](../process-managers.md). Blocked routes, localization degradation, source contamination/depletion, payload anomaly, or missed handoff cause safe stop/reroute only within authority; otherwise operations halt and escalate with known custody and vehicle state.

## Implementation acceptance

Domain invariants must be executable and property-tested; API/event contracts require compatibility tests; persistence requires migration/rollback and concurrency tests; adapters require fault-injection and replay tests; operational promotion requires the applicable evidence in the [production readiness standard](../../operations/production-readiness.md).
