//! TDD Tests for TypeScript example integration in script selection dialog
//! Following TDD principles, these tests define the expected behavior
//! for easy addition of TypeScript examples in the UI.

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
    fn test_typescript_should_have_script_selection_dialog() {
        let mut inspector = create_test_inspector();
        
        // TypeScript should support script selection like Lua
        inspector.script_creation_language = ScriptLanguage::TypeScript;
        
        // Should be able to access TypeScript example scripts
        let examples = inspector.get_typescript_example_scripts();
        assert!(examples.len() > 0, "TypeScript should have example scripts available");
        
        // Examples should be categorized for easy selection
        let beginner_examples = inspector.get_typescript_examples_by_difficulty("beginner");
        assert!(beginner_examples.len() >= 2, "Should have at least 2 beginner TypeScript examples");
        
        let intermediate_examples = inspector.get_typescript_examples_by_difficulty("intermediate");
        assert!(intermediate_examples.len() >= 1, "Should have intermediate TypeScript examples");
    }

    #[test]
    fn test_typescript_examples_easy_addition_ui() {
        let mut inspector = create_test_inspector();
        let (mut world, entity) = create_test_world_with_entity();
        
        inspector.script_creation_language = ScriptLanguage::TypeScript;
        
        // Should be able to get example metadata for UI display
        let example_metadata = inspector.get_typescript_example_metadata();
        assert!(example_metadata.len() >= 5, "Should have multiple TypeScript examples");
        
        // Each example should have required metadata for UI
        for (name, description, difficulty, category) in example_metadata {
            assert!(!name.is_empty(), "Example name should not be empty");
            assert!(!description.is_empty(), "Example description should not be empty");
            assert!(description.len() <= 100, "Description should be concise for UI");
            assert!(difficulty == "beginner" || difficulty == "intermediate" || difficulty == "advanced", 
                   "Difficulty should be valid");
            assert!(!category.is_empty(), "Category should not be empty");
        }
    }

    #[test]
    fn test_typescript_example_attachment() {
        let mut inspector = create_test_inspector();
        let (mut world, entity) = create_test_world_with_entity();
        
        // Should be able to attach TypeScript examples directly
        let result = inspector.attach_typescript_example(&mut world, entity, "typescript_hello_world");
        assert!(result.is_ok(), "Should be able to attach TypeScript example");
        
        // Entity should now have TypeScript component with example script
        let typescript_script = world.get_component::<TypeScriptScript>(entity);
        assert!(typescript_script.is_some(), "Entity should have TypeScript component");
        
        let script = typescript_script.unwrap();
        assert!(script.get_path().contains("hello_world"), "Should contain example script");
    }

    #[test]
    fn test_typescript_example_file_creation() {
        let mut inspector = create_test_inspector();
        let (mut world, entity) = create_test_world_with_entity();
        
        // Should be able to create files from TypeScript examples
        let result = inspector.create_typescript_example_file(&mut world, entity, "typescript_input_handling", "my_input_controller.ts");
        assert!(result.is_ok(), "Should be able to create TypeScript example file");
        
        // File should be created with proper content
        let file_path = std::path::Path::new("assets/scripts/my_input_controller.ts");
        assert!(file_path.exists(), "TypeScript example file should be created");
        
        let content = std::fs::read_to_string(file_path).expect("Should be able to read file");
        assert!(content.contains("export class"), "Should contain TypeScript class");
        assert!(content.contains("Engine.input"), "Should contain Engine input API");
        assert!(content.contains("init(): void"), "Should contain init method");
        
        // Clean up
        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_typescript_examples_categorization() {
        let inspector = create_test_inspector();
        
        // Should be able to get examples by category
        let basic_examples = inspector.get_typescript_examples_by_category("basic_syntax");
        assert!(basic_examples.len() > 0, "Should have basic syntax examples");
        
        let input_examples = inspector.get_typescript_examples_by_category("input_handling");
        assert!(input_examples.len() > 0, "Should have input handling examples");
        
        let physics_examples = inspector.get_typescript_examples_by_category("physics");
        assert!(physics_examples.len() > 0, "Should have physics examples");
        
        let game_logic_examples = inspector.get_typescript_examples_by_category("game_logic");
        assert!(game_logic_examples.len() > 0, "Should have game logic examples");
    }

    #[test]
    fn test_typescript_examples_search_and_filter() {
        let inspector = create_test_inspector();
        
        // Should be able to search examples by keyword
        let input_related = inspector.search_typescript_examples("input");
        assert!(input_related.len() > 0, "Should find input-related examples");
        
        let physics_related = inspector.search_typescript_examples("physics");
        assert!(physics_related.len() > 0, "Should find physics-related examples");
        
        // Should be able to filter by multiple criteria
        let beginner_basic = inspector.get_typescript_examples_filtered("beginner", "basic_syntax");
        assert!(beginner_basic.len() > 0, "Should have beginner basic syntax examples");
    }

    #[test]
    fn test_typescript_example_ui_integration() {
        let mut inspector = create_test_inspector();
        let (mut world, entity) = create_test_world_with_entity();
        
        // Should track which examples are most popular for UI recommendations
        let popular_examples = inspector.get_popular_typescript_examples();
        assert!(popular_examples.len() >= 3, "Should have popular examples for recommendations");
        
        // Should be able to get examples suitable for new users
        let newcomer_examples = inspector.get_newcomer_friendly_typescript_examples();
        assert!(newcomer_examples.len() >= 2, "Should have newcomer-friendly examples");
        
        // Examples should have proper display names for UI
        for example_name in newcomer_examples {
            let display_info = inspector.get_typescript_example_display_info(&example_name);
            assert!(display_info.is_some(), "Should have display info for examples");
            
            let (display_name, icon, tooltip) = display_info.unwrap();
            assert!(!display_name.is_empty(), "Should have display name");
            assert!(!icon.is_empty(), "Should have icon for UI");
            assert!(!tooltip.is_empty(), "Should have tooltip for UI");
        }
    }

    #[test]
    fn test_typescript_example_dialog_integration() {
        let mut inspector = create_test_inspector();
        let (mut world, entity) = create_test_world_with_entity();
        
        inspector.script_creation_language = ScriptLanguage::TypeScript;
        
        // Should be able to show TypeScript script selection dialog
        assert!(inspector.should_show_script_selection_for_typescript(), 
               "TypeScript should support script selection dialog");
        
        // Should be able to populate dialog with TypeScript examples
        let dialog_items = inspector.get_typescript_script_selection_items();
        assert!(dialog_items.len() > 0, "Should have items for script selection dialog");
        
        // Dialog items should include both project scripts and examples
        let has_examples = dialog_items.iter().any(|item| item.item_type == "example");
        let has_templates = dialog_items.iter().any(|item| item.item_type == "template");
        
        assert!(has_examples, "Dialog should include TypeScript examples");
        assert!(has_templates, "Dialog should include TypeScript templates");
    }

    #[test]
    fn test_typescript_example_error_handling() {
        let mut inspector = create_test_inspector();
        let (mut world, entity) = create_test_world_with_entity();
        
        // Should handle invalid example names gracefully
        let result = inspector.attach_typescript_example(&mut world, entity, "nonexistent_example");
        assert!(result.is_err(), "Should return error for invalid example name");
        
        // Should handle file creation errors gracefully
        let result = inspector.create_typescript_example_file(&mut world, entity, "typescript_hello_world", "");
        assert!(result.is_err(), "Should return error for invalid file name");
        
        // Should handle missing directory
        let result = inspector.create_typescript_example_file(&mut world, entity, "typescript_hello_world", "/invalid/path/script.ts");
        assert!(result.is_err(), "Should return error for invalid path");
    }
}