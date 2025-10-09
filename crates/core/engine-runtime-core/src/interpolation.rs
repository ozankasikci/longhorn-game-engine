//! Interpolation framework for smooth rendering between fixed physics updates
//!
//! This system allows components to be smoothly interpolated for rendering
//! while keeping physics deterministic at fixed timesteps.

use std::fmt::Debug;
use std::collections::HashMap;
use std::any::{Any, TypeId};

/// Trait for components that can be interpolated
pub trait Interpolatable: Send + Sync + Clone + Debug + 'static {
    /// Interpolate between two states with the given factor (0.0 to 1.0)
    fn interpolate(&self, other: &Self, factor: f32) -> Self;
}

/// Error that can occur during interpolation
#[derive(Debug, Clone)]
pub enum InterpolationError {
    /// Component type not registered
    ComponentNotRegistered(String),
    /// Entity not found
    EntityNotFound(u32),
    /// Invalid interpolation factor
    InvalidFactor(f32),
}

impl std::fmt::Display for InterpolationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpolationError::ComponentNotRegistered(msg) => write!(f, "Component not registered: {}", msg),
            InterpolationError::EntityNotFound(id) => write!(f, "Entity not found: {}", id),
            InterpolationError::InvalidFactor(factor) => write!(f, "Invalid interpolation factor: {}", factor),
        }
    }
}

impl std::error::Error for InterpolationError {}

/// Storage for component states for interpolation
#[derive(Debug)]
struct ComponentStateStorage {
    /// Previous state (from last fixed update)
    previous_states: HashMap<u32, Box<dyn Any + Send + Sync>>,
    /// Current state (from current fixed update)
    current_states: HashMap<u32, Box<dyn Any + Send + Sync>>,
    /// Component type name for debugging
    type_name: String,
}

impl ComponentStateStorage {
    fn new(type_name: String) -> Self {
        Self {
            previous_states: HashMap::new(),
            current_states: HashMap::new(),
            type_name,
        }
    }
    
    /// Set the current state for an entity
    fn set_current_state<T: Interpolatable>(&mut self, entity_id: u32, state: T) {
        self.current_states.insert(entity_id, Box::new(state));
    }
    
    /// Set the previous state for an entity
    fn set_previous_state<T: Interpolatable>(&mut self, entity_id: u32, state: T) {
        self.previous_states.insert(entity_id, Box::new(state));
    }
    
    /// Get interpolated state for an entity
    fn get_interpolated_state<T: Interpolatable>(&self, entity_id: u32, factor: f32) -> Result<T, InterpolationError> {
        if factor < 0.0 || factor > 1.0 {
            return Err(InterpolationError::InvalidFactor(factor));
        }
        
        let current = self.current_states.get(&entity_id)
            .ok_or(InterpolationError::EntityNotFound(entity_id))?
            .downcast_ref::<T>()
            .ok_or_else(|| InterpolationError::ComponentNotRegistered(self.type_name.clone()))?;
        
        // If no previous state exists, return current state
        let previous = match self.previous_states.get(&entity_id) {
            Some(prev) => prev.downcast_ref::<T>()
                .ok_or_else(|| InterpolationError::ComponentNotRegistered(self.type_name.clone()))?,
            None => return Ok(current.clone()),
        };
        
        Ok(previous.interpolate(current, factor))
    }
    
    /// Advance to next frame (current becomes previous)
    fn advance_frame(&mut self) {
        std::mem::swap(&mut self.previous_states, &mut self.current_states);
        self.current_states.clear();
    }
    
    /// Remove entity from storage
    fn remove_entity(&mut self, entity_id: u32) {
        self.previous_states.remove(&entity_id);
        self.current_states.remove(&entity_id);
    }
}

/// Manager for component interpolation across all entities
#[derive(Debug)]
pub struct InterpolationManager {
    /// Storage for each component type
    component_storages: HashMap<TypeId, ComponentStateStorage>,
}

impl InterpolationManager {
    /// Create a new interpolation manager
    pub fn new() -> Self {
        Self {
            component_storages: HashMap::new(),
        }
    }
    
