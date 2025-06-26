//! Runtime system for the mobile game engine
//!
//! This crate orchestrates all engine systems and provides the main
//! game loop and application lifecycle management.

pub mod application;
pub mod engine;
pub mod loop_;
pub mod scheduler;
pub mod systems;
pub mod hybrid_game_loop;
pub mod standalone_runtime;

pub use application::{Application, ApplicationBuilder, ApplicationEvent};
pub use engine::Engine;
pub use loop_::GameLoop;
pub use scheduler::{Schedule, System, SystemScheduler};
pub use hybrid_game_loop::{HybridGameLoop, EngineMode, HybridFrameResult, HybridApplication};
pub use standalone_runtime::{StandaloneRuntime, StandaloneConfig, StandaloneConfigBuilder};

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
    #[error("Project load error: {0}")]
    ProjectLoadError(String),
}

/// Runtime system result type
pub type RuntimeResult<T> = Result<T, RuntimeError>;

#[cfg(test)]
mod tests {

    #[test]
    fn test_engine_creation() {
        // Placeholder test - this test just ensures the module compiles
    }
}
