//! Event system core abstractions
//! 
//! This crate provides core event system abstractions for the mobile game engine.
//! It includes event types, dispatching mechanisms, and component integration.

pub mod events;
pub mod dispatcher;
pub mod handlers;
pub mod queue;
pub mod filters;
pub mod system;
pub mod impls;

// Re-export main types
pub use events::*;
pub use dispatcher::*;
pub use handlers::*;
pub use queue::*;
pub use filters::*;
pub use system::*;
pub use impls::*;

/// Common error type for event system operations
pub type Result<T> = std::result::Result<T, EventError>;

/// Event system errors
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Event queue is full")]
    QueueFull,
    
    #[error("Handler not found: {0}")]
    HandlerNotFound(String),
    
    #[error("Event type not registered: {0}")]
    EventTypeNotRegistered(String),
    
    #[error("Invalid event data: {0}")]
    InvalidEventData(String),
    
    #[error("Event system not initialized")]
    NotInitialized,
}