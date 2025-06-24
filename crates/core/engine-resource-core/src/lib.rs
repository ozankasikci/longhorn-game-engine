//! Resource management abstractions for the mobile game engine
//!
//! This crate provides type-safe resource handles, loading states, and cache
//! management abstractions. It defines the contracts for resource loading
//! without implementing specific loaders.

pub mod cache;
pub mod handle;
pub mod loader;
pub mod manager;
pub mod metadata;
pub mod state;

// Re-export main types for convenience
pub use cache::ResourceCache;
pub use handle::{Resource, ResourceHandle, ResourceId, WeakResourceHandle};
pub use loader::{LoaderError, ResourceLoader};
pub use manager::CachePolicy;
pub use manager::{ResourceManager, ResourceManagerError};
pub use metadata::{ResourceMetadata, ResourceType};
pub use state::{LoadingState, ResourceState};

/// Resource system errors
#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("Resource not found: {id}")]
    NotFound { id: ResourceId },

    #[error("Resource loading failed: {reason}")]
    LoadingFailed { reason: String },

    #[error("Resource type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("Resource is not loaded")]
    NotLoaded,

    #[error("Resource loading in progress")]
    LoadingInProgress,

    #[error("Resource cache full")]
    CacheFull,

    #[error("Invalid resource path: {path}")]
    InvalidPath { path: String },

    #[error("Resource dependency cycle detected")]
    DependencyCycle,

    #[error("Loader error: {0}")]
    Loader(#[from] LoaderError),

    #[error("Manager error: {0}")]
    Manager(#[from] ResourceManagerError),
}

/// Resource system result type
pub type ResourceResult<T> = Result<T, ResourceError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_error_creation() {
        let error = ResourceError::NotFound {
            id: ResourceId::new(42),
        };
        assert!(matches!(error, ResourceError::NotFound { .. }));
    }
}
