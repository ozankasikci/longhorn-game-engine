//! System scheduler for managing fixed and variable timestep systems
//!
//! The scheduler separates systems into two categories:
//! - Fixed systems: Run at fixed intervals (physics, logic, AI)
//! - Variable systems: Run every frame (rendering, effects, input)

use crate::GameContext;
use std::collections::HashMap;
use std::fmt::Debug;

/// Trait for systems that can be executed by the scheduler
pub trait System: Send + Sync + Debug {
    /// Execute the system
    fn execute(&mut self, context: &mut GameContext, delta_time: f32) -> Result<(), SystemError>;
    
    /// Get the system's name for debugging
    fn name(&self) -> &str;
    
    /// Get the system's dependencies (other systems that must run before this one)
    fn dependencies(&self) -> Vec<&str> {
        Vec::new()
    }
    
    /// Whether this system should run at fixed timestep
    fn is_fixed_timestep(&self) -> bool {
        false
    }
    
    /// Allow downcasting to concrete types
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

/// Error that can occur during system execution
#[derive(Debug, Clone)]
pub enum SystemError {
    /// System execution failed
    ExecutionFailed(String),
    /// Dependency cycle detected
    DependencyCycle(String),
    /// System not found
    SystemNotFound(String),
}

impl std::fmt::Display for SystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemError::ExecutionFailed(msg) => write!(f, "System execution failed: {}", msg),
            SystemError::DependencyCycle(msg) => write!(f, "Dependency cycle detected: {}", msg),
            SystemError::SystemNotFound(msg) => write!(f, "System not found: {}", msg),
        }
    }
}

impl std::error::Error for SystemError {}

/// System scheduler that manages fixed and variable timestep system execution
#[derive(Debug)]
pub struct SystemScheduler {
    /// Systems that run at fixed timestep (physics, logic)
    fixed_systems: Vec<Box<dyn System>>,
    /// Systems that run every frame (rendering, effects)
    variable_systems: Vec<Box<dyn System>>,
    /// Resolved execution order for fixed systems
    fixed_execution_order: Vec<String>,
    /// Resolved execution order for variable systems
    variable_execution_order: Vec<String>,
    /// Whether dependency resolution has been performed
    dependencies_resolved: bool,
}

impl SystemScheduler {
    /// Create a new system scheduler
    pub fn new() -> Self {
        Self {
            fixed_systems: Vec::new(),
            variable_systems: Vec::new(),
            fixed_execution_order: Vec::new(),
            variable_execution_order: Vec::new(),
            dependencies_resolved: false,
        }
    }
    
    /// Add a system to the scheduler
    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.dependencies_resolved = false;
        
