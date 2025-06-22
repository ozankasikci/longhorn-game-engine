// Test-Driven Development for Phase 19.6: Create Editor Framework
// 
// This test verifies that the editor framework functionality is properly extracted
// from engine-editor-egui into a dedicated engine-editor-framework crate.

use std::path::Path;

#[test]
fn test_framework_crate_exists() {
    // Test 1: Verify the new editor framework crate exists
    let framework_crate_path = Path::new("../engine-editor-framework/Cargo.toml");
    assert!(framework_crate_path.exists(), "engine-editor-framework crate should exist");
}

#[test]
fn test_framework_crate_structure() {
    // Test 2: Verify the crate has the expected structure
    let expected_files = vec![
        "../engine-editor-framework/src/lib.rs",
        "../engine-editor-framework/src/editor_state.rs",
        "../engine-editor-framework/src/editor_coordinator.rs",
        "../engine-editor-framework/src/play_state.rs",
        "../engine-editor-framework/src/bridge/mod.rs",
        "../engine-editor-framework/src/bridge/ecs_scene.rs",
        "../engine-editor-framework/src/world_setup.rs",
        "../engine-editor-framework/src/types.rs",
    ];
    
    for file in expected_files {
        let path = Path::new(file);
        assert!(path.exists(), "File {} should exist", file);
    }
}

#[test]
fn test_framework_files_removed_from_original() {
    // Test 3: Verify framework-related files have been removed/reduced from original
    let files_to_check = vec![
        ("src/editor_state.rs", 50),  // Should be minimal or removed
        ("src/editor_coordinator.rs", 20),  // Should be minimal or removed
        ("src/play_state.rs", 20),  // Should be minimal or removed
        ("src/bridge/mod.rs", 20),  // Should be minimal or removed
    ];
    
    for (file, max_lines) in files_to_check {
        let path = Path::new(file);
        if path.exists() {
            let contents = std::fs::read_to_string(path).unwrap();
            let line_count = contents.lines().count();
            assert!(line_count < max_lines, 
                "File {} should be removed or minimal (less than {} lines, found {})", 
                file, max_lines, line_count);
        }
    }
}

#[test]
fn test_framework_crate_dependencies() {
    // Test 4: Verify the framework crate has the correct dependencies
    let cargo_path = Path::new("../engine-editor-framework/Cargo.toml");
    if cargo_path.exists() {
        let contents = std::fs::read_to_string(cargo_path).unwrap();
        
        // Check for essential dependencies
        assert!(contents.contains("engine-ecs-core"), "Should depend on ECS core");
        assert!(contents.contains("engine-components-3d"), "Should depend on 3D components");
        assert!(contents.contains("engine-editor-assets"), "Should depend on asset crate");
        assert!(contents.contains("engine-editor-scene-view"), "Should depend on scene view");
    }
}

#[test]
fn test_editor_state_functionality() {
    // Test 5: Verify EditorState can be created and used
    use engine_editor_framework::{EditorState, SceneObject};
    
    let mut editor_state = EditorState::new();
    
    // Test object creation
    let obj_id = editor_state.create_object("TestObject", SceneObject::default());
    assert!(editor_state.get_object(obj_id).is_some());
    
    // Test object retrieval
    let obj = editor_state.get_object(obj_id).unwrap();
    assert_eq!(obj.name, "TestObject");
    
    // Test object deletion
    editor_state.delete_object(obj_id);
    assert!(editor_state.get_object(obj_id).is_none());
}

#[test]
fn test_editor_coordinator() {
    // Test 6: Verify EditorCoordinator manages play states correctly
    use engine_editor_framework::{EditorCoordinator, PlayState};
    
    let mut coordinator = EditorCoordinator::new();
    
    // Initially should be in editing mode
    assert_eq!(coordinator.get_play_state(), PlayState::Editing);
    
    // Test play state transitions
    coordinator.start_play();
    assert_eq!(coordinator.get_play_state(), PlayState::Playing);
    
    coordinator.pause_play();
    assert_eq!(coordinator.get_play_state(), PlayState::Paused);
    
    coordinator.resume_play();
    assert_eq!(coordinator.get_play_state(), PlayState::Playing);
    
    coordinator.stop_play();
    assert_eq!(coordinator.get_play_state(), PlayState::Editing);
}

