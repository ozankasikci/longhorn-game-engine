//! Core material and shader abstractions

pub mod material;
pub mod shader;
pub mod color;
pub mod texture;

pub use material::{Material, MaterialHandle, PbrMaterial, MaterialTextures, AlphaMode};
pub use shader::{
    Shader, ShaderHandle, ShaderProgram, ShaderSource, ShaderType, ShaderLanguage,
    VertexFormat, VertexAttribute, VertexBufferLayout, VertexStepMode,
    UniformBuffer, UniformField, UniformType
};
pub use color::{Color, ColorSpace};
pub use texture::{TextureDescriptor, TextureUsage, TextureFormat, TextureSize};

pub type Handle = u64;