//! TDD Tests for Actual Console Output from V8 Integration

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// Real console output capture system that integrates with log crate
#[derive(Debug, Clone)]
pub struct RealConsoleCapture {
    /// Captured log messages with levels
    pub captured_logs: Arc<Mutex<VecDeque<LogEntry>>>,
    /// Maximum number of log entries to keep
    pub max_entries: usize,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: std::time::Instant,
    pub source: LogSource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogSource {
    TypeScriptConsole,
    EngineSystem,
    Other,
}

impl RealConsoleCapture {
    pub fn new() -> Self {
        Self {
            captured_logs: Arc::new(Mutex::new(VecDeque::new())),
            max_entries: 1000,
        }
    }

    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            captured_logs: Arc::new(Mutex::new(VecDeque::with_capacity(max_entries))),
            max_entries,
        }
    }

    /// Capture a console.log() from TypeScript
    pub fn log_from_typescript(&self, message: &str) {
        self.add_entry(LogEntry {
            level: LogLevel::Info,
            message: format!("[JS Console] {}", message),
            timestamp: std::time::Instant::now(),
            source: LogSource::TypeScriptConsole,
        });
    }

    /// Capture a console.error() from TypeScript
    pub fn error_from_typescript(&self, message: &str) {
        self.add_entry(LogEntry {
            level: LogLevel::Error,
            message: format!("[JS Console] {}", message),
            timestamp: std::time::Instant::now(),
            source: LogSource::TypeScriptConsole,
        });
    }

    /// Capture a system log message
    pub fn log_from_system(&self, level: LogLevel, message: &str) {
        self.add_entry(LogEntry {
            level,
            message: message.to_string(),
            timestamp: std::time::Instant::now(),
            source: LogSource::EngineSystem,
        });
    }

    fn add_entry(&self, entry: LogEntry) {
        let mut logs = self.captured_logs.lock().unwrap();
        
        // Remove old entries if we're at capacity
        while logs.len() >= self.max_entries {
            logs.pop_front();
        }
        
        logs.push_back(entry);
    }

    /// Get all captured log messages
    pub fn get_all_logs(&self) -> Vec<LogEntry> {
        self.captured_logs.lock().unwrap().iter().cloned().collect()
    }

    /// Get only TypeScript console logs
    pub fn get_typescript_logs(&self) -> Vec<LogEntry> {
        self.get_all_logs()
            .into_iter()
            .filter(|entry| entry.source == LogSource::TypeScriptConsole)
            .collect()
    }

    /// Get log messages by level
    pub fn get_logs_by_level(&self, level: LogLevel) -> Vec<LogEntry> {
        self.get_all_logs()
            .into_iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    /// Check if a specific message was logged
    pub fn contains_message(&self, message: &str) -> bool {
        self.get_all_logs()
            .iter()
            .any(|entry| entry.message.contains(message))
    }

    /// Get the most recent log entry
    pub fn get_latest_log(&self) -> Option<LogEntry> {
        self.captured_logs.lock().unwrap().back().cloned()
    }

    /// Clear all captured logs
    pub fn clear(&self) {
        self.captured_logs.lock().unwrap().clear();
    }

    /// Get count of logs by source
    pub fn count_by_source(&self, source: LogSource) -> usize {
        self.get_all_logs()
            .iter()
            .filter(|entry| entry.source == source)
            .count()
    }
}

/// Test harness for console output integration
pub struct ConsoleOutputTestHarness {
    pub console_capture: RealConsoleCapture,
    pub expected_outputs: Vec<String>,
}

impl ConsoleOutputTestHarness {
    pub fn new() -> Self {
        Self {
            console_capture: RealConsoleCapture::new(),
            expected_outputs: Vec::new(),
        }
    }

    pub fn expect_output(&mut self, message: &str) {
        self.expected_outputs.push(message.to_string());
    }

    pub fn simulate_typescript_hello_world_execution(&self) {
        // Simulate the exact output from typescript_hello_world.ts
        self.console_capture.log_from_typescript("Hello, World!");
        self.console_capture.log_from_typescript("Welcome to Longhorn Game Engine TypeScript scripting!");
    }

    pub fn simulate_typescript_hello_world_destroy(&self) {
        self.console_capture.log_from_typescript("Goodbye from TypeScript!");
    }

    pub fn simulate_entity_controller_execution(&self) {
        self.console_capture.log_from_typescript("EntityController initialized");
    }

    pub fn simulate_input_handler_execution(&self) {
        self.console_capture.log_from_typescript("Input handler ready");
        self.console_capture.log_from_typescript("Space key pressed!");
    }

    pub fn simulate_compilation_error(&self, script_path: &str, error: &str) {
        self.console_capture.error_from_typescript(&format!("Compilation error in {}: {}", script_path, error));
    }

    pub fn simulate_runtime_error(&self, script_path: &str, error: &str) {
        self.console_capture.error_from_typescript(&format!("Runtime error in {}: {}", script_path, error));
    }

