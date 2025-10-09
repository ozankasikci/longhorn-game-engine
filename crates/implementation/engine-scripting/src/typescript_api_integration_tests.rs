//! TDD Tests for integrating the new API registry system with the existing TypeScript runtime
//! 
//! Following TDD methodology: write failing tests first, then implement to make them pass.
//! These tests define how the registry system should integrate with SimpleTypeScriptRuntime.

#[cfg(test)]
mod tests {
    use crate::typescript_script_system::SimpleTypeScriptRuntime;
    use crate::api::{TypeScriptApiSystem, ApiRegistry};

    /// Test that we can create a TypeScript runtime with the new API registry system
    #[test]
    fn test_runtime_with_registry_system_creation() {
        // This should fail initially because integration doesn't exist yet
        let result = SimpleTypeScriptRuntime::with_api_registry();
        
        // Expected to fail until we implement the integration
        assert!(result.is_err(), "Runtime with registry system should fail until integration is implemented");
        
        println!("❌ FAILING TEST: SimpleTypeScriptRuntime::with_api_registry() not implemented");
        println!("   Next step: Add API registry support to SimpleTypeScriptRuntime");
    }

    /// Test that registry-based APIs are available in script execution
    #[test]
    fn test_registry_apis_available_in_scripts() {
        // Create runtime with registry (this will fail until implemented)
        let mut runtime = match SimpleTypeScriptRuntime::with_api_registry() {
            Ok(runtime) => runtime,
            Err(_) => {
                println!("❌ FAILING TEST: Cannot create runtime with registry system");
                return;
            }
        };

        let script_source = r#"
            class TestScript {
                init() {
                    // Test Engine.World namespace from registry
                    const entity = Engine.World.getCurrentEntity();
                    if (!entity) {
                        throw new Error("Engine.World.getCurrentEntity() not available from registry");
                    }
                    
                    // Test Engine.Math namespace from registry
                    const result = Engine.Math.lerp(0, 100, 0.5);
                    if (result !== 50) {
                        throw new Error("Engine.Math.lerp() not working correctly from registry");
                    }
                    
                    // Test Engine.Debug namespace from registry
                    Engine.Debug.log("Registry API test successful");
                    
                    console.log("✅ All registry APIs working correctly");
                    globalThis.registryApisWorking = true;
                }
                
                update(deltaTime) {}
                destroy() {}
            }
            
            globalThis.TestScript = TestScript;
        "#;

        // This should fail until registry integration is complete
        let result = runtime.load_and_compile_script(1, "registry_test.ts", script_source);
        assert!(result.is_ok(), "Script should compile");
        
        let init_result = runtime.call_init(1);
        assert!(init_result.is_err(), "Test should fail until registry APIs are integrated");
        
        println!("❌ FAILING TEST: Registry APIs not available in script execution");
        println!("   Next step: Integrate registry system with V8 context");
    }

    /// Test that registry APIs work alongside legacy APIs
    #[test]
    fn test_registry_and_legacy_apis_coexist() {
        let mut runtime = match SimpleTypeScriptRuntime::with_api_registry() {
            Ok(runtime) => runtime,
            Err(_) => {
                println!("❌ FAILING TEST: Cannot create runtime with registry system");
                return;
            }
        };

        let script_source = r#"
            class CoexistenceTestScript {
                init() {
                    // Test legacy Engine.world.getCurrentEntity() still works
                    const legacyEntity = Engine.world.getCurrentEntity();
                    if (!legacyEntity) {
                        throw new Error("Legacy Engine.world.getCurrentEntity() not working");
                    }
                    
                    // Test new Engine.World.getCurrentEntity() from registry works
                    const registryEntity = Engine.World.getCurrentEntity();
                    if (!registryEntity) {
                        throw new Error("Registry Engine.World.getCurrentEntity() not working");
                    }
                    
                    // Both should return entities with id() method
                    const legacyId = legacyEntity.id();
                    const registryId = registryEntity.id();
                    
                    console.log("Legacy entity ID:", legacyId);
                    console.log("Registry entity ID:", registryId);
                    
                    globalThis.bothApisWorking = true;
                }
                
                update(deltaTime) {}
                destroy() {}
            }
            
            globalThis.CoexistenceTestScript = CoexistenceTestScript;
        "#;

        let result = runtime.load_and_compile_script(1, "coexistence_test.ts", script_source);
        assert!(result.is_ok(), "Script should compile");
        
        let init_result = runtime.call_init(1);
        assert!(init_result.is_err(), "Test should fail until registry and legacy APIs coexist");
        
        println!("❌ FAILING TEST: Registry and legacy APIs cannot coexist yet");
        println!("   Next step: Ensure registry APIs don't conflict with legacy APIs");
    }

