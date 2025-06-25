# Phase 25: Structure Standardization Plan

## Overview
Establish consistent file organization, module templates, and architecture validation tests across all crates to improve maintainability and developer experience.

## Goals
1. Define and enforce consistent file/module organization
2. Create reusable module templates
3. Implement architecture validation tests
4. Establish coding standards and conventions

## Current State Analysis

### Inconsistencies Found

1. **File Organization Patterns**:
   - Some crates: Everything in `lib.rs` (e.g., small core crates)
   - Some crates: Well-organized modules (e.g., `engine-renderer-3d`)
   - Mixed approaches within same tier

2. **Module Structure Variations**:
   ```rust
   // Pattern A: Flat re-exports
   pub use self::types::*;
   pub use self::traits::*;
   
   // Pattern B: Module hierarchy
   pub mod types;
   pub mod traits;
   
   // Pattern C: Mixed approach
   mod internal;
   pub use internal::SomeType;
   ```

3. **Test Organization**:
   - Unit tests: Sometimes in source, sometimes in `tests/` module
   - Integration tests: Inconsistent location and naming
   - Doc tests: Rarely used

4. **Documentation Standards**:
   - Some modules well-documented
   - Others have minimal docs
   - No consistent format

## Implementation Plan

### Step 1: Define Standard Structure (Week 1)

1. **Crate Organization Template**:
   ```
   crate-name/
   ├── Cargo.toml
   ├── README.md
   ├── src/
   │   ├── lib.rs          # Public API and re-exports only
   │   ├── types.rs        # Public type definitions
   │   ├── traits.rs       # Public trait definitions
   │   ├── error.rs        # Error types
   │   ├── internal/       # Private implementation
   │   │   ├── mod.rs
   │   │   └── ...
   │   └── implementation/ # Public implementations
   │       ├── mod.rs
   │       └── ...
   ├── tests/
   │   ├── integration/    # Integration tests
   │   │   └── ...
   │   └── common/        # Shared test utilities
   │       └── mod.rs
   ├── benches/           # Benchmarks
   │   └── ...
   └── examples/          # Usage examples
       └── ...
   ```

2. **Module Organization Rules**:
   - `lib.rs`: Only public API exports and crate documentation
   - One type/trait per file for large definitions
   - Group related small items in single file
   - Private modules in `internal/` directory
   - Implementation modules separate from trait definitions

3. **Naming Conventions**:
   ```rust
   // File names
   - types.rs for type definitions
   - traits.rs for trait definitions
   - error.rs for error types
   - utils.rs for utility functions
   
   // Module names
   - Singular for single-type modules (e.g., renderer.rs)
   - Plural for collections (e.g., components.rs)
   - Descriptive names for feature modules
   ```

### Step 2: Create Module Templates (Week 1-2)

1. **Core Crate Template**:
   ```rust
   // lib.rs template
   #![doc = include_str!("../README.md")]
   #![warn(missing_docs)]
   
   //! # Crate Name
   //! 
   //! Brief description of the crate's purpose.
   
   mod types;
   mod traits;
   mod error;
   
   pub use error::{Error, Result};
   pub use traits::*;
   pub use types::*;
   ```

