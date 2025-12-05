use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Entity identifier - wraps hecs::Entity
pub type EntityId = hecs::Entity;

/// Asset identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(pub u64);

impl AssetId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Result type alias for Longhorn operations
pub type Result<T> = std::result::Result<T, LonghornError>;

/// Error types for Longhorn engine
#[derive(Debug, Error)]
pub enum LonghornError {
    #[error("Entity not found: {0:?}")]
    EntityNotFound(EntityId),

    #[error("Component not found for entity: {0:?}")]
    ComponentNotFound(EntityId),

    #[error("Asset not found: {0:?}")]
    AssetNotFound(AssetId),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Rendering error: {0}")]
    Rendering(String),

    #[error("Scripting error: {0}")]
    Scripting(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}
