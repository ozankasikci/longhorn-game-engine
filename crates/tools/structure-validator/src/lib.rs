//! Structure validation tool for ensuring consistent crate organization
//! 
//! This tool validates that all crates in the workspace follow a standardized structure.

pub mod migration;

use std::path::{Path, PathBuf};
use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Missing required file: {0}")]
    MissingFile(String),
    
    #[error("Missing required directory: {0}")]
    MissingDirectory(String),
    
    #[error("Invalid Cargo.toml: {0}")]
    InvalidCargoToml(String),
    
    #[error("Missing required field in Cargo.toml: {0}")]
    MissingCargoField(String),
    
    #[error("Inconsistent structure: {0}")]
    InconsistentStructure(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Represents the expected structure of a crate
#[derive(Debug, Clone)]
pub struct CrateStructure {
    /// Required files at the crate root
    pub required_files: Vec<String>,
    
    /// Required directories
    pub required_directories: Vec<String>,
    
    /// Required fields in Cargo.toml
    pub required_cargo_fields: Vec<String>,
    
    /// Whether this crate should have integration tests
    pub requires_integration_tests: bool,
    
    /// Whether this crate should have benchmarks
    pub requires_benchmarks: bool,
    
    /// Whether this crate should use workspace inheritance
    pub requires_workspace_inheritance: bool,
}

impl Default for CrateStructure {
    fn default() -> Self {
        Self {
            required_files: vec![
                "Cargo.toml".to_string(),
                "README.md".to_string(),
                "src/lib.rs".to_string(),
            ],
            required_directories: vec![
                "src".to_string(),
            ],
            required_cargo_fields: vec![
                "package.name".to_string(),
                "package.version".to_string(),
                "package.edition".to_string(),
                "package.license".to_string(),
                "package.description".to_string(),
            ],
            requires_integration_tests: false,
            requires_benchmarks: false,
            requires_workspace_inheritance: true,
        }
    }
}

/// Validator for crate structure
pub struct StructureValidator {
    expected_structure: CrateStructure,
}

impl StructureValidator {
    pub fn new(expected_structure: CrateStructure) -> Self {
        Self { expected_structure }
    }
    
    /// Validate a single crate
    pub fn validate_crate(&self, crate_path: &Path) -> Result<ValidationReport> {
        let mut errors = Vec::new();
        
        // Check required files
        for file in &self.expected_structure.required_files {
            let file_path = crate_path.join(file);
            if !file_path.exists() {
                errors.push(ValidationError::MissingFile(file.clone()));
            }
        }
        
        // Check required directories
        for dir in &self.expected_structure.required_directories {
            let dir_path = crate_path.join(dir);
            if !dir_path.exists() || !dir_path.is_dir() {
                errors.push(ValidationError::MissingDirectory(dir.clone()));
            }
        }
        
        // Check Cargo.toml
        let cargo_path = crate_path.join("Cargo.toml");
        if cargo_path.exists() {
            self.validate_cargo_toml(&cargo_path, &mut errors)?;
        }
        
        // Check for tests directory if required
        if self.expected_structure.requires_integration_tests {
            let tests_path = crate_path.join("tests");
            if !tests_path.exists() || !tests_path.is_dir() {
                errors.push(ValidationError::MissingDirectory("tests".to_string()));
            }
        }
        
        // Check for benches directory if required
        if self.expected_structure.requires_benchmarks {
            let benches_path = crate_path.join("benches");
            if !benches_path.exists() || !benches_path.is_dir() {
                errors.push(ValidationError::MissingDirectory("benches".to_string()));
            }
        }
        
        Ok(ValidationReport {
            crate_path: crate_path.to_path_buf(),
            errors,
        })
    }
    
    /// Validate Cargo.toml structure
    fn validate_cargo_toml(&self, cargo_path: &Path, errors: &mut Vec<ValidationError>) -> Result<()> {
        let content = std::fs::read_to_string(cargo_path)?;
        let cargo_toml: toml::Value = toml::from_str(&content)
            .map_err(|e| ValidationError::InvalidCargoToml(e.to_string()))?;
        
        // Check required fields
        for field in &self.expected_structure.required_cargo_fields {
            if !has_field(&cargo_toml, field) {
                errors.push(ValidationError::MissingCargoField(field.clone()));
            }
        }
        
        // Check workspace inheritance
        if self.expected_structure.requires_workspace_inheritance {
            if !uses_workspace_inheritance(&cargo_toml) {
                errors.push(ValidationError::InconsistentStructure(
                    "Not using workspace inheritance for version/edition/license".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate all crates in a workspace
    pub fn validate_workspace(&self, workspace_root: &Path) -> Result<WorkspaceValidationReport> {
        let mut reports = Vec::new();
        
        // Find all crate directories
        let crate_dirs = find_crate_directories(workspace_root)?;
        
        for crate_dir in crate_dirs {
            let report = self.validate_crate(&crate_dir)?;
            reports.push(report);
        }
        
        Ok(WorkspaceValidationReport { reports })
    }
}

/// Report for a single crate validation
#[derive(Debug)]
pub struct ValidationReport {
    pub crate_path: PathBuf,
    pub errors: Vec<ValidationError>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Report for entire workspace validation
#[derive(Debug)]
pub struct WorkspaceValidationReport {
    pub reports: Vec<ValidationReport>,
}

impl WorkspaceValidationReport {
    pub fn is_valid(&self) -> bool {
        self.reports.iter().all(|r| r.is_valid())
    }
    
    pub fn total_errors(&self) -> usize {
        self.reports.iter().map(|r| r.errors.len()).sum()
    }
    
    pub fn print_summary(&self) {
        println!("Workspace Structure Validation Report");
        println!("=====================================");
        println!("Total crates: {}", self.reports.len());
        println!("Valid crates: {}", self.reports.iter().filter(|r| r.is_valid()).count());
        println!("Total errors: {}", self.total_errors());
        println!();
        
        for report in &self.reports {
            if !report.is_valid() {
                println!("❌ {}", report.crate_path.display());
                for error in &report.errors {
                    println!("   - {}", error);
                }
                println!();
            }
        }
    }
}

/// Helper function to check if a field exists in TOML
fn has_field(toml: &toml::Value, field_path: &str) -> bool {
    let parts: Vec<&str> = field_path.split('.').collect();
    let mut current = toml;
    
    for part in parts {
        match current.get(part) {
            Some(value) => current = value,
            None => return false,
        }
    }
    
    true
}

/// Helper function to check if using workspace inheritance
fn uses_workspace_inheritance(toml: &toml::Value) -> bool {
    if let Some(package) = toml.get("package") {
        if let Some(version) = package.get("version") {
            if let toml::Value::Table(table) = version {
                return table.contains_key("workspace");
            }
        }
    }
    false
}

/// Find all crate directories in the workspace
fn find_crate_directories(workspace_root: &Path) -> Result<Vec<PathBuf>> {
    let mut crate_dirs = Vec::new();
    
    // Standard crate locations
    let search_dirs = vec![
        "crates/core",
        "crates/implementation",
        "crates/integration",
        "crates/application",
    ];
    
    for search_dir in search_dirs {
        let dir_path = workspace_root.join(search_dir);
        if dir_path.exists() && dir_path.is_dir() {
            for entry in std::fs::read_dir(dir_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() && path.join("Cargo.toml").exists() {
                    crate_dirs.push(path);
                }
            }
        }
    }
    
    Ok(crate_dirs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    
    #[test]
    fn test_default_structure() {
        let structure = CrateStructure::default();
        assert_eq!(structure.required_files.len(), 3);
        assert!(structure.required_files.contains(&"Cargo.toml".to_string()));
        assert!(structure.required_files.contains(&"README.md".to_string()));
        assert!(structure.required_files.contains(&"src/lib.rs".to_string()));
    }
    
    #[test]
    fn test_validate_valid_crate() {
        let temp = TempDir::new().unwrap();
        
        // Create a valid crate structure
        temp.child("Cargo.toml").write_str(r#"
[package]
name = "test-crate"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "A test crate"
        "#).unwrap();
        
        temp.child("README.md").write_str("# Test Crate").unwrap();
        temp.child("src").create_dir_all().unwrap();
        temp.child("src/lib.rs").write_str("//! Test crate").unwrap();
        
        let validator = StructureValidator::new(CrateStructure::default());
        let report = validator.validate_crate(temp.path()).unwrap();
        
        assert!(report.is_valid());
        assert_eq!(report.errors.len(), 0);
    }
    
    #[test]
    fn test_validate_missing_readme() {
        let temp = TempDir::new().unwrap();
        
        // Create crate without README
        temp.child("Cargo.toml").write_str(r#"
[package]
name = "test-crate"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "A test crate"
        "#).unwrap();
        
        temp.child("src").create_dir_all().unwrap();
        temp.child("src/lib.rs").write_str("//! Test crate").unwrap();
        
        let validator = StructureValidator::new(CrateStructure::default());
        let report = validator.validate_crate(temp.path()).unwrap();
        
        assert!(!report.is_valid());
        assert_eq!(report.errors.len(), 2); // Missing README and not using workspace inheritance
    }
    
    #[test]
    fn test_validate_missing_description() {
        let temp = TempDir::new().unwrap();
        
        // Create crate without description
        temp.child("Cargo.toml").write_str(r#"
[package]
name = "test-crate"
version.workspace = true
edition.workspace = true
license.workspace = true
        "#).unwrap();
        
        temp.child("README.md").write_str("# Test Crate").unwrap();
        temp.child("src").create_dir_all().unwrap();
        temp.child("src/lib.rs").write_str("//! Test crate").unwrap();
        
        let validator = StructureValidator::new(CrateStructure::default());
        let report = validator.validate_crate(temp.path()).unwrap();
        
        assert!(!report.is_valid());
        assert_eq!(report.errors.len(), 1); // Missing description
    }
    
    #[test]
    fn test_has_field() {
        let toml_str = r#"
[package]
name = "test"
version = "1.0"

[dependencies]
serde = "1.0"
        "#;
        
        let toml: toml::Value = toml::from_str(toml_str).unwrap();
        
        assert!(has_field(&toml, "package.name"));
        assert!(has_field(&toml, "package.version"));
        assert!(has_field(&toml, "dependencies.serde"));
        assert!(!has_field(&toml, "package.description"));
        assert!(!has_field(&toml, "dev-dependencies"));
    }
    
    #[test]
    fn test_workspace_inheritance_check() {
        let toml_with_inheritance = r#"
[package]
name = "test"
version.workspace = true
edition.workspace = true
        "#;
        
        let toml_without_inheritance = r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
        "#;
        
        let toml1: toml::Value = toml::from_str(toml_with_inheritance).unwrap();
        let toml2: toml::Value = toml::from_str(toml_without_inheritance).unwrap();
        
        assert!(uses_workspace_inheritance(&toml1));
        assert!(!uses_workspace_inheritance(&toml2));
    }
}