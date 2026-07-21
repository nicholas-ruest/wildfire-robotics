# Bounded Context Catalog

## Aggregate and contract baseline

| Context | Primary aggregates | Commands | Published events |
|---|---|---|---|
| Hazard Intelligence | Source, IngestionRun, ObservationSet, HazardPicture | RegisterSource, IngestBatch, QuarantineBatch | ObservationAccepted, HazardPictureUpdated, DataQualityDegraded |
| Predictive Planning | ModelRelease, ForecastRun, SpreadScenario, Recommendation | ApproveModel, RunForecast, CompareScenario | ForecastPublished, ModelDriftDetected, RecommendationPublished |
| Incident Command | Incident, OperationalPeriod, Objective, Assignment, Restriction | OpenIncident, ApproveObjectives, IssueAssignment | IncidentOpened, AssignmentIssued, RestrictionChanged |
| Mission Control | Mission, Allocation, MissionLease, ConflictSet | PlanMission, AuthorizeMission, AbortMission | MissionAuthorized, MissionStateChanged, ConflictDetected |
| Fleet Operations | Vehicle, BatteryAsset, CapabilityRecord, HealthAssessment, Configuration, FleetCell, CollaborationProfile | RegisterVehicle, RegisterBattery, AttestCapability, GroundVehicle, FormCell | VehicleHealthChanged, BatteryEligibilityChanged, CapabilityAttested, VehicleGrounded, FleetCellChanged |
| Vehicle Integration | GatewaySession, CommandDelivery, TelemetryStream | DeliverIntent, RevokeIntent, RotateDeviceTrust | IntentAcknowledged, TelemetryNormalized, LinkDegraded |
| Station Operations | Station, RobotHabitat, Microgrid, EnergyStore, ChargeSession, EdgeDeployment, MaintenanceBay | ActivateStation, CommissionHabitat, OptimizeEnergy, ScheduleCharge, ReserveEnergy | StationAvailabilityChanged, HabitatReadinessChanged, ChargeSessionChanged, EnergyCritical |
| Logistics | LogisticsMission, Route, Delivery, ResourceItem, SupplyPlan, TransportPod, Carrier, MobilizationWave, WaterSource, RelayCycle | PlanDelivery, OptimizeSupply, LoadPod, PlanMobilization, DispatchWave | DeliveryCompleted, SupplyShortagePredicted, PodLoaded, MobilizationWaveChanged, RouteBlocked |
| Suppression Operations | SuppressionPlan, ActuationEnvelope, Target, Operation | ApproveEnvelope, StartTeleoperation, ApplyAgent | SuppressionStarted, EnvelopeViolated, SuppressionStopped |
| Safety Assurance | Hazard, SafetyConstraint, ODD, EvidenceCase, SafetyOccurrence | RegisterHazard, ApproveConstraint, PromoteRelease | ConstraintPublished, PromotionApproved, NearMissReported |
| Identity and Access | Principal, DeviceIdentity, RoleGrant, Approval | EnrollDevice, GrantRole, ApproveAction, RevokeTrust | DeviceEnrolled, RoleGranted, TrustRevoked |
| Commercial Operations | Tenant, Contract, Entitlement, Meter, SupportCase | OnboardTenant, ChangeEntitlement, RecordUsage | TenantActivated, UsageRated, SupportEscalated |
| Vegetation Management | TreatmentProgram, Prescription, TreatmentUnit, WorkPackage, EffectivenessAssessment | CreatePrescription, AuthorizeWork, DispatchTreatment, RecordTreatment, AssessEffectiveness | PrescriptionApproved, TreatmentCompleted, EffectivenessAssessed |
| Robot Care and Recovery | ServicePolicy, MaintenancePlan, WorkOrder, RecoveryMission, DamageAssessment, QuarantineCase, RepairCase, RetirementCase | ScheduleMaintenance, RequestRecovery, StabilizeRobot, RepairRobot, RecertifyRobot, RetireRobot | MaintenanceCompleted, RobotRecovered, RobotQuarantined, RobotRecertified, RobotRetired |
| Aerial Deployment Operations | BlanketConfiguration, MembraneAssembly, PayloadManifest, AerialDropMission, ReleaseAuthorization, AirborneDeployment, GroundInstallation | PromoteConfiguration, PlanDropMission, ApproveLoad, ArmRelease, CommitRelease, IsolatePanel, ActivateBlanket, RecoverPanel | BlanketConfigurationPromoted, PayloadLoadApproved, PayloadReleased, DeploymentPhaseChanged, BlanketActivated, BlanketRecovered |

Detailed aggregates must enforce invariants in code and tests. This catalog is a planning contract, not a substitute for implementation.

## Normative companion specifications

- [Tactical model standard](tactical-model-standard.md)
- [Integration contracts](integration-contracts.md)
- [Cross-context process managers](process-managers.md)
- [Architecture and assurance traceability](traceability-model.md)
- [Context map](context-map.md)

The individual [context specifications](contexts/) define aggregate lifecycles and numbered invariants. Together these documents are the domain contract; catalog rows alone are not sufficient.