    /// Register a component type for interpolation
    pub fn register_component_type<T: Interpolatable>(&mut self) {
        let type_id = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>().to_string();
        
        if !self.component_storages.contains_key(&type_id) {
            self.component_storages.insert(type_id, ComponentStateStorage::new(type_name));
        }
    }
    
    /// Update the current state of a component for an entity
    pub fn update_current_state<T: Interpolatable>(&mut self, entity_id: u32, state: T) -> Result<(), InterpolationError> {
        let type_id = TypeId::of::<T>();
        let storage = self.component_storages.get_mut(&type_id)
            .ok_or_else(|| InterpolationError::ComponentNotRegistered(std::any::type_name::<T>().to_string()))?;
        
        storage.set_current_state(entity_id, state);
        Ok(())
    }
    
    /// Get interpolated state for a component
    pub fn get_interpolated_state<T: Interpolatable>(&self, entity_id: u32, factor: f32) -> Result<T, InterpolationError> {
        let type_id = TypeId::of::<T>();
        let storage = self.component_storages.get(&type_id)
            .ok_or_else(|| InterpolationError::ComponentNotRegistered(std::any::type_name::<T>().to_string()))?;
        
        storage.get_interpolated_state(entity_id, factor)
    }
    
    /// Advance all component states to the next frame
    pub fn advance_frame(&mut self) {
        for storage in self.component_storages.values_mut() {
            storage.advance_frame();
        }
    }
    
    /// Remove an entity from all interpolation storages
    pub fn remove_entity(&mut self, entity_id: u32) {
        for storage in self.component_storages.values_mut() {
            storage.remove_entity(entity_id);
        }
    }
    
    /// Get the number of registered component types
    pub fn registered_component_count(&self) -> usize {
        self.component_storages.len()
    }
    
    /// Check if a component type is registered
    pub fn is_component_registered<T: Interpolatable>(&self) -> bool {
        self.component_storages.contains_key(&TypeId::of::<T>())
    }
}

impl Default for InterpolationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Common interpolatable types

/// 3D position vector
#[derive(Debug, Clone, PartialEq)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

impl Interpolatable for Position3D {
    fn interpolate(&self, other: &Self, factor: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * factor,
            y: self.y + (other.y - self.y) * factor,
            z: self.z + (other.z - self.z) * factor,
        }
    }
}

/// 3D rotation quaternion (simplified)
#[derive(Debug, Clone, PartialEq)]
pub struct Rotation3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Rotation3D {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
    
    pub fn identity() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }
    
    /// Normalize the quaternion
    pub fn normalize(&self) -> Self {
        let length = (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt();
        if length > 0.0 {
            Self {
                x: self.x / length,
                y: self.y / length,
                z: self.z / length,
                w: self.w / length,
            }
        } else {
            Self::identity()
        }
    }
}

impl Interpolatable for Rotation3D {
    fn interpolate(&self, other: &Self, factor: f32) -> Self {
        // Simplified spherical linear interpolation (SLERP)
        // For proper SLERP, we'd need to handle the shortest path and dot product checks
        let dot = self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w;
        
        // Simple linear interpolation for now (can be improved to proper SLERP)
        let result = if dot >= 0.0 {
            Self {
                x: self.x + (other.x - self.x) * factor,
                y: self.y + (other.y - self.y) * factor,
                z: self.z + (other.z - self.z) * factor,
                w: self.w + (other.w - self.w) * factor,
            }
        } else {
            // Flip quaternion to take shorter path
            Self {
                x: self.x + (-other.x - self.x) * factor,
                y: self.y + (-other.y - self.y) * factor,
                z: self.z + (-other.z - self.z) * factor,
                w: self.w + (-other.w - self.w) * factor,
            }
        };
        
        result.normalize()
    }
}

/// 3D scale vector
#[derive(Debug, Clone, PartialEq)]
pub struct Scale3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Scale3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    pub fn uniform(scale: f32) -> Self {
        Self::new(scale, scale, scale)
    }
    
    pub fn one() -> Self {
        Self::uniform(1.0)
    }
}

