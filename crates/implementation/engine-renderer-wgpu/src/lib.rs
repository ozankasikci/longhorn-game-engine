//! Core rendering abstractions for the mobile game engine
//! 
//! This crate provides pure rendering interfaces and abstractions with no
//! implementation dependencies. All graphics implementations should implement
//! the traits defined here.

pub mod renderer;
pub mod commands;
pub mod resources;
pub mod capabilities;
pub mod pipeline;
pub mod viewport;

// Re-export core types
pub use renderer::{Renderer, BatchRenderer, RenderState};
pub use commands::{RenderCommand, DrawCall};
pub use resources::{Handle, ResourceManager, TextureHandle, BufferHandle, ShaderHandle};
pub use capabilities::{RendererCapabilities, TextureCompressionFormat};
pub use pipeline::{RenderPipeline, PipelineDescriptor};
pub use viewport::{Viewport, RenderTarget};

/// Error types for the renderer core system
#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
    #[error("Renderer initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Rendering failed: {0}")]
    RenderingFailed(String),
    #[error("Out of memory")]
    OutOfMemory,
}

/// Result type for renderer operations
pub type Result<T> = std::result::Result<T, RendererError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_error_display() {
        let error = RendererError::InitializationFailed("GPU not found".to_string());
        assert!(error.to_string().contains("Renderer initialization failed"));
        assert!(error.to_string().contains("GPU not found"));
    }

    #[test]
    fn test_renderer_error_variants() {
        let errors = vec![
            RendererError::ResourceNotFound("texture.png".to_string()),
            RendererError::InvalidOperation("invalid bind".to_string()),
            RendererError::UnsupportedFeature("compute shaders".to_string()),
            RendererError::OutOfMemory,
        ];

        for error in errors {
            // Ensure all variants can be displayed and debugged
            let _display = error.to_string();
            let _debug = format!("{:?}", error);
        }
    }

    #[test]
    fn test_result_type_usage() {
        let success: Result<i32> = Ok(42);
        let failure: Result<i32> = Err(RendererError::OutOfMemory);
        
        assert_eq!(success.unwrap(), 42);
        assert!(failure.is_err());
        
        match failure {
            Err(RendererError::OutOfMemory) => (),
            _ => panic!("Expected OutOfMemory error"),
        }
    }
}