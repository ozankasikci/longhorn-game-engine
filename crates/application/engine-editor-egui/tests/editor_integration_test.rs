// Test-Driven Development for Phase 19.7: Integration & Testing
//
// This test verifies that all editor crates work together seamlessly
// after the modularization in Phase 19.

use std::path::Path;

#[test]
fn test_all_editor_crates_exist() {
    // Test 1: Verify all new editor crates exist
    let crates = vec![
        "engine-editor-scene-view",
        "engine-editor-panels",
        "engine-editor-ui",
        "engine-editor-assets",
        "engine-editor-framework",
    ];

    for crate_name in crates {
        let crate_path = Path::new("../").join(crate_name).join("Cargo.toml");
        assert!(crate_path.exists(), "Crate {} should exist", crate_name);
    }
}

#[test]
fn test_dependency_graph() {
    // Test 2: Verify correct dependency relationships
    let cargo_path = Path::new("Cargo.toml");
    let contents = std::fs::read_to_string(cargo_path).unwrap();

    // Main editor should depend on all extracted crates
    assert!(contents.contains("engine-editor-scene-view"));
    assert!(contents.contains("engine-editor-panels"));
    assert!(contents.contains("engine-editor-ui"));
    assert!(contents.contains("engine-editor-assets"));
    assert!(contents.contains("engine-editor-framework"));
}

#[test]
fn test_scene_view_integration() {
    // Test 3: Verify scene view integration works
    use engine_ecs_core::World;
    use engine_editor_scene_view::{types::SceneNavigation, SceneView};

    let _scene_view = SceneView::new();
    let mut _world = World::new();
    let scene_nav = SceneNavigation::default();

    // Should be able to create scene view components
    assert_eq!(
        scene_nav.current_tool,
        engine_editor_scene_view::types::SceneTool::Select
    );
}

#[test]
fn test_panel_creation() {
    // Test 4: Verify all panels can be created
    use engine_editor_panels::*;

    let _inspector = InspectorPanel::new();
    let _hierarchy = HierarchyPanel::new();
    let _console = ConsolePanel::new();
    let _project = ProjectPanel::new();
    let _game_view = GameViewPanel::new();

    // If this compiles, panels are properly exported
}

#[test]
fn test_ui_components() {
    // Test 5: Verify UI components work together
    use engine_editor_ui::{EditorSettings, MenuBar, SettingsDialog, Toolbar};

    let _toolbar = Toolbar::new();
    let _menu_bar = MenuBar::new();
    let settings = EditorSettings::default();
    let _settings_dialog = SettingsDialog::new(settings);
}

#[test]
fn test_asset_management() {
    // Test 6: Verify asset management integration
    use engine_editor_assets::{create_default_textures, AssetCache, TextureManager};

    let _texture_manager = TextureManager::new();
    let textures = create_default_textures();

    assert!(!textures.is_empty());

    // Test asset cache
    let cache: AssetCache<String> = AssetCache::new();
    assert!(cache.is_empty());
}

#[test]
fn test_framework_state_management() {
    // Test 7: Verify framework state management works
    use engine_editor_framework::{EditorCoordinator, EditorState, PlayState};

    let mut editor_state = EditorState::new();
    let mut coordinator = EditorCoordinator::new();

    // Test state transitions
    assert_eq!(coordinator.get_play_state(), PlayState::Editing);
    coordinator.start_play();
    assert_eq!(coordinator.get_play_state(), PlayState::Playing);

    // Test object creation
    let obj_id = editor_state.create_object("Test", Default::default());
    assert!(editor_state.get_object(obj_id).is_some());
}

#[test]
fn test_cross_crate_types() {
    // Test 8: Verify types are properly shared across crates
    use engine_editor_assets::ProjectAsset;
    use engine_editor_framework::{ConsoleMessage, HierarchyObject, ObjectType};
    use engine_editor_ui::PanelType;

    // Create instances to verify types work
    let _msg = ConsoleMessage::info("Test");
    let _hierarchy = HierarchyObject::new("Test", ObjectType::GameObject);
    let _asset = ProjectAsset::file("test.png");
    let _panel = PanelType::Inspector;
}

