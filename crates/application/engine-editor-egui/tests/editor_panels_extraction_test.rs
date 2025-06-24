// Phase 19.4: Test-driven extraction of editor panels into separate crates
// This test verifies the extraction of panels, ui, and app layers

use std::path::Path;

#[test]
fn test_editor_panels_extraction_structure() {
    // Test 1: Verify panels have been extracted
    let panels_path = Path::new("../engine-editor-panels");
    assert!(
        panels_path.exists(),
        "engine-editor-panels crate should exist"
    );

    // Expected panel files in new crate
    let expected_panels = [
        "src/lib.rs",
        "src/inspector.rs",
        "src/hierarchy.rs",
        "src/console.rs",
        "src/project.rs",
        "src/game_view.rs",
    ];

    for file in &expected_panels {
        let file_path = panels_path.join(file);
        assert!(
            file_path.exists(),
            "Panel file {} should exist in new crate",
            file
        );
    }
}

#[test]
fn test_editor_ui_extraction_structure() {
    // Test 2: Verify UI components have been extracted
    let ui_path = Path::new("../engine-editor-ui");
    assert!(ui_path.exists(), "engine-editor-ui crate should exist");

    // Expected UI files
    let expected_ui_files = [
        "src/lib.rs",
        "src/toolbar.rs",
        "src/menu_bar.rs",
        "src/settings_dialog.rs",
        "src/tab_viewer.rs",
    ];

    for file in &expected_ui_files {
        let file_path = ui_path.join(file);
        assert!(
            file_path.exists(),
            "UI file {} should exist in new crate",
            file
        );
    }

    // Check styling module
    let styling_files = [
        "src/styling/mod.rs",
        "src/styling/theme.rs",
        "src/styling/colors.rs",
        "src/styling/fonts.rs",
        "src/styling/spacing.rs",
        "src/styling/widgets.rs",
    ];

    for file in &styling_files {
        let file_path = ui_path.join(file);
        assert!(file_path.exists(), "Styling file {} should exist", file);
    }
}

#[test]
fn test_editor_app_extraction_structure() {
    // Test 3: Verify app layer extraction
    let app_path = Path::new("../engine-editor-app");
    assert!(app_path.exists(), "engine-editor-app crate should exist");

    // Main app should be moved there
    let main_path = app_path.join("src/main.rs");
    assert!(
        main_path.exists(),
        "Main app file should be in engine-editor-app"
    );

    let world_setup_path = app_path.join("src/world_setup.rs");
    assert!(
        world_setup_path.exists(),
        "World setup should be in app crate"
    );
}

#[test]
fn test_original_files_removed() {
    // Test 4: Verify files have been removed from original location
    let original_panels = [
        "src/panels/inspector.rs",
        "src/panels/hierarchy.rs",
        "src/panels/console.rs",
        "src/panels/project.rs",
        "src/panels/game_view.rs",
    ];

    for file in &original_panels {
        let file_path = Path::new(file);
        assert!(
            !file_path.exists(),
            "Panel file {} should be removed from original location",
            file
        );
    }

    let original_ui = [
        "src/ui/toolbar.rs",
        "src/ui/menu_bar.rs",
        "src/ui/settings_dialog.rs",
        "src/ui/tab_viewer.rs",
    ];

    for file in &original_ui {
        let file_path = Path::new(file);
        assert!(
            !file_path.exists(),
            "UI file {} should be removed from original location",
            file
        );
    }
}

#[test]
fn test_crate_dependencies() {
    // Test 5: Verify new crates have proper dependencies
    use std::fs;

    // Check panels crate dependencies
    let panels_cargo = Path::new("../engine-editor-panels/Cargo.toml");
    if panels_cargo.exists() {
        let content = fs::read_to_string(panels_cargo).unwrap();
        assert!(
            content.contains("egui"),
            "Panels crate should depend on egui"
        );
        assert!(
            content.contains("engine-ecs-core"),
            "Panels crate should depend on ECS"
        );
        assert!(
            content.contains("engine-components-3d"),
            "Panels crate should depend on components"
        );
    }

    // Check UI crate dependencies
    let ui_cargo = Path::new("../engine-editor-ui/Cargo.toml");
    if ui_cargo.exists() {
        let content = fs::read_to_string(ui_cargo).unwrap();
        assert!(content.contains("egui"), "UI crate should depend on egui");
        assert!(
            content.contains("eframe"),
            "UI crate should depend on eframe"
        );
    }

    // Check app crate dependencies
    let app_cargo = Path::new("../engine-editor-app/Cargo.toml");
    if app_cargo.exists() {
        let content = fs::read_to_string(app_cargo).unwrap();
        assert!(
            content.contains("engine-editor-panels"),
            "App should depend on panels crate"
        );
        assert!(
            content.contains("engine-editor-ui"),
            "App should depend on UI crate"
        );
        assert!(
            content.contains("engine-editor-scene-view"),
            "App should depend on scene view"
        );
    }
}

#[test]
fn test_interface_definitions() {
    // Test 6: Verify proper interfaces are defined
    use std::fs;

    // Check panels lib.rs exports
    let panels_lib = Path::new("../engine-editor-panels/src/lib.rs");
    if panels_lib.exists() {
        let content = fs::read_to_string(panels_lib).unwrap();
        assert!(
            content.contains("pub trait Panel"),
            "Should define Panel trait"
        );
        assert!(
            content.contains("pub use inspector::InspectorPanel"),
            "Should export InspectorPanel"
        );
        assert!(
            content.contains("pub use hierarchy::HierarchyPanel"),
            "Should export HierarchyPanel"
        );
    }

    // Check UI lib.rs exports
    let ui_lib = Path::new("../engine-editor-ui/src/lib.rs");
    if ui_lib.exists() {
        let content = fs::read_to_string(ui_lib).unwrap();
        assert!(
            content.contains("pub use toolbar::Toolbar"),
            "Should export Toolbar"
        );
        assert!(
            content.contains("pub use menu_bar::MenuBar"),
            "Should export MenuBar"
        );
        assert!(
            content.contains("pub mod styling"),
            "Should export styling module"
        );
    }
}

#[test]
fn test_editor_still_builds() {
    // Test 7: The final test - does everything still compile?
    // This is a smoke test that will be run after extraction

    // Just verify the bin target exists for now
    let cargo_toml = Path::new("Cargo.toml");
    if cargo_toml.exists() {
        let content = std::fs::read_to_string(cargo_toml).unwrap();
        // After extraction, the binary should be in engine-editor-app
        if !content.contains("[[bin]]") {
            // Binary has been moved to app crate
            let app_cargo = Path::new("../engine-editor-app/Cargo.toml");
            if app_cargo.exists() {
                let app_content = std::fs::read_to_string(app_cargo).unwrap();
                assert!(
                    app_content.contains("[[bin]]"),
                    "Binary should be in app crate"
                );
                assert!(
                    app_content.contains("longhorn-editor"),
                    "Should still be named longhorn-editor"
                );
            }
        }
    }
}
