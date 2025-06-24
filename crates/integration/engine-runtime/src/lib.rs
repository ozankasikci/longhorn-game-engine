//! Runtime system for the mobile game engine
//!
//! This crate orchestrates all engine systems and provides the main
//! game loop and application lifecycle management.

pub mod application;
pub mod engine;
pub mod loop_;
pub mod scheduler;
pub mod systems;

pub use application::{Application, ApplicationBuilder, ApplicationEvent};
pub use engine::Engine;
pub use loop_::GameLoop;
pub use scheduler::{Schedule, System, SystemScheduler};

/// Runtime system errors
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Engine initialization failed: {0}")]
    InitializationError(String),
    #[error("System error: {0}")]
    SystemError(String),
    #[error("Application lifecycle error: {0}")]
    LifecycleError(String),
    #[error("Runtime configuration error: {0}")]
    ConfigurationError(String),
}

/// Runtime system result type
pub type RuntimeResult<T> = Result<T, RuntimeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        // Placeholder test
        assert!(true);
    }
}
