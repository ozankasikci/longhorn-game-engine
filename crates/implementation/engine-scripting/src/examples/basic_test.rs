//! Basic test to verify example framework compiles and works
//! This is a simpler alternative to the complex integration test

#[cfg(test)]
mod tests {
    use super::super::{ExampleValidator, ExampleScript, DifficultyLevel, ExampleCategory};
    
    #[test]
    fn test_example_validator_creation() {
        // Test that we can create an ExampleValidator
        let validator = ExampleValidator::new();
        assert!(validator.is_ok(), "Should be able to create ExampleValidator");
        
        let validator = validator.unwrap();
        assert_eq!(validator.example_count(), 0, "New validator should have no examples");
        
        println!("✅ Basic example validator creation works");
    }
    
    #[test]
    fn test_adding_simple_example() {
        let mut validator = ExampleValidator::new().unwrap();
        
        // Create a simple working example
        let example = ExampleScript {
            name: "test_simple".to_string(),
            description: "Simple test example".to_string(),
            code: r#"
                local result = "Hello from Lua!"
                return result
            "#.to_string(),
            expected_outputs: vec!["Hello from Lua!".to_string()],
            api_features: vec!["lua_basic".to_string()],
            difficulty_level: DifficultyLevel::Beginner,
            category: ExampleCategory::BasicSyntax,
        };
        
        validator.add_example(example);
        assert_eq!(validator.example_count(), 1, "Should have one example after adding");
        
        println!("✅ Adding simple example works");
    }
    
    #[test]
    fn test_category_filtering() {
        let mut validator = ExampleValidator::new().unwrap();
        
        // Add examples in different categories
        let basic_example = ExampleScript {
            name: "basic".to_string(),
            description: "Basic example".to_string(),
            code: "return 42".to_string(),
            expected_outputs: vec!["42".to_string()],
            api_features: vec!["lua_basic".to_string()],
            difficulty_level: DifficultyLevel::Beginner,
            category: ExampleCategory::BasicSyntax,
        };
        
        let physics_example = ExampleScript {
            name: "physics".to_string(),
            description: "Physics example".to_string(),
            code: "return 'physics'".to_string(),
            expected_outputs: vec!["physics".to_string()],
            api_features: vec!["physics_api".to_string()],
            difficulty_level: DifficultyLevel::Intermediate,
            category: ExampleCategory::Physics,
        };
        
        validator.add_example(basic_example);
        validator.add_example(physics_example);
        
        let basic_examples = validator.get_examples_by_category(ExampleCategory::BasicSyntax);
        let physics_examples = validator.get_examples_by_category(ExampleCategory::Physics);
        
        assert_eq!(basic_examples.len(), 1, "Should have one basic example");
        assert_eq!(physics_examples.len(), 1, "Should have one physics example");
        assert_eq!(basic_examples[0].name, "basic");
        assert_eq!(physics_examples[0].name, "physics");
        
        println!("✅ Category filtering works");
    }
}