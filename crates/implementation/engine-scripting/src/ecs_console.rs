//! ECS-based console system to replace global CONSOLE_MESSAGES
//! This implements proper console message handling through ECS resources

use crate::{ScriptId};
use engine_ecs_core::Entity;
use std::collections::VecDeque;
use std::time::SystemTime;

/// Console message that will be stored as ECS resource instead of global
#[derive(Debug, Clone)]
pub struct ConsoleMessage {
    pub message: String,
    pub timestamp: SystemTime,
    pub script_id: Option<ScriptId>,
    pub entity_id: Option<Entity>,
}

/// ECS Resource for console messages instead of global variable
#[derive(Debug)]
pub struct ScriptConsoleResource {
    pub messages: VecDeque<ConsoleMessage>,
    pub max_messages: usize,
}

impl Default for ScriptConsoleResource {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl ScriptConsoleResource {
    pub fn new(max_messages: usize) -> Self {
        Self {
            messages: VecDeque::with_capacity(max_messages),
            max_messages,
        }
    }
    
    pub fn add_message(&mut self, message: ConsoleMessage) {
        if self.messages.len() >= self.max_messages {
            self.messages.pop_front();
        }
        self.messages.push_back(message);
    }
    
    pub fn get_messages(&self) -> &VecDeque<ConsoleMessage> {
        &self.messages
    }
    
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }
    
    pub fn get_and_clear_messages(&mut self) -> Vec<ConsoleMessage> {
        let messages: Vec<_> = self.messages.drain(..).collect();
        messages
    }
}

/// Non-global console handler that can be passed to script engines
pub struct ScriptConsoleHandler {
    console: ScriptConsoleResource,
}

impl ScriptConsoleHandler {
    pub fn new(max_messages: usize) -> Self {
        Self {
            console: ScriptConsoleResource::new(max_messages),
        }
    }
    
    pub fn log(&mut self, message: String, script_id: Option<ScriptId>, entity_id: Option<Entity>) {
        let console_message = ConsoleMessage {
            message,
            timestamp: SystemTime::now(),
            script_id,
            entity_id,
        };
        self.console.add_message(console_message);
    }
    
    pub fn get_messages(&self) -> &VecDeque<ConsoleMessage> {
        self.console.get_messages()
    }
    
    pub fn clear_messages(&mut self) {
        self.console.clear_messages();
    }
    
    pub fn get_and_clear_messages(&mut self) -> Vec<ConsoleMessage> {
        self.console.get_and_clear_messages()
    }
}

/// Trait for script engines to use console without global state
pub trait ConsoleProvider {
    fn log_message(&mut self, message: String, script_id: Option<ScriptId>, entity_id: Option<Entity>);
}

impl ConsoleProvider for ScriptConsoleHandler {
    fn log_message(&mut self, message: String, script_id: Option<ScriptId>, entity_id: Option<Entity>) {
        self.log(message, script_id, entity_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_handler_no_globals() {
        let mut console = ScriptConsoleHandler::new(100);
        
        // Add messages
        console.log("Test message 1".to_string(), Some(ScriptId(1)), None);
        console.log("Test message 2".to_string(), Some(ScriptId(2)), None);
        
        // Verify messages are stored locally, not globally
        assert_eq!(console.get_messages().len(), 2);
        assert_eq!(console.get_messages()[0].message, "Test message 1");
        assert_eq!(console.get_messages()[1].message, "Test message 2");
        
        // Test bounds
        for i in 0..150 {
            console.log(format!("Spam {}", i), Some(ScriptId(100)), None);
        }
        
        // Should be bounded
        assert!(console.get_messages().len() <= 100);
    }
    
    #[test]
    fn test_multiple_console_handlers_isolated() {
        let mut console1 = ScriptConsoleHandler::new(50);
        let mut console2 = ScriptConsoleHandler::new(50);
        
        console1.log("Engine 1 message".to_string(), Some(ScriptId(1)), None);
        console2.log("Engine 2 message".to_string(), Some(ScriptId(2)), None);
        
        // Each console should have its own messages
        assert_eq!(console1.get_messages().len(), 1);
        assert_eq!(console2.get_messages().len(), 1);
        
        assert_eq!(console1.get_messages()[0].message, "Engine 1 message");
        assert_eq!(console2.get_messages()[0].message, "Engine 2 message");
        
        // Messages should not interfere
        assert_ne!(console1.get_messages()[0].message, console2.get_messages()[0].message);
    }
}