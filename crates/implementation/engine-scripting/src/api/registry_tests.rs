//! Tests for the API registry system

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::registry::{ApiRegistry, NamespaceDescriptor, MethodBindingBuilder, TypeDescriptor, Value, MethodContext};
    use crate::api::codegen::TypeScriptGenerator;

    #[test]
    fn test_api_registry_creation() {
        let registry = ApiRegistry::new();
        assert!(!registry.is_initialized());
        assert_eq!(registry.get_namespaces().len(), 0);
        assert_eq!(registry.get_methods().len(), 0);
    }

    #[test]
    fn test_namespace_registration() {
        let mut registry = ApiRegistry::new();
        
        let namespace = NamespaceDescriptor::new("Test.Namespace".to_string())
            .with_methods(vec!["testMethod".to_string()])
            .with_documentation("Test namespace for unit tests".to_string());
        
        let result = registry.register_namespace(namespace);
        assert!(result.is_ok());
        assert_eq!(registry.get_namespaces().len(), 1);
    }

    #[test]
    fn test_method_registration() {
        let mut registry = ApiRegistry::new();
        
        // First register a namespace
        let namespace = NamespaceDescriptor::new("Test.Namespace".to_string())
            .with_methods(vec!["testMethod".to_string()]);
        registry.register_namespace(namespace).unwrap();
        
        // Then register a method
        let method = MethodBindingBuilder::new("Test.Namespace", "testMethod")
            .with_return_type(TypeDescriptor::String)
            .with_documentation("A test method")
            .build(|_context: &MethodContext, _args: &[Value]| -> Result<Value, crate::api::registry::ApiError> {
                Ok(Value::String("Hello World".to_string()))
            });
        
        let result = registry.register_method(method);
        assert!(result.is_ok());
        assert_eq!(registry.get_methods().len(), 1);
    }

    #[test]
    fn test_registry_finalization() {
        let mut registry = ApiRegistry::new();
        
        // Register namespace and method
        let namespace = NamespaceDescriptor::new("Test.Namespace".to_string())
            .with_methods(vec!["testMethod".to_string()]);
        registry.register_namespace(namespace).unwrap();
        
        let method = MethodBindingBuilder::new("Test.Namespace", "testMethod")
            .with_return_type(TypeDescriptor::String)
            .build(|_context: &MethodContext, _args: &[Value]| -> Result<Value, crate::api::registry::ApiError> {
                Ok(Value::String("Hello World".to_string()))
            });
        registry.register_method(method).unwrap();
        
        // Finalize registry
        let result = registry.finalize();
        assert!(result.is_ok());
        assert!(registry.is_initialized());
    }

    #[test]
    fn test_missing_method_implementation_error() {
        let mut registry = ApiRegistry::new();
        
        // Register namespace with method but don't register method implementation
        let namespace = NamespaceDescriptor::new("Test.Namespace".to_string())
            .with_methods(vec!["missingMethod".to_string()]);
        registry.register_namespace(namespace).unwrap();
        
        // Finalization should fail
        let result = registry.finalize();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::api::registry::ApiError::MissingMethodImplementation(_)));
    }

    #[test]
    fn test_method_call() {
        let mut registry = ApiRegistry::new();
        
        // Register namespace and method
        let namespace = NamespaceDescriptor::new("Test.Namespace".to_string())
            .with_methods(vec!["echo".to_string()]);
        registry.register_namespace(namespace).unwrap();
        
        let method = MethodBindingBuilder::new("Test.Namespace", "echo")
            .with_parameter(TypeDescriptor::String)
            .with_return_type(TypeDescriptor::String)
            .build(|_context: &MethodContext, args: &[Value]| -> Result<Value, crate::api::registry::ApiError> {
                let input = args[0].as_string().unwrap_or("").to_string();
                Ok(Value::String(format!("Echo: {}", input)))
            });
        registry.register_method(method).unwrap();
        registry.finalize().unwrap();
        
        // Test method call
        let method_binding = registry.get_method("Test.Namespace", "echo").unwrap();
        let context = MethodContext::new(0);
        let args = vec![Value::String("test".to_string())];
        
        let result = method_binding.call(&context, &args);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert_eq!(output.as_string(), Some("Echo: test"));
    }

    #[test]
    fn test_typescript_generation() {
        let mut registry = ApiRegistry::new();
        
        // Register a test namespace
        let namespace = NamespaceDescriptor::new("Engine.Test".to_string())
            .with_methods(vec!["testMethod".to_string()])
            .with_documentation("Test namespace for TypeScript generation".to_string());
        registry.register_namespace(namespace).unwrap();
        
        let method = MethodBindingBuilder::new("Engine.Test", "testMethod")
            .with_parameter(TypeDescriptor::Number)
            .with_return_type(TypeDescriptor::String)
            .with_documentation("A test method that converts number to string")
            .build(|_context: &MethodContext, args: &[Value]| -> Result<Value, crate::api::registry::ApiError> {
                let num = args[0].as_number().unwrap_or(0.0);
                Ok(Value::String(num.to_string()))
            });
        registry.register_method(method).unwrap();
        registry.finalize().unwrap();
        
        // Generate TypeScript definitions
        let registry_arc = std::sync::Arc::new(registry);
        let generator = TypeScriptGenerator::new(registry_arc);
        let definitions = generator.generate_definitions();
        
        assert!(definitions.contains("declare namespace Engine.Test"));
        assert!(definitions.contains("export function testMethod"));
        assert!(definitions.contains("param0: number"));
        assert!(definitions.contains("): string"));
    }

    #[test]
    fn test_engine_world_namespace_registration() {
        let mut registry = ApiRegistry::new();
        
        // Register Engine.World namespace
        let result = crate::api::namespaces::register_engine_world(&mut registry);
        assert!(result.is_ok());
        
        // Check that methods were registered
        assert!(registry.get_namespaces().contains_key("Engine.World"));
        
        let result = registry.finalize();
        assert!(result.is_ok());
        
        // Test that we can get methods
        assert!(registry.get_method("Engine.World", "getCurrentEntity").is_some());
        assert!(registry.get_method("Engine.World", "createEntity").is_some());
    }

    #[test]
    fn test_complete_api_system_creation() {
        // Test that we can create the complete TypeScript API system
        let result = crate::api::TypeScriptApiSystem::new();
        assert!(result.is_ok());
        
        let api_system = result.unwrap();
        
        // Test TypeScript generation
        let definitions = api_system.generate_type_definitions();
        assert!(definitions.contains("Engine.World"));
        assert!(definitions.contains("Engine.Math"));
        assert!(definitions.contains("Engine.Debug"));
    }
}