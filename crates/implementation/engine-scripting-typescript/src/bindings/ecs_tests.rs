//! Tests for TypeScript ECS API bindings
//! 
//! These tests define the expected behavior of the ECS bindings for TypeScript scripts.
//! Following TDD principles, these tests are written before implementation.

use crate::initialize_v8_platform;
use crate::runtime::TypeScriptRuntime;
use engine_scripting::{
    runtime::ScriptRuntime,
    types::{ScriptId, ScriptMetadata, ScriptType},
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            // Test entity creation through global engine API
            const entity = engine.world.createEntity();
            
            function getEntityId(): number {
                return entity.id;
            }
        "#;
        
        let script_id = ScriptId(1);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_entity_creation.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test that entity was created and has a valid ID
        let entity_id = runtime.execute_function("getEntityId", vec![]).unwrap();
        let id: u32 = entity_id.parse().unwrap();
        assert!(id > 0, "Entity should have a valid ID");
    }

    #[test]
    fn test_component_operations() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            interface Transform {
                position: { x: number, y: number, z: number };
                rotation: { x: number, y: number, z: number };
                scale: { x: number, y: number, z: number };
            }
            
            const entity = engine.world.createEntity();
            
            // Add Transform component
            entity.addComponent("Transform", {
                position: { x: 1.0, y: 2.0, z: 3.0 },
                rotation: { x: 0.0, y: 0.0, z: 0.0 },
                scale: { x: 1.0, y: 1.0, z: 1.0 }
            });
            
            function getPosition(): string {
                const transform = entity.getComponent("Transform") as Transform;
                return JSON.stringify(transform.position);
            }
            
            function hasTransform(): boolean {
                return entity.hasComponent("Transform");
            }
            
            function removeTransform(): void {
                entity.removeComponent("Transform");
            }
        "#;
        
        let script_id = ScriptId(2);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_components.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test component was added
        let has_transform = runtime.execute_function("hasTransform", vec![]).unwrap();
        assert_eq!(has_transform, "true", "Entity should have Transform component");
        
        // Test component data access
        let position = runtime.execute_function("getPosition", vec![]).unwrap();
        assert!(position.contains("1") && position.contains("2") && position.contains("3"), 
                "Position should contain correct values");
        
        // Test component removal
        runtime.execute_function("removeTransform", vec![]).unwrap();
        let has_transform_after = runtime.execute_function("hasTransform", vec![]).unwrap();
        assert_eq!(has_transform_after, "false", "Entity should not have Transform component after removal");
    }

    #[test]
    fn test_world_queries() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            // Create multiple entities with Transform components
            const entity1 = engine.world.createEntity();
            entity1.addComponent("Transform", {
                position: { x: 1.0, y: 0.0, z: 0.0 },
                rotation: { x: 0.0, y: 0.0, z: 0.0 },
                scale: { x: 1.0, y: 1.0, z: 1.0 }
            });
            
            const entity2 = engine.world.createEntity();
            entity2.addComponent("Transform", {
                position: { x: 2.0, y: 0.0, z: 0.0 },
                rotation: { x: 0.0, y: 0.0, z: 0.0 },
                scale: { x: 1.0, y: 1.0, z: 1.0 }
            });
            
            function countTransformEntities(): number {
                const entities = engine.world.query("Transform");
                return entities.length;
            }
            
            function getFirstEntityPosition(): string {
                const entities = engine.world.query("Transform");
                if (entities.length > 0) {
                    const transform = entities[0].getComponent("Transform");
                    return JSON.stringify(transform.position);
                }
                return "{}";
            }
        "#;
        
        let script_id = ScriptId(3);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_queries.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test entity query returns correct count
        let count = runtime.execute_function("countTransformEntities", vec![]).unwrap();
        assert_eq!(count, "2", "Should find 2 entities with Transform components");
        
        // Test query results contain valid data
        let position = runtime.execute_function("getFirstEntityPosition", vec![]).unwrap();
        assert!(position.contains("1") || position.contains("2"), 
                "First entity position should be valid");
    }

    #[test]
    fn test_current_entity_access() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            // Test accessing the current entity (the entity this script is attached to)
            // In a real scenario, 'self.entity' would be automatically injected
            
            function initScript(): void {
                // Simulate having a current entity
                if (typeof self !== 'undefined' && self.entity) {
                    self.entity.addComponent("Health", {
                        current: 100,
                        max: 100
                    });
                }
            }
            
            function getCurrentHealth(): number {
                if (typeof self !== 'undefined' && self.entity) {
                    const health = self.entity.getComponent("Health");
                    return health ? health.current : 0;
                }
                return 0;
            }
        "#;
        
        let script_id = ScriptId(4);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_current_entity.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Note: This test shows the desired API structure
        // The actual implementation will need to inject the 'self' object
        // For now, we test that the script compiles and runs without error
        let health = runtime.execute_function("getCurrentHealth", vec![]).unwrap();
        assert_eq!(health, "0", "Should return 0 when no entity is available");
    }
}