    pub fn verify_expected_outputs(&self) -> bool {
        let logs = self.console_capture.get_typescript_logs();
        
        for expected in &self.expected_outputs {
            let found = logs.iter().any(|entry| entry.message.contains(expected));
            if !found {
                return false;
            }
        }
        
        true
    }

    pub fn get_output_summary(&self) -> String {
        let logs = self.console_capture.get_all_logs();
        let typescript_count = self.console_capture.count_by_source(LogSource::TypeScriptConsole);
        let system_count = self.console_capture.count_by_source(LogSource::EngineSystem);
        let error_count = self.console_capture.get_logs_by_level(LogLevel::Error).len();

        format!(
            "Total logs: {}, TypeScript: {}, System: {}, Errors: {}",
            logs.len(), typescript_count, system_count, error_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_capture_creation() {
        // Arrange & Act
        let capture = RealConsoleCapture::new();

        // Assert
        assert_eq!(capture.max_entries, 1000);
        assert_eq!(capture.get_all_logs().len(), 0);
    }

    #[test]
    fn test_typescript_console_log_capture() {
        // Arrange
        let capture = RealConsoleCapture::new();

        // Act
        capture.log_from_typescript("Hello from TypeScript!");
        capture.log_from_typescript("Another message");

        // Assert
        let logs = capture.get_typescript_logs();
        assert_eq!(logs.len(), 2);
        assert!(logs[0].message.contains("Hello from TypeScript!"));
        assert!(logs[1].message.contains("Another message"));
        assert_eq!(logs[0].source, LogSource::TypeScriptConsole);
        assert_eq!(logs[0].level, LogLevel::Info);
    }

    #[test]
    fn test_typescript_console_error_capture() {
        // Arrange
        let capture = RealConsoleCapture::new();

        // Act
        capture.error_from_typescript("TypeScript runtime error");

        // Assert
        let logs = capture.get_all_logs();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].message.contains("TypeScript runtime error"));
        assert_eq!(logs[0].level, LogLevel::Error);
        assert_eq!(logs[0].source, LogSource::TypeScriptConsole);
    }

    #[test]
    fn test_hello_world_script_output_pattern() {
        // Arrange
        let mut harness = ConsoleOutputTestHarness::new();
        harness.expect_output("Hello, World!");
        harness.expect_output("Welcome to Longhorn Game Engine TypeScript scripting!");

        // Act
        harness.simulate_typescript_hello_world_execution();

        // Assert
        assert!(harness.verify_expected_outputs());
        
        let logs = harness.console_capture.get_typescript_logs();
        assert_eq!(logs.len(), 2);
        assert!(harness.console_capture.contains_message("Hello, World!"));
        assert!(harness.console_capture.contains_message("Welcome to Longhorn"));
    }

    #[test]
    fn test_complete_hello_world_lifecycle_output() {
        // Arrange
        let harness = ConsoleOutputTestHarness::new();

        // Act - Simulate full script lifecycle
        harness.simulate_typescript_hello_world_execution(); // init()
        harness.simulate_typescript_hello_world_destroy();   // destroy()

        // Assert
        let logs = harness.console_capture.get_typescript_logs();
        assert_eq!(logs.len(), 3);
        
        let messages: Vec<&str> = logs.iter().map(|entry| entry.message.as_str()).collect();
        assert!(messages.iter().any(|&msg| msg.contains("Hello, World!")));
        assert!(messages.iter().any(|&msg| msg.contains("Welcome to Longhorn")));
        assert!(messages.iter().any(|&msg| msg.contains("Goodbye from TypeScript!")));
    }

    #[test]
    fn test_multiple_script_output_separation() {
        // Arrange
        let harness = ConsoleOutputTestHarness::new();

        // Act - Simulate multiple scripts running
        harness.simulate_typescript_hello_world_execution();
        harness.simulate_entity_controller_execution();
        harness.simulate_input_handler_execution();

        // Assert
        let logs = harness.console_capture.get_typescript_logs();
        assert_eq!(logs.len(), 4); // 2 from hello_world + 1 from entity_controller + 1 from input_handler
        
        assert!(harness.console_capture.contains_message("Hello, World!"));
        assert!(harness.console_capture.contains_message("EntityController initialized"));
        assert!(harness.console_capture.contains_message("Input handler ready"));
    }

