//! Binary for running structure validation on the workspace

use anyhow::Result;
use std::path::PathBuf;
use structure_validator::{CrateStructure, StructureValidator};

fn main() -> Result<()> {
    // Find workspace root (go up from current directory until we find workspace Cargo.toml)
    let workspace_root = find_workspace_root()?;

    println!(
        "🔍 Validating workspace structure at: {}",
        workspace_root.display()
    );
    println!();

    // Create validator with default structure requirements
    let validator = StructureValidator::new(CrateStructure::default());

    // Validate the workspace
    let report = validator.validate_workspace(&workspace_root)?;

    // Print the report
    report.print_summary();

    // Exit with error if validation failed
    if !report.is_valid() {
        std::process::exit(1);
    }

    println!("✅ All crates have valid structure!");
    Ok(())
}

fn find_workspace_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml)?;
            if content.contains("[workspace]") {
                return Ok(current);
            }
        }

        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => anyhow::bail!("Could not find workspace root"),
        }
    }
}
