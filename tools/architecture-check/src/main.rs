#![forbid(unsafe_code)]
//! Machine-enforced bounded-context and documentation checks (Prompt 00).

use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
struct Registry {
    schema_version: u16,
    contexts: Vec<Context>,
}

#[derive(Debug, Deserialize)]
struct Context {
    name: String,
    #[serde(rename = "crate")]
    crate_name: String,
    schemas: Vec<String>,
    migrations: Vec<String>,
    deployables: Vec<String>,
    owners: Vec<String>,
    adrs: Vec<u16>,
    invariant_prefixes: Vec<String>,
}

fn main() {
    let root = workspace_root();
    let errors = validate(&root);
    if errors.is_empty() {
        println!("architecture-check: all ownership, dependency, and documentation checks passed");
        return;
    }
    for error in &errors {
        eprintln!("architecture-check: {error}");
    }
    std::process::exit(1);
}

fn workspace_root() -> PathBuf {
    env::var_os("CARGO_MANIFEST_DIR").map_or_else(
        || PathBuf::from("."),
        |dir| PathBuf::from(dir).join("../.."),
    )
}

fn validate(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let registry_path = root.join("docs/architecture/context-ownership.toml");
    let registry_text = match fs::read_to_string(&registry_path) {
        Ok(value) => value,
        Err(error) => return vec![format!("cannot read {}: {error}", registry_path.display())],
    };
    let registry: Registry = match toml::from_str(&registry_text) {
        Ok(value) => value,
        Err(error) => return vec![format!("invalid ownership registry: {error}")],
    };
    validate_registry(root, &registry, &mut errors);
    validate_context_dependencies(root, &registry, &mut errors);
    validate_documentation(root, &registry, &mut errors);
    errors
}

fn validate_registry(root: &Path, registry: &Registry, errors: &mut Vec<String>) {
    if registry.schema_version != 1 {
        errors.push("unsupported ownership registry schema_version".into());
    }
    if registry.contexts.len() != 15 {
        errors.push(format!(
            "expected 15 contexts, found {}",
            registry.contexts.len()
        ));
    }
    let mut names = BTreeSet::new();
    let mut crates = BTreeSet::new();
    let mut schemas = BTreeSet::new();
    for context in &registry.contexts {
        if !names.insert(&context.name) {
            errors.push(format!("duplicate context name: {}", context.name));
        }
        if !crates.insert(&context.crate_name) {
            errors.push(format!("duplicate context crate: {}", context.crate_name));
        }
        if !root
            .join("crates")
            .join(&context.crate_name)
            .join("Cargo.toml")
            .is_file()
        {
            errors.push(format!("missing context crate: {}", context.crate_name));
        }
        if context.schemas.is_empty()
            || context.migrations.is_empty()
            || context.deployables.is_empty()
            || context.owners.is_empty()
            || context.adrs.is_empty()
            || context.invariant_prefixes.is_empty()
        {
            errors.push(format!(
                "{} has incomplete ownership metadata",
                context.name
            ));
        }
        for schema in &context.schemas {
            if !schemas.insert(schema) {
                errors.push(format!("database schema is owned more than once: {schema}"));
            }
        }
        for adr in &context.adrs {
            if !(1..=74).contains(adr) {
                errors.push(format!("{} references invalid ADR-{adr:03}", context.name));
            }
        }
    }
}

fn validate_context_dependencies(root: &Path, registry: &Registry, errors: &mut Vec<String>) {
    let context_crates: BTreeSet<_> = registry
        .contexts
        .iter()
        .map(|context| context.crate_name.as_str())
        .collect();
    let forbidden_domain_dependencies = [
        "tokio",
        "sqlx",
        "diesel",
        "reqwest",
        "tonic",
        "axum",
        "actix-web",
        "r2d2",
        "nats",
        "async-nats",
        "rdkafka",
        "aws-sdk-s3",
        "kube",
        "rosrust",
        "mavlink",
        "wildfire-contracts-generated",
    ];
    for context in &registry.contexts {
        let manifest_path = root
            .join("crates")
            .join(&context.crate_name)
            .join("Cargo.toml");
        let Ok(text) = fs::read_to_string(&manifest_path) else {
            continue;
        };
        let Ok(document) = text.parse::<toml::Table>() else {
            errors.push(format!(
                "invalid Cargo manifest: {}",
                manifest_path.display()
            ));
            continue;
        };
        let dependencies = document
            .get("dependencies")
            .and_then(toml::Value::as_table)
            .cloned()
            .unwrap_or_default();
        validate_dependency_names(
            &context.crate_name,
            dependencies.keys().map(String::as_str),
            &context_crates,
            &forbidden_domain_dependencies,
            errors,
        );
    }
}

