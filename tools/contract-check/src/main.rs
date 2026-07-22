#![forbid(unsafe_code)]
//! Validates descriptor policy, registry coverage, and reviewed conformance fixtures.

use prost::Message;
use prost_types::{DescriptorProto, FileDescriptorSet};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};
use wildfire_contracts_generated::{FILE_DESCRIPTOR_SET, wildfire::v1::EventEnvelope};

const EXPECTED_CONTRACTS: usize = 49;

#[derive(Deserialize)]
struct Registry {
    schema_version: u32,
    contracts: Vec<Contract>,
}

#[derive(Deserialize)]
struct Contract {
    name: String,
    kind: String,
    owner: String,
    version: String,
    example: PathBuf,
    fixture: PathBuf,
    authorized_consumers: Vec<String>,
    classification: String,
    replay_policy: String,
    failure_policy: String,
}

#[derive(Deserialize)]
struct ReleaseRegistry {
    schema_version: u32,
    process_managers: Vec<ProcessManager>,
}

#[derive(Deserialize)]
struct ProcessManager {
    name: String,
    owner: String,
    timeout_seconds: u64,
    compensation: String,
    escalation: String,
    observable_containment: String,
}

fn message_names(prefix: &str, messages: &[DescriptorProto], output: &mut BTreeSet<String>) {
    for message in messages {
        if let Some(name) = &message.name {
            let qualified = format!("{prefix}.{name}");
            output.insert(qualified.clone());
            message_names(&qualified, &message.nested_type, output);
        }
    }
}

fn validates_reserved_governance_range(message: &DescriptorProto) -> bool {
    message
        .reserved_range
        .iter()
        .any(|range| range.start == Some(100) && range.end == Some(200))
}

fn fixture_envelope(contract: &Contract) -> EventEnvelope {
    let payload = Vec::new();
    EventEnvelope {
        message_id: format!("fixture-{}", contract.name),
        event_type: contract.name.clone(),
        schema_version: contract.version.clone(),
        producer: contract.owner.clone(),
        producer_version: "0.1.0-fixture".into(),
        aggregate_type: contract
            .name
            .trim_end_matches("Changed")
            .trim_end_matches("Published")
            .into(),
        aggregate_id: "00000000-0000-4000-8000-000000000001".into(),
        aggregate_version: 1,
        correlation_id: "00000000-0000-4000-8000-000000000002".into(),
        classification: classification_number(&contract.classification),
        subject: format!(
            "wr.fixture.ca-central-1.fixture.{}.event.{}.v1",
            contract.owner,
            contract.name.to_ascii_lowercase()
        ),
        content_type: "application/protobuf".into(),
        payload_sha256: format!("{:x}", Sha256::digest(&payload)),
        payload: Some(prost_types::Any {
            type_url: format!("type.googleapis.com/wildfire.v1.{}", contract.name),
            value: payload,
        }),
        ..Default::default()
    }
}

fn classification_number(value: &str) -> i32 {
    match value {
        "PUBLIC" => 1,
        "INTERNAL" => 2,
        "CONFIDENTIAL" => 3,
        "RESTRICTED" => 4,
        _ => 0,
    }
}

fn write_fixtures(root: &Path, registry: &Registry) -> Result<(), Box<dyn std::error::Error>> {
    for contract in &registry.contracts {
        let envelope = fixture_envelope(contract);
        let fixture_path = root.join(&contract.fixture);
        let example_path = root.join(&contract.example);
        if let Some(parent) = fixture_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = example_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&fixture_path, envelope.encode_to_vec())?;
        let example = json!({
            "messageId": envelope.message_id,
            "eventType": envelope.event_type,
            "schemaVersion": envelope.schema_version,
            "producer": envelope.producer,
            "aggregateVersion": envelope.aggregate_version,
            "classification": contract.classification,
            "subject": envelope.subject,
            "contentType": envelope.content_type,
            "payloadSha256": envelope.payload_sha256,
            "payloadTypeUrl": envelope.payload.as_ref().map(|value| &value.type_url),
        });
        let mut bytes = serde_json::to_vec_pretty(&example)?;
        bytes.push(b'\n');
        fs::write(example_path, bytes)?;
    }
    Ok(())
}

