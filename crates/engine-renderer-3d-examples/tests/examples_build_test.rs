//! Test that examples can build in the new crate

use std::process::Command;

#[test]
fn test_simple_triangle_example_builds() {
    let output = Command::new("cargo")
        .arg("build")
        .arg("--example")
        .arg("simple_triangle")
        .output()
        .expect("Failed to execute cargo build");

    assert!(
        output.status.success(),
        "simple_triangle example should build successfully. Error: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_all_examples_listed() {
    // Verify that cargo recognizes all examples
    let output = Command::new("cargo")
        .arg("build")
        .arg("--examples") // Build all examples
        .output()
        .expect("Failed to execute cargo build");

    // This will build all examples, ensuring they compile
    if !output.status.success() {
        eprintln!(
            "Error building examples: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    assert!(
        output.status.success(),
        "All examples should build successfully"
    );
}

#[test]
fn test_example_count() {
    let examples_dir = std::path::Path::new("examples");
    assert!(examples_dir.exists(), "examples directory should exist");

    let example_count = std::fs::read_dir(examples_dir)
        .unwrap()
        .filter(|entry| {
            entry
                .as_ref()
                .unwrap()
                .path()
                .extension()
                .and_then(|s| s.to_str())
                == Some("rs")
        })
        .count();

    assert_eq!(example_count, 9, "Should have exactly 9 examples");
}