        if system.is_fixed_timestep() {
            self.fixed_systems.push(system);
        } else {
            self.variable_systems.push(system);
        }
    }
    
    /// Resolve system dependencies and determine execution order
    pub fn resolve_dependencies(&mut self) -> Result<(), SystemError> {
        self.fixed_execution_order = self.resolve_system_dependencies(&self.fixed_systems)?;
        self.variable_execution_order = self.resolve_system_dependencies(&self.variable_systems)?;
        self.dependencies_resolved = true;
        Ok(())
    }
    
    /// Execute fixed timestep systems
    pub fn execute_fixed_systems(&mut self, context: &mut GameContext, delta_time: f32) -> Result<(), SystemError> {
        if !self.dependencies_resolved {
            self.resolve_dependencies()?;
        }
        
        // Clone the execution order to avoid borrow conflicts
        let execution_order = self.fixed_execution_order.clone();
        
        for system_name in execution_order {
            if let Some(system) = self.find_fixed_system_mut(&system_name) {
                system.execute(context, delta_time)?;
            } else {
                return Err(SystemError::SystemNotFound(system_name));
            }
        }
        
        Ok(())
    }
    
    /// Execute variable timestep systems
    pub fn execute_variable_systems(&mut self, context: &mut GameContext, delta_time: f32) -> Result<(), SystemError> {
        if !self.dependencies_resolved {
            self.resolve_dependencies()?;
        }
        
        // Clone the execution order to avoid borrow conflicts
        let execution_order = self.variable_execution_order.clone();
        
        for system_name in execution_order {
            if let Some(system) = self.find_variable_system_mut(&system_name) {
                system.execute(context, delta_time)?;
            } else {
                return Err(SystemError::SystemNotFound(system_name));
            }
        }
        
        Ok(())
    }
    
    /// Get the number of fixed systems
    pub fn fixed_system_count(&self) -> usize {
        self.fixed_systems.len()
    }
    
    /// Get the number of variable systems
    pub fn variable_system_count(&self) -> usize {
        self.variable_systems.len()
    }
    
    /// Get the execution order for fixed systems
    pub fn fixed_execution_order(&self) -> &[String] {
        &self.fixed_execution_order
    }
    
    /// Get the execution order for variable systems
    pub fn variable_execution_order(&self) -> &[String] {
        &self.variable_execution_order
    }
    
    /// Check if dependencies have been resolved
    pub fn are_dependencies_resolved(&self) -> bool {
        self.dependencies_resolved
    }
    
    /// Resolve dependencies for a set of systems using topological sort
    fn resolve_system_dependencies(&self, systems: &[Box<dyn System>]) -> Result<Vec<String>, SystemError> {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        
        // Build dependency graph
        for system in systems {
            let system_name = system.name().to_string();
            graph.insert(system_name.clone(), Vec::new());
            in_degree.insert(system_name, 0);
        }
        
        // Add edges and calculate in-degrees
        for system in systems {
            let system_name = system.name().to_string();
            for dep in system.dependencies() {
                let dep_name = dep.to_string();
                
                // Check if dependency exists
                if !graph.contains_key(&dep_name) {
                    return Err(SystemError::SystemNotFound(format!(
                        "Dependency '{}' for system '{}' not found", 
                        dep_name, system_name
                    )));
                }
                
                // Add edge from dependency to system
                graph.get_mut(&dep_name).unwrap().push(system_name.clone());
                *in_degree.get_mut(&system_name).unwrap() += 1;
            }
        }
        
        // Topological sort using Kahn's algorithm
        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(name, _)| name.clone())
            .collect();
        
        let mut result = Vec::new();
        
        while let Some(current) = queue.pop() {
            result.push(current.clone());
            
            if let Some(neighbors) = graph.get(&current) {
                for neighbor in neighbors {
                    let degree = in_degree.get_mut(neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(neighbor.clone());
                    }
                }
            }
        }
        
        // Check for cycles
        if result.len() != systems.len() {
            return Err(SystemError::DependencyCycle(
                "Circular dependency detected in system dependencies".to_string()
            ));
        }
        
        Ok(result)
    }
    
    /// Find a mutable reference to a fixed system by name
    fn find_fixed_system_mut(&mut self, name: &str) -> Option<&mut Box<dyn System>> {
        self.fixed_systems.iter_mut().find(|s| s.name() == name)
    }
    
    /// Find a mutable reference to a variable system by name
    fn find_variable_system_mut(&mut self, name: &str) -> Option<&mut Box<dyn System>> {
        self.variable_systems.iter_mut().find(|s| s.name() == name)
    }
    
    /// Find a mutable reference to any system by name (public access)
    pub fn find_system_mut(&mut self, name: &str) -> Option<&mut Box<dyn System>> {
        // First try fixed systems
        if let Some(system) = self.fixed_systems.iter_mut().find(|s| s.name() == name) {
            return Some(system);
        }
        // Then try variable systems
        self.variable_systems.iter_mut().find(|s| s.name() == name)
    }
}

