//! Documentation generator for the API registry

use crate::api::registry::{ApiRegistry, NamespaceDescriptor, ClassDescriptor, MethodBinding};
use std::sync::Arc;

/// Generator for API documentation
pub struct DocumentationGenerator {
    registry: Arc<ApiRegistry>,
}

impl DocumentationGenerator {
    pub fn new(registry: Arc<ApiRegistry>) -> Self {
        Self { registry }
    }

    /// Generate comprehensive API documentation in Markdown format
    pub fn generate_markdown(&self) -> String {
        let mut output = String::new();
        
        output.push_str("# Longhorn Engine TypeScript API Reference\n\n");
        output.push_str("Auto-generated API documentation for TypeScript scripting.\n\n");
        
        // Generate table of contents
        output.push_str("## Table of Contents\n\n");
        for namespace_name in self.registry.get_namespaces().keys() {
            let anchor = namespace_name.to_lowercase().replace(".", "-");
            output.push_str(&format!("- [{}](#{})\n", namespace_name, anchor));
        }
        output.push_str("\n");

        // Generate namespace documentation
        for (namespace_name, descriptor) in self.registry.get_namespaces() {
            output.push_str(&self.generate_namespace_docs(namespace_name, descriptor));
            output.push_str("\n");
        }

        output
    }

    /// Generate documentation for a single namespace
    fn generate_namespace_docs(&self, name: &str, descriptor: &NamespaceDescriptor) -> String {
        let mut output = String::new();
        
        let anchor = name.to_lowercase().replace(".", "-");
        output.push_str(&format!("## {} {{#{}}}\n\n", name, anchor));
        
        if !descriptor.documentation.is_empty() {
            output.push_str(&format!("{}\n\n", descriptor.documentation));
        }

        // Document methods
        if !descriptor.methods.is_empty() {
            output.push_str("### Methods\n\n");
            
            for method_name in &descriptor.methods {
                if let Some(method_binding) = self.registry.get_method(name, method_name) {
                    output.push_str(&self.generate_method_docs(method_binding));
                    output.push_str("\n");
                }
            }
        }

        // Document classes
        if !descriptor.classes.is_empty() {
            output.push_str("### Classes\n\n");
            
            for class in &descriptor.classes {
                output.push_str(&self.generate_class_docs(class));
                output.push_str("\n");
            }
        }

        output
    }