fn validate(
    root: &Path,
    registry: &Registry,
    descriptors: &FileDescriptorSet,
) -> Result<(), Box<dyn std::error::Error>> {
    if registry.schema_version != 1 || registry.contracts.len() != EXPECTED_CONTRACTS {
        return Err(format!(
            "registry must contain exactly {EXPECTED_CONTRACTS} version-1 contracts"
        )
        .into());
    }
    let mut descriptor_messages = BTreeSet::new();
    let mut governed_messages = BTreeSet::new();
    for file in &descriptors.file {
        if file.package.as_deref() != Some("wildfire.v1") {
            continue;
        }
        message_names("wildfire.v1", &file.message_type, &mut descriptor_messages);
        for message in &file.message_type {
            if validates_reserved_governance_range(message)
                && let Some(name) = &message.name
            {
                governed_messages.insert(name.clone());
            }
        }
    }
    let mut names = BTreeSet::new();
    for contract in &registry.contracts {
        if !names.insert(&contract.name) {
            return Err(format!("duplicate registry contract {}", contract.name).into());
        }
        if contract.kind != "event"
            || contract.owner.is_empty()
            || contract.version != "1.0.0"
            || contract.authorized_consumers.is_empty()
            || contract.replay_policy.is_empty()
            || contract.failure_policy.is_empty()
        {
            return Err(format!("incomplete registry metadata for {}", contract.name).into());
        }
        if contract
            .authorized_consumers
            .iter()
            .any(|consumer| consumer.trim().is_empty())
            || contract
                .authorized_consumers
                .iter()
                .collect::<BTreeSet<_>>()
                .len()
                != contract.authorized_consumers.len()
        {
            return Err(format!("invalid consumer compatibility for {}", contract.name).into());
        }
        if !descriptor_messages.contains(&format!("wildfire.v1.{}", contract.name)) {
            return Err(format!("registry contract {} has no descriptor", contract.name).into());
        }
        if !governed_messages.contains(&contract.name) {
            return Err(format!(
                "{} does not reserve governance fields 100-199",
                contract.name
            )
            .into());
        }
        let fixture = fs::read(root.join(&contract.fixture))?;
        let envelope = EventEnvelope::decode(fixture.as_slice())?;
        if envelope.event_type != contract.name || envelope.schema_version != contract.version {
            return Err(format!("fixture envelope mismatch for {}", contract.name).into());
        }
        let expected_type = format!("type.googleapis.com/wildfire.v1.{}", contract.name);
        if envelope
            .payload
            .as_ref()
            .map(|value| value.type_url.as_str())
            != Some(expected_type.as_str())
        {
            return Err(format!("fixture payload type mismatch for {}", contract.name).into());
        }
        let example: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join(&contract.example))?)?;
        if example["eventType"] != contract.name {
            return Err(format!("example mismatch for {}", contract.name).into());
        }
    }
    println!(
        "validated {} registered contracts, descriptors, examples, and fixtures",
        names.len()
    );
    Ok(())
}

fn validate_release_registry(registry: &ReleaseRegistry) -> Result<(), Box<dyn std::error::Error>> {
    const EXPECTED_PROCESS_MANAGERS: usize = 18;
    if registry.schema_version != 1 || registry.process_managers.len() != EXPECTED_PROCESS_MANAGERS
    {
        return Err(format!(
            "release registry must contain exactly {EXPECTED_PROCESS_MANAGERS} version-1 process managers"
        )
        .into());
    }
    let mut names = BTreeSet::new();
    for manager in &registry.process_managers {
        if !names.insert(manager.name.as_str())
            || manager.owner.trim().is_empty()
            || manager.timeout_seconds == 0
            || manager.compensation.trim().is_empty()
            || manager.escalation.trim().is_empty()
            || manager.observable_containment.trim().is_empty()
        {
            return Err(format!("incomplete process-manager policy for {}", manager.name).into());
        }
    }
    println!(
        "validated {} process-manager timeout, compensation, escalation, and containment policies",
        names.len()
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let registry: Registry =
        toml::from_str(&fs::read_to_string(root.join("contracts/registry.toml"))?)?;
    let release_registry: ReleaseRegistry = toml::from_str(&fs::read_to_string(
        root.join("contracts/release-registry.toml"),
    )?)?;
    let descriptors = FileDescriptorSet::decode(FILE_DESCRIPTOR_SET)?;
    if env::args().any(|argument| argument == "--update") {
        write_fixtures(&root, &registry)?;
    }
    validate(&root, &registry, &descriptors)?;
    validate_release_registry(&release_registry)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> ReleaseRegistry {
        toml::from_str(include_str!("../../../contracts/release-registry.toml"))
            .unwrap_or_else(|error| unreachable!("checked-in release registry parses: {error}"))
    }

    #[test]
    fn checked_in_process_managers_have_all_failure_controls() {
        validate_release_registry(&registry())
            .unwrap_or_else(|error| unreachable!("release registry validates: {error}"));
    }

    #[test]
    fn missing_timeout_compensation_escalation_or_observation_fails_closed() {
        let mut value = registry();
        value.process_managers[0].timeout_seconds = 0;
        assert!(validate_release_registry(&value).is_err());
        let mut value = registry();
        value.process_managers[0].compensation.clear();
        assert!(validate_release_registry(&value).is_err());
        let mut value = registry();
        value.process_managers[0].escalation.clear();
        assert!(validate_release_registry(&value).is_err());
        let mut value = registry();
        value.process_managers[0].observable_containment.clear();
        assert!(validate_release_registry(&value).is_err());
    }
}
