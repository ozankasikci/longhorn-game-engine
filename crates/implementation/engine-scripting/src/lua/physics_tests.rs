//! Tests for Lua physics system integration

#[cfg(test)]
mod tests {
    use crate::lua::physics::{
        LuaPhysicsManager, RigidBodyHandle, RigidBodyType, ColliderShape,
        PhysicsEvent, CollisionInfo, Vector3
    };
    use crate::ScriptError;
    use mlua::{Lua, Function as LuaFunction};
    use std::sync::{Arc, Mutex};
    
    #[test]
    fn test_physics_manager_creation() {
        let lua = Lua::new();
        let physics_manager = LuaPhysicsManager::new(&lua).unwrap();
        
        assert_eq!(physics_manager.rigid_body_count(), 0);
        assert!(!physics_manager.is_simulation_running());
    }
    
    #[test]
    fn test_add_rigid_body() {
        let lua = Lua::new();
        let mut physics_manager = LuaPhysicsManager::new(&lua).unwrap();
        
        // Add dynamic rigid body
        let handle = physics_manager.add_rigid_body(
            Vector3::new(0.0, 10.0, 0.0),
            RigidBodyType::Dynamic,
            1.0 // mass
        ).unwrap();
        
        assert_eq!(physics_manager.rigid_body_count(), 1);
        assert!(physics_manager.has_rigid_body(handle));
        
        // Verify position
        let position = physics_manager.get_rigid_body_position(handle).unwrap();
        assert_eq!(position.x, 0.0);
        assert_eq!(position.y, 10.0);
        assert_eq!(position.z, 0.0);
    }
    
    #[test]
    fn test_add_static_body() {
        let lua = Lua::new();
        let mut physics_manager = LuaPhysicsManager::new(&lua).unwrap();
        
        // Add static rigid body (no mass)
        let handle = physics_manager.add_rigid_body(
            Vector3::new(0.0, 0.0, 0.0),
            RigidBodyType::Static,
            0.0
        ).unwrap();
        
        assert!(physics_manager.has_rigid_body(handle));
    }
    
    #[test]
    fn test_remove_rigid_body() {
        let lua = Lua::new();
        let mut physics_manager = LuaPhysicsManager::new(&lua).unwrap();
        
        let handle = physics_manager.add_rigid_body(
            Vector3::new(0.0, 0.0, 0.0),
            RigidBodyType::Dynamic,
            1.0
        ).unwrap();
        
        assert_eq!(physics_manager.rigid_body_count(), 1);
        
        physics_manager.remove_rigid_body(handle).unwrap();
        
        assert_eq!(physics_manager.rigid_body_count(), 0);
        assert!(!physics_manager.has_rigid_body(handle));
    }
    
    #[test]
    fn test_apply_force() {
        let lua = Lua::new();
        let mut physics_manager = LuaPhysicsManager::new(&lua).unwrap();
        
        let handle = physics_manager.add_rigid_body(
            Vector3::new(0.0, 0.0, 0.0),
            RigidBodyType::Dynamic,
            1.0
        ).unwrap();
        
        // Apply force
        physics_manager.apply_force(handle, Vector3::new(10.0, 0.0, 0.0)).unwrap();
        
        // Get velocity after force application
        let velocity = physics_manager.get_rigid_body_velocity(handle).unwrap();
        assert!(velocity.x > 0.0); // Should have positive x velocity
    }
    
    #[test]
    fn test_apply_impulse() {
        let lua = Lua::new();
        let mut physics_manager = LuaPhysicsManager::new(&lua).unwrap();
        
        let handle = physics_manager.add_rigid_body(
            Vector3::new(0.0, 0.0, 0.0),
            RigidBodyType::Dynamic,
            1.0
        ).unwrap();
        
        // Apply impulse
        physics_manager.apply_impulse(handle, Vector3::new(0.0, 10.0, 0.0)).unwrap();
        
        // Velocity should be immediately changed
        let velocity = physics_manager.get_rigid_body_velocity(handle).unwrap();
        assert_eq!(velocity.y, 10.0); // impulse = mass * velocity_change
    }
    
    #[test]
    fn test_set_velocity() {
        let lua = Lua::new();
        let mut physics_manager = LuaPhysicsManager::new(&lua).unwrap();
        
        let handle = physics_manager.add_rigid_body(
            Vector3::new(0.0, 0.0, 0.0),
            RigidBodyType::Dynamic,
            1.0
        ).unwrap();
        
        // Set velocity directly
        physics_manager.set_rigid_body_velocity(handle, Vector3::new(5.0, 10.0, -3.0)).unwrap();
        
        let velocity = physics_manager.get_rigid_body_velocity(handle).unwrap();
        assert_eq!(velocity.x, 5.0);
        assert_eq!(velocity.y, 10.0);
        assert_eq!(velocity.z, -3.0);
    }
    
