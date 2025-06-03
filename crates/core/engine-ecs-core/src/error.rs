//! ECS-specific error types

use std::fmt;

/// Errors that can occur in the ECS system
#[derive(Debug, Clone, PartialEq)]
pub enum EcsError {
    /// Entity not found in the world
    EntityNotFound(crate::Entity),
    
    /// Component type not registered
    ComponentNotRegistered(std::any::TypeId),
    
    /// Component not found on entity
    ComponentNotFound {
        entity: crate::Entity,
        component_type: &'static str,
    },
    
    /// Invalid entity (generation mismatch)
    InvalidEntity(crate::Entity),
    
    /// Archetype not found
    ArchetypeNotFound,
    
    /// Invalid component index
    InvalidComponentIndex {
        index: usize,
        max: usize,
    },
    
    /// Type mismatch when casting components
    TypeMismatch {
        expected: &'static str,
        found: &'static str,
    },
    
    /// Bundle operation failed
    BundleError(String),
}

impl fmt::Display for EcsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EcsError::EntityNotFound(entity) => {
                write!(f, "Entity {:?} not found in world", entity)
            }
            EcsError::ComponentNotRegistered(type_id) => {
                write!(f, "Component type {:?} not registered", type_id)
            }
            EcsError::ComponentNotFound { entity, component_type } => {
                write!(f, "Component {} not found on entity {:?}", component_type, entity)
            }
            EcsError::InvalidEntity(entity) => {
                write!(f, "Invalid entity {:?} (generation mismatch)", entity)
            }
            EcsError::ArchetypeNotFound => {
                write!(f, "Archetype not found")
            }
            EcsError::InvalidComponentIndex { index, max } => {
                write!(f, "Invalid component index {} (max: {})", index, max)
            }
            EcsError::TypeMismatch { expected, found } => {
                write!(f, "Type mismatch: expected {}, found {}", expected, found)
            }
            EcsError::BundleError(msg) => {
                write!(f, "Bundle error: {}", msg)
            }
        }
    }
}

impl std::error::Error for EcsError {}

/// Result type for ECS operations
pub type EcsResult<T> = Result<T, EcsError>;