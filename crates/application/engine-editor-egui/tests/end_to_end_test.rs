// End-to-end functionality tests for the modularized editor
// Tests complete workflows across all crates

#[test]
fn test_complete_object_creation_workflow() {
    // Test creating an object through the full editor stack
    use engine_components_3d::Transform;
    use engine_editor_framework::{world_setup, EditorState};

    // 1. Create world and editor state
    let (mut world, _) = world_setup::create_default_world();
    let mut editor_state = EditorState::new();

    // 2. Create a new object in editor state
    let obj_id = editor_state.create_object("TestCube", Default::default());

    // 3. Create corresponding entity in ECS world
    let entity = world.spawn();
    world.add_component(entity, Transform::default()).unwrap();
    world
        .add_component(entity, engine_components_ui::Name::new("TestCube"))
        .unwrap();

    // 4. Verify object exists in both systems
    assert!(editor_state.get_object(obj_id).is_some());
    assert!(world.get_component::<Transform>(entity).is_some());
}

#[test]
fn test_complete_asset_loading_workflow() {
    // Test loading and using assets across crates
    use engine_editor_assets::{create_default_textures, TextureManager};
    use engine_editor_panels::ProjectPanel;

    // 1. Create texture manager and load default textures
    let mut texture_manager = TextureManager::new();
    let default_textures = create_default_textures();

    // 2. Register textures
    for (_id, texture) in default_textures.iter() {
        texture_manager.register_texture(
            texture.name.clone(),
            texture.id,
            texture.size,
            texture.path.clone(),
        );
    }

    // 3. Create project panel that would display these assets
    let _project_panel = ProjectPanel::new();
    let project_assets = engine_editor_assets::create_default_project_assets();

    // 4. Verify assets are available
    assert!(!project_assets.is_empty());
    assert!(texture_manager.all_handles().len() > 0);
}

#[test]
fn test_play_mode_transition_workflow() {
    // Test transitioning between edit and play modes
    use engine_editor_framework::{EditorCoordinator, PlayState};
    use engine_editor_ui::Toolbar;

    let mut coordinator = EditorCoordinator::new();
    let _toolbar = Toolbar::new();

    // 1. Start in editing mode
    assert_eq!(coordinator.get_play_state(), PlayState::Editing);

    // 2. Start play mode
    coordinator.start_play();
    assert_eq!(coordinator.get_play_state(), PlayState::Playing);

    // 3. Pause
    coordinator.pause_play();
    assert_eq!(coordinator.get_play_state(), PlayState::Paused);

    // 4. Resume
    coordinator.resume_play();
    assert_eq!(coordinator.get_play_state(), PlayState::Playing);

    // 5. Stop and return to editing
    coordinator.stop_play();
    assert_eq!(coordinator.get_play_state(), PlayState::Editing);
}

#[test]
fn test_scene_navigation_workflow() {
    // Test scene navigation across view and input systems
    use engine_editor_scene_view::types::{SceneNavigation, SceneTool};

    let mut scene_nav = SceneNavigation::default();

    // 1. Test tool switching
    assert_eq!(scene_nav.current_tool, SceneTool::Select);

    scene_nav.current_tool = SceneTool::Move;
    assert_eq!(scene_nav.current_tool, SceneTool::Move);

    scene_nav.current_tool = SceneTool::Rotate;
    assert_eq!(scene_nav.current_tool, SceneTool::Rotate);

    scene_nav.current_tool = SceneTool::Scale;
    assert_eq!(scene_nav.current_tool, SceneTool::Scale);
}

#[test]
fn test_console_message_workflow() {
    // Test console message flow from various sources
    use engine_editor_panels::{ConsoleMessage as PanelMsg, ConsolePanel};

    let mut console = ConsolePanel::new();

    // 1. Add messages from different sources
    console.add_messages(vec![
        PanelMsg::info("System initialized"),
        PanelMsg::warning("Low memory"),
        PanelMsg::error("Failed to load asset"),
    ]);

    // 2. Verify messages are stored
    assert_eq!(console.console_messages.len(), 3);

    // 3. Clear console (manually since no clear method)
    console.console_messages.clear();
    assert_eq!(console.console_messages.len(), 0);
}

