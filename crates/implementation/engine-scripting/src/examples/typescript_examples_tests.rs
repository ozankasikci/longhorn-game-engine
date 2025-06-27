//! TDD Tests for TypeScript example scripts system
//! Following TDD principles, these tests define the expected behavior
//! for TypeScript example scripts and their integration.

use super::{ExampleScript, DifficultyLevel, ExampleCategory, ExampleValidator};
use super::typescript_examples::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typescript_hello_world_example() {
        let example = get_typescript_example_by_name("typescript_hello_world").unwrap();
        
        assert_eq!(example.name, "typescript_hello_world");
        assert_eq!(example.description, "The classic Hello World example in TypeScript");
        assert_eq!(example.difficulty_level, DifficultyLevel::Beginner);
        assert_eq!(example.category, ExampleCategory::BasicSyntax);
        
        // Should contain TypeScript syntax
        assert!(example.code.contains("console.log"));
        assert!(example.code.contains("export"));
        assert!(example.code.contains(": void"));
        
        // Should have expected outputs
        assert!(example.expected_outputs.contains(&"Hello, World!".to_string()));
        
        // Should cover console.log API
        assert!(example.api_features.contains(&"console.log".to_string()));
    }

    #[test]
    fn test_typescript_entity_controller_example() {
        let example = get_typescript_example_by_name("typescript_entity_controller").unwrap();
        
        assert_eq!(example.name, "typescript_entity_controller");
        assert_eq!(example.difficulty_level, DifficultyLevel::Intermediate);
        assert_eq!(example.category, ExampleCategory::GameLogic);
        
        // Should contain TypeScript class syntax
        assert!(example.code.contains("export class"));
        assert!(example.code.contains("init(): void"));
        assert!(example.code.contains("update(deltaTime: number): void"));
        assert!(example.code.contains("destroy(): void"));
        
        // Should use Engine APIs
        assert!(example.code.contains("Engine.world.getCurrentEntity"));
        assert!(example.code.contains("getComponent<Transform>"));
        
        // Should cover Engine APIs
        assert!(example.api_features.contains(&"Engine.world".to_string()));
        assert!(example.api_features.contains(&"getComponent".to_string()));
    }

    #[test]
    fn test_typescript_input_handling_example() {
        let example = get_typescript_example_by_name("typescript_input_handling").unwrap();
        
        assert_eq!(example.name, "typescript_input_handling");
        assert_eq!(example.difficulty_level, DifficultyLevel::Beginner);
        assert_eq!(example.category, ExampleCategory::InputHandling);
        
        // Should contain input handling code
        assert!(example.code.contains("Engine.input"));
        assert!(example.code.contains("isKeyDown"));
        assert!(example.code.contains("getMousePosition"));
        
        // Should cover input APIs
        assert!(example.api_features.contains(&"Engine.input".to_string()));
        assert!(example.api_features.contains(&"isKeyDown".to_string()));
    }

    #[test]
    fn test_typescript_physics_example() {
        let example = get_typescript_example_by_name("typescript_physics_basic").unwrap();
        
        assert_eq!(example.name, "typescript_physics_basic");
        assert_eq!(example.difficulty_level, DifficultyLevel::Intermediate);
        assert_eq!(example.category, ExampleCategory::Physics);
        
        // Should contain physics code
        assert!(example.code.contains("Engine.physics"));
        assert!(example.code.contains("Vector3"));
        assert!(example.code.contains("velocity"));
        
        // Should cover physics APIs
        assert!(example.api_features.contains(&"Engine.physics".to_string()));
        assert!(example.api_features.contains(&"Vector3".to_string()));
    }

    #[test]
    fn test_typescript_event_system_example() {
        let example = get_typescript_example_by_name("typescript_event_system").unwrap();
        
        assert_eq!(example.name, "typescript_event_system");
        assert_eq!(example.difficulty_level, DifficultyLevel::Advanced);
        assert_eq!(example.category, ExampleCategory::EventSystem);
        
        // Should contain event handling code
        assert!(example.code.contains("Engine.events"));
        assert!(example.code.contains("addEventListener"));
        assert!(example.code.contains("dispatchEvent"));
        
        // Should cover event APIs
        assert!(example.api_features.contains(&"Engine.events".to_string()));
        assert!(example.api_features.contains(&"addEventListener".to_string()));
    }

    #[test]
    fn test_get_all_typescript_examples() {
        let examples = get_all_typescript_examples();
        
        // Should have at least 5 examples
        assert!(examples.len() >= 5);
        
        // Should have examples from different categories
        let categories: Vec<_> = examples.iter().map(|e| &e.category).collect();
        assert!(categories.contains(&&ExampleCategory::BasicSyntax));
        assert!(categories.contains(&&ExampleCategory::GameLogic));
        assert!(categories.contains(&&ExampleCategory::InputHandling));
        assert!(categories.contains(&&ExampleCategory::Physics));
        assert!(categories.contains(&&ExampleCategory::EventSystem));
        
        // Should have examples from different difficulty levels
        let difficulties: Vec<_> = examples.iter().map(|e| &e.difficulty_level).collect();
        assert!(difficulties.contains(&&DifficultyLevel::Beginner));
        assert!(difficulties.contains(&&DifficultyLevel::Intermediate));
        assert!(difficulties.contains(&&DifficultyLevel::Advanced));
    }

    #[test]
    fn test_typescript_examples_validator() {
        let mut validator = ExampleValidator::new().expect("Failed to create validator");
        let examples = get_all_typescript_examples();
        
        // Add all TypeScript examples
        for example in examples {
            validator.add_example(example);
        }
        
        // Should have added examples
        assert!(validator.example_count() >= 5);
        
        // Should be able to get examples by category
        let basic_examples = validator.get_examples_by_category(ExampleCategory::BasicSyntax);
        assert!(basic_examples.len() > 0);
        
        let input_examples = validator.get_examples_by_category(ExampleCategory::InputHandling);
        assert!(input_examples.len() > 0);
    }

    #[test]
    fn test_typescript_examples_coverage_report() {
        let mut validator = ExampleValidator::new().expect("Failed to create validator");
        let examples = get_all_typescript_examples();
        
        for example in examples {
            validator.add_example(example);
        }
        
        let coverage = validator.generate_coverage_report();
        
        // Should cover key TypeScript/Engine APIs
        assert!(coverage.contains_key("console.log"));
        assert!(coverage.contains_key("Engine.world"));
        assert!(coverage.contains_key("Engine.input"));
        assert!(coverage.contains_key("Vector3"));
    }

    #[test]
    fn test_typescript_examples_easy_addition() {
        // Test that examples can be easily found and added to script selection
        let examples = get_all_typescript_examples();
        
        // Should be able to filter beginner examples for easy addition
        let beginner_examples = get_beginner_typescript_examples();
        
        assert!(beginner_examples.len() >= 2);
        
        // Each beginner example should have simple, clear descriptions
        for example in beginner_examples {
            assert!(!example.description.is_empty());
            assert!(example.description.len() < 100); // Keep descriptions concise
        }
    }

    #[test]
    fn test_typescript_examples_filtering() {
        let all_examples = get_all_typescript_examples();
        
        // Test category filtering
        let basic_examples = get_typescript_examples_by_category(ExampleCategory::BasicSyntax);
        assert!(basic_examples.len() > 0);
        
        let physics_examples = get_typescript_examples_by_category(ExampleCategory::Physics);
        assert!(physics_examples.len() > 0);
        
        // Test difficulty filtering
        let beginner_examples = get_typescript_examples_by_difficulty(DifficultyLevel::Beginner);
        assert!(beginner_examples.len() > 0);
        
        let intermediate_examples = get_typescript_examples_by_difficulty(DifficultyLevel::Intermediate);
        assert!(intermediate_examples.len() > 0);
        
        // Test convenience function
        let easy_examples = get_beginner_typescript_examples();
        assert_eq!(easy_examples.len(), beginner_examples.len());
    }
}