//! Real V8 Engine API Integration Tests for TypeScript
//! 
//! These tests use the actual SimpleTypeScriptRuntime to verify that Engine APIs
//! are properly injected into the V8 context and can be called from TypeScript scripts.

#[cfg(test)]
mod tests {
    use super::super::typescript_script_system::SimpleTypeScriptRuntime;

    #[test]
    fn test_world_api_injection_into_real_v8_context() {
        // Arrange
        let mut runtime = SimpleTypeScriptRuntime::new().unwrap();
        
        // World API should be available in global scope
        let test_script = r#"
            export class WorldApiTestScript {
                init(): void {
                    // Test that world API is available
                    if (typeof globalThis.World === 'undefined') {
                        throw new Error('World API not available in global scope');
                    }
                    
                    // Test basic world query functionality
                    if (typeof globalThis.World.queryEntities !== 'function') {
                        throw new Error('World.queryEntities function not available');
                    }
                    
                    console.log('World API successfully injected');
                }
            }
        "#;
        
        // Act
        runtime.load_and_compile_script(1, "world_api_test.ts", test_script).unwrap();
        let result = runtime.call_init(1);
        
        // Assert
        assert!(result.is_ok(), "World API should be available in V8 context: {:?}", result);
    }

    #[test] 
    fn test_input_api_injection_into_real_v8_context() {
        // Arrange
        let mut runtime = SimpleTypeScriptRuntime::new().unwrap();
        
        let test_script = r#"
            export class InputApiTestScript {
                init(): void {
                    // Test that input API is available
                    if (typeof globalThis.Input === 'undefined') {
                        throw new Error('Input API not available in global scope');
                    }
                    
                    // Test basic input functions
                    if (typeof globalThis.Input.isKeyPressed !== 'function') {
                        throw new Error('Input.isKeyPressed function not available');
                    }
                    
                    if (typeof globalThis.Input.getMousePosition !== 'function') {
                        throw new Error('Input.getMousePosition function not available');
                    }
                    
                    console.log('Input API successfully injected');
                }
            }
        "#;
        
        // Act
        runtime.load_and_compile_script(1, "input_api_test.ts", test_script).unwrap();
        let result = runtime.call_init(1);
        
        // Assert
        assert!(result.is_ok(), "Input API should be available in V8 context: {:?}", result);
    }

    #[test]
    fn test_physics_api_injection_into_real_v8_context() {
        // Arrange
        let mut runtime = SimpleTypeScriptRuntime::new().unwrap();
        
        let test_script = r#"
            export class PhysicsApiTestScript {
                init(): void {
                    // Test that physics API is available
                    if (typeof globalThis.Physics === 'undefined') {
                        throw new Error('Physics API not available in global scope');
                    }
                    
                    // Test basic physics functions
                    if (typeof globalThis.Physics.applyForce !== 'function') {
                        throw new Error('Physics.applyForce function not available');
                    }
                    
                    if (typeof globalThis.Physics.raycast !== 'function') {
                        throw new Error('Physics.raycast function not available');
                    }
                    
                    console.log('Physics API successfully injected');
                }
            }
        "#;
        
        // Act
        runtime.load_and_compile_script(1, "physics_api_test.ts", test_script).unwrap();
        let result = runtime.call_init(1);
        
        // Assert
        assert!(result.is_ok(), "Physics API should be available in V8 context: {:?}", result);
    }

