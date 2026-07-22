#![forbid(unsafe_code)]
//! Fail-closed validation and deterministic reporting for an integrated release candidate.

use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Debug, Deserialize)]
struct Traceability {
    schema_version: u16,
    release_id: String,
    invariant_registry: String,
    process_registry: String,
    expected: Expected,
    #[serde(default)]
    required_artifact: Vec<RequiredArtifact>,
}

#[derive(Debug, Deserialize)]
struct InvariantRegistry {
    schema_version: u16,
    invariant: Vec<InvariantLink>,
}

#[derive(Debug, Deserialize)]
struct InvariantLink {
    id: String,
    context: String,
    code: String,
    test: String,
    evidence: String,
}

#[derive(Debug, Deserialize)]
struct ProcessRegistry {
    schema_version: u16,
    process_managers: Vec<ProcessAssurance>,
}

#[derive(Debug, Deserialize)]
struct ProcessAssurance {
    name: String,
    owner: String,
    timeout_seconds: u64,
    compensation: String,
    escalation: String,
    observable_containment: String,
}

#[derive(Debug, Deserialize)]
struct Expected {
    adrs: usize,
    contexts: usize,
    invariants: usize,
    contracts: usize,
    process_managers: usize,
}

#[derive(Debug, Deserialize)]
struct RequiredArtifact {
    id: String,
    path: String,
    kind: String,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    schema_version: u16,
    release_id: String,
    status: String,
    production_ready: bool,
    field_ready: bool,
    known_release_blockers: u32,
    evidence_boundary: String,
    risk_register: String,
    architecture_deviations: String,
}

#[derive(Debug, Deserialize)]
struct RiskRegistry {
    schema_version: u16,
    #[serde(default)]
    risk: Vec<Risk>,
}
#[derive(Debug, Deserialize)]
struct Risk {
    id: String,
    owner: String,
    expires: String,
    disposition: String,
    release_blocking: bool,
}

#[derive(Debug, Deserialize)]
struct Deviations {
    schema_version: u16,
    #[serde(default)]
    deviation: Vec<Deviation>,
}
#[derive(Debug, Deserialize)]
struct Deviation {
    id: String,
    status: String,
    approval: String,
    expires: String,
}

#[derive(Debug, Deserialize)]
struct Boundary {
    schema_version: u16,
    software_release_label: String,
    production_claim_allowed: bool,
    field_claim_allowed: bool,
    #[serde(default)]
    gate: Vec<ExternalGate>,
}
#[derive(Debug, Deserialize)]
struct ExternalGate {
    id: String,
    status: String,
    owner: String,
    evidence: String,
    blocks: String,
}

#[derive(Debug, Serialize)]
struct Report {
    schema_version: u16,
    release_id: String,
    result: &'static str,
    inventory: Inventory,
    validated_artifacts: Vec<ArtifactDigest>,
    external_gates_outstanding: usize,
    production_ready: bool,
    field_ready: bool,
}

#[derive(Debug, Serialize)]
struct Inventory {
    adrs: usize,
    contexts: usize,
    invariants: usize,
    contracts: usize,
    process_managers: usize,
}
#[derive(Debug, Serialize)]
struct ArtifactDigest {
    id: String,
    kind: String,
    path: String,
    sha256: String,
}

