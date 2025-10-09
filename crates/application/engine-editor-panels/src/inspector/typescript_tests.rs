//! Tests for TypeScript integration in the Inspector panel
//! Following TDD principles, these tests define the expected behavior
//! for TypeScript script creation and management in the UI.

use crate::{InspectorPanel, ScriptLanguage, ScriptTemplate};
use engine_scripting::components::TypeScriptScript;
use engine_ecs_core::{Entity, World, register_component};
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test helper to create a mock inspector panel
    fn create_test_inspector() -> InspectorPanel {
        InspectorPanel::new()
    }

    /// Test helper to create a test world with entity
    fn create_test_world_with_entity() -> (World, Entity) {
        // Register TypeScript component for tests
        register_component::<TypeScriptScript>();
        
        let mut world = World::new();
        let entity = world.spawn();
        (world, entity)
    }

    #[test]
    fn test_script_creation_defaults_to_typescript() {
        let inspector = create_test_inspector();
        
        // When creating a new script, it should default to TypeScript
        assert_eq!(inspector.script_creation_language, ScriptLanguage::TypeScript);
        assert_eq!(inspector.script_creation_template, ScriptTemplate::Entity);
    }

    #[test]
    fn test_script_creation_generates_typescript_files() {
        let mut inspector = create_test_inspector();
        let (mut world, entity) = create_test_world_with_entity();
        
        // Set up script creation
        inspector.script_creation_name = "player_controller".to_string();
        inspector.script_creation_template = ScriptTemplate::Entity;
        inspector.script_creation_language = ScriptLanguage::TypeScript;
        
        // Create script should generate .ts file
        let result = inspector.create_script_file(&mut world, entity);
        assert!(result.is_ok());
        
        // Should create TypeScript file with .ts extension
        let expected_path = "assets/scripts/player_controller.ts";
        assert!(Path::new(expected_path).exists());
        
        // Clean up
        let _ = std::fs::remove_file(expected_path);
    }

    #[test]
    fn test_entity_template_generates_typescript_class() {
        let mut inspector = create_test_inspector();
        let (mut world, entity) = create_test_world_with_entity();
        
        inspector.script_creation_name = "enemy_ai".to_string();
        inspector.script_creation_template = ScriptTemplate::Entity;
        inspector.script_creation_language = ScriptLanguage::TypeScript;
        
        let result = inspector.create_script_file(&mut world, entity);
        assert!(result.is_ok());
        
        // Read the generated file content
        let script_path = "assets/scripts/enemy_ai.ts";
        let content = std::fs::read_to_string(script_path).unwrap();
        
        // Should contain TypeScript class structure
        assert!(content.contains("export class EnemyAi"));
        assert!(content.contains("init(): void"));
        assert!(content.contains("update(deltaTime: number): void"));
        assert!(content.contains("destroy(): void"));
        assert!(content.contains("Engine.world.getCurrentEntity()"));
        
        // Clean up
        let _ = std::fs::remove_file(script_path);
    }

    #[test]
    fn test_behavior_template_generates_typescript_interface() {
        let mut inspector = create_test_inspector();
        let (mut world, entity) = create_test_world_with_entity();
        
        inspector.script_creation_name = "jump_behavior".to_string();
        inspector.script_creation_template = ScriptTemplate::Behavior;
        inspector.script_creation_language = ScriptLanguage::TypeScript;
        
        let result = inspector.create_script_file(&mut world, entity);
        assert!(result.is_ok());
        
        let script_path = "assets/scripts/jump_behavior.ts";
        let content = std::fs::read_to_string(script_path).unwrap();
        
        // Should contain TypeScript behavior pattern
        assert!(content.contains("interface Behavior"));
        assert!(content.contains("export class JumpBehavior implements Behavior"));
        assert!(content.contains("start(entity: Entity): void"));
        assert!(content.contains("update(entity: Entity, deltaTime: number): void"));
        
        // Clean up
        let _ = std::fs::remove_file(script_path);
    }

    #[test]
    fn test_system_template_generates_typescript_system() {
        let mut inspector = create_test_inspector();
        let (mut world, entity) = create_test_world_with_entity();
        
        inspector.script_creation_name = "physics_system".to_string();
        inspector.script_creation_template = ScriptTemplate::System;
        inspector.script_creation_language = ScriptLanguage::TypeScript;
        
        let result = inspector.create_script_file(&mut world, entity);
        assert!(result.is_ok());
        
        let script_path = "assets/scripts/physics_system.ts";
        let content = std::fs::read_to_string(script_path).unwrap();
        
        // Should contain TypeScript system pattern
        assert!(content.contains("interface System"));
        assert!(content.contains("export class PhysicsSystem implements System"));
        assert!(content.contains("initialize(): void"));
        assert!(content.contains("update(world: World, deltaTime: number): void"));
        assert!(content.contains("Engine.world.query"));
        
        // Clean up
        let _ = std::fs::remove_file(script_path);
    }

    #[test]
    fn test_script_creation_dialog_shows_typescript_options() {
        let mut inspector = create_test_inspector();
        
        // Script creation dialog should show TypeScript as default
        assert!(!inspector.show_script_creation_dialog);
        
        inspector.show_script_creation_dialog = true;
        
        // Should have TypeScript language option available
        assert_eq!(inspector.script_creation_language, ScriptLanguage::TypeScript);
        
        // Should have all template options available for TypeScript
        let available_templates = vec![
            ScriptTemplate::Entity,
            ScriptTemplate::Behavior,
            ScriptTemplate::System,
        ];
        
        for template in available_templates {
            inspector.script_creation_template = template;
            // Should be valid combination
            assert!(inspector.is_valid_template_language_combination());
        }
    }

    #[test]
    fn test_lua_option_hidden_by_default() {
        let inspector = create_test_inspector();
        
        // Lua should not be the default option in UI
        assert_ne!(inspector.script_creation_language, ScriptLanguage::Lua);
        
        // TypeScript should be the default
        assert_eq!(inspector.script_creation_language, ScriptLanguage::TypeScript);
    }

    #[test]
    fn test_script_attachment_works_with_typescript() {
        let mut inspector = create_test_inspector();
        let (mut world, entity) = create_test_world_with_entity();
        
        // Create a TypeScript script
        inspector.script_creation_name = "test_script".to_string();
        inspector.script_creation_template = ScriptTemplate::Entity;
        inspector.script_creation_language = ScriptLanguage::TypeScript;
        
        let result = inspector.create_script_file(&mut world, entity);
        assert!(result.is_ok());
        
        // Should be able to attach TypeScript script to entity
        let script_path = "assets/scripts/test_script.ts";
        let attach_result = inspector.attach_script_to_entity(&mut world, entity, script_path);
        assert!(attach_result.is_ok());
        
        // Entity should have TypeScript script component attached
        assert!(world.has_component::<TypeScriptScript>(entity));
        
        // Clean up
        let _ = std::fs::remove_file(script_path);
    }

    #[test]
    fn test_script_validation_rejects_invalid_names() {
        let inspector = create_test_inspector();
        
        // Empty names should be invalid
        assert!(!inspector.is_valid_script_name(""));
        
        // Names with invalid characters should be invalid
        assert!(!inspector.is_valid_script_name("script with spaces"));
        assert!(!inspector.is_valid_script_name("script-with-dashes"));
        assert!(!inspector.is_valid_script_name("script/with/slashes"));
        
        // Valid names should be accepted
        assert!(inspector.is_valid_script_name("PlayerController"));
        assert!(inspector.is_valid_script_name("enemy_ai"));
        assert!(inspector.is_valid_script_name("physics_system"));
    }

    #[test]
    fn test_typescript_script_component_metadata() {
        let mut inspector = create_test_inspector();
        let (mut world, entity) = create_test_world_with_entity();
        
        // Create and attach TypeScript script
        inspector.script_creation_name = "metadata_test".to_string();
        let _ = inspector.create_script_file(&mut world, entity);
        let _ = inspector.attach_script_to_entity(&mut world, entity, "assets/scripts/metadata_test.ts");
        
        // Should be able to read script metadata
        let script_component = world.get_component::<TypeScriptScript>(entity).unwrap();
        // TypeScript scripts should have .ts extension
        assert_eq!(script_component.get_file_extension(), "ts");
        assert!(script_component.get_path().ends_with(".ts"));
        
        // Clean up
        let _ = std::fs::remove_file("assets/scripts/metadata_test.ts");
    }
}