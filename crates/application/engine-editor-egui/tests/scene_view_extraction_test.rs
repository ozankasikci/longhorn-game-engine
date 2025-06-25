//! Phase 19.3: Test for extracting scene view from editor
//! TDD approach - write tests first, then implement

use std::path::Path;

#[test]
fn test_scene_view_exists() {
    // Post-extraction state: scene_view has been moved to engine-editor-scene-view crate
    let scene_view_crate = Path::new("../engine-editor-scene-view");
    assert!(scene_view_crate.exists(), "Scene view crate should exist");

    // Count files in scene view src
    let scene_view_src = scene_view_crate.join("src/scene_view");
    let file_count = std::fs::read_dir(scene_view_src)
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
    // Verify scene view crate is substantial
    let mut total_lines = 0;
    let scene_view_src = Path::new("../engine-editor-scene-view/src/scene_view");

    for entry in std::fs::read_dir(scene_view_src).unwrap() {
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
    // Check that scene view crate has proper dependencies
    let cargo_path = Path::new("../engine-editor-scene-view/Cargo.toml");
    let cargo_content = std::fs::read_to_string(cargo_path).unwrap();

    // Check for key dependencies
    assert!(cargo_content.contains("egui"), "Should depend on egui");
    assert!(
        cargo_content.contains("engine-ecs-core"),
        "Should depend on ECS"
    );
    assert!(
        cargo_content.contains("engine-renderer-3d"),
        "Should depend on renderer"
    );

    // Check key files exist in the extracted crate
    assert!(Path::new("../engine-editor-scene-view/src/scene_view/gizmo_3d_input.rs").exists());
    assert!(Path::new("../engine-editor-scene-view/src/scene_view/scene_view_impl.rs").exists());
    assert!(Path::new("../engine-editor-scene-view/src/scene_view/navigation.rs").exists());
}
