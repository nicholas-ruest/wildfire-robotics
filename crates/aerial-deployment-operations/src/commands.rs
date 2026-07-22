#![allow(clippy::wildcard_imports)]
use crate::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandMeta {
    pub id: CommandId,
    pub scope: OperationScope,
    pub expected_version: u64,
    pub issued_at: DateTime<Utc>,
    pub evidence: Vec<EvidenceRef>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    RegisterConfiguration {
        meta: CommandMeta,
        configuration: BlanketConfigurationId,
    },
    AttachQualification {
        meta: CommandMeta,
        configuration: BlanketConfigurationId,
        stage: QualificationStage,
    },
    PromoteStage {
        meta: CommandMeta,
        configuration: BlanketConfigurationId,
        stage: QualificationStage,
    },
    SuspendConfiguration {
        meta: CommandMeta,
        configuration: BlanketConfigurationId,
    },
    AssemblePanels {
        meta: CommandMeta,
        assembly: AssemblyId,
        panels: Vec<PanelId>,
    },
    InspectAssembly {
        meta: CommandMeta,
        assembly: AssemblyId,
    },
    PackAssembly {
        meta: CommandMeta,
        assembly: AssemblyId,
    },
    RecoverAssembly {
        meta: CommandMeta,
        assembly: AssemblyId,
    },
    AddPayloadItem {
        meta: CommandMeta,
        manifest: PayloadManifestId,
        component: ComponentId,
    },
    ReconcileManifest {
        meta: CommandMeta,
        manifest: PayloadManifestId,
    },
    ApproveLoad {
        meta: CommandMeta,
        manifest: PayloadManifestId,
        aircraft: AircraftConfigurationId,
    },
    AccountItem {
        meta: CommandMeta,
        manifest: PayloadManifestId,
        component: ComponentId,
        disposition: ComponentDisposition,
    },
    PlanDropMission {
        meta: CommandMeta,
        mission: AerialDropMissionId,
        corridor: ReleaseCorridorId,
    },
    ModelDispersion {
        meta: CommandMeta,
        mission: AerialDropMissionId,
        nominal: FootprintId,
        failure: Vec<FootprintId>,
    },
    AuthorizeMission {
        meta: CommandMeta,
        mission: AerialDropMissionId,
    },
    AbortMission {
        meta: CommandMeta,
        mission: AerialDropMissionId,
    },
    RequestRelease {
        meta: CommandMeta,
        authorization: ReleaseAuthorizationId,
    },
    RecordAircraftDecision {
        meta: CommandMeta,
        authorization: ReleaseAuthorizationId,
        evidence: EvidenceRef,
    },
    RecordGroundDecision {
        meta: CommandMeta,
        authorization: ReleaseAuthorizationId,
        evidence: EvidenceRef,
    },
    CommitRelease {
        meta: CommandMeta,
        authorization: ReleaseAuthorizationId,
    },
    AbortRelease {
        meta: CommandMeta,
        authorization: ReleaseAuthorizationId,
    },
    RecordExtraction {
        meta: CommandMeta,
        deployment: AirborneDeploymentId,
    },
    ReleaseSection {
        meta: CommandMeta,
        deployment: AirborneDeploymentId,
        panel: PanelId,
    },
    ChangeVent {
        meta: CommandMeta,
        deployment: AirborneDeploymentId,
        vent: VentId,
    },
    IsolatePanel {
        meta: CommandMeta,
        deployment: AirborneDeploymentId,
        panel: PanelId,
    },
    JettisonSection {
        meta: CommandMeta,
        deployment: AirborneDeploymentId,
        panel: PanelId,
        zone: JettisonZoneId,
    },
    RecordLanding {
        meta: CommandMeta,
        deployment: AirborneDeploymentId,
    },
    TransitionGroundMode {
        meta: CommandMeta,
        installation: GroundInstallationId,
    },
    InstallAnchor {
        meta: CommandMeta,
        installation: GroundInstallationId,
        anchor: AnchorId,
    },
    SealJoint {
        meta: CommandMeta,
        installation: GroundInstallationId,
        joint: JointId,
    },
    ActivateBlanket {
        meta: CommandMeta,
        installation: GroundInstallationId,
    },
    DeactivateBlanket {
        meta: CommandMeta,
        installation: GroundInstallationId,
    },
    RecoverPanel {
        meta: CommandMeta,
        installation: GroundInstallationId,
        panel: PanelId,
    },
}
