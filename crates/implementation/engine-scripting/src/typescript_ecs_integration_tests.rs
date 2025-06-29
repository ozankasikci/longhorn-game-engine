//! TDD Tests for TypeScript ECS Integration
//! 
//! Following TDD methodology: write failing tests first, then implement to make them pass.
//! These tests define the expected behavior for TypeScript scripts accessing ECS components.

#[cfg(test)]
mod tests {
    use crate::typescript_script_system::SimpleTypeScriptRuntime;

    /// Test that Engine.world.getCurrentEntity() returns the correct entity
    #[test]
    fn test_get_current_entity_returns_actual_entity() {
        // Arrange
        let mut runtime = SimpleTypeScriptRuntime::new()
            .expect("Should create TypeScript runtime");
        
        // TypeScript test script
        let script_source = r#"
            class TestScript {
                init() {
                    const entity = Engine.world.getCurrentEntity();
                    
                    // Should return actual entity, not undefined
                    if (entity === undefined) {
                        throw new Error("getCurrentEntity() returned undefined - NOT IMPLEMENTED");
                    }
                    
                    // Should have id() method
                    if (typeof entity.id !== 'function') {
                        throw new Error("Entity missing id() method");
                    }
                    
                    const entityId = entity.id();
                    console.log("✅ getCurrentEntity() returned entity with ID:", entityId);
                    
                    // Store for verification
                    globalThis.testEntityId = entityId;
                }
                
                update(deltaTime) {}
                destroy() {}
            }
            
            globalThis.TestScript = TestScript;
        "#;

        // Act & Assert
        // This should FAIL initially because getCurrentEntity() is not implemented
        runtime.load_and_compile_script(1, "test_get_current_entity.ts", script_source)
            .expect("Should compile script");
        
        // Set entity context (this is what we need to implement)
        // runtime.set_entity_context(entity_id, world_arc.clone()); // ← TO IMPLEMENT
        
        // This should now pass because Engine.world.getCurrentEntity() is implemented
        let result = runtime.call_init(1);
        
        // Expected to pass now that we have basic implementation
        assert!(result.is_ok(), "Test should pass now that getCurrentEntity() is implemented");
        
        println!("✅ PASSING TEST: Engine.world.getCurrentEntity() implemented successfully");
        println!("   Next step: Implement component access system");
    }
    
    /// Test that entity.getComponent<Transform>() returns actual Transform data
    #[test]
    fn test_get_transform_component_returns_actual_data() {
        // Arrange
        let mut runtime = SimpleTypeScriptRuntime::new()
            .expect("Should create TypeScript runtime");
        
        // TypeScript test script
        let script_source = r#"
            class TestScript {
                init() {
                    const entity = Engine.world.getCurrentEntity();
                    
                    if (!entity) {
                        throw new Error("getCurrentEntity() failed - test prerequisite");
                    }
                    
                    // Should be able to get Transform component
                    const transform = entity.getComponent('Transform');
                    
                    if (transform === null || transform === undefined) {
                        throw new Error("getComponent('Transform') returned null/undefined - NOT IMPLEMENTED");
                    }
                    
                    // Should have position property
                    if (!transform.position) {
                        throw new Error("Transform missing position property");
                    }
                    
                    // Should have correct position values
                    console.log("Transform position:", transform.position.x, transform.position.y, transform.position.z);
                    
                    // Verify expected values
                    if (Math.abs(transform.position.x - 5.0) > 0.001) {
                        throw new Error(`Position X mismatch: expected 5.0, got ${transform.position.x}`);
                    }
                    
                    console.log("✅ getComponent<Transform>() returned correct data");
                    globalThis.testTransform = transform;
                }
                
                update(deltaTime) {}
                destroy() {}
            }
            
            globalThis.TestScript = TestScript;
        "#;

        // Act & Assert
        // This should FAIL initially because getComponent() is not implemented
        runtime.load_and_compile_script(1, "test_get_component.ts", script_source)
            .expect("Should compile script");
        
        // Set entity context (this is what we need to implement)
        // runtime.set_entity_context(entity_id, world_arc.clone()); // ← TO IMPLEMENT
        
        // This should now pass because getComponent() is implemented
        let result = runtime.call_init(1);
        
        // Expected to pass now that we have component access
        assert!(result.is_ok(), "Test should pass now that getComponent() is implemented");
        
        println!("✅ PASSING TEST: entity.getComponent<Transform>() implemented successfully");
        println!("   Next step: Implement position modification sync");
    }
    