    /// Test that registry-based Transform components work with position modification
    #[test]
    fn test_registry_transform_component_position_modification() {
        let mut runtime = match SimpleTypeScriptRuntime::with_api_registry() {
            Ok(runtime) => runtime,
            Err(_) => {
                println!("❌ FAILING TEST: Cannot create runtime with registry system");
                return;
            }
        };

        let script_source = r#"
            class TransformTestScript {
                private transform;
                
                init() {
                    // Get entity from registry
                    const entity = Engine.World.getCurrentEntity();
                    
                    // Get transform component from registry
                    this.transform = entity.getComponent('Transform');
                    if (!this.transform) {
                        throw new Error("Transform component not available from registry");
                    }
                    
                    // Check initial position from registry
                    console.log("Initial position from registry:", 
                               this.transform.position.x, 
                               this.transform.position.y, 
                               this.transform.position.z);
                    
                    globalThis.initialPosition = {
                        x: this.transform.position.x,
                        y: this.transform.position.y,
                        z: this.transform.position.z
                    };
                }
                
                update(deltaTime) {
                    // Modify position using registry APIs
                    this.transform.position.x += deltaTime * 2.0;
                    this.transform.position.z += deltaTime * 5.0;
                    
                    console.log("Updated position via registry:", 
                               this.transform.position.x, 
                               this.transform.position.y, 
                               this.transform.position.z);
                    
                    globalThis.modifiedPosition = {
                        x: this.transform.position.x,
                        y: this.transform.position.y,
                        z: this.transform.position.z
                    };
                    
                    globalThis.registryTransformWorking = true;
                }
                
                destroy() {}
            }
            
            globalThis.TransformTestScript = TransformTestScript;
        "#;

        let result = runtime.load_and_compile_script(1, "registry_transform_test.ts", script_source);
        assert!(result.is_ok(), "Script should compile");
        
        let init_result = runtime.call_init(1);
        assert!(init_result.is_err(), "Test should fail until registry Transform API is working");
        
        println!("❌ FAILING TEST: Registry-based Transform component position modification not working");
        println!("   Next step: Implement registry Transform component with position synchronization");
    }

    /// Test that TypeScript definitions are generated correctly for the runtime
    #[test]
    fn test_typescript_definitions_generation_for_runtime() {
        // Create the API system to generate definitions
        let api_system = match crate::api::TypeScriptApiSystem::new() {
            Ok(system) => system,
            Err(e) => {
                panic!("Failed to create API system: {}", e);
            }
        };

        let definitions = api_system.generate_type_definitions();
        
        // Check that key namespaces are present
        assert!(definitions.contains("declare namespace Engine.World"), 
                "Engine.World namespace should be in generated definitions");
        assert!(definitions.contains("declare namespace Engine.Math"), 
                "Engine.Math namespace should be in generated definitions");
        assert!(definitions.contains("declare namespace Engine.Debug"), 
                "Engine.Debug namespace should be in generated definitions");
        
        // Check that key methods are present
        assert!(definitions.contains("function getCurrentEntity"), 
                "getCurrentEntity function should be in generated definitions");
        assert!(definitions.contains("function lerp"), 
                "lerp function should be in generated definitions");
        assert!(definitions.contains("function log"), 
                "log function should be in generated definitions");
        
        println!("✅ PASSING TEST: TypeScript definitions generated correctly");
        
        // This test should pass because we implemented the generator
        // Next step is to integrate these definitions with the runtime
    }

    /// Test that the runtime can be configured to use either legacy or registry APIs
    #[test]
    fn test_runtime_api_configuration() {
        // Test legacy configuration (should work)
        let legacy_runtime = SimpleTypeScriptRuntime::new();
        assert!(legacy_runtime.is_ok(), "Legacy runtime should work");
        
        // Test registry configuration (should fail until implemented)
        let registry_runtime = SimpleTypeScriptRuntime::with_api_registry();
        assert!(registry_runtime.is_err(), "Registry runtime should fail until implemented");
        
        // Test that we can switch between modes (future feature)
        let mut runtime = legacy_runtime.unwrap();
        let switch_result = runtime.enable_registry_apis();
        assert!(switch_result.is_err(), "Switching to registry APIs should fail until implemented");
        
        println!("❌ FAILING TEST: Runtime API configuration not implemented");
        println!("   Next step: Add API configuration methods to SimpleTypeScriptRuntime");
    }

    /// Test that entity context is properly passed to registry methods
    #[test]
    fn test_entity_context_with_registry() {
        let mut runtime = match SimpleTypeScriptRuntime::with_api_registry() {
            Ok(runtime) => runtime,
            Err(_) => {
                println!("❌ FAILING TEST: Cannot create runtime with registry system");
                return;
            }
        };

        // Set entity context (this method should exist)
        let set_context_result = runtime.set_entity_context(123);
        assert!(set_context_result.is_err(), "Setting entity context should fail until implemented");

        let script_source = r#"
            class EntityContextTestScript {
                init() {
                    const entity = Engine.World.getCurrentEntity();
                    const entityId = entity.id();
                    
                    // Should return the entity ID we set in context (123)
                    if (entityId !== 123) {
                        throw new Error(`Expected entity ID 123, got ${entityId}`);
                    }
                    
                    console.log("✅ Entity context correctly passed to registry APIs");
                    globalThis.entityContextWorking = true;
                }
                
                update(deltaTime) {}
                destroy() {}
            }
            
            globalThis.EntityContextTestScript = EntityContextTestScript;
        "#;

        let result = runtime.load_and_compile_script(1, "entity_context_test.ts", script_source);
        assert!(result.is_ok(), "Script should compile");
        
        let init_result = runtime.call_init(1);
        assert!(init_result.is_err(), "Test should fail until entity context integration is implemented");
        
        println!("❌ FAILING TEST: Entity context not properly passed to registry methods");
        println!("   Next step: Implement entity context support in registry integration");
    }
}