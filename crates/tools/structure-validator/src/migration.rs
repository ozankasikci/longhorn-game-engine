//! Migration tool for fixing crate structure issues

use crate::{CrateStructure, ValidationError, ValidationReport};
use std::path::Path;
use anyhow::Result;

/// Migrator for fixing crate structure issues
pub struct StructureMigrator {
    expected_structure: CrateStructure,
    dry_run: bool,
}

impl StructureMigrator {
    pub fn new(expected_structure: CrateStructure, dry_run: bool) -> Self {
        Self {
            expected_structure,
            dry_run,
        }
    }
    
    /// Fix issues in a single crate based on validation report
    pub fn fix_crate(&self, crate_path: &Path, report: &ValidationReport) -> Result<MigrationReport> {
        let mut actions = Vec::new();
        
        for error in &report.errors {
            match error {
                ValidationError::MissingFile(file) => {
                    self.fix_missing_file(crate_path, file, &mut actions)?;
                }
                ValidationError::MissingDirectory(dir) => {
                    self.fix_missing_directory(crate_path, dir, &mut actions)?;
                }
                ValidationError::MissingCargoField(field) => {
                    self.fix_missing_cargo_field(crate_path, field, &mut actions)?;
                }
                ValidationError::InconsistentStructure(msg) if msg.contains("workspace inheritance") => {
                    self.fix_workspace_inheritance(crate_path, &mut actions)?;
                }
                _ => {
                    actions.push(MigrationAction::Skipped {
                        reason: format!("Cannot auto-fix: {}", error),
                    });
                }
            }
        }
        
        Ok(MigrationReport {
            crate_path: crate_path.to_path_buf(),
            actions,
        })
    }
    
    fn fix_missing_file(&self, crate_path: &Path, file: &str, actions: &mut Vec<MigrationAction>) -> Result<()> {
        let file_path = crate_path.join(file);
        
        let content = match file {
            "README.md" => self.generate_readme(crate_path)?,
            "src/lib.rs" => self.generate_lib_rs(crate_path)?,
            _ => return Ok(()),
        };
        
        if self.dry_run {
            actions.push(MigrationAction::WouldCreate {
                path: file_path,
                content,
            });
        } else {
            std::fs::write(&file_path, &content)?;
            actions.push(MigrationAction::Created {
                path: file_path,
                content,
            });
        }
        
        Ok(())
    }
    
    fn fix_missing_directory(&self, crate_path: &Path, dir: &str, actions: &mut Vec<MigrationAction>) -> Result<()> {
        let dir_path = crate_path.join(dir);
        
        if self.dry_run {
            actions.push(MigrationAction::WouldCreateDir {
                path: dir_path,
            });
        } else {
            std::fs::create_dir_all(&dir_path)?;
            actions.push(MigrationAction::CreatedDir {
                path: dir_path,
            });
        }
        
        Ok(())
    }
    
    fn fix_missing_cargo_field(&self, crate_path: &Path, field: &str, actions: &mut Vec<MigrationAction>) -> Result<()> {
        let cargo_path = crate_path.join("Cargo.toml");
        
        if field == "package.description" {
            let description = self.generate_description(crate_path)?;
            
            if self.dry_run {
                actions.push(MigrationAction::WouldUpdateCargo {
                    path: cargo_path,
                    field: field.to_string(),
                    value: description,
                });
            } else {
                self.add_cargo_field(&cargo_path, field, &description)?;
                actions.push(MigrationAction::UpdatedCargo {
                    path: cargo_path,
                    field: field.to_string(),
                    value: description,
                });
            }
        }
        
        Ok(())
    }
    
    fn fix_workspace_inheritance(&self, crate_path: &Path, actions: &mut Vec<MigrationAction>) -> Result<()> {
        let cargo_path = crate_path.join("Cargo.toml");
        
        if self.dry_run {
            actions.push(MigrationAction::WouldFixWorkspaceInheritance {
                path: cargo_path.clone(),
            });
        } else {
            self.convert_to_workspace_inheritance(&cargo_path)?;
            actions.push(MigrationAction::FixedWorkspaceInheritance {
                path: cargo_path,
            });
        }
        
        Ok(())
    }
    
