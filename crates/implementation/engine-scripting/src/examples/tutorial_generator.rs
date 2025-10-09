//! Tutorial Series Generator
//! Creates structured learning paths with validated examples

use super::{ExampleScript, DifficultyLevel, ExampleCategory};
use super::script_examples::get_all_examples;
use super::api_documentation::{TutorialStep, ApiDocumentationGenerator};

/// Tutorial series generator with progressive learning
pub struct TutorialGenerator {
    examples: Vec<ExampleScript>,
    generator: ApiDocumentationGenerator,
}

impl TutorialGenerator {
    /// Create new tutorial generator
    pub fn new() -> Self {
        Self {
            examples: get_all_examples(),
            generator: ApiDocumentationGenerator::new(),
        }
    }
    
    /// Generate complete tutorial series
    pub fn generate_complete_series(&self) -> TutorialSeries {
        let steps = self.generator.generate_tutorial_series();
        
        TutorialSeries {
            title: "Longhorn Game Engine Scripting Mastery".to_string(),
            description: "Complete guide to mastering Lua scripting in Longhorn Game Engine".to_string(),
            steps,
            total_examples: self.examples.len(),
            estimated_hours: 8,
        }
    }
    
    /// Generate tutorial as markdown
    pub fn generate_tutorial_markdown(&self) -> String {
        let series = self.generate_complete_series();
        let mut markdown = String::new();
        
        markdown.push_str(&format!("# {}\n\n", series.title));
        markdown.push_str(&format!("{}\n\n", series.description));
        markdown.push_str(&format!("**Estimated Time:** {} hours\n", series.estimated_hours));
        markdown.push_str(&format!("**Total Examples:** {}\n\n", series.total_examples));
        
        markdown.push_str("## Tutorial Progression\n\n");
        
        for (i, step) in series.steps.iter().enumerate() {
            markdown.push_str(&format!("### Step {}: {}\n\n", i + 1, step.title));
            markdown.push_str(&format!("{}\n\n", step.description));
            
            markdown.push_str("**APIs Covered:**\n");
            for api in &step.apis_covered {
                markdown.push_str(&format!("- `{}`\n", api));
            }
            markdown.push_str("\n");
            
            markdown.push_str("**Practice Examples:**\n\n");
            for (j, example) in step.examples.iter().take(3).enumerate() {
                markdown.push_str(&format!("#### Example {}: {}\n\n", j + 1, example.description));
                markdown.push_str("```lua\n");
                markdown.push_str(&example.code.trim());
                markdown.push_str("\n```\n\n");
            }
            
            markdown.push_str("---\n\n");
        }
        
        markdown
    }
}

/// Complete tutorial series
#[derive(Debug)]
pub struct TutorialSeries {
    pub title: String,
    pub description: String,
    pub steps: Vec<TutorialStep>,
    pub total_examples: usize,
    pub estimated_hours: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tutorial_generator_creation() {
        let generator = TutorialGenerator::new();
        assert!(!generator.examples.is_empty(), "Should have examples");
        println!("✅ Tutorial generator created successfully");
    }
    
    #[test]
    fn test_complete_series_generation() {
        let generator = TutorialGenerator::new();
        let series = generator.generate_complete_series();
        
        assert!(!series.steps.is_empty(), "Should have tutorial steps");
        assert!(series.estimated_hours > 0, "Should have estimated time");
        assert!(series.total_examples > 0, "Should have examples");
        
        println!("✅ Complete tutorial series: {} steps, {} examples", 
                 series.steps.len(), series.total_examples);
    }
    
    #[test]
    fn test_markdown_generation() {
        let generator = TutorialGenerator::new();
        let markdown = generator.generate_tutorial_markdown();
        
        assert!(markdown.contains("# Longhorn Game Engine"), "Should have title");
        assert!(markdown.contains("## Tutorial Progression"), "Should have progression");
        assert!(markdown.contains("```lua"), "Should have code examples");
        
        println!("✅ Tutorial markdown generated successfully");
    }
}