impl Interpolatable for Scale3D {
    fn interpolate(&self, other: &Self, factor: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * factor,
            y: self.y + (other.y - self.y) * factor,
            z: self.z + (other.z - self.z) * factor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_interpolation_manager_creation() {
        let manager = InterpolationManager::new();
        assert_eq!(manager.registered_component_count(), 0);
    }
    
    #[test]
    fn test_register_component_type() {
        let mut manager = InterpolationManager::new();
        
        manager.register_component_type::<Position3D>();
        assert_eq!(manager.registered_component_count(), 1);
        assert!(manager.is_component_registered::<Position3D>());
    }
    
    #[test]
    fn test_register_multiple_component_types() {
        let mut manager = InterpolationManager::new();
        
        manager.register_component_type::<Position3D>();
        manager.register_component_type::<Rotation3D>();
        manager.register_component_type::<Scale3D>();
        
        assert_eq!(manager.registered_component_count(), 3);
        assert!(manager.is_component_registered::<Position3D>());
        assert!(manager.is_component_registered::<Rotation3D>());
        assert!(manager.is_component_registered::<Scale3D>());
    }
    
    #[test]
    fn test_register_same_type_twice() {
        let mut manager = InterpolationManager::new();
        
        manager.register_component_type::<Position3D>();
        manager.register_component_type::<Position3D>(); // Should not duplicate
        
        assert_eq!(manager.registered_component_count(), 1);
    }
    
    #[test]
    fn test_update_current_state_unregistered() {
        let mut manager = InterpolationManager::new();
        
        let result = manager.update_current_state(1, Position3D::new(1.0, 2.0, 3.0));
        assert!(result.is_err());
        match result.unwrap_err() {
            InterpolationError::ComponentNotRegistered(_) => (),
            _ => panic!("Expected ComponentNotRegistered error"),
        }
    }
    
    #[test]
    fn test_update_and_get_current_state() {
        let mut manager = InterpolationManager::new();
        manager.register_component_type::<Position3D>();
        
        let pos = Position3D::new(1.0, 2.0, 3.0);
        manager.update_current_state(1, pos.clone()).unwrap();
        
        // With no previous state, should return current state
        let interpolated = manager.get_interpolated_state::<Position3D>(1, 0.5).unwrap();
        assert_eq!(interpolated, pos);
    }
    
    #[test]
    fn test_interpolation_with_previous_state() {
        let mut manager = InterpolationManager::new();
        manager.register_component_type::<Position3D>();
        
        // Set up two states for interpolation
        let prev_pos = Position3D::new(0.0, 0.0, 0.0);
        let curr_pos = Position3D::new(10.0, 20.0, 30.0);
        
        // Simulate a frame advance
        manager.update_current_state(1, prev_pos).unwrap();
        manager.advance_frame();
        manager.update_current_state(1, curr_pos).unwrap();
        
        // Test interpolation at various factors
        let interpolated_0 = manager.get_interpolated_state::<Position3D>(1, 0.0).unwrap();
        let interpolated_half = manager.get_interpolated_state::<Position3D>(1, 0.5).unwrap();
        let interpolated_1 = manager.get_interpolated_state::<Position3D>(1, 1.0).unwrap();
        
        assert_eq!(interpolated_0, Position3D::new(0.0, 0.0, 0.0));
        assert_eq!(interpolated_half, Position3D::new(5.0, 10.0, 15.0));
        assert_eq!(interpolated_1, Position3D::new(10.0, 20.0, 30.0));
    }
    
    #[test]
    fn test_invalid_interpolation_factor() {
        let mut manager = InterpolationManager::new();
        manager.register_component_type::<Position3D>();
        
        manager.update_current_state(1, Position3D::new(1.0, 2.0, 3.0)).unwrap();
        
        let result_negative = manager.get_interpolated_state::<Position3D>(1, -0.1);
        let result_too_large = manager.get_interpolated_state::<Position3D>(1, 1.1);
        
        assert!(result_negative.is_err());
        assert!(result_too_large.is_err());
        
        match result_negative.unwrap_err() {
            InterpolationError::InvalidFactor(_) => (),
            _ => panic!("Expected InvalidFactor error"),
        }
    }
    
    #[test]
    fn test_entity_not_found() {
        let mut manager = InterpolationManager::new();
        manager.register_component_type::<Position3D>();
        
        let result = manager.get_interpolated_state::<Position3D>(999, 0.5);
        assert!(result.is_err());
        match result.unwrap_err() {
            InterpolationError::EntityNotFound(999) => (),
            _ => panic!("Expected EntityNotFound error"),
        }
    }
    
    #[test]
    fn test_remove_entity() {
        let mut manager = InterpolationManager::new();
        manager.register_component_type::<Position3D>();
        
        manager.update_current_state(1, Position3D::new(1.0, 2.0, 3.0)).unwrap();
        manager.advance_frame();
        manager.update_current_state(1, Position3D::new(2.0, 4.0, 6.0)).unwrap();
        
        // Verify entity exists
        let result = manager.get_interpolated_state::<Position3D>(1, 0.5);
        assert!(result.is_ok());
        
        // Remove entity
        manager.remove_entity(1);
        
        // Verify entity no longer exists
        let result = manager.get_interpolated_state::<Position3D>(1, 0.5);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_advance_frame() {
        let mut manager = InterpolationManager::new();
        manager.register_component_type::<Position3D>();
        
        // Set current state
        manager.update_current_state(1, Position3D::new(1.0, 2.0, 3.0)).unwrap();
        
        // Without advance_frame, no previous state exists
        let result = manager.get_interpolated_state::<Position3D>(1, 0.0).unwrap();
        assert_eq!(result, Position3D::new(1.0, 2.0, 3.0));
        
        // Advance frame (current becomes previous)
        manager.advance_frame();
        
        // Set new current state
        manager.update_current_state(1, Position3D::new(10.0, 20.0, 30.0)).unwrap();
        
        // Now interpolation should work between previous and current
        let result = manager.get_interpolated_state::<Position3D>(1, 0.0).unwrap();
        assert_eq!(result, Position3D::new(1.0, 2.0, 3.0)); // Previous state
    }
    
    #[test]
    fn test_position3d_interpolation() {
        let pos1 = Position3D::new(0.0, 0.0, 0.0);
        let pos2 = Position3D::new(10.0, 20.0, 30.0);
        
        let interpolated = pos1.interpolate(&pos2, 0.5);
        assert_eq!(interpolated, Position3D::new(5.0, 10.0, 15.0));
    }
    
    #[test]
    fn test_rotation3d_interpolation() {
        let rot1 = Rotation3D::identity();
        let rot2 = Rotation3D::new(0.5, 0.5, 0.5, 0.5);
        
        let interpolated = rot1.interpolate(&rot2, 0.5);
        
        // Result should be normalized
        let length = (interpolated.x * interpolated.x + 
                     interpolated.y * interpolated.y + 
                     interpolated.z * interpolated.z + 
                     interpolated.w * interpolated.w).sqrt();
        assert!((length - 1.0).abs() < 0.001); // Should be approximately 1.0
    }
    
    #[test]
    fn test_scale3d_interpolation() {
        let scale1 = Scale3D::new(1.0, 1.0, 1.0);
        let scale2 = Scale3D::new(2.0, 3.0, 4.0);
        
        let interpolated = scale1.interpolate(&scale2, 0.5);
        assert_eq!(interpolated, Scale3D::new(1.5, 2.0, 2.5));
    }
    
    #[test]
    fn test_multiple_entities() {
        let mut manager = InterpolationManager::new();
        manager.register_component_type::<Position3D>();
        
        // Set up states for multiple entities
        manager.update_current_state(1, Position3D::new(0.0, 0.0, 0.0)).unwrap();
        manager.update_current_state(2, Position3D::new(100.0, 200.0, 300.0)).unwrap();
        
        manager.advance_frame();
        
        manager.update_current_state(1, Position3D::new(10.0, 20.0, 30.0)).unwrap();
        manager.update_current_state(2, Position3D::new(110.0, 220.0, 330.0)).unwrap();
        
        // Test interpolation for both entities
        let entity1_interp = manager.get_interpolated_state::<Position3D>(1, 0.5).unwrap();
        let entity2_interp = manager.get_interpolated_state::<Position3D>(2, 0.5).unwrap();
        
        assert_eq!(entity1_interp, Position3D::new(5.0, 10.0, 15.0));
        assert_eq!(entity2_interp, Position3D::new(105.0, 210.0, 315.0));
    }
}