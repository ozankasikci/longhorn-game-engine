//! Binary for migrating crate structures to standard format

use anyhow::Result;
use std::env;
use std::path::PathBuf;
use structure_validator::{migration::StructureMigrator, CrateStructure, StructureValidator};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let dry_run = args.contains(&"--dry-run".to_string());

    // Find workspace root
    let workspace_root = find_workspace_root()?;

    println!("🔧 Structure Migration Tool");
    println!("==========================");
    println!("Workspace: {}", workspace_root.display());
    println!(
        "Mode: {}",
        if dry_run { "DRY RUN" } else { "APPLY CHANGES" }
    );
    println!();

    // Create validator and migrator
    let expected_structure = CrateStructure::default();
    let validator = StructureValidator::new(expected_structure.clone());
    let migrator = StructureMigrator::new(expected_structure, dry_run);

    // Validate workspace first
    let validation_report = validator.validate_workspace(&workspace_root)?;

    if validation_report.is_valid() {
        println!("✅ All crates already have valid structure!");
        return Ok(());
    }

    println!(
        "Found {} crates with issues",
        validation_report
            .reports
            .iter()
            .filter(|r| !r.is_valid())
            .count()
    );
    println!();

    // Migrate each crate with issues
    let mut total_actions = 0;
    for report in &validation_report.reports {
        if !report.is_valid() {
            let migration_report = migrator.fix_crate(&report.crate_path, report)?;
            migration_report.print_summary();
            total_actions += migration_report.actions.len();
            println!();
        }
    }

    println!("Summary");
    println!("=======");
    println!("Total actions: {}", total_actions);

    if dry_run {
        println!();
        println!("This was a dry run. To apply changes, run without --dry-run flag.");
    } else {
        println!();
        println!("✅ Migration complete! Run validate-structure to verify.");
    }

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
