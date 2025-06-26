//! Comprehensive error handling for the scripting system
//! Provides detailed error types with context and recovery options

use crate::ScriptId;
use std::fmt;
use thiserror::Error;

/// Comprehensive scripting system errors with rich context
#[derive(Debug, Error)]
pub enum ScriptError {
    #[error("Script compilation failed: {message}")]
    CompilationError {
        message: String,
        script_id: Option<ScriptId>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Script runtime error: {message}")]
    RuntimeError {
        message: String,
        script_id: Option<ScriptId>,
        line: Option<u32>,
        column: Option<u32>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Script not found: {path}")]
    NotFound {
        path: String,
        script_id: Option<ScriptId>,
    },

    #[error("Invalid API call: {message}")]
    InvalidApiCall {
        message: String,
        api_name: String,
        script_id: Option<ScriptId>,
    },

    #[error("Security violation: {violation_type} - {message}")]
    SecurityViolation {
        script_id: ScriptId,
        violation_type: String,
        message: String,
        severity: SecuritySeverity,
    },

    #[error("Resource limit exceeded: {limit_type} (limit: {limit_value}, actual: {actual_value})")]
    ResourceLimitExceeded {
        script_id: ScriptId,
        limit_type: String,
        limit_value: String,
        actual_value: String,
    },

    #[error("Script not loaded: {script_id:?}")]
    ScriptNotLoaded {
        script_id: ScriptId,
    },

    #[error("Invalid script type: {script_type}")]
    InvalidScriptType {
        script_type: String,
        supported_types: Vec<String>,
    },

    #[error("Initialization error: {message}")]
    InitializationError {
        message: String,
        component: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Serialization error: {message}")]
    SerializationError {
        message: String,
        data_type: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Permission denied: {resource} - {action}")]
    PermissionDenied {
        script_id: ScriptId,
        resource: String,
        action: String,
        required_permission: String,
    },

    #[error("Script panic: {message}")]
    ScriptPanic {
        script_id: ScriptId,
        message: String,
        stack_trace: Option<String>,
    },

    #[error("Invalid arguments: {function_name} - {message}")]
    InvalidArguments {
        script_id: Option<ScriptId>,
        function_name: String,
        message: String,
        expected: String,
        actual: String,
    },

    #[error("State corruption detected: {message}")]
    StateCorruption {
        message: String,
        component: String,
        recovery_action: String,
    },

    #[error("Multiple errors occurred")]
    Multiple(Vec<ScriptError>),
}

/// Security violation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

// Backward compatibility - provide function-style constructors
impl ScriptError {
    /// Create a simple compilation error (backward compatible)
    pub fn CompilationError(message: String) -> Self {
        ScriptError::CompilationError {
            message,
            script_id: None,
            source: None,
        }
    }

    /// Create a simple runtime error (backward compatible)
    pub fn RuntimeError(message: String) -> Self {
        ScriptError::RuntimeError {
            message,
            script_id: None,
            line: None,
            column: None,
            source: None,
        }
    }

    /// Create a simple not found error (backward compatible)
    pub fn NotFound(path: String) -> Self {
        ScriptError::NotFound {
            path,
            script_id: None,
        }
    }

    /// Create a simple invalid API call error (backward compatible)
    pub fn InvalidApiCall(message: String) -> Self {
        ScriptError::InvalidApiCall {
            message: message.clone(),
            api_name: "unknown".to_string(),
            script_id: None,
        }
    }
}

impl ScriptError {
    /// Check if the error has context information
    pub fn has_context(&self) -> bool {
        match self {
            ScriptError::RuntimeError { line, column, .. } => line.is_some() || column.is_some(),
            ScriptError::CompilationError { source, .. } => source.is_some(),
            ScriptError::InitializationError { source, .. } => source.is_some(),
            ScriptError::SerializationError { source, .. } => source.is_some(),
            ScriptError::Multiple(errors) => errors.iter().all(|e| e.has_context()),
            _ => true, // Other errors have context through their fields
        }
    }

    /// Get the script ID associated with this error, if any
    pub fn script_id(&self) -> Option<ScriptId> {
        match self {
            ScriptError::CompilationError { script_id, .. } => *script_id,
            ScriptError::RuntimeError { script_id, .. } => *script_id,
            ScriptError::NotFound { script_id, .. } => *script_id,
            ScriptError::InvalidApiCall { script_id, .. } => *script_id,
            ScriptError::SecurityViolation { script_id, .. } => Some(*script_id),
            ScriptError::ResourceLimitExceeded { script_id, .. } => Some(*script_id),
            ScriptError::ScriptNotLoaded { script_id } => Some(*script_id),
            ScriptError::PermissionDenied { script_id, .. } => Some(*script_id),
            ScriptError::ScriptPanic { script_id, .. } => Some(*script_id),
            ScriptError::InvalidArguments { script_id, .. } => *script_id,
            _ => None,
        }
    }

    /// Create a compilation error with context
    pub fn compilation(message: impl Into<String>) -> Self {
        ScriptError::CompilationError {
            message: message.into(),
            script_id: None,
            source: None,
        }
    }

    /// Create a runtime error with context
    pub fn runtime(message: impl Into<String>) -> Self {
        ScriptError::RuntimeError {
            message: message.into(),
            script_id: None,
            line: None,
            column: None,
            source: None,
        }
    }

    /// Add script context to an error
    pub fn with_script_id(mut self, script_id: ScriptId) -> Self {
        match &mut self {
            ScriptError::CompilationError { script_id: id, .. } => *id = Some(script_id),
            ScriptError::RuntimeError { script_id: id, .. } => *id = Some(script_id),
            ScriptError::NotFound { script_id: id, .. } => *id = Some(script_id),
            ScriptError::InvalidApiCall { script_id: id, .. } => *id = Some(script_id),
            ScriptError::InvalidArguments { script_id: id, .. } => *id = Some(script_id),
            _ => {}
        }
        self
    }

    /// Add line/column context to a runtime error
    pub fn with_location(mut self, line: u32, column: u32) -> Self {
        if let ScriptError::RuntimeError { line: l, column: c, .. } = &mut self {
            *l = Some(line);
            *c = Some(column);
        }
        self
    }

    /// Convert old-style error strings to new comprehensive errors
    pub fn from_string(error_type: &str, message: String) -> Self {
        match error_type {
            "CompilationError" => ScriptError::compilation(message),
            "RuntimeError" => ScriptError::runtime(message),
            "NotFound" => ScriptError::NotFound {
                path: message,
                script_id: None,
            },
            "InvalidApiCall" => ScriptError::InvalidApiCall {
                message: message.clone(),
                api_name: "unknown".to_string(),
                script_id: None,
            },
            _ => ScriptError::runtime(message),
        }
    }
}