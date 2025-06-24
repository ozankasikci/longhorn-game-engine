pub mod command;
pub mod pipeline;
pub mod resource;
pub mod surface;

pub use command::*;
pub use pipeline::*;
pub use resource::*;
pub use surface::*;

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