    fn generate_readme(&self, crate_path: &Path) -> Result<String> {
        let crate_name = crate_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        let category = if crate_path.to_string_lossy().contains("/core/") {
            "Core"
        } else if crate_path.to_string_lossy().contains("/implementation/") {
            "Implementation"
        } else if crate_path.to_string_lossy().contains("/integration/") {
            "Integration"
        } else if crate_path.to_string_lossy().contains("/application/") {
            "Application"
        } else {
            "Unknown"
        };
        
        Ok(format!(
            r#"# {}

{} crate for the Longhorn Game Engine.

## Overview

TODO: Add overview of this crate's purpose and functionality.

## Features

TODO: List key features provided by this crate.

## Usage

```rust
// TODO: Add usage example
```

## Dependencies

See `Cargo.toml` for a full list of dependencies.

## License

This crate is part of the Longhorn Game Engine and is licensed under the same terms.
"#,
            crate_name,
            category
        ))
    }
    
    fn generate_lib_rs(&self, crate_path: &Path) -> Result<String> {
        let crate_name = crate_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        Ok(format!(
            r#"//! {}
//! 
//! TODO: Add crate-level documentation here.

// TODO: Add modules and exports
"#,
            crate_name.replace('-', " ").replace("engine ", "")
        ))
    }
    