    #[test]
    fn test_add_collider() {
        let lua = Lua::new();
        let mut physics_manager = LuaPhysicsManager::new(&lua).unwrap();
        
        let handle = physics_manager.add_rigid_body(
            Vector3::new(0.0, 0.0, 0.0),
            RigidBodyType::Dynamic,
            1.0
        ).unwrap();
        
        // Add box collider
        physics_manager.add_collider(
            handle,
            ColliderShape::Box { 
                half_extents: Vector3::new(1.0, 1.0, 1.0) 
            }
        ).unwrap();
        
        assert!(physics_manager.has_collider(handle));
    }
    
    #[test]
    fn test_collision_detection() {
        let lua = Lua::new();
        let mut physics_manager = LuaPhysicsManager::new(&lua).unwrap();
        
        // Create two bodies that will collide
        let body1 = physics_manager.add_rigid_body(
            Vector3::new(0.0, 10.0, 0.0),
            RigidBodyType::Dynamic,
            1.0
        ).unwrap();
        physics_manager.add_collider(
            body1,
            ColliderShape::Box { half_extents: Vector3::new(1.0, 1.0, 1.0) }
        ).unwrap();
        
        let body2 = physics_manager.add_rigid_body(
            Vector3::new(0.0, 0.0, 0.0),
            RigidBodyType::Static,
            0.0
        ).unwrap();
        physics_manager.add_collider(
            body2,
            ColliderShape::Box { half_extents: Vector3::new(10.0, 1.0, 10.0) }
        ).unwrap();
        
        // Register collision callback
        let collision_detected = Arc::new(Mutex::new(false));
        let collision_detected_clone = collision_detected.clone();
        
        let callback: LuaFunction = lua.create_function(move |_, (handle1, handle2): (u64, u64)| {
            *collision_detected_clone.lock().unwrap() = true;
            Ok(())
        }).unwrap();
        
        physics_manager.on_collision_start(callback).unwrap();
        
        // Simulate physics for a few steps
        for _ in 0..10 {
            physics_manager.step(0.016); // 60 FPS
        }
        
        // Check if collision was detected
        assert!(*collision_detected.lock().unwrap());
    }
    
    #[test]
    fn test_raycast() {
        let lua = Lua::new();
        let mut physics_manager = LuaPhysicsManager::new(&lua).unwrap();
        
        // Create a body to hit
        let body = physics_manager.add_rigid_body(
            Vector3::new(5.0, 0.0, 0.0),
            RigidBodyType::Static,
            0.0
        ).unwrap();
        physics_manager.add_collider(
            body,
            ColliderShape::Box { half_extents: Vector3::new(1.0, 1.0, 1.0) }
        ).unwrap();
        
        // Cast ray from origin towards the body
        let hit_info = physics_manager.raycast(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            10.0 // max distance
        );
        
        assert!(hit_info.is_some());
        let hit = hit_info.unwrap();
        assert_eq!(hit.body_handle, body);
        assert!(hit.distance > 3.0 && hit.distance < 5.0); // Should hit the near face
    }
    
    #[test]
    fn test_gravity_settings() {
        let lua = Lua::new();
        let mut physics_manager = LuaPhysicsManager::new(&lua).unwrap();
        
        // Default gravity
        let gravity = physics_manager.get_gravity();
        assert_eq!(gravity.y, -9.81);
        
        // Set custom gravity
        physics_manager.set_gravity(Vector3::new(0.0, -20.0, 0.0));
        
        let new_gravity = physics_manager.get_gravity();
        assert_eq!(new_gravity.y, -20.0);
    }
    
    #[test]
    fn test_physics_api_registration() {
        let lua = Lua::new();
        let physics_manager = LuaPhysicsManager::new(&lua).unwrap();
        
        // Register API
        physics_manager.register_api(&lua).unwrap();
        
        // Check that functions are available
        let globals = lua.globals();
        assert!(globals.get::<_, LuaFunction>("add_rigid_body").is_ok());
        assert!(globals.get::<_, LuaFunction>("remove_rigid_body").is_ok());
        assert!(globals.get::<_, LuaFunction>("apply_force").is_ok());
        assert!(globals.get::<_, LuaFunction>("apply_impulse").is_ok());
        assert!(globals.get::<_, LuaFunction>("raycast").is_ok());
        
        // Test using the API from Lua
        lua.load(r#"
            local gravity_x, gravity_y, gravity_z = get_gravity()
            assert(gravity_y == -9.81)
        "#).exec().unwrap();
    }
    
    #[test]
    fn test_simulation_control() {
        let lua = Lua::new();
        let mut physics_manager = LuaPhysicsManager::new(&lua).unwrap();
        
        // Initially not running
        assert!(!physics_manager.is_simulation_running());
        
        // Start simulation
        physics_manager.start_simulation();
        assert!(physics_manager.is_simulation_running());
        
        // Pause simulation
        physics_manager.pause_simulation();
        assert!(!physics_manager.is_simulation_running());
    }
}