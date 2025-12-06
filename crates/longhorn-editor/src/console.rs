// crates/longhorn-editor/src/console.rs
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Log level for console entries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleLevel {
    Log,
    Warn,
    Error,
}

/// A single console entry
#[derive(Debug, Clone)]
pub struct ConsoleEntry {
    pub level: ConsoleLevel,
    pub message: String,
    pub timestamp: Instant,
}

/// Maximum number of console entries to retain
const MAX_ENTRIES: usize = 1000;

/// Shared console buffer for script output
#[derive(Clone)]
pub struct ScriptConsole {
    entries: Arc<Mutex<Vec<ConsoleEntry>>>,
}

impl ScriptConsole {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a log entry
    pub fn log(&self, message: String) {
        self.push(ConsoleLevel::Log, message);
    }

    /// Add a warning entry
    pub fn warn(&self, message: String) {
        self.push(ConsoleLevel::Warn, message);
    }

    /// Add an error entry
    pub fn error(&self, message: String) {
        self.push(ConsoleLevel::Error, message);
    }

    fn push(&self, level: ConsoleLevel, message: String) {
        let mut entries = self.entries.lock().unwrap();

        // Drop oldest entries if at capacity
        if entries.len() >= MAX_ENTRIES {
            entries.remove(0);
        }

        entries.push(ConsoleEntry {
            level,
            message,
            timestamp: Instant::now(),
        });

        // Also log to file via log crate
        match level {
            ConsoleLevel::Log => log::info!(target: "script", "{}", entries.last().unwrap().message),
            ConsoleLevel::Warn => log::warn!(target: "script", "{}", entries.last().unwrap().message),
            ConsoleLevel::Error => log::error!(target: "script", "{}", entries.last().unwrap().message),
        }
    }

    /// Get all entries (for UI display)
    pub fn entries(&self) -> Vec<ConsoleEntry> {
        self.entries.lock().unwrap().clone()
    }

    /// Clear all entries
    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.lock().unwrap().is_empty()
    }
}

impl Default for ScriptConsole {
    fn default() -> Self {
        Self::new()
    }
}
