//! Physics system integration for Lua scripting

use crate::ScriptError;
use mlua::{Lua, Function as LuaFunction, Table};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// 3D Vector for physics calculations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
}

/// Rigid body types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RigidBodyType {
    Static,    // No movement
    Dynamic,   // Affected by forces and gravity
    Kinematic, // Controlled by user, not affected by forces
}

/// Collider shapes
#[derive(Debug, Clone)]
pub enum ColliderShape {
    Box { half_extents: Vector3 },
    Sphere { radius: f32 },
    Capsule { height: f32, radius: f32 },
}

/// Handle to a rigid body
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RigidBodyHandle(u64);

static NEXT_BODY_ID: AtomicU64 = AtomicU64::new(1);

impl RigidBodyHandle {
    fn new() -> Self {
        RigidBodyHandle(NEXT_BODY_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Physics event types
#[derive(Debug, Clone)]
pub enum PhysicsEvent {
    CollisionStart(CollisionInfo),
    CollisionEnd(CollisionInfo),
}

/// Collision information
#[derive(Debug, Clone)]
pub struct CollisionInfo {
    pub body_a: RigidBodyHandle,
    pub body_b: RigidBodyHandle,
    pub contact_point: Vector3,
    pub normal: Vector3,
}

/// Ray hit information
#[derive(Debug, Clone)]
pub struct RaycastHit {
    pub body_handle: RigidBodyHandle,
    pub point: Vector3,
    pub normal: Vector3,
    pub distance: f32,
}

/// Simplified rigid body data
struct RigidBody {
    position: Vector3,
    velocity: Vector3,
    body_type: RigidBodyType,
    mass: f32,
    has_collider: bool,
}

/// Lua physics manager for handling physics in scripts
pub struct LuaPhysicsManager {
    rigid_bodies: HashMap<RigidBodyHandle, RigidBody>,
    gravity: Vector3,
    simulation_running: bool,
    collision_callbacks: Vec<String>, // Simplified callback storage
}

impl LuaPhysicsManager {
    /// Create a new physics manager
    pub fn new(_lua: &Lua) -> Result<Self, ScriptError> {
        Ok(Self {
            rigid_bodies: HashMap::new(),
            gravity: Vector3::new(0.0, -9.81, 0.0),
            simulation_running: false,
            collision_callbacks: Vec::new(),
        })
    }
    
    /// Get the number of rigid bodies
    pub fn rigid_body_count(&self) -> usize {
        self.rigid_bodies.len()
    }
    
    /// Check if simulation is running
    pub fn is_simulation_running(&self) -> bool {
        self.simulation_running
    }
    
    /// Start physics simulation
    pub fn start_simulation(&mut self) {
        self.simulation_running = true;
    }
    
    /// Pause physics simulation
    pub fn pause_simulation(&mut self) {
        self.simulation_running = false;
    }
    
    /// Add a rigid body
    pub fn add_rigid_body(
        &mut self,
        position: Vector3,
        body_type: RigidBodyType,
        mass: f32
    ) -> Result<RigidBodyHandle, ScriptError> {
        let handle = RigidBodyHandle::new();
        
        let body = RigidBody {
            position,
            velocity: Vector3::zero(),
            body_type,
            mass,
            has_collider: false,
        };
        
        self.rigid_bodies.insert(handle, body);
        Ok(handle)
    }
    
    /// Remove a rigid body
    pub fn remove_rigid_body(&mut self, handle: RigidBodyHandle) -> Result<(), ScriptError> {
        self.rigid_bodies.remove(&handle)
            .ok_or_else(|| ScriptError::InvalidArguments {
                script_id: None,
                function_name: "remove_rigid_body".to_string(),
                message: "Invalid rigid body handle".to_string(),
                expected: "valid handle".to_string(),
                actual: format!("{:?}", handle),
            })?;
        Ok(())
    }
    
    /// Check if a rigid body exists
    pub fn has_rigid_body(&self, handle: RigidBodyHandle) -> bool {
        self.rigid_bodies.contains_key(&handle)
    }
    
    /// Get rigid body position
    pub fn get_rigid_body_position(&self, handle: RigidBodyHandle) -> Result<Vector3, ScriptError> {
        self.rigid_bodies.get(&handle)
            .map(|body| body.position)
            .ok_or_else(|| ScriptError::InvalidArguments {
                script_id: None,
                function_name: "get_rigid_body_position".to_string(),
                message: "Invalid rigid body handle".to_string(),
                expected: "valid handle".to_string(),
                actual: format!("{:?}", handle),
            })
    }
    
    /// Get rigid body velocity
    pub fn get_rigid_body_velocity(&self, handle: RigidBodyHandle) -> Result<Vector3, ScriptError> {
        self.rigid_bodies.get(&handle)
            .map(|body| body.velocity)
            .ok_or_else(|| ScriptError::InvalidArguments {
                script_id: None,
                function_name: "get_rigid_body_velocity".to_string(),
                message: "Invalid rigid body handle".to_string(),
                expected: "valid handle".to_string(),
                actual: format!("{:?}", handle),
            })
    }
    
    /// Set rigid body velocity
    pub fn set_rigid_body_velocity(&mut self, handle: RigidBodyHandle, velocity: Vector3) -> Result<(), ScriptError> {
        let body = self.rigid_bodies.get_mut(&handle)
            .ok_or_else(|| ScriptError::InvalidArguments {
                script_id: None,
                function_name: "set_rigid_body_velocity".to_string(),
                message: "Invalid rigid body handle".to_string(),
                expected: "valid handle".to_string(),
                actual: format!("{:?}", handle),
            })?;
        
        if body.body_type != RigidBodyType::Static {
            body.velocity = velocity;
        }
        Ok(())
    }
    
    /// Apply force to a rigid body
    pub fn apply_force(&mut self, handle: RigidBodyHandle, force: Vector3) -> Result<(), ScriptError> {
        let body = self.rigid_bodies.get_mut(&handle)
            .ok_or_else(|| ScriptError::InvalidArguments {
                script_id: None,
                function_name: "apply_force".to_string(),
                message: "Invalid rigid body handle".to_string(),
                expected: "valid handle".to_string(),
                actual: format!("{:?}", handle),
            })?;
        
        if body.body_type == RigidBodyType::Dynamic && body.mass > 0.0 {
            // F = ma, so a = F/m
            // For simplicity, directly modify velocity (should be acceleration * dt)
            body.velocity.x += force.x / body.mass * 0.016; // Assume 60 FPS
            body.velocity.y += force.y / body.mass * 0.016;
            body.velocity.z += force.z / body.mass * 0.016;
        }
        Ok(())
    }
    
    /// Apply impulse to a rigid body
    pub fn apply_impulse(&mut self, handle: RigidBodyHandle, impulse: Vector3) -> Result<(), ScriptError> {
        let body = self.rigid_bodies.get_mut(&handle)
            .ok_or_else(|| ScriptError::InvalidArguments {
                script_id: None,
                function_name: "apply_impulse".to_string(),
                message: "Invalid rigid body handle".to_string(),
                expected: "valid handle".to_string(),
                actual: format!("{:?}", handle),
            })?;
        
        if body.body_type == RigidBodyType::Dynamic && body.mass > 0.0 {
            // Impulse = mass * velocity_change
            body.velocity.x += impulse.x / body.mass;
            body.velocity.y += impulse.y / body.mass;
            body.velocity.z += impulse.z / body.mass;
        }
        Ok(())
    }
    
    /// Add a collider to a rigid body
    pub fn add_collider(&mut self, handle: RigidBodyHandle, _shape: ColliderShape) -> Result<(), ScriptError> {
        let body = self.rigid_bodies.get_mut(&handle)
            .ok_or_else(|| ScriptError::InvalidArguments {
                script_id: None,
                function_name: "add_collider".to_string(),
                message: "Invalid rigid body handle".to_string(),
                expected: "valid handle".to_string(),
                actual: format!("{:?}", handle),
            })?;
        
        body.has_collider = true;
        Ok(())
    }
    
    /// Check if a rigid body has a collider
    pub fn has_collider(&self, handle: RigidBodyHandle) -> bool {
        self.rigid_bodies.get(&handle)
            .map(|body| body.has_collider)
            .unwrap_or(false)
    }
    
    /// Register collision start callback
    pub fn on_collision_start(&mut self, _callback: LuaFunction) -> Result<(), ScriptError> {
        // Simplified - just track that we have callbacks
        self.collision_callbacks.push("collision_start".to_string());
        Ok(())
    }
    
    /// Perform a raycast
    pub fn raycast(&self, origin: Vector3, direction: Vector3, max_distance: f32) -> Option<RaycastHit> {
        // Simplified raycast - check against all bodies
        for (handle, body) in &self.rigid_bodies {
            if !body.has_collider {
                continue;
            }
            
            // Simple sphere check for demonstration
            let to_body = Vector3::new(
                body.position.x - origin.x,
                body.position.y - origin.y,
                body.position.z - origin.z,
            );
            
            let distance = (to_body.x * to_body.x + to_body.y * to_body.y + to_body.z * to_body.z).sqrt();
            
            if distance <= max_distance {
                return Some(RaycastHit {
                    body_handle: *handle,
                    point: body.position,
                    normal: Vector3::new(0.0, 1.0, 0.0), // Simplified
                    distance,
                });
            }
        }
        None
    }
    
    /// Get current gravity
    pub fn get_gravity(&self) -> Vector3 {
        self.gravity
    }
    
    /// Set gravity
    pub fn set_gravity(&mut self, gravity: Vector3) {
        self.gravity = gravity;
    }
    
    /// Step the physics simulation
    pub fn step(&mut self, _delta_time: f32) {
        if !self.simulation_running {
            return;
        }
        
        // Simplified physics step
        let bodies_to_update: Vec<_> = self.rigid_bodies.iter()
            .filter(|(_, body)| body.body_type == RigidBodyType::Dynamic)
            .map(|(handle, _)| *handle)
            .collect();
        
        for handle in bodies_to_update {
            if let Some(body) = self.rigid_bodies.get_mut(&handle) {
                // Apply gravity
                if body.mass > 0.0 {
                    body.velocity.y += self.gravity.y * 0.016; // Assume 60 FPS
                }
                
                // Update position
                body.position.x += body.velocity.x * 0.016;
                body.position.y += body.velocity.y * 0.016;
                body.position.z += body.velocity.z * 0.016;
                
                // Simple ground collision at y=0
                if body.position.y < 0.0 {
                    body.position.y = 0.0;
                    body.velocity.y = 0.0;
                    
                    // Trigger collision callbacks (simplified)
                    if !self.collision_callbacks.is_empty() {
                        // In real implementation, would call Lua callbacks
                    }
                }
            }
        }
    }
    
    /// Register the physics API in Lua globals
    pub fn register_api(&self, lua: &Lua) -> Result<(), ScriptError> {
        let globals = lua.globals();
        
        // add_rigid_body function
        let add_rigid_body = lua.create_function(|_, (x, y, z, body_type, mass): (f32, f32, f32, String, f32)| {
            // TODO: Get physics manager instance and add body
            Ok(1u64) // Return dummy handle
        }).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to create add_rigid_body function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        globals.set("add_rigid_body", add_rigid_body).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to set add_rigid_body function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        // remove_rigid_body function
        let remove_rigid_body = lua.create_function(|_, _handle: u64| {
            // TODO: Get physics manager instance and remove body
            Ok(())
        }).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to create remove_rigid_body function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        globals.set("remove_rigid_body", remove_rigid_body).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to set remove_rigid_body function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        // apply_force function
        let apply_force = lua.create_function(|_, (_handle, _fx, _fy, _fz): (u64, f32, f32, f32)| {
            // TODO: Get physics manager instance and apply force
            Ok(())
        }).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to create apply_force function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        globals.set("apply_force", apply_force).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to set apply_force function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        // apply_impulse function
        let apply_impulse = lua.create_function(|_, (_handle, _ix, _iy, _iz): (u64, f32, f32, f32)| {
            // TODO: Get physics manager instance and apply impulse
            Ok(())
        }).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to create apply_impulse function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        globals.set("apply_impulse", apply_impulse).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to set apply_impulse function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        // raycast function
        let raycast = lua.create_function(|_, (_ox, _oy, _oz, _dx, _dy, _dz, _max_dist): (f32, f32, f32, f32, f32, f32, f32)| {
            // TODO: Get physics manager instance and perform raycast
            Ok(mlua::Value::Nil)
        }).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to create raycast function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        globals.set("raycast", raycast).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to set raycast function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        // get_gravity function
        let get_gravity = lua.create_function(|_, ()| {
            // TODO: Get physics manager instance and return gravity
            Ok((0.0, -9.81, 0.0))
        }).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to create get_gravity function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        globals.set("get_gravity", get_gravity).map_err(|e| ScriptError::RuntimeError {
            message: format!("Failed to set get_gravity function: {}", e),
            script_id: None,
            line: None,
            column: None,
            source: None,
        })?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::Lua;
    
    #[test]
    fn test_physics_manager_basic() {
        let lua = Lua::new();
        let mut physics_manager = LuaPhysicsManager::new(&lua).unwrap();
        
        // Test initial state
        assert_eq!(physics_manager.rigid_body_count(), 0);
        assert!(!physics_manager.is_simulation_running());
        
        // Add a rigid body
        let body_handle = physics_manager.add_rigid_body(
            Vector3::new(0.0, 10.0, 0.0),
            RigidBodyType::Dynamic,
            1.0
        ).unwrap();
        
        // Check it was added
        assert_eq!(physics_manager.rigid_body_count(), 1);
        assert!(physics_manager.has_rigid_body(body_handle));
        
        // Get position
        let position = physics_manager.get_rigid_body_position(body_handle).unwrap();
        assert_eq!(position.x, 0.0);
        assert_eq!(position.y, 10.0);
        assert_eq!(position.z, 0.0);
        
        println!("✅ Basic physics manager test passed!");
    }
    
    #[test]
    fn test_physics_simulation() {
        let lua = Lua::new();
        let mut physics_manager = LuaPhysicsManager::new(&lua).unwrap();
        
        // Add a dynamic body
        let body_handle = physics_manager.add_rigid_body(
            Vector3::new(0.0, 10.0, 0.0),
            RigidBodyType::Dynamic,
            1.0
        ).unwrap();
        
        // Start simulation
        physics_manager.start_simulation();
        assert!(physics_manager.is_simulation_running());
        
        // Step simulation several times
        for _ in 0..60 { // Simulate 1 second at 60 FPS
            physics_manager.step(0.016);
        }
        
        // Body should have fallen due to gravity
        let final_position = physics_manager.get_rigid_body_position(body_handle).unwrap();
        assert!(final_position.y < 10.0, "Body should have fallen from initial position");
        
        println!("✅ Physics simulation test passed!");
    }
    
    #[test]
    fn test_forces_and_impulses() {
        let lua = Lua::new();
        let mut physics_manager = LuaPhysicsManager::new(&lua).unwrap();
        
        let body_handle = physics_manager.add_rigid_body(
            Vector3::new(0.0, 0.0, 0.0),
            RigidBodyType::Dynamic,
            1.0
        ).unwrap();
        
        // Apply force
        physics_manager.apply_force(body_handle, Vector3::new(10.0, 0.0, 0.0)).unwrap();
        
        // Get velocity after force application
        let velocity = physics_manager.get_rigid_body_velocity(body_handle).unwrap();
        assert!(velocity.x > 0.0); // Should have positive x velocity
        
        // Apply impulse
        physics_manager.apply_impulse(body_handle, Vector3::new(0.0, 10.0, 0.0)).unwrap();
        
        // Velocity should be immediately changed
        let velocity = physics_manager.get_rigid_body_velocity(body_handle).unwrap();
        assert_eq!(velocity.y, 10.0); // impulse = mass * velocity_change
        
        println!("✅ Forces and impulses test passed!");
    }
}