#[test]
fn test_console_message_system() {
    // Test 7: Verify ConsoleMessage system works
    use engine_editor_framework::ConsoleMessage;
    
    let info_msg = ConsoleMessage::info("Test info");
    let warning_msg = ConsoleMessage::warning("Test warning");
    let error_msg = ConsoleMessage::error("Test error");
    
    // Test message creation
    match &info_msg {
        ConsoleMessage::Message { message, message_type, .. } => {
            assert_eq!(message, "Test info");
            assert!(matches!(message_type, engine_editor_framework::ConsoleMessageType::Info));
        }
        _ => panic!("Expected Message variant"),
    }
}

#[test]
fn test_world_setup_functionality() {
    // Test 8: Verify world setup creates default entities
    use engine_editor_framework::world_setup;
    use engine_ecs_core::World;
    
    let (world, cube_entity) = world_setup::create_default_world();
    
    // Verify world has entities
    assert!(world.entity_count() > 0);
    
    // Verify cube entity exists by checking if it has components
    assert!(world.get_component::<engine_components_ui::Name>(cube_entity).is_some());
    
    // Check for transform component
    let has_transform = world.get_component::<engine_components_3d::Transform>(cube_entity).is_some();
    assert!(has_transform, "Cube should have Transform component");
}

#[test]
fn test_bridge_system() {
    // Test 9: Verify bridge system exists and exports necessary types
    use engine_editor_framework::bridge::EcsSceneBridge;
    
    // This is a compile-time test - if it compiles, the type exists
    let _bridge: Option<EcsSceneBridge> = None;
}

#[test]
fn test_play_state_management() {
    // Test 10: Verify PlayState and timing functionality
    use engine_editor_framework::{PlayStateManager, PlayState};
    
    let mut play_state_manager = PlayStateManager::new();
    
    // Test initial state
    assert_eq!(play_state_manager.get_state(), PlayState::Editing);
    assert_eq!(play_state_manager.get_play_time(), 0.0);
    
    // Test state transitions
    play_state_manager.start();
    assert_eq!(play_state_manager.get_state(), PlayState::Playing);
    
    // Update time (simulate frame)
    play_state_manager.update_time(0.016); // 16ms frame
    assert!(play_state_manager.get_play_time() > 0.0);
}

#[test]
fn test_hierarchy_object_types() {
    // Test 11: Verify HierarchyObject and ObjectType are available
    use engine_editor_framework::{HierarchyObject, ObjectType};
    
    let camera_obj = HierarchyObject::new("Main Camera", ObjectType::Camera);
    assert_eq!(camera_obj.name, "Main Camera");
    assert!(matches!(camera_obj.object_type, ObjectType::Camera));
    
    let parent_obj = HierarchyObject::parent("Parent", vec![
        HierarchyObject::new("Child1", ObjectType::GameObject),
        HierarchyObject::new("Child2", ObjectType::GameObject),
    ]);
    
    assert!(parent_obj.children.is_some());
    assert_eq!(parent_obj.children.unwrap().len(), 2);
}

#[test]
fn test_editor_integration() {
    // Test 12: Verify that engine-editor-egui properly depends on engine-editor-framework
    let cargo_path = Path::new("Cargo.toml");
    let contents = std::fs::read_to_string(cargo_path).unwrap();
    
    assert!(contents.contains("engine-editor-framework"), 
        "engine-editor-egui should depend on engine-editor-framework");
}

#[test]
fn test_default_hierarchy_creation() {
    // Test 13: Verify default hierarchy creation
    use engine_editor_framework::world_setup;
    
    let hierarchy = world_setup::create_default_hierarchy();
    assert!(!hierarchy.is_empty(), "Should create default hierarchy");
    
    // Should have at least camera and cube
    let has_camera = hierarchy.iter().any(|obj| obj.name.contains("Camera"));
    let has_cube = hierarchy.iter().any(|obj| obj.name == "Cube");
    
    assert!(has_camera, "Should have a camera in hierarchy");
    assert!(has_cube, "Should have a cube in hierarchy");
}

#[test]
fn test_delta_time_tracking() {
    // Test 14: Verify delta time tracking in coordinator
    use engine_editor_framework::EditorCoordinator;
    
    let mut coordinator = EditorCoordinator::new();
    
    // Initial delta time should be 0
    assert_eq!(coordinator.get_delta_time(), 0.0);
    
    // Update delta time
    coordinator.update_delta_time();
    
    // Sleep a bit to ensure time passes
    std::thread::sleep(std::time::Duration::from_millis(10));
    coordinator.update_delta_time();
    
    // Delta time should now be positive
    assert!(coordinator.get_delta_time() > 0.0);
}