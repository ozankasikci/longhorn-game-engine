//! Phase 19.3: Test for extracting scene view from editor
//! TDD approach - write tests first, then implement

use std::path::Path;

#[test]
fn test_scene_view_exists() {
    // Pre-extraction state: scene_view module should exist
    let scene_view_path = Path::new("src/panels/scene_view");
    assert!(scene_view_path.exists(), "Scene view module should exist");

    // Count files in scene view
    let file_count = std::fs::read_dir(scene_view_path)
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

    println!("Found {} files in scene_view module", file_count);
    assert!(
        file_count >= 10,
        "Should have at least 10 files in scene view"
    );
}

#[test]
fn test_scene_view_line_count() {
    // Verify scene view is substantial (should be ~2,319 lines)
    let mut total_lines = 0;
    let scene_view_path = Path::new("src/panels/scene_view");

    for entry in std::fs::read_dir(scene_view_path).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let content = std::fs::read_to_string(&path).unwrap();
            let lines = content.lines().count();
            total_lines += lines;
            println!(
                "{}: {} lines",
                path.file_name().unwrap().to_string_lossy(),
                lines
            );
        }
    }

    println!("Total lines in scene_view: {}", total_lines);
    assert!(total_lines > 2000, "Scene view should be >2000 lines");
}

#[test]
fn test_scene_view_dependencies() {
    // Analyze what scene_view depends on from parent crate
    let scene_view_mod = std::fs::read_to_string("src/panels/scene_view/mod.rs").unwrap();

    // Check for parent imports
    let has_parent_imports =
        scene_view_mod.contains("use super::") || scene_view_mod.contains("use crate::");

    println!("Scene view has parent imports: {}", has_parent_imports);

    // Check key files exist
    assert!(Path::new("src/panels/scene_view/gizmo_3d_input.rs").exists());
    assert!(Path::new("src/panels/scene_view/scene_view_impl.rs").exists());
    assert!(Path::new("src/panels/scene_view/navigation.rs").exists());
}