#[test]
fn test_settings_update_workflow() {
    // Test settings changes propagating through the system
    use engine_editor_ui::{EditorSettings, SettingsDialog};

    let mut settings = EditorSettings::default();
    let mut dialog = SettingsDialog::new(settings.clone());

    // 1. Modify settings
    dialog.settings.camera.movement_speed = 10.0;
    dialog.settings.vsync = false;
    dialog.settings.theme = "Light".to_string();

    // 2. Apply settings (in real app, this would update scene navigation)
    settings = dialog.settings.clone();

    // 3. Verify changes
    assert_eq!(settings.camera.movement_speed, 10.0);
    assert!(!settings.vsync);
    assert_eq!(settings.theme, "Light");
}

#[test]
fn test_hierarchy_selection_workflow() {
    // Test selecting objects in hierarchy and updating inspector
    use engine_editor_framework::EditorState;
    use engine_editor_panels::{HierarchyPanel, InspectorPanel};

    let mut editor_state = EditorState::new();
    let _hierarchy = HierarchyPanel::new();
    let _inspector = InspectorPanel::new();

    // 1. Create some objects
    let obj1 = editor_state.create_object("Object1", Default::default());
    let obj2 = editor_state.create_object("Object2", Default::default());

    // 2. Select first object
    editor_state.select_object(obj1);
    assert_eq!(editor_state.selected_object, Some(obj1));

    // 3. Select second object
    editor_state.select_object(obj2);
    assert_eq!(editor_state.selected_object, Some(obj2));

    // 4. Deselect by deleting
    editor_state.delete_object(obj2);
    assert_eq!(editor_state.selected_object, None);
}

#[test]
fn test_multi_panel_coordination() {
    // Test that multiple panels can work together
    use engine_ecs_core::World;
    use engine_editor_framework::EditorState;
    use engine_editor_panels::*;

    let _world = World::new();
    let mut editor_state = EditorState::new();

    // Create all panels
    let _inspector = InspectorPanel::new();
    let _hierarchy = HierarchyPanel::new();
    let mut console = ConsolePanel::new();
    let _project = ProjectPanel::new();
    let _game_view = GameViewPanel::new();

    // Simulate workflow
    // 1. Log message when creating object
    console.add_messages(vec![ConsoleMessage::info("Creating new object")]);

    // 2. Create object
    let obj_id = editor_state.create_object("TestObject", Default::default());

    // 3. Select it (hierarchy would handle this)
    editor_state.select_object(obj_id);

    // 4. Log selection
    console.add_messages(vec![ConsoleMessage::info("Selected: TestObject")]);

    // Verify state
    assert!(editor_state.selected_object.is_some());
    assert_eq!(console.console_messages.len(), 2);
}

#[test]
fn test_error_handling_workflow() {
    // Test error handling and reporting across crates
    use engine_editor_assets::AssetLoadError;
    use engine_editor_panels::{ConsoleMessage, ConsolePanel};

    let mut console = ConsolePanel::new();

    // Simulate asset loading error
    let error = AssetLoadError::NotFound("missing.png".to_string());

    // Log error to console
    console.add_messages(vec![ConsoleMessage::error(&format!(
        "Asset load failed: {}",
        error
    ))]);

    // Verify error was logged
    assert_eq!(console.console_messages.len(), 1);
    match &console.console_messages[0] {
        ConsoleMessage::Message { message_type, .. } => {
            assert!(matches!(
                message_type,
                engine_editor_panels::ConsoleMessageType::Error
            ));
        }
        _ => panic!("Expected error message"),
    }
}
