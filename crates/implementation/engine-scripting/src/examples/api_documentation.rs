//! Comprehensive API Documentation with Working Examples
//! 
//! This module provides complete documentation for all Longhorn Game Engine
//! scripting APIs with verified working examples.

use super::{ExampleScript, DifficultyLevel, ExampleCategory};
use super::script_examples::get_all_examples;
use std::collections::HashMap;

/// API Documentation Generator
pub struct ApiDocumentationGenerator {
    examples_by_api: HashMap<String, Vec<ExampleScript>>,
    api_categories: HashMap<String, Vec<String>>,
}

impl ApiDocumentationGenerator {
    /// Create a new documentation generator with all examples
    pub fn new() -> Self {
        let all_examples = get_all_examples();
        let mut examples_by_api = HashMap::new();
        let mut api_categories = HashMap::new();
        
        // Group examples by API features
        for example in all_examples {
            for api_feature in &example.api_features {
                examples_by_api.entry(api_feature.clone())
                    .or_insert_with(Vec::new)
                    .push(example.clone());
            }
        }
        
        // Define API categories
        api_categories.insert("Basic Lua".to_string(), vec!["print".to_string()]);
        api_categories.insert("Input System".to_string(), vec![
            "is_key_pressed".to_string(),
            "get_mouse_position".to_string(),
            "bind_key".to_string(),
        ]);
        api_categories.insert("Physics System".to_string(), vec![
            "add_rigid_body".to_string(),
            "apply_force".to_string(),
            "get_gravity".to_string(),
        ]);
        api_categories.insert("Event System".to_string(), vec![
            "emit_event".to_string(),
            "listen_for_event".to_string(),
        ]);
        api_categories.insert("Debugging".to_string(), vec![
            "debug_print".to_string(),
            "debug_inspect".to_string(),
            "debug_break".to_string(),
        ]);
        api_categories.insert("Performance".to_string(), vec![
            "profile_start".to_string(),
            "profile_mark".to_string(),
            "profile_stop".to_string(),
        ]);
        
        Self {
            examples_by_api,
            api_categories,
        }
    }
    
    /// Generate complete API documentation as markdown
    pub fn generate_markdown_documentation(&self) -> String {
        let mut doc = String::new();
        
        doc.push_str("# Longhorn Game Engine - Lua Scripting API Documentation\n\n");
        doc.push_str("Complete reference for all available scripting APIs with working examples.\n\n");
        
        doc.push_str("## Table of Contents\n\n");
        for category in self.api_categories.keys() {
            doc.push_str(&format!("- [{}](#{})\n", category, category.to_lowercase().replace(" ", "-")));
        }
        doc.push_str("\n");
        
        // Generate documentation for each category
        for (category, apis) in &self.api_categories {
            doc.push_str(&format!("## {}\n\n", category));
            
            for api in apis {
                if let Some(examples) = self.examples_by_api.get(api) {
                    doc.push_str(&self.generate_api_section(api, examples));
                }
            }
        }
        
        doc
    }
    
    /// Generate documentation section for a specific API
    fn generate_api_section(&self, api_name: &str, examples: &[ExampleScript]) -> String {
        let mut section = String::new();
        
        section.push_str(&format!("### `{}`\n\n", api_name));
        
        // Add API description based on name
        section.push_str(&self.get_api_description(api_name));
        section.push_str("\n\n");
        
        // Add examples
        section.push_str("#### Examples\n\n");
        
        for example in examples.iter().take(2) { // Show up to 2 examples per API
            section.push_str(&format!("**{}**\n\n", example.description));
            section.push_str("```lua\n");
            section.push_str(&example.code.trim());
            section.push_str("\n```\n\n");
            
            if !example.expected_outputs.is_empty() {
                section.push_str("Expected output:\n");
                for output in &example.expected_outputs {
                    section.push_str(&format!("- `{}`\n", output));
                }
                section.push_str("\n");
            }
        }
        
        section.push_str("---\n\n");
        section
    }
    