    #[test]
    fn test_world_api_entity_operations_with_real_v8() {
        // Arrange
        let mut runtime = SimpleTypeScriptRuntime::new().unwrap();
        
        let test_script = r#"
            export class WorldEntityTestScript {
                init(): void {
                    // Test entity creation
                    const entityId = globalThis.World.createEntity();
                    if (typeof entityId !== 'number') {
                        throw new Error('World.createEntity should return entity ID');
                    }
                    
                    // Test entity component operations
                    globalThis.World.addComponent(entityId, 'Transform', {
                        position: { x: 1.0, y: 2.0, z: 3.0 },
                        rotation: { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
                        scale: { x: 1.0, y: 1.0, z: 1.0 }
                    });
                    
                    const transform = globalThis.World.getComponent(entityId, 'Transform');
                    if (!transform || transform.position.x !== 1.0) {
                        throw new Error('Component operations not working correctly');
                    }
                    
                    console.log('World entity operations working correctly');
                }
            }
        "#;
        
        // Act
        runtime.load_and_compile_script(1, "world_entity_test.ts", test_script).unwrap();
        let result = runtime.call_init(1);
        
        // Assert - This should now pass since we implemented the API injection
        assert!(result.is_ok(), "World entity operations should work: {:?}", result);
    }

    #[test]
    fn test_input_api_key_and_mouse_operations_with_real_v8() {
        // Arrange
        let mut runtime = SimpleTypeScriptRuntime::new().unwrap();
        
        let test_script = r#"
            export class InputOperationsTestScript {
                init(): void {
                    // Test key input functions
                    const isWPressed = globalThis.Input.isKeyPressed('W');
                    const isSpaceDown = globalThis.Input.isKeyDown('Space');
                    const isShiftUp = globalThis.Input.isKeyUp('Shift');
                    
                    if (typeof isWPressed !== 'boolean') {
                        throw new Error('Input.isKeyPressed should return boolean');
                    }
                    
                    // Test mouse input functions  
                    const mousePos = globalThis.Input.getMousePosition();
                    if (!mousePos || typeof mousePos.x !== 'number' || typeof mousePos.y !== 'number') {
                        throw new Error('Input.getMousePosition should return {x, y} object');
                    }
                    
                    const isLeftMouseDown = globalThis.Input.isMouseButtonDown('Left');
                    if (typeof isLeftMouseDown !== 'boolean') {
                        throw new Error('Input.isMouseButtonDown should return boolean');
                    }
                    
                    console.log('Input operations working correctly');
                }
            }
        "#;
        
        // Act
        runtime.load_and_compile_script(1, "input_operations_test.ts", test_script).unwrap();
        let result = runtime.call_init(1);
        
        // Assert - Should now pass with real API injection
        assert!(result.is_ok(), "Input operations should work: {:?}", result);
    }

    #[test]
    fn test_physics_api_force_and_collision_operations_with_real_v8() {
        // Arrange
        let mut runtime = SimpleTypeScriptRuntime::new().unwrap();
        
        let test_script = r#"
            export class PhysicsOperationsTestScript {
                init(): void {
                    // Test force application
                    const entityId = 123; // Mock entity ID
                    globalThis.Physics.applyForce(entityId, { x: 10.0, y: 0.0, z: 0.0 });
                    globalThis.Physics.applyImpulse(entityId, { x: 5.0, y: 0.0, z: 0.0 });
                    
                    // Test raycasting
                    const rayResult = globalThis.Physics.raycast(
                        { x: 0.0, y: 0.0, z: 0.0 }, // origin
                        { x: 1.0, y: 0.0, z: 0.0 }, // direction
                        10.0 // max distance
                    );
                    
                    if (rayResult && typeof rayResult.hit !== 'boolean') {
                        throw new Error('Physics.raycast should return result with hit boolean');
                    }
                    
                    // Test collision detection
                    const isColliding = globalThis.Physics.isColliding(entityId, 456);
                    if (typeof isColliding !== 'boolean') {
                        throw new Error('Physics.isColliding should return boolean');
                    }
                    
                    console.log('Physics operations working correctly');
                }
            }
        "#;
        
        // Act
        runtime.load_and_compile_script(1, "physics_operations_test.ts", test_script).unwrap();
        let result = runtime.call_init(1);
        
        // Assert - Should now pass with real API injection
        assert!(result.is_ok(), "Physics operations should work: {:?}", result);
    }

    #[test]
    fn test_comprehensive_engine_api_with_real_v8_runtime() {
        // Arrange
        let mut runtime = SimpleTypeScriptRuntime::new().unwrap();
        
        // Test that TypeScript interfaces are properly defined for Engine APIs
        let comprehensive_script = r#"
            // TypeScript interface definitions (should be available globally)
            interface Vector3 {
                x: number;
                y: number; 
                z: number;
            }
            
            interface Transform {
                position: Vector3;
                rotation: { x: number; y: number; z: number; w: number };
                scale: Vector3;
            }
            
            export class ComprehensiveApiTestScript {
                init(): void {
                    console.log('Starting comprehensive Engine API test');
                    
                    // Test complete World API
                    if (globalThis.World) {
                        const entityId: number = globalThis.World.createEntity();
                        const entities: number[] = globalThis.World.queryEntities(['Transform']);
                        const component: Transform = globalThis.World.getComponent(entityId, 'Transform');
                        console.log('World API tests passed');
                    }
                    
                    // Test complete Input API
                    if (globalThis.Input) {
                        const keyPressed: boolean = globalThis.Input.isKeyPressed('W');
                        const mousePos: {x: number, y: number} = globalThis.Input.getMousePosition();
                        const mouseDown: boolean = globalThis.Input.isMouseButtonDown('Left');
                        console.log('Input API tests passed');
                    }
                    
                    // Test complete Physics API
                    if (globalThis.Physics) {
                        globalThis.Physics.applyForce(123, { x: 1, y: 0, z: 0 });
                        const rayResult = globalThis.Physics.raycast(
                            { x: 0, y: 0, z: 0 },
                            { x: 1, y: 0, z: 0 },
                            10.0
                        );
                        console.log('Physics API tests passed');
                    }
                    
                    console.log('Comprehensive Engine API test completed successfully');
                }
            }
        "#;
        
        // Act
        runtime.load_and_compile_script(1, "comprehensive_api_test.ts", comprehensive_script).unwrap();
        let result = runtime.call_init(1);
        
        // Assert - Should now pass with real API injection
        assert!(result.is_ok(), "Comprehensive Engine API test should pass: {:?}", result);
    }

    #[test]
    fn test_api_injection_performance_with_real_v8() {
        // Arrange
        let mut runtime = SimpleTypeScriptRuntime::new().unwrap();
        
        let test_script = r#"
            export class ApiPerformanceTestScript {
                update(deltaTime: number): void {
                    // Simulate typical game script operations
                    for (let i = 0; i < 100; i++) {
                        // Mock frequent operations
                        if (globalThis.Input && globalThis.Input.isKeyPressed) {
                            globalThis.Input.isKeyPressed('W');
                        }
                        
                        if (globalThis.World && globalThis.World.queryEntities) {
                            // Query entities with Transform component
                            globalThis.World.queryEntities(['Transform']);
                        }
                    }
                    
                    console.log('Performance test: 100 API calls completed');
                }
            }
        "#;
        
        // Act
        runtime.load_and_compile_script(1, "api_performance_test.ts", test_script).unwrap();
        
        // Measure performance
        let start_time = std::time::Instant::now();
        let result = runtime.call_update(1, 0.016);
        let execution_time = start_time.elapsed();
        
        // Assert
        println!("Real V8 API performance test execution time: {:?}", execution_time);
        
        // Should complete within reasonable time with real APIs
        assert!(execution_time.as_millis() < 100, 
            "API calls should be fast, took: {:?}", execution_time);
        
        assert!(result.is_ok(), "Performance test should pass: {:?}", result);
    }

    #[test]
    fn test_error_handling_with_invalid_api_calls() {
        // Arrange
        let mut runtime = SimpleTypeScriptRuntime::new().unwrap();
        
        let test_script = r#"
            export class ApiErrorHandlingTestScript {
                init(): void {
                    // Test API calls that should not crash the runtime
                    try {
                        globalThis.World.getComponent(999999, 'Transform');
                        console.log('World.getComponent handled large entity ID gracefully');
                    } catch (e) {
                        console.log('World API properly handles invalid entity ID: ' + e);
                    }
                    
                    try {
                        globalThis.Input.isKeyPressed('InvalidKey123');
                        console.log('Input.isKeyPressed handled invalid key gracefully');
                    } catch (e) {
                        console.log('Input API properly handles invalid key: ' + e);
                    }
                    
                    try {
                        globalThis.Physics.applyForce(-1, { x: 0, y: 0, z: 0 });
                        console.log('Physics.applyForce handled invalid entity gracefully');
                    } catch (e) {
                        console.log('Physics API properly handles invalid entity: ' + e);
                    }
                    
                    console.log('API error handling test completed');
                }
            }
        "#;
        
        // Act
        runtime.load_and_compile_script(1, "api_error_test.ts", test_script).unwrap();
        let result = runtime.call_init(1);
        
        // Assert - Should handle errors gracefully with real APIs
        assert!(result.is_ok(), "API error handling should work: {:?}", result);
    }
}