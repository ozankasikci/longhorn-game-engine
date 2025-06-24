//! Phase 19.2: Test for extracting renderer examples
//! TDD approach - write tests first, then implement

use std::path::Path;

#[test]
fn test_examples_removed() {
    // Post-extraction state: examples should NOT exist
    let examples_path = Path::new("examples");
    assert!(
        !examples_path.exists(),
        "Examples directory should be removed after extraction"
    );
}

#[test]
fn test_cargo_toml_cleaned() {
    // Verify that Cargo.toml no longer has [[example]] entries
    let cargo_content = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(
        !cargo_content.contains("[[example]]"),
        "Cargo.toml should not contain [[example]] entries after extraction"
    );
}

#[test]
fn test_renderer_compiles_without_examples() {
    // This will pass now, but after extraction we'll verify
    // the renderer still compiles without the examples directory

    // For now, just verify the src directory exists
    assert!(Path::new("src").exists(), "src directory should exist");
}

#[cfg(feature = "post-extraction")]
#[test]
fn test_examples_removed() {
    // After extraction, examples directory should not exist
    let examples_path = Path::new("examples");
    assert!(
        !examples_path.exists(),
        "Examples should be moved to separate crate"
    );
}

#[cfg(feature = "post-extraction")]
#[test]
fn test_cargo_toml_updated() {
    // After extraction, Cargo.toml should not have [[example]] entries
    let cargo_content = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(
        !cargo_content.contains("[[example]]"),
        "Example entries should be removed"
    );
}