    /// Generate documentation for a method
    fn generate_method_docs(&self, binding: &MethodBinding) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("#### `{}`\n\n", binding.method_name));
        
        if !binding.documentation.is_empty() {
            output.push_str(&format!("{}\n\n", binding.documentation));
        }

        // Method signature
        let params = binding.parameter_types
            .iter()
            .enumerate()
            .map(|(i, param_type)| format!("param{}: {}", i, param_type.to_typescript()))
            .collect::<Vec<_>>()
            .join(", ");

        let return_type = binding.return_type.to_typescript();
        
        output.push_str("**Signature:**\n");
        output.push_str(&format!("```typescript\n{}({}): {}\n```\n\n", 
                binding.method_name, params, return_type));

        // Parameters
        if !binding.parameter_types.is_empty() {
            output.push_str("**Parameters:**\n");
            for (i, param_type) in binding.parameter_types.iter().enumerate() {
                output.push_str(&format!("- `param{}`: `{}` - Parameter {}\n", 
                        i, param_type.to_typescript(), i + 1));
            }
            output.push_str("\n");
        }

        // Return value
        output.push_str("**Returns:**\n");
        output.push_str(&format!("- `{}` - Return value\n\n", return_type));

        // Example usage
        output.push_str("**Example:**\n");
        output.push_str("```typescript\n");
        let example_args = (0..binding.parameter_types.len())
            .map(|i| format!("arg{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!("const result = {}.{}({});\n", 
                binding.namespace, binding.method_name, example_args));
        output.push_str("```\n");

        output
    }

    /// Generate documentation for a class
    fn generate_class_docs(&self, class: &ClassDescriptor) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("#### Class: `{}`\n\n", class.name));
        
        if !class.documentation.is_empty() {
            output.push_str(&format!("{}\n\n", class.documentation));
        }

        // Properties
        if !class.properties.is_empty() {
            output.push_str("**Properties:**\n");
            for property in &class.properties {
                let readonly = if property.readonly { " (readonly)" } else { "" };
                output.push_str(&format!("- `{}`: `{}`{}", 
                        property.name, property.property_type.to_typescript(), readonly));
                if !property.documentation.is_empty() {
                    output.push_str(&format!(" - {}", property.documentation));
                }
                output.push_str("\n");
            }
            output.push_str("\n");
        }

        // Methods
        if !class.methods.is_empty() {
            output.push_str("**Methods:**\n");
            for method_name in &class.methods {
                output.push_str(&format!("- `{}()` - Class method\n", method_name));
            }
            output.push_str("\n");
        }

        // Example usage
        output.push_str("**Example:**\n");
        output.push_str("```typescript\n");
        output.push_str(&format!("const instance = new {}();\n", class.name));
        if let Some(first_method) = class.methods.first() {
            output.push_str(&format!("instance.{}();\n", first_method));
        }
        output.push_str("```\n");

        output
    }

    /// Generate a quick reference guide
    pub fn generate_quick_reference(&self) -> String {
        let mut output = String::new();
        
        output.push_str("# Longhorn Engine API Quick Reference\n\n");
        
        for (namespace_name, descriptor) in self.registry.get_namespaces() {
            output.push_str(&format!("## {}\n\n", namespace_name));
            
            // List methods
            for method_name in &descriptor.methods {
                if let Some(method_binding) = self.registry.get_method(namespace_name, method_name) {
                    let params = method_binding.parameter_types.iter()
                        .map(|t| t.to_typescript())
                        .collect::<Vec<_>>()
                        .join(", ");
                    output.push_str(&format!("- `{}({})` → `{}`\n", 
                            method_name, params, method_binding.return_type.to_typescript()));
                }
            }
            
            // List classes
            for class in &descriptor.classes {
                output.push_str(&format!("- `class {}` - {}\n", 
                        class.name, class.documentation));
            }
            
            output.push_str("\n");
        }
        
        output
    }

    /// Generate example code snippets
    pub fn generate_examples(&self) -> String {
        let mut output = String::new();
        
        output.push_str("# Longhorn Engine API Examples\n\n");
        output.push_str("Common usage patterns and examples for the TypeScript API.\n\n");
        
        // Entity management example
        if self.registry.get_namespaces().contains_key("Engine.World") {
            output.push_str("## Entity Management\n\n");
            output.push_str("```typescript\n");
            output.push_str("// Get the current entity\n");
            output.push_str("const entity = Engine.World.getCurrentEntity();\n");
            output.push_str("console.log('Entity ID:', entity.id());\n\n");
            output.push_str("// Create a new entity\n");
            output.push_str("const newEntity = Engine.World.createEntity('MyEntity');\n\n");
            output.push_str("// Get component\n");
            output.push_str("const transform = entity.getComponent('Transform');\n");
            output.push_str("if (transform) {\n");
            output.push_str("    console.log('Position:', transform.position.x, transform.position.y, transform.position.z);\n");
            output.push_str("}\n");
            output.push_str("```\n\n");
        }

        // Math utilities example
        if self.registry.get_namespaces().contains_key("Engine.Math") {
            output.push_str("## Math Utilities\n\n");
            output.push_str("```typescript\n");
            output.push_str("// Linear interpolation\n");
            output.push_str("const lerped = Engine.Math.lerp(0, 100, 0.5); // 50\n\n");
            output.push_str("// Clamp values\n");
            output.push_str("const clamped = Engine.Math.clamp(150, 0, 100); // 100\n\n");
            output.push_str("// Trigonometry\n");
            output.push_str("const angle = Math.PI / 4;\n");
            output.push_str("const sine = Engine.Math.sin(angle);\n");
            output.push_str("const cosine = Engine.Math.cos(angle);\n");
            output.push_str("```\n\n");
        }

        // Debug utilities example
        if self.registry.get_namespaces().contains_key("Engine.Debug") {
            output.push_str("## Debug and Logging\n\n");
            output.push_str("```typescript\n");
            output.push_str("// Logging methods\n");
            output.push_str("Engine.Debug.log('Information message');\n");
            output.push_str("Engine.Debug.warn('Warning message');\n");
            output.push_str("Engine.Debug.error('Error message');\n");
            output.push_str("```\n\n");
        }
        
        output
    }
}