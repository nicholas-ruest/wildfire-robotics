use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregateKind {
    BlanketConfiguration,
    MembraneAssembly,
    PayloadManifest,
    AerialDropMission,
    ReleaseAuthorization,
    AirborneDeployment,
    GroundInstallation,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalAuthority {
    Aircraft,
    IncidentCommand,
    SafetyAssurance,
    MissionControl,
    FleetOperations,
    SuppressionOperations,
    VegetationManagement,
    Logistics,
    RobotCare,
}
