// Graphics rendering system for the mobile game engine

pub mod renderer;
pub mod multi_camera_renderer;
pub mod materials;
pub mod shaders;
pub mod mesh;

pub use renderer::{Renderer, Vertex, Uniform};
pub use multi_camera_renderer::{MultiCameraRenderer, RenderTarget, TextureRenderTarget};

// Error types for the graphics system
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Camera component not found")]
    CameraNotFound,
    #[error("Transform component not found")]
    TransformNotFound,
    #[error("Mesh component not found")]
    MeshNotFound,
    #[error("Render target not found")]
    RenderTargetNotFound,
    #[error("WGPU error: {0}")]
    WgpuError(String),
}