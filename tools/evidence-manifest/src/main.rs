#![forbid(unsafe_code)]
//! Deterministic quality orchestration and attributable evidence generation.

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

const STEPS: &[(&str, &[&str])] = &[
    ("format", &["cargo", "fmt", "--all", "--", "--check"]),
    (
        "build",
        &[
            "cargo",
            "build",
            "--workspace",
            "--all-features",
            "--locked",
        ],
    ),
    (
        "clippy",
        &[
            "cargo",
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
    ),
    (
        "test",
        &[
            "cargo",
            "nextest",
            "run",
            "--workspace",
            "--all-features",
            "--locked",
        ],
    ),
    (
        "docs",
        &["cargo", "test", "--workspace", "--doc", "--locked"],
    ),
    (
        "architecture",
        &["cargo", "run", "--locked", "-p", "architecture-check", "--"],
    ),
    ("licenses", &["cargo", "deny", "check"]),
    ("advisories", &["cargo", "audit", "--deny", "warnings"]),
    (
        "secrets",
        &[
            "gitleaks",
            "detect",
            "--no-banner",
            "--redact",
            "--source",
            ".",
        ],
    ),
    (
        "sast",
        &[
            "semgrep",
            "scan",
            "--config",
            ".semgrep.yml",
            "--error",
            "--metrics",
            "off",
        ],
    ),
    (
        "coverage",
        &[
            "cargo",
            "llvm-cov",
            "nextest",
            "--workspace",
            "--all-features",
            "--lcov",
            "--output-path",
            "target/evidence/coverage.lcov",
        ],
    ),
    ("sbom", &["bash", "scripts/generate-sbom.sh"]),
];

#[derive(Debug, Deserialize)]
struct ToolVersions {
    schema_version: u16,
    rust: String,
    cargo_nextest: String,
    cargo_deny: String,
    cargo_audit: String,
    cargo_llvm_cov: String,
    cargo_cyclonedx: String,
    gitleaks: String,
    semgrep: String,
}

#[derive(Debug, Serialize)]
struct Manifest {
    schema_version: u16,
    generated_at_utc: String,
    repository: String,
    revision: String,
    source_dirty: bool,
    rustc: String,
    cargo: String,
    cargo_lock_sha256: String,
    tools: Vec<ToolEvidence>,
    steps: Vec<StepEvidence>,
    artifacts: Vec<ArtifactEvidence>,
    overall_success: bool,
    promotion_authority: bool,
}

#[derive(Debug, Serialize)]
struct ToolEvidence {
    name: String,
    required_version: String,
    actual_version: String,
    verified: bool,
}

#[derive(Debug, Serialize)]
struct StepEvidence {
    name: String,
    command: Vec<String>,
    success: bool,
    exit_code: Option<i32>,
    log: String,
    log_sha256: String,
}

#[derive(Debug, Serialize)]
struct ArtifactEvidence {
    path: String,
    sha256: String,
}

fn main() -> ExitCode {
    let root = workspace_root();
    let versions = match read_versions(&root) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("quality-gate: {error}");
            return ExitCode::FAILURE;
        }
    };
    if env::args().nth(1).as_deref() != Some("verify") {
        eprintln!("usage: cargo quality");
        return ExitCode::FAILURE;
    }
    match verify(&root, versions) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) | Err(_) => ExitCode::FAILURE,
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_versions(root: &Path) -> Result<ToolVersions, String> {
    let path = root.join("tools/quality-tools.toml");
    let text =
        fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let versions: ToolVersions =
        toml::from_str(&text).map_err(|error| format!("parse tools: {error}"))?;
    if versions.schema_version != 1 {
        return Err("unsupported tool version schema".into());
    }
    Ok(versions)
}

fn verify(root: &Path, versions: ToolVersions) -> Result<bool, String> {
    let evidence_dir = root.join("target/evidence");
    let logs_dir = evidence_dir.join("logs");
    fs::create_dir_all(&logs_dir).map_err(|error| format!("create evidence directory: {error}"))?;
    fs::create_dir_all(evidence_dir.join("sbom"))
        .map_err(|error| format!("create SBOM directory: {error}"))?;

    let mut steps = Vec::new();
    for (name, command) in STEPS {
        println!("quality-gate: running {name}: {}", command.join(" "));
        let output = Command::new(command[0])
            .args(&command[1..])
            .current_dir(root)
            .env("PROPTEST_RNG_SEED", "6289646323763466821")
            .output();
        let log_path = logs_dir.join(format!("{name}.log"));
        let (success, exit_code, bytes) = match output {
            Ok(output) => {
                let mut bytes = output.stdout;
                bytes.extend_from_slice(&output.stderr);
                (output.status.success(), output.status.code(), bytes)
            }
            Err(error) => (
                false,
                None,
                format!("unable to execute {}: {error}\n", command[0]).into_bytes(),
            ),
        };
        fs::write(&log_path, &bytes)
            .map_err(|error| format!("write {}: {error}", log_path.display()))?;
        if !success {
            eprintln!("quality-gate: {name} failed; see {}", log_path.display());
        }
        steps.push(StepEvidence {
            name: (*name).into(),
            command: command.iter().map(ToString::to_string).collect(),
            success,
            exit_code,
            log: relative(root, &log_path),
            log_sha256: sha256(&log_path)?,
        });
    }

    let manifest = build_manifest(root, versions, steps)?;
    let success = manifest.overall_success;
    let manifest_path = evidence_dir.join("quality-manifest.json");
    let bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| format!("serialize evidence: {error}"))?;
    fs::write(&manifest_path, bytes).map_err(|error| format!("write evidence: {error}"))?;
    println!(
        "quality-gate: evidence written to {}",
        manifest_path.display()
    );
    Ok(success)
}

