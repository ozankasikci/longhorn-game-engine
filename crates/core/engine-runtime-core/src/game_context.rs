//! Game context that provides access to engine resources during system execution
//!
//! The GameContext acts as a centralized access point for systems to interact with
//! the ECS world, input, resources, and other engine services.

use crate::{TimeManager, RuntimeError};
use engine_input::InputManager;
use std::collections::HashMap;
use std::any::{Any, TypeId};

/// Central context for game systems providing access to engine resources
#[derive(Debug)]
pub struct GameContext {
    /// Time management (delta time, total time, etc.)
    pub time: TimeManager,
    /// Input management
    pub input: InputManager,
    /// Resource storage for arbitrary data
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl GameContext {
    /// Create a new game context
    pub fn new() -> Self {
        Self {
            time: TimeManager::new(), // Default to 60 FPS
            input: InputManager::new().expect("Failed to create InputManager"),
            resources: HashMap::new(),
        }
    }
    
    /// Create a new game context with specified target FPS
    pub fn with_target_fps(target_fps: f64) -> Self {
        let fixed_timestep = std::time::Duration::from_secs_f64(1.0 / target_fps);
        Self {
            time: TimeManager::with_timestep(fixed_timestep),
            input: InputManager::new().expect("Failed to create InputManager"),
            resources: HashMap::new(),
        }
    }
    
    /// Insert a resource into the context
    pub fn insert_resource<T: Send + Sync + 'static>(&mut self, resource: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource));
    }
    
    /// Get a resource from the context
    pub fn get_resource<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|res| res.downcast_ref::<T>())
    }
    
    /// Get a mutable resource from the context
    pub fn get_resource_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .and_then(|res| res.downcast_mut::<T>())
    }
    
    /// Remove a resource from the context
    pub fn remove_resource<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.resources
            .remove(&TypeId::of::<T>())
            .and_then(|res| res.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }
    
    /// Check if a resource exists in the context
    pub fn has_resource<T: Send + Sync + 'static>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<T>())
    }
    
    /// Update the context (typically called once per frame)
    pub fn update(&mut self, _delta_time: f32) -> Result<(), RuntimeError> {
        // Update time manager
        self.time.update();
        
        // Update input manager
        self.input.update();
        
        Ok(())
    }
    
    /// Get the current delta time
    pub fn delta_time(&self) -> f32 {
        self.time.delta_time()
    }
    
    /// Get the current frame count
    pub fn frame_count(&self) -> u64 {
        self.time.frame_count()
    }
}

