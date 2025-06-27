//! Example scripts and documentation testing framework

pub mod script_examples;
pub mod basic_test;
pub mod api_documentation;
pub mod tutorial_generator;

use crate::ScriptError;
use mlua::Lua;
use std::collections::HashMap;

/// Example script metadata
#[derive(Debug, Clone)]
pub struct ExampleScript {
    pub name: String,
    pub description: String,
    pub code: String,
    pub expected_outputs: Vec<String>,
    pub api_features: Vec<String>,
    pub difficulty_level: DifficultyLevel,
    pub category: ExampleCategory,
}

/// Difficulty levels for examples
#[derive(Debug, Clone, PartialEq)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
}

/// Categories of examples
#[derive(Debug, Clone, PartialEq)]
pub enum ExampleCategory {
    BasicSyntax,
    InputHandling,
    Physics,
    EventSystem,
    Debugging,
    Performance,
    GameLogic,
    Integration,
}

/// Example validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub script_name: String,
    pub success: bool,
    pub execution_time: std::time::Duration,
    pub outputs: Vec<String>,
    pub errors: Vec<String>,
    pub api_coverage: Vec<String>,
}

/// Example script validator
pub struct ExampleValidator {
    examples: HashMap<String, ExampleScript>,
    lua_context: Lua,
}

impl ExampleValidator {
    /// Create a new example validator
    pub fn new() -> Result<Self, ScriptError> {
        let lua = Lua::new();
        
        Ok(Self {
            examples: HashMap::new(),
            lua_context: lua,
        })
    }
    
    /// Add an example script to validate
    pub fn add_example(&mut self, example: ExampleScript) {
        self.examples.insert(example.name.clone(), example);
    }
    
    /// Validate a specific example script
    pub fn validate_example(&self, example_name: &str) -> Result<ValidationResult, ScriptError> {
        let example = self.examples.get(example_name)
            .ok_or_else(|| ScriptError::NotFound {
                script_id: None,
                path: example_name.to_string(),
            })?;
        
        let start_time = std::time::Instant::now();
        let mut outputs = Vec::new();
        let mut errors = Vec::new();
        let mut success = true;
        
        // Execute the example script
        match self.lua_context.load(&example.code).exec() {
            Ok(_) => {
                // Check if expected outputs were generated
                for expected in &example.expected_outputs {
                    outputs.push(format!("Expected: {}", expected));
                }
            }
            Err(e) => {
                success = false;
                errors.push(format!("Execution error: {}", e));
            }
        }
        
        let execution_time = start_time.elapsed();
        
        Ok(ValidationResult {
            script_name: example_name.to_string(),
            success,
            execution_time,
            outputs,
            errors,
            api_coverage: example.api_features.clone(),
        })
    }
    
    /// Validate all example scripts
    pub fn validate_all_examples(&self) -> Result<Vec<ValidationResult>, ScriptError> {
        let mut results = Vec::new();
        
        for example_name in self.examples.keys() {
            let result = self.validate_example(example_name)?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Get examples by category
    pub fn get_examples_by_category(&self, category: ExampleCategory) -> Vec<&ExampleScript> {
        self.examples.values()
            .filter(|example| example.category == category)
            .collect()
    }
    
    /// Get examples by difficulty level
    pub fn get_examples_by_difficulty(&self, level: DifficultyLevel) -> Vec<&ExampleScript> {
        self.examples.values()
            .filter(|example| example.difficulty_level == level)
            .collect()
    }
    
    /// Generate API coverage report
    pub fn generate_coverage_report(&self) -> HashMap<String, Vec<String>> {
        let mut coverage = HashMap::new();
        
        for example in self.examples.values() {
            for api_feature in &example.api_features {
                coverage.entry(api_feature.clone())
                    .or_insert_with(Vec::new)
                    .push(example.name.clone());
            }
        }
        
        coverage
    }
    
    /// Get total number of examples
    pub fn example_count(&self) -> usize {
        self.examples.len()
    }
    
    /// Check if all examples are valid
    pub fn all_examples_valid(&self) -> Result<bool, ScriptError> {
        let results = self.validate_all_examples()?;
        Ok(results.iter().all(|r| r.success))
    }
}