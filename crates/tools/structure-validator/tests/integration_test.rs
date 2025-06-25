//! Integration tests for structure validation and migration

use assert_fs::prelude::*;
use assert_fs::TempDir;
use structure_validator::{migration::StructureMigrator, CrateStructure, StructureValidator};

#[test]
fn test_full_migration_workflow() {
    let temp = TempDir::new().unwrap();

    // Create a crate with multiple issues (simulating a real crate)
    temp.child("Cargo.toml")
        .write_str(
            r#"
[package]
name = "engine-test-core"
version = "0.1.0"
edition = "2021"
license = "MIT"
    "#,
        )
        .unwrap();

    temp.child("src").create_dir_all().unwrap();
    temp.child("src/lib.rs").write_str("// Some code").unwrap();

    // Step 1: Validate and find issues
    let validator = StructureValidator::new(CrateStructure::default());
    let initial_report = validator.validate_crate(temp.path()).unwrap();
    assert!(!initial_report.is_valid());
    assert_eq!(initial_report.errors.len(), 3); // Missing README, description, and workspace inheritance

    // Step 2: Migrate
    let migrator = StructureMigrator::new(CrateStructure::default(), false);
    let migration_report = migrator.fix_crate(temp.path(), &initial_report).unwrap();
    assert_eq!(migration_report.actions.len(), 3);

    // Step 3: Validate again - should be fixed
    let final_report = validator.validate_crate(temp.path()).unwrap();
    assert!(final_report.is_valid());
    assert_eq!(final_report.errors.len(), 0);

    // Verify files exist
    assert!(temp.child("README.md").exists());

    // Verify Cargo.toml has description and workspace inheritance
    let cargo_content = std::fs::read_to_string(temp.child("Cargo.toml").path()).unwrap();
    assert!(cargo_content.contains("description"));
    assert!(cargo_content.contains("version.workspace = true"));
    assert!(cargo_content.contains("edition.workspace = true"));
    assert!(cargo_content.contains("license.workspace = true"));
}

#[test]
fn test_migration_preserves_existing_content() {
    let temp = TempDir::new().unwrap();

    // Create a crate with some existing content
    temp.child("Cargo.toml")
        .write_str(
            r#"
[package]
name = "engine-test-impl"
version = "0.1.0"
edition = "2021"
license = "MIT"

[dependencies]
serde = "1.0"
    "#,
        )
        .unwrap();

    temp.child("src").create_dir_all().unwrap();
    temp.child("src/lib.rs")
        .write_str("//! Important documentation\n\npub fn test() {}")
        .unwrap();

    // Validate and migrate
    let validator = StructureValidator::new(CrateStructure::default());
    let report = validator.validate_crate(temp.path()).unwrap();

    let migrator = StructureMigrator::new(CrateStructure::default(), false);
    migrator.fix_crate(temp.path(), &report).unwrap();

    // Check that existing content is preserved
    let cargo_content = std::fs::read_to_string(temp.child("Cargo.toml").path()).unwrap();
    assert!(cargo_content.contains("[dependencies]"));
    assert!(cargo_content.contains("serde = \"1.0\""));

    let lib_content = std::fs::read_to_string(temp.child("src/lib.rs").path()).unwrap();
    assert!(lib_content.contains("//! Important documentation"));
    assert!(lib_content.contains("pub fn test() {}"));
}