    fn generate_description(&self, crate_path: &Path) -> Result<String> {
        // Try to get crate name from Cargo.toml first
        let cargo_path = crate_path.join("Cargo.toml");
        let crate_name = if cargo_path.exists() {
            let content = std::fs::read_to_string(&cargo_path)?;
            let cargo_toml: toml::Value = toml::from_str(&content)?;
            cargo_toml.get("package")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or_else(|| crate_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown"))
                .to_string()
        } else {
            crate_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        };
        
        // Generate description based on crate name
        let description = match crate_name.as_str() {
            name if name.contains("core") => format!("Core {} functionality for Longhorn Game Engine", 
                name.replace("engine-", "").replace("-core", "").replace('-', " ")),
            name if name.contains("impl") => format!("{} implementation for Longhorn Game Engine",
                name.replace("engine-", "").replace("-impl", "").replace('-', " ")),
            name if name.contains("import") => format!("{} functionality for Longhorn Game Engine",
                name.replace("engine-", "").replace('-', " ")),
            name if name.contains("bridge") => format!("{} for Longhorn Game Engine",
                name.replace("engine-", "").replace('-', " ")),
            _ => format!("{} for Longhorn Game Engine", 
                crate_name.replace("engine-", "").replace('-', " ")),
        };
        
        Ok(description)
    }
    
    fn add_cargo_field(&self, cargo_path: &Path, field: &str, value: &str) -> Result<()> {
        let content = std::fs::read_to_string(cargo_path)?;
        let mut doc = content.parse::<toml_edit::DocumentMut>()?;
        
        // Add the field
        if field == "package.description" {
            doc["package"]["description"] = toml_edit::value(value);
        }
        
        std::fs::write(cargo_path, doc.to_string())?;
        Ok(())
    }
    
    fn convert_to_workspace_inheritance(&self, cargo_path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(cargo_path)?;
        
        // Simple text replacement approach for workspace inheritance
        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let mut in_package = false;
        
        for i in 0..lines.len() {
            let line = &lines[i];
            
            if line.trim() == "[package]" {
                in_package = true;
            } else if line.trim().starts_with('[') {
                in_package = false;
            } else if in_package {
                if line.starts_with("version") && line.contains('=') && !line.contains("workspace") {
                    lines[i] = "version.workspace = true".to_string();
                } else if line.starts_with("edition") && line.contains('=') && !line.contains("workspace") {
                    lines[i] = "edition.workspace = true".to_string();
                } else if line.starts_with("license") && line.contains('=') && !line.contains("workspace") {
                    lines[i] = "license.workspace = true".to_string();
                }
            }
        }
        
        let new_content = lines.join("\n");
        std::fs::write(cargo_path, new_content)?;
        Ok(())
    }
}

/// Actions taken during migration
#[derive(Debug)]
pub enum MigrationAction {
    Created { path: std::path::PathBuf, content: String },
    WouldCreate { path: std::path::PathBuf, content: String },
    CreatedDir { path: std::path::PathBuf },
    WouldCreateDir { path: std::path::PathBuf },
    UpdatedCargo { path: std::path::PathBuf, field: String, value: String },
    WouldUpdateCargo { path: std::path::PathBuf, field: String, value: String },
    FixedWorkspaceInheritance { path: std::path::PathBuf },
    WouldFixWorkspaceInheritance { path: std::path::PathBuf },
    Skipped { reason: String },
}

/// Report of migration actions
#[derive(Debug)]
pub struct MigrationReport {
    pub crate_path: std::path::PathBuf,
    pub actions: Vec<MigrationAction>,
}

impl MigrationReport {
    pub fn print_summary(&self) {
        println!("📦 {}", self.crate_path.display());
        for action in &self.actions {
            match action {
                MigrationAction::Created { path, .. } => {
                    println!("   ✅ Created: {}", path.display());
                }
                MigrationAction::WouldCreate { path, .. } => {
                    println!("   🔄 Would create: {}", path.display());
                }
                MigrationAction::CreatedDir { path } => {
                    println!("   ✅ Created directory: {}", path.display());
                }
                MigrationAction::WouldCreateDir { path } => {
                    println!("   🔄 Would create directory: {}", path.display());
                }
                MigrationAction::UpdatedCargo { field, value, .. } => {
                    println!("   ✅ Added {}: {}", field, value);
                }
                MigrationAction::WouldUpdateCargo { field, value, .. } => {
                    println!("   🔄 Would add {}: {}", field, value);
                }
                MigrationAction::FixedWorkspaceInheritance { .. } => {
                    println!("   ✅ Fixed workspace inheritance");
                }
                MigrationAction::WouldFixWorkspaceInheritance { .. } => {
                    println!("   🔄 Would fix workspace inheritance");
                }
                MigrationAction::Skipped { reason } => {
                    println!("   ⏭️  Skipped: {}", reason);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{StructureValidator, CrateStructure};
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    
    #[test]
    fn test_fix_missing_readme() {
        let temp = TempDir::new().unwrap();
        
        // Create a crate without README
        temp.child("Cargo.toml").write_str(r#"
[package]
name = "test-crate"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "Test crate"
        "#).unwrap();
        
        temp.child("src").create_dir_all().unwrap();
        temp.child("src/lib.rs").write_str("//! Test").unwrap();
        
        // Validate to get errors
        let validator = StructureValidator::new(CrateStructure::default());
        let report = validator.validate_crate(temp.path()).unwrap();
        
        // Fix with migrator
        let migrator = StructureMigrator::new(CrateStructure::default(), false);
        let migration_report = migrator.fix_crate(temp.path(), &report).unwrap();
        
        // Check that README was created
        assert!(temp.child("README.md").exists());
        assert_eq!(migration_report.actions.len(), 2); // README + workspace inheritance
    }
    
    #[test]
    fn test_dry_run_mode() {
        let temp = TempDir::new().unwrap();
        
        // Create a crate without README
        temp.child("Cargo.toml").write_str(r#"
[package]
name = "test-crate"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "Test crate"
        "#).unwrap();
        
        temp.child("src").create_dir_all().unwrap();
        temp.child("src/lib.rs").write_str("//! Test").unwrap();
        
        // Validate to get errors
        let validator = StructureValidator::new(CrateStructure::default());
        let report = validator.validate_crate(temp.path()).unwrap();
        
        // Fix with migrator in dry-run mode
        let migrator = StructureMigrator::new(CrateStructure::default(), true);
        let migration_report = migrator.fix_crate(temp.path(), &report).unwrap();
        
        // Check that README was NOT created
        assert!(!temp.child("README.md").exists());
        
        // But action was recorded
        assert!(migration_report.actions.iter().any(|a| matches!(a, MigrationAction::WouldCreate { .. })));
    }
    
    #[test]
    fn test_fix_missing_description() {
        let temp = TempDir::new().unwrap();
        
        // Create a crate without description
        temp.child("Cargo.toml").write_str(r#"
[package]
name = "engine-test-core"
version.workspace = true
edition.workspace = true
license.workspace = true
        "#).unwrap();
        
        temp.child("README.md").write_str("# Test").unwrap();
        temp.child("src").create_dir_all().unwrap();
        temp.child("src/lib.rs").write_str("//! Test").unwrap();
        
        // Validate to get errors
        let validator = StructureValidator::new(CrateStructure::default());
        let report = validator.validate_crate(temp.path()).unwrap();
        
        // Fix with migrator
        let migrator = StructureMigrator::new(CrateStructure::default(), false);
        let migration_report = migrator.fix_crate(temp.path(), &report).unwrap();
        
        // Check that description was added
        let cargo_content = std::fs::read_to_string(temp.child("Cargo.toml").path()).unwrap();
        assert!(cargo_content.contains("description"));
        // Should have generated "Core test functionality for Longhorn Game Engine"
        assert!(cargo_content.contains("Core test functionality for Longhorn Game Engine"));
    }
}