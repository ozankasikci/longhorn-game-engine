//! Core material and shader abstractions

pub mod color;
pub mod material;
pub mod shader;
pub mod texture;

pub use color::{Color, ColorSpace};
pub use material::{AlphaMode, Material, MaterialHandle, MaterialTextures, PbrMaterial};
pub use shader::{
    Shader, ShaderHandle, ShaderLanguage, ShaderProgram, ShaderSource, ShaderType, UniformBuffer,
    UniformField, UniformType, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
};
pub use texture::{TextureDescriptor, TextureFormat, TextureSize, TextureUsage};

pub type Handle = u64;
