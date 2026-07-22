#![allow(missing_docs, clippy::panic, clippy::unwrap_used)]
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

const CONTEXT_CRATES: &[&str] = &[
    "hazard-intelligence",
    "predictive-planning",
    "incident-command",
    "mission-control",
    "fleet-operations",
    "vehicle-integration",
    "station-operations",
    "logistics",
    "suppression-operations",
    "safety-assurance",
    "identity-access",
    "commercial-operations",
    "vegetation-management",
    "robot-care-recovery",
];
const FORBIDDEN: &[&str] = &[
    "ruv-drone",
    "ruv_drone",
    "sqlx",
    "diesel",
    "postgres",
    "nats",
    "rdkafka",
    "axum",
    "actix",
    "rocket",
    "warp",
    "gazebo",
    "carla",
    "airsim",
    "mavsdk",
    "mavlink",
    "px4",
    "ardupilot",
    "boeing",
    "lockheed",
];
const AGGREGATES: &[&str] = &[
    "BlanketConfiguration",
    "MembraneAssembly",
    "PayloadManifest",
    "AerialDropMission",
    "ReleaseAuthorization",
    "AirborneDeployment",
    "GroundInstallation",
];
const VALUES: &[&str] = &[
    "BlanketConfigurationId",
    "MaterialRevisionId",
    "PanelId",
    "JointId",
    "VentId",
    "AnchorId",
    "TetherId",
    "ReelId",
    "ParafoilId",
    "CradleId",
    "RobotId",
    "AircraftConfigurationId",
    "PayloadManifestId",
    "ReleaseCorridorId",
    "FootprintId",
    "ExclusionZoneId",
    "JettisonZoneId",
    "EmergencyLandingZoneId",
    "AssemblyId",
    "AerialDropMissionId",
    "ReleaseAuthorizationId",
    "AirborneDeploymentId",
    "GroundInstallationId",
    "EvidenceId",
    "OddId",
    "CommandId",
    "EventId",
    "ComponentId",
    "EffectivenessStudyId",
    "EvidenceRef",
    "OperationScope",
    "QualificationStage",
    "AssemblyPhase",
    "ManifestPhase",
    "MissionPhase",
    "ReleasePhase",
    "DeploymentPhase",
    "InstallationPhase",
    "ComponentDisposition",
];
const EVENTS: &[&str] = &[
    "BlanketConfigurationRegistered",
    "BlanketConfigurationPromoted",
    "BlanketConfigurationSuspended",
    "MembraneStateChanged",
    "PayloadLoadApproved",
    "PayloadAccounted",
    "AerialDropMissionAuthorized",
    "AerialDropMissionChanged",
    "ReleaseArmed",
    "PayloadReleased",
    "ReleaseAborted",
    "DeploymentPhaseChanged",
    "PanelIsolated",
    "SectionJettisoned",
    "BlanketActivated",
    "BlanketDegraded",
    "BlanketRecovered",
    "ComponentDispositionChanged",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn quoted_array(document: &str, key: &str) -> BTreeSet<String> {
    let marker = format!("{key} = [");
    let start = document
        .find(&marker)
        .unwrap_or_else(|| panic!("missing {key}"))
        + marker.len();
    let end = document[start..]
        .find(']')
        .unwrap_or_else(|| panic!("unterminated {key}"))
        + start;
    document[start..end]
        .split(',')
        .filter_map(|v| {
            let value = v.trim().trim_matches('"');
            (!value.is_empty()).then(|| value.to_owned())
        })
        .collect()
}

#[test]
fn cargo_and_domain_sources_reject_forbidden_runtime_dependencies() {
    let cargo = fs::read_to_string(root().join("crates/aerial-deployment-operations/Cargo.toml"))
        .unwrap()
        .to_ascii_lowercase();
    for name in FORBIDDEN.iter().chain(CONTEXT_CRATES) {
        assert!(!cargo.contains(name), "forbidden aerial dependency: {name}");
    }
    let dependency_section = cargo
        .split("[dependencies]")
        .nth(1)
        .and_then(|s| s.split("[lints]").next())
        .unwrap_or("");
    let allowed = [
        "chrono",
        "serde",
        "thiserror",
        "shared-kernel",
        "contracts-generated",
    ];
    for line in dependency_section.lines().filter(|line| line.contains('=')) {
        let dependency = line.split(['.', '=']).next().unwrap_or("").trim();
        assert!(
            allowed.contains(&dependency),
            "unapproved domain dependency: {dependency}"
        );
    }
    let source = fs::read_to_string(root().join("crates/aerial-deployment-operations/src/lib.rs"))
        .unwrap()
        .to_ascii_lowercase();
    for line in source.lines().filter(|line| {
        line.trim_start().starts_with("use ") || line.trim_start().starts_with("extern crate")
    }) {
        for name in FORBIDDEN.iter().chain(CONTEXT_CRATES) {
            assert!(
                !line.contains(&name.replace('-', "_")),
                "forbidden domain import: {name}"
            );
        }
    }
}

#[test]
fn every_aggregate_value_and_event_has_exact_context_ownership() {
    let manifest =
        fs::read_to_string(root().join("docs/architecture/aerial-deployment-ownership.toml"))
            .unwrap();
    assert!(manifest.contains("context = \"Aerial Deployment Operations\""));
    assert_eq!(
        quoted_array(&manifest, "aggregates"),
        AGGREGATES.iter().map(|v| (*v).to_owned()).collect()
    );
    assert_eq!(
        quoted_array(&manifest, "values"),
        VALUES.iter().map(|v| (*v).to_owned()).collect()
    );
    assert_eq!(
        quoted_array(&manifest, "events"),
        EVENTS.iter().map(|v| (*v).to_owned()).collect()
    );
}

#[test]
fn repository_registry_and_schema_document_use_owned_namespaces() {
    let registry =
        fs::read_to_string(root().join("docs/architecture/context-ownership.toml")).unwrap();
    let Some(section) = registry
        .split("[[contexts]]")
        .find(|s| s.contains("crate = \"aerial-deployment-operations\""))
    else {
        panic!("aerial context registry entry is required");
    };
    for required in [
        "schemas = [\"aerial_deployment_operations\"]",
        "event_namespace = \"wildfire.aerial_deployment_operations.v1\"",
        "protobuf_namespace = \"wildfire.aerial_deployment.v1\"",
        "invariant_prefixes = [\"AD-INV-\"]",
    ] {
        assert!(
            section.contains(required),
            "missing registry ownership: {required}"
        );
    }
    let namespaces =
        fs::read_to_string(root().join("docs/contracts/aerial-deployment-schema-namespaces.md"))
            .unwrap();
    for required in [
        "aerial_deployment_operations",
        "wildfire.aerial_deployment_operations.v1",
        "wildfire.aerial_deployment.v1",
    ] {
        assert!(
            namespaces.contains(required),
            "missing namespace documentation: {required}"
        );
    }
}
