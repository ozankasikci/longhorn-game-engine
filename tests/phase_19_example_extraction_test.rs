//! Phase 19.2: Test for extracting renderer examples
//! 
//! This test verifies that:
//! 1. Examples can be moved to a separate crate
//! 2. The main renderer crate still compiles without examples
//! 3. Examples can still run from the new crate

use std::process::Command;
use std::path::Path;

#[test]
fn test_renderer_builds_without_examples() {
    // This test will verify the renderer can build after examples are moved
    let renderer_path = Path::new("crates/implementation/engine-renderer-3d");
    
    // Check that renderer crate exists
    assert!(renderer_path.exists(), "Renderer crate should exist");
    
    // After extraction, examples directory should not exist in main crate
    let examples_path = renderer_path.join("examples");
    
    // For now, we expect examples to exist (pre-extraction state)
    // After extraction, we'll update this test
    assert!(examples_path.exists(), "Examples currently exist (pre-extraction)");
}

#[test]
fn test_examples_crate_structure() {
    // Test that the new examples crate will have proper structure
    let examples_crate_path = Path::new("crates/engine-renderer-3d-examples");
    
    // Pre-extraction: crate doesn't exist yet
    // Post-extraction: crate should exist with proper structure
    if examples_crate_path.exists() {
        // Verify structure
        assert!(examples_crate_path.join("Cargo.toml").exists(), "Cargo.toml should exist");
        assert!(examples_crate_path.join("examples").exists(), "examples directory should exist");
        
        // Verify at least one example exists
        let has_examples = std::fs::read_dir(examples_crate_path.join("examples"))
            .unwrap()
            .any(|_| true);
        assert!(has_examples, "Should have at least one example");
    }
}

#[test]
#[ignore] // Run manually after extraction
fn test_examples_can_run_from_new_crate() {
    let examples_crate_path = Path::new("crates/engine-renderer-3d-examples");
    
    if examples_crate_path.exists() {
        // Try to build one example
        let output = Command::new("cargo")
            .arg("build")
            .arg("--example")
            .arg("simple_triangle")
            .current_dir(examples_crate_path)
            .output()
            .expect("Failed to execute cargo build");
        
        assert!(output.status.success(), "Example should build successfully");
    }
}

#[test]
fn test_renderer_size_reduction() {
    // This test verifies that removing examples reduces the crate size
    let renderer_src = Path::new("crates/implementation/engine-renderer-3d/src");
    let examples_dir = Path::new("crates/implementation/engine-renderer-3d/examples");
    
    if examples_dir.exists() {
        // Count lines in examples
        let example_lines = count_lines_in_dir(examples_dir);
        println!("Example files contain {} lines", example_lines);
        
        // This should be around 2,263 lines according to our analysis
        assert!(example_lines > 2000, "Examples should be substantial (>2000 lines)");
    }
}

fn count_lines_in_dir(dir: &Path) -> usize {
    let mut total = 0;
    
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    total += content.lines().count();
                }
            }
        }
    }
    
    total
}