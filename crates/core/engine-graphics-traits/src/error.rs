use thiserror::Error;

/// Graphics-related errors
#[derive(Error, Debug)]
pub enum GraphicsError {
    /// Device creation failed
    #[error("Failed to create graphics device: {0}")]
    DeviceCreation(String),
    
    /// Resource creation failed
    #[error("Failed to create resource: {0}")]
    ResourceCreation(String),
    
    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    /// Out of memory
    #[error("Out of memory")]
    OutOfMemory,
    
    /// Backend-specific error
    #[error("Backend error: {0}")]
    BackendError(String),
}

/// Result type for graphics operations
pub type Result<T> = std::result::Result<T, GraphicsError>;