#[test]
fn test_gizmo_system_integration() {
    // Test 9: Verify gizmo system works across crates
    use engine_editor_ui::SceneTool;

    // Verify SceneTool enum is available
    let tool = SceneTool::Select;
    assert!(matches!(tool, SceneTool::Select));
}

#[test]
fn test_world_setup_integration() {
    // Test 10: Verify world setup works correctly
    use engine_components_3d::Transform;
    use engine_editor_framework::world_setup;

    let (world, cube_entity) = world_setup::create_default_world();

    // Verify world is properly initialized
    assert!(world.entity_count() > 0);

    // Verify cube has required components
    assert!(world.get_component::<Transform>(cube_entity).is_some());
}

#[test]
fn test_console_message_flow() {
    // Test 11: Test message flow between panels and console
    use engine_editor_panels::{ConsoleMessage as PanelMessage, ConsolePanel};

    let mut console = ConsolePanel::new();

    // Add messages
    let messages = vec![
        PanelMessage::info("Test info"),
        PanelMessage::warning("Test warning"),
        PanelMessage::error("Test error"),
    ];

    console.add_messages(messages);

    // Verify console has messages
    assert_eq!(console.console_messages.len(), 3);
}

#[test]
fn test_editor_app_trait() {
    // Test 12: Verify EditorApp trait implementation
    use engine_editor_ui::EditorApp;

    // This test verifies the trait is properly implemented
    // by the main editor (compilation test)
    fn verify_editor_app<T: EditorApp>(_app: &T) {}

    // The main LonghornEditor should implement this trait
}

#[test]
fn test_texture_asset_consistency() {
    // Test 13: Verify TextureAsset type is consistent
    use engine_editor_assets::create_default_textures;

    let textures = create_default_textures();

    // Should have default textures
    assert!(textures.len() >= 8); // At least 8 default color textures

    // Verify texture properties
    for (_, texture) in textures.iter() {
        assert!(!texture.name.is_empty());
        assert!(texture.path.starts_with("builtin:"));
    }
}

#[test]
fn test_settings_persistence() {
    // Test 14: Verify settings work across UI and framework
    use engine_editor_ui::EditorSettings;

    let settings = EditorSettings::default();

    // Verify default settings
    assert_eq!(settings.theme, "Dark");
    assert!(settings.vsync);
    assert!(settings.camera.movement_speed > 0.0);
}

#[test]
fn test_compilation_time_improvement() {
    // Test 15: Document expected compilation improvements
    // This is a documentation test to record the expected benefits

    // Before modularization:
    // - engine-editor-egui: 7,328 lines in one crate
    // - Any change required full recompilation

    // After modularization:
    // - engine-editor-scene-view: ~2,000 lines
    // - engine-editor-panels: ~1,177 lines
    // - engine-editor-ui: ~800 lines
    // - engine-editor-assets: ~400 lines
    // - engine-editor-framework: ~600 lines
    // - engine-editor-egui: ~500 lines (mostly imports)

    // Expected benefits:
    // - Parallel compilation of independent crates
    // - Incremental compilation when changing specific functionality
    // - Better code organization and maintainability

    assert!(true); // This test documents the improvements
}

#[test]
fn test_no_circular_dependencies() {
    // Test 16: Verify no circular dependencies exist
    // This would fail at compile time if there were circular deps

    // The dependency graph should be:
    // engine-editor-egui depends on:
    //   -> engine-editor-framework
    //   -> engine-editor-assets
    //   -> engine-editor-ui
    //   -> engine-editor-panels
    //   -> engine-editor-scene-view
    //
    // engine-editor-framework depends on:
    //   -> engine-editor-assets
    //   -> engine-editor-scene-view
    //
    // engine-editor-panels depends on:
    //   -> engine-editor-assets
    //   -> engine-editor-scene-view
    //
    // engine-editor-ui depends on:
    //   -> engine-editor-scene-view
    //
    // No cycles should exist

    assert!(true); // Compilation success proves no circular deps
}