    /// Get description for API function
    fn get_api_description(&self, api_name: &str) -> &'static str {
        match api_name {
            "print" => "Prints text to the console. Basic output function for debugging and logging.",
            "is_key_pressed" => "Checks if a specific key is currently pressed. Returns boolean.",
            "get_mouse_position" => "Gets the current mouse position as x, y coordinates.",
            "bind_key" => "Binds a callback function to a specific key press event.",
            "add_rigid_body" => "Adds a new rigid body to the physics simulation.",
            "apply_force" => "Applies a force vector to a rigid body.",
            "get_gravity" => "Gets the current gravity vector of the physics world.",
            "emit_event" => "Emits a custom event that can be listened to by other scripts.",
            "listen_for_event" => "Registers a listener for custom events.",
            "debug_print" => "Enhanced debug printing with formatting and categorization.",
            "debug_inspect" => "Inspects variables and objects for debugging.",
            "debug_break" => "Sets a breakpoint for debugging (development builds only).",
            "profile_start" => "Starts performance profiling for a named section.",
            "profile_mark" => "Adds a performance marker during profiling.",
            "profile_stop" => "Stops performance profiling and returns results.",
            _ => "API function for game scripting."
        }
    }
    
    /// Generate tutorial progression
    pub fn generate_tutorial_series(&self) -> Vec<TutorialStep> {
        vec![
            TutorialStep {
                title: "Getting Started with Lua Scripting".to_string(),
                description: "Learn the basics of Lua scripting in Longhorn Game Engine".to_string(),
                examples: self.get_examples_for_difficulty(DifficultyLevel::Beginner),
                apis_covered: vec!["print".to_string()],
            },
            TutorialStep {
                title: "Input Handling".to_string(),
                description: "Handle keyboard and mouse input in your scripts".to_string(),
                examples: self.get_examples_for_category(ExampleCategory::InputHandling),
                apis_covered: vec!["is_key_pressed".to_string(), "get_mouse_position".to_string()],
            },
            TutorialStep {
                title: "Physics Integration".to_string(),
                description: "Integrate with the physics system for realistic gameplay".to_string(),
                examples: self.get_examples_for_category(ExampleCategory::Physics),
                apis_covered: vec!["add_rigid_body".to_string(), "apply_force".to_string()],
            },
            TutorialStep {
                title: "Advanced Scripting".to_string(),
                description: "Advanced techniques including debugging and profiling".to_string(),
                examples: self.get_examples_for_difficulty(DifficultyLevel::Advanced),
                apis_covered: vec!["debug_print".to_string(), "profile_start".to_string()],
            },
        ]
    }
    
    /// Get examples for specific difficulty level
    fn get_examples_for_difficulty(&self, level: DifficultyLevel) -> Vec<ExampleScript> {
        self.examples_by_api.values()
            .flatten()
            .filter(|e| e.difficulty_level == level)
            .cloned()
            .collect()
    }
    
    /// Get examples for specific category
    fn get_examples_for_category(&self, category: ExampleCategory) -> Vec<ExampleScript> {
        self.examples_by_api.values()
            .flatten()
            .filter(|e| e.category == category)
            .cloned()
            .collect()
    }
    
    /// Generate API coverage report
    pub fn generate_coverage_report(&self) -> ApiCoverageReport {
        let total_apis = self.api_categories.values().flatten().count();
        let documented_apis = self.examples_by_api.len();
        let coverage_percentage = if total_apis > 0 {
            (documented_apis as f32 / total_apis as f32) * 100.0
        } else {
            0.0
        };
        
        ApiCoverageReport {
            total_apis,
            documented_apis,
            coverage_percentage,
            missing_apis: self.find_missing_apis(),
            example_count: self.examples_by_api.values().map(|v| v.len()).sum(),
        }
    }
    
    /// Find APIs that don't have examples
    fn find_missing_apis(&self) -> Vec<String> {
        self.api_categories.values()
            .flatten()
            .filter(|api| !self.examples_by_api.contains_key(*api))
            .cloned()
            .collect()
    }
}

/// Tutorial step in the learning progression
#[derive(Debug, Clone)]
pub struct TutorialStep {
    pub title: String,
    pub description: String,
    pub examples: Vec<ExampleScript>,
    pub apis_covered: Vec<String>,
}

/// API documentation coverage report
#[derive(Debug)]
pub struct ApiCoverageReport {
    pub total_apis: usize,
    pub documented_apis: usize,
    pub coverage_percentage: f32,
    pub missing_apis: Vec<String>,
    pub example_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_documentation_generator_creation() {
        let generator = ApiDocumentationGenerator::new();
        assert!(!generator.examples_by_api.is_empty(), "Should have examples");
        assert!(!generator.api_categories.is_empty(), "Should have API categories");
        println!("✅ Documentation generator creates successfully");
    }
    
    #[test]
    fn test_markdown_generation() {
        let generator = ApiDocumentationGenerator::new();
        let markdown = generator.generate_markdown_documentation();
        
        assert!(markdown.contains("# Longhorn Game Engine"), "Should have main title");
        assert!(markdown.contains("## Table of Contents"), "Should have TOC");
        assert!(markdown.contains("```lua"), "Should have code examples");
        
        println!("✅ Markdown documentation generates correctly");
    }
    
    #[test]
    fn test_tutorial_series_generation() {
        let generator = ApiDocumentationGenerator::new();
        let tutorials = generator.generate_tutorial_series();
        
        assert!(!tutorials.is_empty(), "Should have tutorial steps");
        assert!(tutorials.iter().any(|t| t.title.contains("Getting Started")), "Should have beginner tutorial");
        
        println!("✅ Tutorial series generates correctly");
    }
    
    #[test]
    fn test_coverage_report() {
        let generator = ApiDocumentationGenerator::new();
        let report = generator.generate_coverage_report();
        
        assert!(report.total_apis > 0, "Should have APIs to document");
        assert!(report.coverage_percentage >= 0.0, "Coverage should be non-negative");
        
        println!("✅ Coverage report: {:.1}% ({}/{})", 
                 report.coverage_percentage, report.documented_apis, report.total_apis);
    }
}