fn main() -> ExitCode {
    let root = env::args_os()
        .nth(1)
        .map_or_else(workspace_root, PathBuf::from);
    match validate(&root) {
        Ok(report) => {
            let json = match serde_json::to_string_pretty(&report) {
                Ok(value) => value,
                Err(error) => {
                    eprintln!("release-gate: serialize report: {error}");
                    return ExitCode::FAILURE;
                }
            };
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(errors) => {
            for error in errors {
                eprintln!("release-gate: {error}");
            }
            ExitCode::FAILURE
        }
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[allow(clippy::too_many_lines)] // One ordered gate makes the fail-closed decision auditable.
fn validate(root: &Path) -> Result<Report, Vec<String>> {
    let mut errors = Vec::new();
    let Some(trace): Option<Traceability> = load(root, "release/traceability.toml", &mut errors)
    else {
        return Err(errors);
    };
    let Some(candidate): Option<Candidate> = load(root, "release/rc-manifest.toml", &mut errors)
    else {
        return Err(errors);
    };
    let Some(invariants): Option<InvariantRegistry> =
        load(root, &trace.invariant_registry, &mut errors)
    else {
        return Err(errors);
    };
    let Some(processes): Option<ProcessRegistry> = load(root, &trace.process_registry, &mut errors)
    else {
        return Err(errors);
    };
    let Some(risks): Option<RiskRegistry> = load(root, &candidate.risk_register, &mut errors)
    else {
        return Err(errors);
    };
    let Some(deviations): Option<Deviations> =
        load(root, &candidate.architecture_deviations, &mut errors)
    else {
        return Err(errors);
    };
    let Some(boundary): Option<Boundary> = load(root, &candidate.evidence_boundary, &mut errors)
    else {
        return Err(errors);
    };
    for version in [
        trace.schema_version,
        candidate.schema_version,
        risks.schema_version,
        deviations.schema_version,
        boundary.schema_version,
        invariants.schema_version,
        processes.schema_version,
    ] {
        require(
            version == 1,
            "unsupported registry schema version",
            &mut errors,
        );
    }
    require(
        trace.release_id == candidate.release_id,
        "release identifiers differ",
        &mut errors,
    );
    require(
        candidate.status == "software-release-candidate",
        "status must be software-release-candidate",
        &mut errors,
    );
    require(
        !candidate.production_ready && !candidate.field_ready,
        "RC must not claim field or production readiness",
        &mut errors,
    );
    require(
        !boundary.production_claim_allowed && !boundary.field_claim_allowed,
        "evidence boundary must prohibit field and production claims",
        &mut errors,
    );
    require(
        boundary.software_release_label == "software-release-candidate",
        "evidence boundary label mismatch",
        &mut errors,
    );
    require(
        candidate.known_release_blockers == 0,
        "known release blockers are nonzero",
        &mut errors,
    );

    let inventory = inventory(root, &mut errors);
    require(
        inventory.adrs == trace.expected.adrs,
        "ADR inventory mismatch",
        &mut errors,
    );
    require(
        inventory.contexts == trace.expected.contexts,
        "context inventory mismatch",
        &mut errors,
    );
    require(
        inventory.invariants == trace.expected.invariants,
        "invariant inventory mismatch",
        &mut errors,
    );
    require(
        inventory.contracts == trace.expected.contracts,
        "contract inventory mismatch",
        &mut errors,
    );
    require(
        inventory.process_managers == trace.expected.process_managers,
        "process-manager inventory mismatch",
        &mut errors,
    );

    validate_process_managers(root, &mut errors);
    validate_process_registry(&processes, trace.expected.process_managers, &mut errors);
    validate_invariant_links(root, &invariants, &mut errors);
    validate_risks(&risks, &mut errors);
    validate_deviations(&deviations, &mut errors);
    validate_boundary(&boundary, &mut errors);

    let mut digests = Vec::new();
    let mut ids = BTreeSet::new();
    for artifact in &trace.required_artifact {
        require(
            ids.insert(&artifact.id),
            &format!("duplicate artifact id {}", artifact.id),
            &mut errors,
        );
        let path = root.join(&artifact.path);
        if !path.is_file() {
            errors.push(format!(
                "missing required artifact {} at {}",
                artifact.id, artifact.path
            ));
            continue;
        }
        match fs::read(&path) {
            Ok(bytes) if !bytes.is_empty() => digests.push(ArtifactDigest {
                id: artifact.id.clone(),
                kind: artifact.kind.clone(),
                path: artifact.path.clone(),
                sha256: format!("{:x}", Sha256::digest(bytes)),
            }),
            Ok(_) => errors.push(format!("required artifact {} is empty", artifact.id)),
            Err(error) => errors.push(format!("read {}: {error}", artifact.path)),
        }
    }
    require(
        ids.len() >= 12,
        "traceability has fewer than twelve cross-cutting artifacts",
        &mut errors,
    );
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(Report {
        schema_version: 1,
        release_id: candidate.release_id,
        result: "pass",
        inventory,
        validated_artifacts: digests,
        external_gates_outstanding: boundary
            .gate
            .iter()
            .filter(|gate| gate.status != "satisfied")
            .count(),
        production_ready: false,
        field_ready: false,
    })
}

fn validate_process_registry(
    registry: &ProcessRegistry,
    expected: usize,
    errors: &mut Vec<String>,
) {
    require(
        registry.process_managers.len() == expected,
        "process assurance registry count mismatch",
        errors,
    );
    let mut names = BTreeSet::new();
    for process in &registry.process_managers {
        require(
            names.insert(&process.name),
            &format!("duplicate process assurance {}", process.name),
            errors,
        );
        require(
            process.timeout_seconds > 0,
            &format!("process {} has no bounded timeout", process.name),
            errors,
        );
        for (field, value) in [
            ("owner", &process.owner),
            ("compensation", &process.compensation),
            ("escalation", &process.escalation),
            ("observable containment", &process.observable_containment),
        ] {
            require(
                !value.trim().is_empty(),
                &format!("process {} lacks {field}", process.name),
                errors,
            );
        }
    }
}

fn validate_invariant_links(root: &Path, registry: &InvariantRegistry, errors: &mut Vec<String>) {
    let mut documented = BTreeSet::new();
    for path in files(&root.join("docs/ddd/contexts")) {
        if let Ok(text) = fs::read_to_string(path) {
            extract_ids(&text, "-INV-", &mut documented);
        }
    }
    let mut linked = BTreeSet::new();
    for link in &registry.invariant {
        require(
            linked.insert(link.id.clone()),
            &format!("duplicate invariant link {}", link.id),
            errors,
        );
        require(
            documented.contains(&link.id),
            &format!("unknown invariant link {}", link.id),
            errors,
        );
        let context_segment = format!("/{}/", link.context);
        require(
            link.code.contains(&context_segment) && link.test.contains(&context_segment),
            &format!(
                "invariant {} paths do not belong to context {}",
                link.id, link.context
            ),
            errors,
        );
        for (kind, relative) in [
            ("code", &link.code),
            ("test", &link.test),
            ("evidence", &link.evidence),
        ] {
            match fs::read_to_string(root.join(relative)) {
                Ok(content) => {
                    require(
                        !content.trim().is_empty(),
                        &format!("invariant {} {kind} artifact is empty", link.id),
                        errors,
                    );
                    if kind == "evidence" {
                        require(
                            content.contains(&link.id),
                            &format!(
                                "invariant {} evidence does not define its identifier",
                                link.id
                            ),
                            errors,
                        );
                    }
                }
                Err(error) => errors.push(format!(
                    "invariant {} cannot read {kind} artifact {relative}: {error}",
                    link.id
                )),
            }
        }
    }
    require(
        linked == documented,
        "invariant traceability is not an exact set match",
        errors,
    );
}

fn load<T: for<'de> Deserialize<'de>>(
    root: &Path,
    relative: &str,
    errors: &mut Vec<String>,
) -> Option<T> {
    let path = root.join(relative);
    let text = fs::read_to_string(&path)
        .map_err(|e| errors.push(format!("read {relative}: {e}")))
        .ok()?;
    toml::from_str(&text)
        .map_err(|e| errors.push(format!("parse {relative}: {e}")))
        .ok()
}

fn inventory(root: &Path, errors: &mut Vec<String>) -> Inventory {
    let adr_dir = root.join("docs/adr");
    let context_dir = root.join("docs/ddd/contexts");
    let adrs = files(&adr_dir)
        .iter()
        .filter(|p| {
            p.file_name().and_then(|v| v.to_str()).is_some_and(|v| {
                v.starts_with("ADR-")
                    && Path::new(v)
                        .extension()
                        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
            })
        })
        .count();
    let contexts = files(&context_dir)
        .iter()
        .filter(|p| p.extension().is_some_and(|v| v == "md"))
        .count();
    let mut invariants = BTreeSet::new();
    for path in files(&context_dir) {
        if let Ok(text) = fs::read_to_string(path) {
            extract_ids(&text, "-INV-", &mut invariants);
        }
    }
    let contracts = fs::read_to_string(root.join("contracts/registry.toml")).map_or_else(
        |e| {
            errors.push(format!("read contract registry: {e}"));
            0
        },
        |v| v.matches("[[contracts]]").count(),
    );
    let process_managers = fs::read_to_string(root.join("docs/ddd/process-managers.md"))
        .map_or_else(
            |e| {
                errors.push(format!("read process managers: {e}"));
                0
            },
            |v| v.lines().filter(|line| line.starts_with("## ")).count(),
        );
    Inventory {
        adrs,
        contexts,
        invariants: invariants.len(),
        contracts,
        process_managers,
    }
}

fn extract_ids(text: &str, marker: &str, output: &mut BTreeSet<String>) {
    for token in text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')) {
        if token.contains(marker) && token.len() >= marker.len() + 5 {
            output.insert(token.to_owned());
        }
    }
}

fn validate_process_managers(root: &Path, errors: &mut Vec<String>) {
    let Ok(text) = fs::read_to_string(root.join("docs/ddd/process-managers.md")) else {
        return;
    };
    for section in text.split("\n## ").skip(1) {
        let name = section.lines().next().unwrap_or("unnamed");
        for field in [
            "**Owner:**",
            "**Flow:**",
            "**Compensation:**",
            "**Escalation:**",
        ] {
            require(
                section.contains(field),
                &format!("process manager {name} lacks {field}"),
                errors,
            );
        }
    }
    require(
        text.contains("deadline"),
        "process managers lack deadline/timeout policy",
        errors,
    );
}

fn validate_risks(registry: &RiskRegistry, errors: &mut Vec<String>) {
    let mut ids = BTreeSet::new();
    for risk in &registry.risk {
        require(
            ids.insert(&risk.id),
            &format!("duplicate risk {}", risk.id),
            errors,
        );
        require(
            !risk.owner.trim().is_empty()
                && !risk.expires.trim().is_empty()
                && !risk.disposition.trim().is_empty(),
            &format!("risk {} lacks owner, expiry, or disposition", risk.id),
            errors,
        );
        validate_expiry(&risk.expires, &format!("risk {}", risk.id), errors);
        require(
            !risk.release_blocking,
            &format!("risk {} is release-blocking", risk.id),
            errors,
        );
    }
}

fn validate_deviations(registry: &Deviations, errors: &mut Vec<String>) {
    for deviation in &registry.deviation {
        require(
            deviation.status == "approved",
            &format!("deviation {} is not approved", deviation.id),
            errors,
        );
        validate_expiry(
            &deviation.expires,
            &format!("deviation {}", deviation.id),
            errors,
        );
        require(
            !deviation.approval.trim().is_empty() && !deviation.expires.trim().is_empty(),
            &format!("deviation {} lacks approval or expiry", deviation.id),
            errors,
        );
    }
}

fn validate_expiry(value: &str, subject: &str, errors: &mut Vec<String>) {
    match NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        Ok(expiry) => require(
            expiry >= Utc::now().date_naive(),
            &format!("{subject} is expired"),
            errors,
        ),
        Err(_) => errors.push(format!("{subject} has invalid YYYY-MM-DD expiry")),
    }
}

fn validate_boundary(boundary: &Boundary, errors: &mut Vec<String>) {
    let mut ids = BTreeSet::new();
    require(
        !boundary.gate.is_empty(),
        "external evidence gate inventory is empty",
        errors,
    );
    for gate in &boundary.gate {
        require(
            ids.insert(&gate.id),
            &format!("duplicate external gate {}", gate.id),
            errors,
        );
        require(
            !gate.owner.trim().is_empty() && !gate.evidence.trim().is_empty(),
            &format!(
                "external gate {} lacks owner or evidence requirement",
                gate.id
            ),
            errors,
        );
        require(
            matches!(gate.blocks.as_str(), "field" | "production" | "both"),
            &format!("external gate {} has invalid blocks value", gate.id),
            errors,
        );
    }
}

fn files(path: &Path) -> Vec<PathBuf> {
    fs::read_dir(path)
        .map(|entries| entries.filter_map(Result::ok).map(|e| e.path()).collect())
        .unwrap_or_default()
}
fn require(condition: bool, message: &str, errors: &mut Vec<String>) {
    if !condition {
        errors.push(message.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extracts_unique_invariant_identifiers() {
        let mut values = BTreeSet::new();
        extract_ids("`HI-INV-001` HI-INV-001 AD-INV-011", "-INV-", &mut values);
        assert_eq!(
            values.into_iter().collect::<Vec<_>>(),
            ["AD-INV-011", "HI-INV-001"]
        );
    }
    #[test]
    fn repository_release_registries_pass() {
        assert!(validate(&workspace_root()).is_ok());
    }

    #[test]
    fn release_blocking_risk_fails_closed() {
        let risks = RiskRegistry {
            schema_version: 1,
            risk: vec![Risk {
                id: "RISK-1".into(),
                owner: "safety".into(),
                expires: "2099-01-01".into(),
                disposition: "unresolved".into(),
                release_blocking: true,
            }],
        };
        let mut errors = Vec::new();
        validate_risks(&risks, &mut errors);
        assert!(
            errors
                .iter()
                .any(|error| error.contains("release-blocking"))
        );
    }

    #[test]
    fn expired_risk_fails_closed() {
        let risks = RiskRegistry {
            schema_version: 1,
            risk: vec![Risk {
                id: "RISK-1".into(),
                owner: "safety".into(),
                expires: "2000-01-01".into(),
                disposition: "accepted".into(),
                release_blocking: false,
            }],
        };
        let mut errors = Vec::new();
        validate_risks(&risks, &mut errors);
        assert!(errors.iter().any(|error| error.contains("expired")));
    }

    #[test]
    fn field_claim_is_not_permitted_without_external_evidence() {
        let boundary = Boundary {
            schema_version: 1,
            software_release_label: "software-release-candidate".into(),
            production_claim_allowed: false,
            field_claim_allowed: false,
            gate: Vec::new(),
        };
        let mut errors = Vec::new();
        validate_boundary(&boundary, &mut errors);
        assert!(errors.iter().any(|error| error.contains("empty")));
    }
}