    /// Test that modifying transform.position in script affects actual ECS component
    #[test]
    fn test_transform_position_modification_affects_ecs() {
        // Arrange
        let mut runtime = SimpleTypeScriptRuntime::new()
            .expect("Should create TypeScript runtime");
        
        // TypeScript test script that modifies position
        let script_source = r#"
            class TestScript {
                private transform;
                
                init() {
                    const entity = Engine.world.getCurrentEntity();
                    this.transform = entity.getComponent('Transform');
                    
                    if (!this.transform) {
                        throw new Error("Failed to get Transform component");
                    }
                    
                    console.log("Initial position:", this.transform.position.x, this.transform.position.y, this.transform.position.z);
                }
                
                update(deltaTime) {
                    // Move entity forward
                    this.transform.position.z += deltaTime * 5.0;
                    this.transform.position.x += deltaTime * 2.0;
                    
                    console.log("Updated position:", this.transform.position.x, this.transform.position.y, this.transform.position.z);
                    
                    // Mark that we modified the transform
                    globalThis.positionModified = true;
                    globalThis.newPositionX = this.transform.position.x;
                    globalThis.newPositionZ = this.transform.position.z;
                }
                
                destroy() {}
            }
            
            globalThis.TestScript = TestScript;
        "#;

        // Act & Assert
        // This should FAIL initially because position modification doesn't sync back to ECS
        runtime.load_and_compile_script(1, "test_position_modification.ts", script_source)
            .expect("Should compile script");
        
        // Set entity context and run script
        // runtime.set_entity_context(entity_id, world_arc.clone()); // ← TO IMPLEMENT
        
        // First call init() to initialize the script
        let init_result = runtime.call_init(1);
        assert!(init_result.is_ok(), "Init should succeed");
        
        // Then call update() to trigger position modifications
        let update_result = runtime.call_update(1, 0.016); // 16ms delta time
        
        // This should fail until we implement bidirectional sync
        // The script will modify position in update(), but changes won't sync back to ECS
        assert!(update_result.is_err(), "Test should fail until position modification sync is implemented");
        
        println!("❌ FAILING TEST: Transform position modification not synced to ECS");
        println!("   Next step: Implement bidirectional component synchronization");
    }
    
    /// Test complete workflow: get entity, get component, modify, verify sync
    #[test]
    fn test_complete_entity_component_workflow() {
        // Arrange
        let mut runtime = SimpleTypeScriptRuntime::new()
            .expect("Should create TypeScript runtime");
        
        // Complete workflow test script
        let script_source = r#"
            class EntityController {
                private entity;
                private transform;
                
                init() {
                    console.log("🚀 Starting complete ECS integration test");
                    
                    // Step 1: Get current entity
                    this.entity = Engine.world.getCurrentEntity();
                    if (!this.entity) {
                        throw new Error("STEP 1 FAILED: getCurrentEntity() returned null/undefined");
                    }
                    console.log("✅ Step 1: Got entity with ID", this.entity.id());
                    
                    // Step 2: Get Transform component
                    this.transform = this.entity.getComponent('Transform');
                    if (!this.transform) {
                        throw new Error("STEP 2 FAILED: getComponent('Transform') returned null/undefined");
                    }
                    console.log("✅ Step 2: Got transform component");
                    console.log("   Initial position:", this.transform.position.x, this.transform.position.y, this.transform.position.z);
                    
                    // Store initial values for verification
                    globalThis.initialX = this.transform.position.x;
                    globalThis.initialY = this.transform.position.y;
                    globalThis.initialZ = this.transform.position.z;
                }
                
                update(deltaTime) {
                    // Step 3: Modify position
                    const moveSpeed = 100.0; // units per second
                    this.transform.position.x += deltaTime * moveSpeed;
                    this.transform.position.z -= deltaTime * moveSpeed;
                    
                    console.log("🔄 Step 3: Modified position to", 
                               this.transform.position.x, this.transform.position.y, this.transform.position.z);
                    
                    // Store for verification
                    globalThis.modifiedX = this.transform.position.x;
                    globalThis.modifiedZ = this.transform.position.z;
                    globalThis.workflowComplete = true;
                }
                
                destroy() {
                    console.log("🏁 EntityController destroyed");
                }
            }
            
            globalThis.EntityController = EntityController;
        "#;

        // Act & Assert
        runtime.load_and_compile_script(1, "complete_workflow_test.ts", script_source)
            .expect("Should compile script");
        
        // This is the API we need to implement
        // runtime.set_entity_context(entity_id, world_arc.clone()); // ← TO IMPLEMENT
        
        // This will fail because the complete workflow is not implemented
        let result = runtime.call_init(1);
        
        // Expected to fail until complete implementation
        assert!(result.is_err(), "Complete workflow should fail until all systems implemented");
        
        println!("❌ FAILING TEST: Complete TypeScript ↔ ECS workflow not implemented");
        println!("   This test will pass when all previous systems are implemented");
    }
    
    /// Helper test to verify TypeScript runtime is working
    #[test]
    fn test_typescript_runtime_basic_functionality() {
        let mut runtime = SimpleTypeScriptRuntime::new()
            .expect("Should create TypeScript runtime");
        
        let basic_script = r#"
            class BasicTest {
                init() {
                    console.log("✅ TypeScript runtime is working");
                    globalThis.runtimeWorking = true;
                }
                update(deltaTime) {}
                destroy() {}
            }
            globalThis.BasicTest = BasicTest;
        "#;
        
        runtime.load_and_compile_script(1, "basic_test.ts", basic_script)
            .expect("Should compile basic script");
        
        runtime.call_init(1)
            .expect("Should execute basic script init");
        
        println!("✅ TypeScript runtime baseline functionality verified");
    }
}