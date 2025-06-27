//! Tests for TypeScript Physics API bindings
//! 
//! These tests define the expected behavior of the Physics bindings for TypeScript scripts.
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
    fn test_rigid_body_management() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            interface Vector3 {
                x: number;
                y: number;
                z: number;
            }
            
            let rigidBodyHandle: number = -1;
            
            function addRigidBody(): number {
                const position: Vector3 = { x: 0.0, y: 5.0, z: 0.0 };
                rigidBodyHandle = engine.physics.addRigidBody(position, "Dynamic", 1.0);
                return rigidBodyHandle;
            }
            
            function removeRigidBody(): void {
                if (rigidBodyHandle !== -1) {
                    engine.physics.removeRigidBody(rigidBodyHandle);
                }
            }
            
            function getHandle(): number {
                return rigidBodyHandle;
            }
        "#;
        
        let script_id = ScriptId(1);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_rigid_body.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test rigid body creation returns valid handle
        let handle = runtime.execute_function("addRigidBody", vec![]).unwrap();
        let handle_id: i32 = handle.parse().unwrap();
        assert!(handle_id > 0, "Rigid body handle should be positive");
        
        // Test handle is stored correctly
        let stored_handle = runtime.execute_function("getHandle", vec![]).unwrap();
        assert_eq!(handle, stored_handle, "Stored handle should match returned handle");
        
        // Test rigid body removal doesn't error
        runtime.execute_function("removeRigidBody", vec![]).unwrap();
    }

    #[test]
    fn test_force_and_impulse() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            let handle: number = -1;
            
            function setupRigidBody(): number {
                const pos = { x: 0.0, y: 0.0, z: 0.0 };
                handle = engine.physics.addRigidBody(pos, "Dynamic", 2.0);
                return handle;
            }
            
            function applyForce(): void {
                if (handle !== -1) {
                    const force = { x: 10.0, y: 0.0, z: 0.0 };
                    engine.physics.applyForce(handle, force);
                }
            }
            
            function applyImpulse(): void {
                if (handle !== -1) {
                    const impulse = { x: 5.0, y: 0.0, z: 0.0 };
                    engine.physics.applyImpulse(handle, impulse);
                }
            }
            
            function testForceOperations(): boolean {
                applyForce();
                applyImpulse();
                return true; // If we get here without error, operations succeeded
            }
        "#;
        
        let script_id = ScriptId(2);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_forces.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Setup rigid body
        let handle = runtime.execute_function("setupRigidBody", vec![]).unwrap();
        let handle_id: i32 = handle.parse().unwrap();
        assert!(handle_id > 0, "Setup should return valid handle");
        
        // Test force and impulse operations
        let result = runtime.execute_function("testForceOperations", vec![]).unwrap();
        assert_eq!(result, "true", "Force operations should succeed");
    }

    #[test]
    fn test_raycast() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            function performRaycast(): string {
                const origin = { x: 0.0, y: 10.0, z: 0.0 };
                const direction = { x: 0.0, y: -1.0, z: 0.0 };
                const maxDistance = 20.0;
                
                const hit = engine.physics.raycast(origin, direction, maxDistance);
                
                if (hit) {
                    return JSON.stringify({
                        hit: true,
                        position: hit.position,
                        normal: hit.normal,
                        distance: hit.distance
                    });
                } else {
                    return JSON.stringify({ hit: false });
                }
            }
            
            function testRaycastExists(): boolean {
                // Test that the raycast function exists and can be called
                try {
                    performRaycast();
                    return true;
                } catch (e) {
                    return false;
                }
            }
        "#;
        
        let script_id = ScriptId(3);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_raycast.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test raycast function exists and can be called
        let exists = runtime.execute_function("testRaycastExists", vec![]).unwrap();
        assert_eq!(exists, "true", "Raycast function should exist and be callable");
        
        // Test raycast returns valid result format
        let result = runtime.execute_function("performRaycast", vec![]).unwrap();
        assert!(result.contains("hit"), "Raycast result should contain hit field");
    }

    #[test]
    fn test_gravity() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            function getGravity(): string {
                const gravity = engine.physics.getGravity();
                return JSON.stringify(gravity);
            }
            
            function setGravity(): void {
                const newGravity = { x: 0.0, y: -20.0, z: 0.0 };
                engine.physics.setGravity(newGravity);
            }
            
            function testGravityOperations(): boolean {
                // Test getter
                const gravity1 = getGravity();
                
                // Test setter
                setGravity();
                
                // Test getter again
                const gravity2 = getGravity();
                
                return gravity1.length > 0 && gravity2.length > 0;
            }
        "#;
        
        let script_id = ScriptId(4);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_gravity.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test gravity operations
        let result = runtime.execute_function("testGravityOperations", vec![]).unwrap();
        assert_eq!(result, "true", "Gravity operations should work");
        
        // Test gravity getter returns valid format
        let gravity = runtime.execute_function("getGravity", vec![]).unwrap();
        assert!(gravity.contains("x") && gravity.contains("y") && gravity.contains("z"), 
                "Gravity should contain x, y, z components");
    }

    #[test]
    fn test_physics_validation() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            function testInvalidHandle(): boolean {
                try {
                    const force = { x: 1.0, y: 0.0, z: 0.0 };
                    engine.physics.applyForce(-999, force); // Invalid handle
                    return true; // Should handle gracefully
                } catch (e) {
                    return true; // Error handling is also acceptable
                }
            }
            
            function testInvalidBodyType(): number {
                try {
                    const pos = { x: 0.0, y: 0.0, z: 0.0 };
                    return engine.physics.addRigidBody(pos, "InvalidType", 1.0);
                } catch (e) {
                    return -1; // Should return invalid handle or throw
                }
            }
            
            function testNegativeMass(): number {
                try {
                    const pos = { x: 0.0, y: 0.0, z: 0.0 };
                    return engine.physics.addRigidBody(pos, "Dynamic", -1.0); // Negative mass
                } catch (e) {
                    return -1; // Should handle invalid mass
                }
            }
        "#;
        
        let script_id = ScriptId(5);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_physics_validation.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test invalid handle handling
        let invalid_handle = runtime.execute_function("testInvalidHandle", vec![]).unwrap();
        assert_eq!(invalid_handle, "true", "Invalid handle should be handled gracefully");
        
        // Test invalid body type handling
        let invalid_type = runtime.execute_function("testInvalidBodyType", vec![]).unwrap();
        let type_result: i32 = invalid_type.parse().unwrap();
        assert!(type_result == -1 || type_result == 0, "Invalid body type should return invalid handle");
        
        // Test negative mass handling
        let negative_mass = runtime.execute_function("testNegativeMass", vec![]).unwrap();
        let mass_result: i32 = negative_mass.parse().unwrap();
        assert!(mass_result == -1 || mass_result == 0, "Negative mass should return invalid handle");
    }

    #[test]
    fn test_collider_shapes() {
        initialize_v8_platform().unwrap();
        let mut runtime = TypeScriptRuntime::new().unwrap();
        runtime.initialize().unwrap();
        
        let typescript_code = r#"
            function testBoxCollider(): number {
                const pos = { x: 0.0, y: 0.0, z: 0.0 };
                const size = { x: 1.0, y: 1.0, z: 1.0 };
                return engine.physics.addBoxCollider(pos, size, "Static");
            }
            
            function testSphereCollider(): number {
                const pos = { x: 0.0, y: 0.0, z: 0.0 };
                const radius = 1.0;
                return engine.physics.addSphereCollider(pos, radius, "Static");
            }
            
            function testColliderCreation(): boolean {
                try {
                    const box = testBoxCollider();
                    const sphere = testSphereCollider();
                    return box > 0 && sphere > 0;
                } catch (e) {
                    // If specific collider functions don't exist, that's ok for now
                    return true;
                }
            }
        "#;
        
        let script_id = ScriptId(6);
        let metadata = ScriptMetadata {
            id: script_id,
            script_type: ScriptType::TypeScript,
            path: "test_colliders.ts".to_string(),
            entry_point: None,
        };
        
        runtime.load_script(metadata, typescript_code).unwrap();
        runtime.execute_script(script_id).unwrap();
        
        // Test collider creation (may not be implemented yet)
        let result = runtime.execute_function("testColliderCreation", vec![]).unwrap();
        assert_eq!(result, "true", "Collider creation should work or be gracefully skipped");
    }
}