fn validate_dependency_names<'a>(
    crate_name: &str,
    dependencies: impl IntoIterator<Item = &'a str>,
    context_crates: &BTreeSet<&str>,
    forbidden_domain_dependencies: &[&str],
    errors: &mut Vec<String>,
) {
    for dependency in dependencies {
        if context_crates.contains(dependency) {
            errors.push(format!(
                "{crate_name} directly depends on bounded context {dependency}; use a port and contract adapter"
            ));
        }
        if forbidden_domain_dependencies.contains(&dependency) {
            errors.push(format!(
                "{crate_name} domain crate directly depends on infrastructure/vendor crate {dependency}"
            ));
        }
    }
}

fn validate_documentation(root: &Path, registry: &Registry, errors: &mut Vec<String>) {
    let adr_dir = root.join("docs/adr");
    let mut adr_files = BTreeMap::new();
    for number in 1..=74 {
        let prefix = format!("ADR-{number:03}-");
        let matches = read_dir_paths(&adr_dir)
            .into_iter()
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| {
                        name.starts_with(&prefix)
                            && Path::new(name)
                                .extension()
                                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
                    })
            })
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            errors.push(format!(
                "expected one {prefix} document, found {}",
                matches.len()
            ));
        } else if let Some(path) = matches.first() {
            adr_files.insert(number, path.clone());
        }
    }
    for context in &registry.contexts {
        for number in &context.adrs {
            if !adr_files.contains_key(number) {
                errors.push(format!(
                    "{} references missing ADR-{number:03}",
                    context.name
                ));
            }
        }
        for prefix in &context.invariant_prefixes {
            let count = markdown_files(root)
                .iter()
                .filter_map(|path| fs::read_to_string(path).ok())
                .filter(|text| text.contains(prefix))
                .count();
            if count == 0 {
                errors.push(format!(
                    "no documentation defines invariant prefix {prefix}"
                ));
            }
        }
    }
    for path in markdown_files(root) {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        validate_local_links(root, &path, &text, errors);
    }
}

fn validate_local_links(root: &Path, source: &Path, text: &str, errors: &mut Vec<String>) {
    for segment in text.split("](").skip(1) {
        let Some(raw_target) = segment.split(')').next() else {
            continue;
        };
        let target = raw_target.split('#').next().unwrap_or_default();
        if target.is_empty()
            || target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with("mailto:")
        {
            continue;
        }
        let resolved = source.parent().unwrap_or(root).join(target);
        if !resolved.exists() {
            errors.push(format!(
                "broken local link in {}: {target}",
                source.display()
            ));
        }
    }
}

fn markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    collect_markdown(&root.join("docs"), &mut result);
    collect_markdown(&root.join(".plans"), &mut result);
    result
}

fn collect_markdown(directory: &Path, result: &mut Vec<PathBuf>) {
    for path in read_dir_paths(directory) {
        if path.is_dir() {
            collect_markdown(&path, result);
        } else if path.extension().is_some_and(|extension| extension == "md") {
            result.push(path);
        }
    }
}

fn read_dir_paths(directory: &Path) -> Vec<PathBuf> {
    fs::read_dir(directory)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_architecture_is_valid() {
        let errors = validate(&workspace_root());
        assert!(errors.is_empty(), "{}", errors.join("\n"));
    }

    #[test]
    fn cross_context_and_vendor_dependencies_are_rejected() {
        let contexts = BTreeSet::from(["mission-control", "safety-assurance"]);
        let mut errors = Vec::new();
        validate_dependency_names(
            "mission-control",
            ["safety-assurance", "sqlx"],
            &contexts,
            &["sqlx"],
            &mut errors,
        );
        assert_eq!(errors.len(), 2);
    }
}