2. **Type Definition Template**:
   ```rust
   // types.rs template
   use serde::{Deserialize, Serialize};
   
   /// Brief description of the type.
   /// 
   /// # Examples
   /// 
   /// ```
   /// use crate_name::TypeName;
   /// 
   /// let example = TypeName::new();
   /// ```
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct TypeName {
       // fields
   }
   
   impl TypeName {
       /// Creates a new instance.
       pub fn new() -> Self {
           Self {
               // initialization
           }
       }
   }
   
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_new() {
           let instance = TypeName::new();
           // assertions
       }
   }
   ```

3. **Implementation Module Template**:
   ```rust
   // implementation/feature.rs template
   use crate::{traits::SomeTrait, types::SomeType, Result};
   
   /// Implementation of SomeTrait for specific use case.
   pub struct FeatureImpl {
       // fields
   }
   
   impl FeatureImpl {
       /// Creates a new instance with the given configuration.
       pub fn new(config: Config) -> Result<Self> {
           Ok(Self {
               // initialization
           })
       }
   }
   
   impl SomeTrait for FeatureImpl {
       fn method(&self) -> Result<()> {
           // implementation
           Ok(())
       }
   }
   ```

### Step 3: Create Validation Tools (Week 2-3)

1. **Architecture Validation Tests**:
   ```rust
   // tests/architecture_validation.rs
   #[test]
   fn test_no_std_imports_in_core() {
       let core_files = glob("src/**/*.rs");
       for file in core_files {
           let content = read_file(file);
           assert!(!content.contains("use std::"), 
                   "Core crates should use core/alloc only");
       }
   }
   
   #[test]
   fn test_dependency_direction() {
       let cargo_toml = parse_cargo_toml();
       for dep in cargo_toml.dependencies {
           assert!(
               is_valid_dependency(current_tier(), dep.tier()),
               "Invalid dependency direction"
           );
       }
   }
   ```

2. **Structure Validation Script**:
   ```bash
   #!/bin/bash
   # validate_structure.sh
   
   check_crate_structure() {
       local crate=$1
       
       # Check required files exist
       [[ -f "$crate/Cargo.toml" ]] || echo "Missing Cargo.toml"
       [[ -f "$crate/README.md" ]] || echo "Missing README.md"
       [[ -f "$crate/src/lib.rs" ]] || echo "Missing lib.rs"
       
       # Check lib.rs is not too large
       lines=$(wc -l < "$crate/src/lib.rs")
       if [[ $lines -gt 100 ]]; then
           echo "lib.rs too large ($lines lines)"
       fi
   }
   ```

3. **Continuous Integration Checks**:
   ```yaml
   # .github/workflows/structure_check.yml
   name: Structure Validation
   
   on: [push, pull_request]
   
   jobs:
     validate:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v2
         
         - name: Check crate structure
           run: ./scripts/validate_structure.sh
         
         - name: Run architecture tests
           run: cargo test --test architecture_validation
         
         - name: Check documentation
           run: cargo doc --no-deps --all-features
   ```

### Step 4: Migration Strategy (Week 3-4)

1. **Tier-by-Tier Migration**:
   - Week 3: Migrate all core crates
   - Week 4: Migrate implementation crates
   - Week 5: Migrate integration/application crates

2. **Per-Crate Process**:
   ```
   1. Run structure analyzer
   2. Generate migration plan
   3. Reorganize files
   4. Update imports
   5. Run tests
   6. Update documentation
   ```

3. **Automated Migration Tool**:
   ```rust
   // tools/migrate_structure/src/main.rs
   fn migrate_crate(path: &Path) -> Result<()> {
       // 1. Analyze current structure
       let analysis = analyze_crate(path)?;
       
       // 2. Generate migration plan
       let plan = generate_plan(&analysis)?;
       
       // 3. Execute migration
       for action in plan.actions {
           match action {
               Action::MoveFile { from, to } => {
                   fs::rename(from, to)?;
               }
               Action::UpdateImports { file, changes } => {
                   update_imports(file, changes)?;
               }
           }
       }
       
       Ok(())
   }
   ```

### Step 5: Documentation Standards (Week 4-5)

1. **Crate-Level Documentation**:
   ```markdown
   # Crate Name
   
   Brief description (1-2 sentences).
   
   ## Overview
   
   Detailed description of the crate's purpose and design.
   
   ## Usage
   
   ```rust
   use crate_name::MainType;
   
   let instance = MainType::new();
   ```
   
   ## Features
   
   - Feature 1: Description
   - Feature 2: Description
   
   ## Architecture
   
   Describe key design decisions and patterns.
   ```

2. **API Documentation Template**:
   ```rust
   /// Brief description (one line).
   /// 
   /// Detailed description explaining the purpose and behavior.
   /// 
   /// # Arguments
   /// 
   /// * `param1` - Description of parameter
   /// * `param2` - Description of parameter
   /// 
   /// # Returns
   /// 
   /// Description of return value.
   /// 
   /// # Errors
   /// 
   /// * `ErrorKind::Variant` - When this error occurs
   /// 
   /// # Examples
   /// 
   /// ```
   /// # use crate_name::function;
   /// let result = function(arg1, arg2)?;
   /// assert_eq!(result, expected);
   /// ```
   /// 
   /// # Panics
   /// 
   /// Panics if invalid state is encountered.
   pub fn function(param1: Type1, param2: Type2) -> Result<ReturnType> {
       // implementation
   }
   ```

3. **Module Documentation**:
   ```rust
   //! # Module Name
   //! 
   //! This module provides functionality for...
   //! 
   //! ## Design
   //! 
   //! Explain the module's design and architecture.
   //! 
   //! ## Examples
   //! 
   //! ```
   //! use crate::module::{Type1, Type2};
   //! 
   //! // Example usage
   //! ```
   ```

### Step 6: Tooling and Automation (Week 5-6)

1. **Code Generation Tools**:
   ```bash
   # Create new crate with standard structure
   cargo make new-crate --name engine-new-system --tier core
   
   # Add new module to existing crate
   cargo make add-module --crate engine-ecs --name query --type implementation
   ```

2. **Linting Configuration**:
   ```toml
   # .clippy.toml
   warn-on-all-wildcard-imports = true
   max-single-char-names = 0
   ```

3. **Pre-commit Hooks**:
   ```bash
   #!/bin/bash
   # .git/hooks/pre-commit
   
   # Check structure
   ./scripts/validate_structure.sh
   
   # Run clippy
   cargo clippy --all -- -D warnings
   
   # Check documentation
   cargo doc --no-deps --all-features
   ```

## Success Criteria

1. **Consistency Achieved**:
   - All crates follow standard structure
   - Module organization is predictable
   - Navigation is intuitive

2. **Automation Working**:
   - CI validates structure
   - Migration tools functional
   - Code generation saves time

3. **Documentation Complete**:
   - All public APIs documented
   - Examples for major features
   - Architecture decisions recorded

4. **Developer Experience**:
   - Easy to find code
   - Clear where to add features
   - Consistent patterns throughout

## Maintenance Plan

1. **Regular Reviews**:
   - Monthly structure audits
   - Quarterly standard updates
   - Annual major revision

2. **Enforcement**:
   - CI/CD checks mandatory
   - PR reviews include structure
   - Automated fixes where possible

3. **Evolution**:
   - Gather developer feedback
   - Adapt to new Rust patterns
   - Keep standards current

## Benefits

1. **Improved Maintainability**:
   - Easier to navigate codebase
   - Consistent patterns reduce cognitive load
   - New developers onboard faster

2. **Better Quality**:
   - Automated validation catches issues
   - Consistent testing approach
   - Better documentation coverage

3. **Increased Productivity**:
   - Less time deciding structure
   - Code generation for boilerplate
   - Clear conventions guide decisions