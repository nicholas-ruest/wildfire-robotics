#![forbid(unsafe_code)]
//! Command-line entry point for the context migration policy validator.

use migration_check::validate;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let policy = root.join("fixtures/persistence-service/migration-policy.toml");
    let validated = validate(&root, &policy)?;
    println!(
        "validated {} ordered migrations for {}",
        validated.migrations.len(),
        validated.context
    );
    Ok(())
}
