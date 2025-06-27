//! TDD Tests for TypeScript component creation issue
//! This file contains tests that define the expected behavior for TypeScript component creation.

use crate::{InspectorPanel, ScriptTemplate, ScriptLanguage};
use engine_ecs_core::{Entity, World};
use engine_scripting::components::TypeScriptScript;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test helper to create a test inspector panel
    fn create_test_inspector() -> InspectorPanel {
        InspectorPanel::new()
    }

    /// Test helper to create a test world with an entity
    fn create_test_world_with_entity() -> (World, Entity) {
        let mut world = World::new();
        // Register TypeScriptScript component
        engine_ecs_core::register_component::<TypeScriptScript>();
        let entity = world.spawn();
        (world, entity)
    }

    #[test]
    fn test_typescript_script_component_creation() {
        let mut inspector = create_test_inspector();
        let (mut world, entity) = create_test_world_with_entity();

        // Setup script creation state
        inspector.script_creation_name = "test_script".to_string();
        inspector.script_creation_template = ScriptTemplate::Entity;
        inspector.script_creation_language = ScriptLanguage::TypeScript;

        // Should be able to attach TypeScript script to entity
        let result = inspector.attach_script_to_entity(&mut world, entity, "assets/scripts/test_script.ts");
        
        assert!(result.is_ok(), "Failed to attach TypeScript script: {:?}", result);
        
        // Entity should now have TypeScriptScript component
        let typescript_script = world.get_component::<TypeScriptScript>(entity);
        assert!(typescript_script.is_some(), "Entity should have TypeScriptScript component");
        
        let script = typescript_script.unwrap();
        assert_eq!(script.get_path(), "assets/scripts/test_script.ts");
        assert!(script.is_enabled());
    }

    #[test]
    fn test_typescript_script_file_creation() {
        let mut inspector = create_test_inspector();
        let (mut world, entity) = create_test_world_with_entity();

        inspector.script_creation_name = "player_controller".to_string();
        inspector.script_creation_template = ScriptTemplate::Entity;
        inspector.script_creation_language = ScriptLanguage::TypeScript;

        // Should be able to create TypeScript script file
        let result = inspector.create_script_file(&mut world, entity);
        assert!(result.is_ok(), "Failed to create TypeScript script file: {:?}", result);

        // File should exist at expected location
        let script_path = std::path::Path::new("assets/scripts/player_controller.ts");
        assert!(script_path.exists(), "TypeScript script file should be created");

        // File should contain TypeScript template content
        let content = std::fs::read_to_string(script_path).expect("Failed to read script file");
        assert!(content.contains("export class PlayerController"));
        assert!(content.contains("init(): void"));
        assert!(content.contains("update(deltaTime: number): void"));
        assert!(content.contains("destroy(): void"));

        // Clean up
        let _ = std::fs::remove_file(script_path);
    }

    #[test]
    fn test_typescript_behavior_template() {
        let mut inspector = create_test_inspector();
        let (mut world, entity) = create_test_world_with_entity();

        inspector.script_creation_name = "jump_behavior".to_string();
        inspector.script_creation_template = ScriptTemplate::Behavior;
        inspector.script_creation_language = ScriptLanguage::TypeScript;

        let result = inspector.create_script_file(&mut world, entity);
        assert!(result.is_ok());

        let script_path = std::path::Path::new("assets/scripts/jump_behavior.ts");
        assert!(script_path.exists());

        let content = std::fs::read_to_string(script_path).expect("Failed to read script file");
        assert!(content.contains("export class JumpBehavior"));
        assert!(content.contains("implements Behavior"));
        assert!(content.contains("start(entity: Entity): void"));

        // Clean up
        let _ = std::fs::remove_file(script_path);
    }

    #[test]
    fn test_typescript_system_template() {
        let mut inspector = create_test_inspector();
        let (mut world, entity) = create_test_world_with_entity();

        inspector.script_creation_name = "physics_system".to_string();
        inspector.script_creation_template = ScriptTemplate::System;
        inspector.script_creation_language = ScriptLanguage::TypeScript;

        let result = inspector.create_script_file(&mut world, entity);
        assert!(result.is_ok());

        let script_path = std::path::Path::new("assets/scripts/physics_system.ts");
        assert!(script_path.exists());

        let content = std::fs::read_to_string(script_path).expect("Failed to read script file");
        assert!(content.contains("export class PhysicsSystem"));
        assert!(content.contains("implements System"));
        assert!(content.contains("update(world: World, deltaTime: number): void"));

        // Clean up
        let _ = std::fs::remove_file(script_path);
    }

    #[test]
    fn test_typescript_script_name_validation() {
        let inspector = create_test_inspector();

        // Valid names
        assert!(inspector.is_valid_script_name("player_controller"));
        assert!(inspector.is_valid_script_name("test123"));
        assert!(inspector.is_valid_script_name("simple"));

        // Invalid names
        assert!(!inspector.is_valid_script_name(""));
        assert!(!inspector.is_valid_script_name("player-controller")); // dash not allowed
        assert!(!inspector.is_valid_script_name("player controller")); // space not allowed
        assert!(!inspector.is_valid_script_name("player/controller")); // slash not allowed
    }

    #[test]
    fn test_multiple_typescript_scripts_on_entity() {
        let mut inspector = create_test_inspector();
        let (mut world, entity) = create_test_world_with_entity();

        // Attach first TypeScript script
        let result1 = inspector.attach_script_to_entity(&mut world, entity, "assets/scripts/script1.ts");
        assert!(result1.is_ok());

        // Attach second TypeScript script
        let result2 = inspector.attach_script_to_entity(&mut world, entity, "assets/scripts/script2.ts");
        assert!(result2.is_ok());

        // Entity should have multiple TypeScript scripts
        let typescript_script = world.get_component::<TypeScriptScript>(entity);
        assert!(typescript_script.is_some());
        
        let script = typescript_script.unwrap();
        let all_scripts = script.get_all_scripts();
        assert_eq!(all_scripts.len(), 2);
        assert!(all_scripts.contains(&&"assets/scripts/script1.ts".to_string()));
        assert!(all_scripts.contains(&&"assets/scripts/script2.ts".to_string()));
    }

    #[test]
    fn test_typescript_script_component_serialization() {
        let script = TypeScriptScript::new("test.ts".to_string());
        
        // Component should have proper path and be enabled by default
        assert_eq!(script.get_path(), "test.ts");
        assert!(script.is_enabled());
        
        // Component should support execution order
        let script_with_order = TypeScriptScript::with_execution_order("test.ts".to_string(), 10);
        assert_eq!(script_with_order.get_execution_order(), 10);
    }
}