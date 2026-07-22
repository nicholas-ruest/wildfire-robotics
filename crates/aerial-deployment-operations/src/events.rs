#![allow(clippy::wildcard_imports)]
use crate::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventMeta {
    pub id: EventId,
    pub scope: OperationScope,
    pub aggregate_id: String,
    pub aggregate_version: u64,
    pub occurred_at: DateTime<Utc>,
    pub command_id: CommandId,
    pub evidence: Vec<EvidenceRef>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    BlanketConfigurationRegistered {
        meta: EventMeta,
        configuration: BlanketConfigurationId,
    },
    BlanketConfigurationPromoted {
        meta: EventMeta,
        configuration: BlanketConfigurationId,
        stage: QualificationStage,
    },
    BlanketConfigurationSuspended {
        meta: EventMeta,
        configuration: BlanketConfigurationId,
    },
    MembraneStateChanged {
        meta: EventMeta,
        assembly: AssemblyId,
        phase: AssemblyPhase,
    },
    PayloadLoadApproved {
        meta: EventMeta,
        manifest: PayloadManifestId,
        aircraft: AircraftConfigurationId,
    },
    PayloadAccounted {
        meta: EventMeta,
        manifest: PayloadManifestId,
    },
    AerialDropMissionAuthorized {
        meta: EventMeta,
        mission: AerialDropMissionId,
    },
    AerialDropMissionChanged {
        meta: EventMeta,
        mission: AerialDropMissionId,
        phase: MissionPhase,
    },
    ReleaseArmed {
        meta: EventMeta,
        authorization: ReleaseAuthorizationId,
    },
    PayloadReleased {
        meta: EventMeta,
        authorization: ReleaseAuthorizationId,
        manifest: PayloadManifestId,
    },
    ReleaseAborted {
        meta: EventMeta,
        authorization: ReleaseAuthorizationId,
    },
    DeploymentPhaseChanged {
        meta: EventMeta,
        deployment: AirborneDeploymentId,
        phase: DeploymentPhase,
    },
    PanelIsolated {
        meta: EventMeta,
        deployment: AirborneDeploymentId,
        panel: PanelId,
    },
    SectionJettisoned {
        meta: EventMeta,
        deployment: AirborneDeploymentId,
        panel: PanelId,
        zone: JettisonZoneId,
    },
    BlanketActivated {
        meta: EventMeta,
        installation: GroundInstallationId,
        footprint: FootprintId,
    },
    BlanketDegraded {
        meta: EventMeta,
        installation: GroundInstallationId,
    },
    BlanketRecovered {
        meta: EventMeta,
        installation: GroundInstallationId,
    },
    ComponentDispositionChanged {
        meta: EventMeta,
        component: ComponentId,
        disposition: ComponentDisposition,
    },
}
