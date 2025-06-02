//! Physics world abstractions

use glam::Vec3;
use serde::{Serialize, Deserialize};
use crate::Result;

/// Physics world configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicsWorldConfig {
    /// Gravity vector
    pub gravity: Vec3,
    /// Number of velocity iterations per step
    pub velocity_iterations: u32,
    /// Number of position iterations per step
    pub position_iterations: u32,
    /// Fixed timestep for physics simulation
    pub fixed_timestep: f32,
    /// Maximum sub-steps per frame
    pub max_substeps: u32,
    /// Whether to allow sleeping bodies
    pub allow_sleeping: bool,
    /// Sleep threshold velocity
    pub sleep_threshold: f32,
    /// Time before a body can sleep
    pub time_to_sleep: f32,
    /// Default contact skin width
    pub contact_skin_width: f32,
    /// Broad phase algorithm
    pub broad_phase: BroadPhaseAlgorithm,
}

impl Default for PhysicsWorldConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            velocity_iterations: 8,
            position_iterations: 3,
            fixed_timestep: 1.0 / 60.0,
            max_substeps: 4,
            allow_sleeping: true,
            sleep_threshold: 0.01,
            time_to_sleep: 0.5,
            contact_skin_width: 0.01,
            broad_phase: BroadPhaseAlgorithm::DynamicBvh,
        }
    }
}

/// Broad phase collision detection algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BroadPhaseAlgorithm {
    /// Dynamic bounding volume hierarchy (good for general use)
    DynamicBvh,
    /// Spatial hashing (good for uniform distribution)
    SpatialHash,
    /// Sweep and prune (good for many objects in a line)
    SweepAndPrune,
}

/// Physics world statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicsWorldStats {
    /// Number of active rigid bodies
    pub active_bodies: u32,
    /// Number of sleeping bodies
    pub sleeping_bodies: u32,
    /// Number of colliders
    pub colliders: u32,
    /// Number of joints
    pub joints: u32,
    /// Number of collision pairs this frame
    pub collision_pairs: u32,
    /// Time spent in physics simulation (ms)
    pub simulation_time: f32,
    /// Time spent in broad phase (ms)
    pub broad_phase_time: f32,
    /// Time spent in narrow phase (ms)
    pub narrow_phase_time: f32,
    /// Time spent in constraint solving (ms)
    pub solver_time: f32,
    /// Memory used by physics world (bytes)
    pub memory_usage: usize,
}

/// Physics world trait for implementation by specific physics backends
pub trait PhysicsWorld {
    /// Initialize the physics world
    fn initialize(&mut self, config: PhysicsWorldConfig) -> Result<()>;
    
    /// Step the physics simulation
    fn step(&mut self, delta_time: f32) -> Result<()>;
    
    /// Set gravity
    fn set_gravity(&mut self, gravity: Vec3) -> Result<()>;
    
    /// Get current gravity
    fn get_gravity(&self) -> Vec3;
    
    /// Add a rigid body to the world
    fn add_rigid_body(&mut self, body: crate::RigidBody) -> Result<crate::BodyHandle>;
    
    /// Remove a rigid body from the world
    fn remove_rigid_body(&mut self, handle: crate::BodyHandle) -> Result<()>;
    
    /// Add a collider to the world
    fn add_collider(&mut self, collider: crate::Collider) -> Result<crate::ColliderHandle>;
    
    /// Remove a collider from the world
    fn remove_collider(&mut self, handle: crate::ColliderHandle) -> Result<()>;
    
    /// Add a joint to the world
    fn add_joint(&mut self, joint: crate::Joint) -> Result<u32>;
    
    /// Remove a joint from the world
    fn remove_joint(&mut self, handle: u32) -> Result<()>;
    
    /// Get world statistics
    fn get_stats(&self) -> PhysicsWorldStats;
    
    /// Clear all physics objects
    fn clear(&mut self) -> Result<()>;
    
    /// Enable/disable physics simulation
    fn set_enabled(&mut self, enabled: bool);
    
    /// Check if physics simulation is enabled
    fn is_enabled(&self) -> bool;
    
    /// Set simulation timestep
    fn set_timestep(&mut self, timestep: f32);
    
    /// Get current timestep
    fn get_timestep(&self) -> f32;
}

impl PhysicsWorldConfig {
    /// Create config for 2D physics
    pub fn config_2d() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            ..Self::default()
        }
    }
    
    /// Create config for 3D physics
    pub fn config_3d() -> Self {
        Self::default()
    }
    
    /// Create config for platformer games
    pub fn platformer() -> Self {
        Self {
            gravity: Vec3::new(0.0, -20.0, 0.0),
            fixed_timestep: 1.0 / 60.0,
            velocity_iterations: 10,
            position_iterations: 4,
            ..Self::default()
        }
    }
    
    /// Create config for space/zero gravity
    pub fn zero_gravity() -> Self {
        Self {
            gravity: Vec3::ZERO,
            ..Self::default()
        }
    }
    
    /// Create config optimized for mobile
    pub fn mobile_optimized() -> Self {
        Self {
            velocity_iterations: 6,
            position_iterations: 2,
            max_substeps: 2,
            allow_sleeping: true,
            broad_phase: BroadPhaseAlgorithm::SpatialHash,
            ..Self::default()
        }
    }
}