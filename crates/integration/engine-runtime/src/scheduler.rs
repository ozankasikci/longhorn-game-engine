//! System scheduler for managing engine systems

use crate::{RuntimeError, RuntimeResult};
use std::collections::HashMap;

/// System trait for engine systems
pub trait System {
    /// Initialize the system
    fn initialize(&mut self) -> RuntimeResult<()>;

    /// Update the system
    fn update(&mut self, delta_time: f32) -> RuntimeResult<()>;

    /// Shutdown the system
    fn shutdown(&mut self) -> RuntimeResult<()>;

    /// Get system name
    fn name(&self) -> &str;
}

/// System scheduler for managing system execution order
pub struct SystemScheduler {
    systems: HashMap<String, Box<dyn System>>,
    execution_order: Vec<String>,
}

/// Schedule definition for system execution
pub struct Schedule {
    systems: Vec<String>,
}

impl SystemScheduler {
    /// Create a new system scheduler
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
            execution_order: Vec::new(),
        }
    }

    /// Add a system to the scheduler
    pub fn add_system<S: System + 'static>(&mut self, system: S) -> RuntimeResult<()> {
        let name = system.name().to_string();
        self.systems.insert(name.clone(), Box::new(system));
        self.execution_order.push(name);
        Ok(())
    }

    /// Set system execution order
    pub fn set_schedule(&mut self, schedule: Schedule) -> RuntimeResult<()> {
        // Validate that all systems in schedule exist
        for system_name in &schedule.systems {
            if !self.systems.contains_key(system_name) {
                return Err(RuntimeError::ConfigurationError(format!(
                    "System '{}' not found",
                    system_name
                )));
            }
        }

        self.execution_order = schedule.systems;
        Ok(())
    }

    /// Initialize all systems
    pub fn initialize(&mut self) -> RuntimeResult<()> {
        for system_name in &self.execution_order.clone() {
            if let Some(system) = self.systems.get_mut(system_name) {
                system.initialize()?;
            }
        }
        Ok(())
    }

    /// Update all systems
    pub fn update(&mut self, delta_time: f32) -> RuntimeResult<()> {
        for system_name in &self.execution_order.clone() {
            if let Some(system) = self.systems.get_mut(system_name) {
                system.update(delta_time)?;
            }
        }
        Ok(())
    }

    /// Shutdown all systems
    pub fn shutdown(&mut self) -> RuntimeResult<()> {
        // Shutdown in reverse order
        for system_name in self.execution_order.iter().rev() {
            if let Some(system) = self.systems.get_mut(system_name) {
                system.shutdown()?;
            }
        }
        Ok(())
    }
}

impl Schedule {
    /// Create a new schedule
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    /// Add a system to the schedule
    pub fn add_system(mut self, system_name: &str) -> Self {
        self.systems.push(system_name.to_string());
        self
    }
}
