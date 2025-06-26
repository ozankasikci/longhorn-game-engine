//! Resource limits for script execution safety
//! This module provides mechanisms to prevent scripts from consuming excessive resources

use std::time::{Duration, Instant};

/// Resource limits for script execution
#[derive(Debug, Clone)]
pub struct ScriptResourceLimits {
    /// Maximum memory usage in bytes (default: 1GB)
    pub max_memory_bytes: usize,
    /// Maximum execution time per script call (default: 10 seconds)
    pub max_execution_time: Duration,
    /// Maximum recursion depth (default: 10,000)
    pub max_recursion_depth: u32,
    /// Maximum string length (default: 10MB)
    pub max_string_length: usize,
}

impl Default for ScriptResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 1024 * 1024 * 1024, // 1GB
            max_execution_time: Duration::from_secs(10), // 10 seconds
            max_recursion_depth: 10000, // 10,000 levels
            max_string_length: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// Execution context with resource tracking
pub struct ScriptExecutionContext {
    limits: ScriptResourceLimits,
    start_time: Option<Instant>,
    recursion_depth: u32,
}

impl ScriptExecutionContext {
    pub fn new(limits: ScriptResourceLimits) -> Self {
        Self {
            limits,
            start_time: None,
            recursion_depth: 0,
        }
    }

    pub fn start_execution(&mut self) {
        self.start_time = Some(Instant::now());
        self.recursion_depth = 0;
    }

    pub fn check_timeout(&self) -> Result<(), crate::ScriptError> {
        if let Some(start) = self.start_time {
            if start.elapsed() > self.limits.max_execution_time {
                return Err(crate::ScriptError::RuntimeError(
                    "Script execution timeout".to_string()
                ));
            }
        }
        Ok(())
    }

    pub fn check_recursion_depth(&mut self) -> Result<(), crate::ScriptError> {
        self.recursion_depth += 1;
        if self.recursion_depth > self.limits.max_recursion_depth {
            return Err(crate::ScriptError::RuntimeError(
                "Maximum recursion depth exceeded".to_string()
            ));
        }
        Ok(())
    }

    pub fn decrease_recursion_depth(&mut self) {
        if self.recursion_depth > 0 {
            self.recursion_depth -= 1;
        }
    }

    pub fn limits(&self) -> &ScriptResourceLimits {
        &self.limits
    }

    pub fn current_recursion_depth(&self) -> u32 {
        self.recursion_depth
    }

    pub fn elapsed_time(&self) -> Option<Duration> {
        self.start_time.map(|start| start.elapsed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_limits_default() {
        let limits = ScriptResourceLimits::default();
        assert_eq!(limits.max_memory_bytes, 1024 * 1024 * 1024);
        assert_eq!(limits.max_execution_time, Duration::from_secs(10));
        assert_eq!(limits.max_recursion_depth, 10000);
        assert_eq!(limits.max_string_length, 10 * 1024 * 1024);
    }

    #[test]
    fn test_execution_context_timeout() {
        let limits = ScriptResourceLimits {
            max_execution_time: Duration::from_millis(10),
            ..Default::default()
        };
        let mut context = ScriptExecutionContext::new(limits);
        
        context.start_execution();
        
        // Should not timeout immediately
        assert!(context.check_timeout().is_ok());
        
        // Wait longer than timeout
        std::thread::sleep(Duration::from_millis(15));
        
        // Should now timeout
        assert!(context.check_timeout().is_err());
    }

    #[test]
    fn test_recursion_depth_tracking() {
        let limits = ScriptResourceLimits {
            max_recursion_depth: 3,
            ..Default::default()
        };
        let mut context = ScriptExecutionContext::new(limits);
        
        // Should allow up to max depth
        assert!(context.check_recursion_depth().is_ok()); // depth 1
        assert!(context.check_recursion_depth().is_ok()); // depth 2  
        assert!(context.check_recursion_depth().is_ok()); // depth 3
        
        // Should fail on exceeding max depth
        assert!(context.check_recursion_depth().is_err()); // depth 4 - should fail
    }

    #[test]
    fn test_recursion_depth_decrease() {
        let limits = ScriptResourceLimits {
            max_recursion_depth: 2,
            ..Default::default()
        };
        let mut context = ScriptExecutionContext::new(limits);
        
        assert!(context.check_recursion_depth().is_ok()); // depth 1
        assert!(context.check_recursion_depth().is_ok()); // depth 2
        
        // Should fail on next increase
        assert!(context.check_recursion_depth().is_err()); // depth 3 - should fail
        
        // Decrease and try again
        context.decrease_recursion_depth(); // back to depth 2
        context.decrease_recursion_depth(); // back to depth 1
        
        // Should work again
        assert!(context.check_recursion_depth().is_ok()); // depth 2
    }
}