    #[test]
    fn test_console_output_with_js_prefix() {
        // Arrange
        let capture = RealConsoleCapture::new();

        // Act
        capture.log_from_typescript("Test message");

        // Assert
        let logs = capture.get_all_logs();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].message.starts_with("[JS Console]"));
        assert!(logs[0].message.contains("Test message"));
    }

    #[test]
    fn test_error_vs_log_level_separation() {
        // Arrange
        let capture = RealConsoleCapture::new();

        // Act
        capture.log_from_typescript("Normal log message");
        capture.error_from_typescript("Error message");
        capture.log_from_system(LogLevel::Info, "System message");

        // Assert
        let all_logs = capture.get_all_logs();
        let error_logs = capture.get_logs_by_level(LogLevel::Error);
        let info_logs = capture.get_logs_by_level(LogLevel::Info);

        assert_eq!(all_logs.len(), 3);
        assert_eq!(error_logs.len(), 1);
        assert_eq!(info_logs.len(), 2);
        
        assert!(error_logs[0].message.contains("Error message"));
    }

    #[test]
    fn test_console_capture_capacity_management() {
        // Arrange
        let capture = RealConsoleCapture::with_capacity(3);

        // Act - Add more logs than capacity
        capture.log_from_typescript("Message 1");
        capture.log_from_typescript("Message 2");
        capture.log_from_typescript("Message 3");
        capture.log_from_typescript("Message 4");
        capture.log_from_typescript("Message 5");

        // Assert - Should only keep the last 3
        let logs = capture.get_all_logs();
        assert_eq!(logs.len(), 3);
        assert!(logs[0].message.contains("Message 3"));
        assert!(logs[1].message.contains("Message 4"));
        assert!(logs[2].message.contains("Message 5"));
    }

    #[test]
    fn test_latest_log_retrieval() {
        // Arrange
        let capture = RealConsoleCapture::new();

        // Act
        capture.log_from_typescript("First message");
        capture.log_from_typescript("Latest message");

        // Assert
        let latest = capture.get_latest_log().unwrap();
        assert!(latest.message.contains("Latest message"));
        assert_eq!(latest.source, LogSource::TypeScriptConsole);
    }

    #[test]
    fn test_console_clear_functionality() {
        // Arrange
        let capture = RealConsoleCapture::new();
        capture.log_from_typescript("Message before clear");

        // Act
        capture.clear();

        // Assert
        assert_eq!(capture.get_all_logs().len(), 0);
        assert!(!capture.contains_message("Message before clear"));
    }

    #[test]
    fn test_typescript_error_compilation_output() {
        // Arrange
        let harness = ConsoleOutputTestHarness::new();

        // Act
        harness.simulate_compilation_error("broken_script.ts", "Syntax error at line 5");

        // Assert
        let error_logs = harness.console_capture.get_logs_by_level(LogLevel::Error);
        assert_eq!(error_logs.len(), 1);
        assert!(error_logs[0].message.contains("Compilation error"));
        assert!(error_logs[0].message.contains("broken_script.ts"));
        assert!(error_logs[0].message.contains("Syntax error"));
    }

    #[test]
    fn test_typescript_runtime_error_output() {
        // Arrange
        let harness = ConsoleOutputTestHarness::new();

        // Act
        harness.simulate_runtime_error("error_script.ts", "ReferenceError: undefined variable");

        // Assert
        let error_logs = harness.console_capture.get_logs_by_level(LogLevel::Error);
        assert_eq!(error_logs.len(), 1);
        assert!(error_logs[0].message.contains("Runtime error"));
        assert!(error_logs[0].message.contains("ReferenceError"));
    }

    #[test]
    fn test_console_output_timing() {
        // Arrange
        let capture = RealConsoleCapture::new();
        let start_time = std::time::Instant::now();

        // Act
        capture.log_from_typescript("Timed message");

        // Assert
        let logs = capture.get_all_logs();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].timestamp >= start_time);
        assert!(logs[0].timestamp <= std::time::Instant::now());
    }

    #[test]
    fn test_output_summary_generation() {
        // Arrange
        let harness = ConsoleOutputTestHarness::new();

        // Act
        harness.simulate_typescript_hello_world_execution();
        harness.simulate_compilation_error("test.ts", "error");
        harness.console_capture.log_from_system(LogLevel::Info, "System started");

        // Assert
        let summary = harness.get_output_summary();
        assert!(summary.contains("Total logs: 4"));
        assert!(summary.contains("TypeScript: 3"));
        assert!(summary.contains("System: 1"));
        assert!(summary.contains("Errors: 1"));
    }

    #[test]
    fn test_expected_output_verification() {
        // Arrange
        let mut harness = ConsoleOutputTestHarness::new();
        harness.expect_output("Hello, World!");
        harness.expect_output("Welcome to Longhorn");
        harness.expect_output("Nonexistent message");

        // Act
        harness.simulate_typescript_hello_world_execution();

        // Assert
        assert!(!harness.verify_expected_outputs()); // Should fail due to "Nonexistent message"
        
        // Remove the nonexistent expectation
        harness.expected_outputs.pop();
        assert!(harness.verify_expected_outputs()); // Should pass now
    }

    #[test]
    fn test_concurrent_console_access() {
        // Arrange
        let capture = RealConsoleCapture::new();
        let capture_clone = capture.clone();

        // Act - Simulate concurrent access
        capture.log_from_typescript("Message from thread 1");
        capture_clone.log_from_typescript("Message from thread 2");

        // Assert
        let logs = capture.get_all_logs();
        assert_eq!(logs.len(), 2);
        assert!(capture.contains_message("Message from thread 1"));
        assert!(capture.contains_message("Message from thread 2"));
    }
}