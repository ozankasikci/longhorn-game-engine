//! Scripting system for the mobile game engine
//!
//! This crate provides scripting capabilities for game logic,
//! allowing runtime behavior modification and extensibility.

pub mod api;
pub mod bindings;
pub mod manager;
pub mod runtime;
pub mod types;

pub use api::ScriptApi;
pub use manager::ScriptManager;
pub use runtime::ScriptRuntime;
pub use types::{ScriptId, ScriptMetadata, ScriptType};

/// Scripting system errors
#[derive(Debug, thiserror::Error)]
pub enum ScriptError {
    #[error("Script compilation failed: {0}")]
    CompilationError(String),
    #[error("Script runtime error: {0}")]
    RuntimeError(String),
    #[error("Script not found: {0}")]
    NotFound(String),
    #[error("Invalid script API call: {0}")]
    InvalidApiCall(String),
}

/// Scripting system result type
pub type ScriptResult<T> = Result<T, ScriptError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_manager_creation() {
        // Placeholder test
        assert!(true);
    }
}
