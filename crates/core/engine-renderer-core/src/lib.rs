pub mod pipeline;
pub mod command;
pub mod surface;
pub mod resource;

pub use pipeline::*;
pub use command::*;
pub use surface::*;
pub use resource::*;

#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("Device creation failed: {0}")]
    DeviceCreation(String),
    #[error("Surface configuration failed: {0}")]
    SurfaceConfiguration(String),
    #[error("Shader compilation failed: {0}")]
    ShaderCompilation(String),
    #[error("Resource binding failed: {0}")]
    ResourceBinding(String),
}

pub type RendererResult<T> = Result<T, RendererError>;