impl Default for SystemScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GameContext;
    use std::sync::{Arc, Mutex};
    
    // Mock system for testing
    #[derive(Debug)]
    struct MockSystem {
        name: String,
        dependencies: Vec<String>,
        is_fixed: bool,
        executed: Arc<Mutex<bool>>,
        execution_order: Arc<Mutex<Vec<String>>>,
    }
    
    impl MockSystem {
        fn new(name: &str, is_fixed: bool) -> Self {
            Self {
                name: name.to_string(),
                dependencies: Vec::new(),
                is_fixed,
                executed: Arc::new(Mutex::new(false)),
                execution_order: Arc::new(Mutex::new(Vec::new())),
            }
        }
        
        fn with_dependencies(mut self, deps: Vec<&str>) -> Self {
            self.dependencies = deps.into_iter().map(|s| s.to_string()).collect();
            self
        }
        
        fn with_execution_tracker(mut self, tracker: Arc<Mutex<Vec<String>>>) -> Self {
            self.execution_order = tracker;
            self
        }
        
        fn was_executed(&self) -> bool {
            *self.executed.lock().unwrap()
        }
    }
    
    impl System for MockSystem {
        fn execute(&mut self, _context: &mut GameContext, _delta_time: f32) -> Result<(), SystemError> {
            *self.executed.lock().unwrap() = true;
            self.execution_order.lock().unwrap().push(self.name.clone());
            Ok(())
        }
        
        fn name(&self) -> &str {
            &self.name
        }
        
        fn dependencies(&self) -> Vec<&str> {
            self.dependencies.iter().map(|s| s.as_str()).collect()
        }
        
        fn is_fixed_timestep(&self) -> bool {
            self.is_fixed
        }
        
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }
    
    // Error system for testing
    #[derive(Debug)]
    struct ErrorSystem {
        name: String,
        is_fixed: bool,
    }
    
    impl ErrorSystem {
        fn new(name: &str, is_fixed: bool) -> Self {
            Self {
                name: name.to_string(),
                is_fixed,
            }
        }
    }
    
    impl System for ErrorSystem {
        fn execute(&mut self, _context: &mut GameContext, _delta_time: f32) -> Result<(), SystemError> {
            Err(SystemError::ExecutionFailed("Test error".to_string()))
        }
        
        fn name(&self) -> &str {
            &self.name
        }
        
        fn is_fixed_timestep(&self) -> bool {
            self.is_fixed
        }
        
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }
    
    #[test]
    fn test_scheduler_creation() {
        let scheduler = SystemScheduler::new();
        assert_eq!(scheduler.fixed_system_count(), 0);
        assert_eq!(scheduler.variable_system_count(), 0);
        assert!(!scheduler.are_dependencies_resolved());
    }
    
    #[test]
    fn test_add_fixed_system() {
        let mut scheduler = SystemScheduler::new();
        let system = MockSystem::new("PhysicsSystem", true);
        
        scheduler.add_system(Box::new(system));
        
        assert_eq!(scheduler.fixed_system_count(), 1);
        assert_eq!(scheduler.variable_system_count(), 0);
    }
    
    #[test]
    fn test_add_variable_system() {
        let mut scheduler = SystemScheduler::new();
        let system = MockSystem::new("RenderSystem", false);
        
        scheduler.add_system(Box::new(system));
        
        assert_eq!(scheduler.fixed_system_count(), 0);
        assert_eq!(scheduler.variable_system_count(), 1);
    }
    
    #[test]
    fn test_add_mixed_systems() {
        let mut scheduler = SystemScheduler::new();
        
        scheduler.add_system(Box::new(MockSystem::new("PhysicsSystem", true)));
        scheduler.add_system(Box::new(MockSystem::new("RenderSystem", false)));
        scheduler.add_system(Box::new(MockSystem::new("AISystem", true)));
        
        assert_eq!(scheduler.fixed_system_count(), 2);
        assert_eq!(scheduler.variable_system_count(), 1);
    }
    
    #[test]
    fn test_resolve_dependencies_no_deps() {
        let mut scheduler = SystemScheduler::new();
        
        scheduler.add_system(Box::new(MockSystem::new("SystemA", true)));
        scheduler.add_system(Box::new(MockSystem::new("SystemB", true)));
        
        let result = scheduler.resolve_dependencies();
        assert!(result.is_ok());
        assert!(scheduler.are_dependencies_resolved());
        assert_eq!(scheduler.fixed_execution_order().len(), 2);
    }
    
    #[test]
    fn test_resolve_dependencies_with_deps() {
        let mut scheduler = SystemScheduler::new();
        
        // SystemB depends on SystemA
        scheduler.add_system(Box::new(MockSystem::new("SystemA", true)));
        scheduler.add_system(Box::new(MockSystem::new("SystemB", true).with_dependencies(vec!["SystemA"])));
        
        let result = scheduler.resolve_dependencies();
        assert!(result.is_ok());
        
        let order = scheduler.fixed_execution_order();
        assert_eq!(order.len(), 2);
        assert_eq!(order[0], "SystemA");
        assert_eq!(order[1], "SystemB");
    }
    
    #[test]
    fn test_resolve_dependencies_complex() {
        let mut scheduler = SystemScheduler::new();
        
        // Create dependency chain: A -> B -> C, A -> D
        scheduler.add_system(Box::new(MockSystem::new("SystemA", true)));
        scheduler.add_system(Box::new(MockSystem::new("SystemB", true).with_dependencies(vec!["SystemA"])));
        scheduler.add_system(Box::new(MockSystem::new("SystemC", true).with_dependencies(vec!["SystemB"])));
        scheduler.add_system(Box::new(MockSystem::new("SystemD", true).with_dependencies(vec!["SystemA"])));
        
        let result = scheduler.resolve_dependencies();
        assert!(result.is_ok());
        
        let order = scheduler.fixed_execution_order();
        assert_eq!(order.len(), 4);
        assert_eq!(order[0], "SystemA");
        // SystemB and SystemD can be in any order after SystemA
        assert!(order[1] == "SystemB" || order[1] == "SystemD");
        assert!(order[2] == "SystemB" || order[2] == "SystemD");
        assert_eq!(order[3], "SystemC"); // SystemC must be last
    }
    
    #[test]
    fn test_resolve_dependencies_cycle_detection() {
        let mut scheduler = SystemScheduler::new();
        
        // Create circular dependency: A -> B -> A
        scheduler.add_system(Box::new(MockSystem::new("SystemA", true).with_dependencies(vec!["SystemB"])));
        scheduler.add_system(Box::new(MockSystem::new("SystemB", true).with_dependencies(vec!["SystemA"])));
        
        let result = scheduler.resolve_dependencies();
        assert!(result.is_err());
        match result.unwrap_err() {
            SystemError::DependencyCycle(_) => (),
            _ => panic!("Expected DependencyCycle error"),
        }
    }
    
    #[test]
    fn test_resolve_dependencies_missing_dependency() {
        let mut scheduler = SystemScheduler::new();
        
        // SystemA depends on non-existent SystemB
        scheduler.add_system(Box::new(MockSystem::new("SystemA", true).with_dependencies(vec!["SystemB"])));
        
        let result = scheduler.resolve_dependencies();
        assert!(result.is_err());
        match result.unwrap_err() {
            SystemError::SystemNotFound(_) => (),
            _ => panic!("Expected SystemNotFound error"),
        }
    }
    
    #[test]
    fn test_separate_fixed_and_variable_dependencies() {
        let mut scheduler = SystemScheduler::new();
        
        // Fixed systems: A -> B
        scheduler.add_system(Box::new(MockSystem::new("FixedA", true)));
        scheduler.add_system(Box::new(MockSystem::new("FixedB", true).with_dependencies(vec!["FixedA"])));
        
        // Variable systems: C -> D
        scheduler.add_system(Box::new(MockSystem::new("VariableC", false)));
        scheduler.add_system(Box::new(MockSystem::new("VariableD", false).with_dependencies(vec!["VariableC"])));
        
        let result = scheduler.resolve_dependencies();
        assert!(result.is_ok());
        
        let fixed_order = scheduler.fixed_execution_order();
        let variable_order = scheduler.variable_execution_order();
        
        assert_eq!(fixed_order.len(), 2);
        assert_eq!(fixed_order[0], "FixedA");
        assert_eq!(fixed_order[1], "FixedB");
        
        assert_eq!(variable_order.len(), 2);
        assert_eq!(variable_order[0], "VariableC");
        assert_eq!(variable_order[1], "VariableD");
    }
    
    #[test]
    fn test_execute_fixed_systems() {
        let mut scheduler = SystemScheduler::new();
        let execution_tracker = Arc::new(Mutex::new(Vec::new()));
        
        let system_a = MockSystem::new("SystemA", true)
            .with_execution_tracker(execution_tracker.clone());
        let system_b = MockSystem::new("SystemB", true)
            .with_dependencies(vec!["SystemA"])
            .with_execution_tracker(execution_tracker.clone());
        
        scheduler.add_system(Box::new(system_a));
        scheduler.add_system(Box::new(system_b));
        
        // Mock GameContext (you'll need to implement this)
        let mut context = GameContext::new();  // This will need to be implemented
        
        let result = scheduler.execute_fixed_systems(&mut context, 0.016);
        assert!(result.is_ok());
        
        let execution_order = execution_tracker.lock().unwrap();
        assert_eq!(execution_order.len(), 2);
        assert_eq!(execution_order[0], "SystemA");
        assert_eq!(execution_order[1], "SystemB");
    }
    
    #[test]
    fn test_execute_variable_systems() {
        let mut scheduler = SystemScheduler::new();
        let execution_tracker = Arc::new(Mutex::new(Vec::new()));
        
        let system_a = MockSystem::new("SystemA", false)
            .with_execution_tracker(execution_tracker.clone());
        let system_b = MockSystem::new("SystemB", false)
            .with_dependencies(vec!["SystemA"])
            .with_execution_tracker(execution_tracker.clone());
        
        scheduler.add_system(Box::new(system_a));
        scheduler.add_system(Box::new(system_b));
        
        let mut context = GameContext::new();
        
        let result = scheduler.execute_variable_systems(&mut context, 0.016);
        assert!(result.is_ok());
        
        let execution_order = execution_tracker.lock().unwrap();
        assert_eq!(execution_order.len(), 2);
        assert_eq!(execution_order[0], "SystemA");
        assert_eq!(execution_order[1], "SystemB");
    }
    
    #[test]
    fn test_execute_systems_with_error() {
        let mut scheduler = SystemScheduler::new();
        
        scheduler.add_system(Box::new(MockSystem::new("SystemA", true)));
        scheduler.add_system(Box::new(ErrorSystem::new("ErrorSystem", true))); // Make it a fixed system
        
        let mut context = GameContext::new();
        
        let result = scheduler.execute_fixed_systems(&mut context, 0.016);
        assert!(result.is_err());
        match result.unwrap_err() {
            SystemError::ExecutionFailed(_) => (),
            _ => panic!("Expected ExecutionFailed error"),
        }
    }
    
    #[test]
    fn test_auto_resolve_dependencies_on_execute() {
        let mut scheduler = SystemScheduler::new();
        
        scheduler.add_system(Box::new(MockSystem::new("SystemA", true)));
        scheduler.add_system(Box::new(MockSystem::new("SystemB", true).with_dependencies(vec!["SystemA"])));
        
        assert!(!scheduler.are_dependencies_resolved());
        
        let mut context = GameContext::new();
        let result = scheduler.execute_fixed_systems(&mut context, 0.016);
        assert!(result.is_ok());
        assert!(scheduler.are_dependencies_resolved());
    }
    
    #[test]
    fn test_adding_system_invalidates_resolution() {
        let mut scheduler = SystemScheduler::new();
        
        scheduler.add_system(Box::new(MockSystem::new("SystemA", true)));
        scheduler.resolve_dependencies().unwrap();
        assert!(scheduler.are_dependencies_resolved());
        
        // Adding a new system should invalidate resolution
        scheduler.add_system(Box::new(MockSystem::new("SystemB", true)));
        assert!(!scheduler.are_dependencies_resolved());
    }
}