fn build_manifest(
    root: &Path,
    versions: ToolVersions,
    steps: Vec<StepEvidence>,
) -> Result<Manifest, String> {
    let mut artifact_paths = vec!["target/evidence/coverage.lcov".to_owned()];
    collect_files(
        root,
        &root.join("target/evidence/sbom"),
        &mut artifact_paths,
    );
    let artifacts = artifact_paths
        .into_iter()
        .filter(|path| root.join(path).is_file())
        .map(|path| {
            let absolute = root.join(&path);
            Ok(ArtifactEvidence {
                path,
                sha256: sha256(&absolute)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let tools = tool_evidence(root, versions);
    let overall_success =
        steps.iter().all(|step| step.success) && tools.iter().all(|tool| tool.verified);
    Ok(Manifest {
        schema_version: 1,
        generated_at_utc: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        repository: command_text(root, "git", &["config", "--get", "remote.origin.url"]),
        revision: command_text(root, "git", &["rev-parse", "HEAD"]),
        source_dirty: !command_text(root, "git", &["status", "--porcelain"]).is_empty(),
        rustc: command_text(root, "rustc", &["--version", "--verbose"]),
        cargo: command_text(root, "cargo", &["--version", "--verbose"]),
        cargo_lock_sha256: sha256(&root.join("Cargo.lock"))?,
        tools,
        steps,
        artifacts,
        overall_success,
        promotion_authority: false,
    })
}

fn tool_evidence(root: &Path, value: ToolVersions) -> Vec<ToolEvidence> {
    [
        ("rust", value.rust, "rustc", &["--version"][..]),
        (
            "cargo-nextest",
            value.cargo_nextest,
            "cargo",
            &["nextest", "--version"],
        ),
        (
            "cargo-deny",
            value.cargo_deny,
            "cargo",
            &["deny", "--version"],
        ),
        (
            "cargo-audit",
            value.cargo_audit,
            "cargo",
            &["audit", "--version"],
        ),
        (
            "cargo-llvm-cov",
            value.cargo_llvm_cov,
            "cargo",
            &["llvm-cov", "--version"],
        ),
        (
            "cargo-cyclonedx",
            value.cargo_cyclonedx,
            "cargo",
            &["cyclonedx", "--version"],
        ),
        ("gitleaks", value.gitleaks, "gitleaks", &["version"]),
        ("semgrep", value.semgrep, "semgrep", &["--version"]),
    ]
    .into_iter()
    .map(|(name, required_version, program, arguments)| {
        let actual_version = command_text(root, program, arguments);
        ToolEvidence {
            name: name.into(),
            verified: actual_version.contains(&required_version),
            required_version,
            actual_version,
        }
    })
    .collect()
}

fn command_text(root: &Path, program: &str, arguments: &[&str]) -> String {
    Command::new(program)
        .args(arguments)
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_default()
}

fn sha256(path: &Path) -> Result<String, String> {
    let mut file =
        fs::File::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| format!("read {}: {error}", path.display()))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn collect_files(root: &Path, directory: &Path, result: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for path in entries.filter_map(Result::ok).map(|entry| entry.path()) {
        if path.is_dir() {
            collect_files(root, &path, result);
        } else if path.is_file() {
            result.push(relative(root, &path));
        }
    }
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_gate_contains_every_required_category() {
        let names: Vec<_> = STEPS.iter().map(|(name, _)| *name).collect();
        for required in [
            "format",
            "build",
            "clippy",
            "test",
            "licenses",
            "advisories",
            "secrets",
            "sast",
            "coverage",
            "sbom",
        ] {
            assert!(names.contains(&required), "missing {required}");
        }
    }

    #[test]
    fn tool_versions_are_pinned() -> Result<(), String> {
        let versions = read_versions(&workspace_root())?;
        assert!(!versions.rust.contains('*'));
        assert!(!versions.cargo_nextest.contains('*'));
        Ok(())
    }
}