impl Default for GameContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Debug, PartialEq)]
    struct TestResource {
        value: i32,
    }
    
    #[derive(Debug, PartialEq)]
    struct AnotherResource {
        data: String,
    }
    
    #[test]
    fn test_context_creation() {
        let context = GameContext::new();
        assert!(!context.has_resource::<TestResource>());
    }
    
    #[test]
    fn test_context_with_target_fps() {
        let context = GameContext::with_target_fps(120.0);
        let target_fps = context.time.target_fps();
        assert!((target_fps - 120.0).abs() < 0.01); // Allow for floating point precision
    }
    
    #[test]
    fn test_insert_and_get_resource() {
        let mut context = GameContext::new();
        
        let resource = TestResource { value: 42 };
        context.insert_resource(resource);
        
        assert!(context.has_resource::<TestResource>());
        let retrieved = context.get_resource::<TestResource>().unwrap();
        assert_eq!(retrieved.value, 42);
    }
    
    #[test]
    fn test_get_resource_mut() {
        let mut context = GameContext::new();
        
        let resource = TestResource { value: 42 };
        context.insert_resource(resource);
        
        {
            let resource_mut = context.get_resource_mut::<TestResource>().unwrap();
            resource_mut.value = 100;
        }
        
        let retrieved = context.get_resource::<TestResource>().unwrap();
        assert_eq!(retrieved.value, 100);
    }
    
    #[test]
    fn test_remove_resource() {
        let mut context = GameContext::new();
        
        let resource = TestResource { value: 42 };
        context.insert_resource(resource);
        
        assert!(context.has_resource::<TestResource>());
        
        let removed = context.remove_resource::<TestResource>().unwrap();
        assert_eq!(removed.value, 42);
        assert!(!context.has_resource::<TestResource>());
    }
    
    #[test]
    fn test_multiple_resource_types() {
        let mut context = GameContext::new();
        
        let test_resource = TestResource { value: 42 };
        let another_resource = AnotherResource { data: "hello".to_string() };
        
        context.insert_resource(test_resource);
        context.insert_resource(another_resource);
        
        assert!(context.has_resource::<TestResource>());
        assert!(context.has_resource::<AnotherResource>());
        
        let test_res = context.get_resource::<TestResource>().unwrap();
        let another_res = context.get_resource::<AnotherResource>().unwrap();
        
        assert_eq!(test_res.value, 42);
        assert_eq!(another_res.data, "hello");
    }
    
    #[test]
    fn test_resource_not_found() {
        let context = GameContext::new();
        
        assert!(!context.has_resource::<TestResource>());
        assert!(context.get_resource::<TestResource>().is_none());
    }
    
    #[test]
    fn test_context_update() {
        let mut context = GameContext::new();
        
        let initial_frame = context.frame_count();
        let result = context.update(0.016);
        
        assert!(result.is_ok());
        // Frame count should increment during update
        assert!(context.frame_count() > initial_frame);
    }
    
    #[test]
    fn test_time_access() {
        let mut context = GameContext::new();
        
        // Initial delta time should be 0
        assert_eq!(context.delta_time(), 0.0);
        
        // After update, delta time should be updated
        context.update(0.016).unwrap();
        // Note: The actual delta time will be calculated by TimeManager based on real time
    }
    
    #[test]
    fn test_replace_resource() {
        let mut context = GameContext::new();
        
        // Insert initial resource
        context.insert_resource(TestResource { value: 42 });
        assert_eq!(context.get_resource::<TestResource>().unwrap().value, 42);
        
        // Replace with new resource
        context.insert_resource(TestResource { value: 100 });
        assert_eq!(context.get_resource::<TestResource>().unwrap().value, 100);
    }
    
    #[test]
    fn test_integration_with_system_scheduler() {
        use crate::{SystemScheduler, System, SystemError};
        
        #[derive(Debug)]
        struct TestSystem {
            name: String,
            execution_count: std::sync::Arc<std::sync::Mutex<u32>>,
        }
        
        impl TestSystem {
            fn new(name: &str) -> (Self, std::sync::Arc<std::sync::Mutex<u32>>) {
                let counter = std::sync::Arc::new(std::sync::Mutex::new(0));
                (Self {
                    name: name.to_string(),
                    execution_count: counter.clone(),
                }, counter)
            }
        }
        
        impl System for TestSystem {
            fn execute(&mut self, context: &mut GameContext, _delta_time: f32) -> Result<(), SystemError> {
                // Access a resource from the context
                if let Some(resource) = context.get_resource_mut::<TestResource>() {
                    resource.value += 10;
                }
                
                *self.execution_count.lock().unwrap() += 1;
                Ok(())
            }
            
            fn name(&self) -> &str {
                &self.name
            }
            
            fn is_fixed_timestep(&self) -> bool {
                true
            }
            
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
        
        let mut scheduler = SystemScheduler::new();
        let mut context = GameContext::new();
        
        // Add a resource to the context
        context.insert_resource(TestResource { value: 0 });
        
        // Add systems to the scheduler
        let (system1, counter1) = TestSystem::new("System1");
        let (system2, counter2) = TestSystem::new("System2");
        
        scheduler.add_system(Box::new(system1));
        scheduler.add_system(Box::new(system2));
        
        // Execute systems
        scheduler.execute_fixed_systems(&mut context, 0.016).unwrap();
        
        // Check that both systems executed
        assert_eq!(*counter1.lock().unwrap(), 1);
        assert_eq!(*counter2.lock().unwrap(), 1);
        
        // Check that the resource was modified by both systems
        let resource = context.get_resource::<TestResource>().unwrap();
        assert_eq!(resource.value, 20); // 0 + 10 + 10
    }
}