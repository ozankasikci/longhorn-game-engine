//! Core shader abstractions and management

use serde::{Serialize, Deserialize};

/// Handle for shader resources
pub type ShaderHandle = u64;

/// Shader stage/type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Compute,
    Geometry,
    TessellationControl,
    TessellationEvaluation,
}

/// Shader source language
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShaderLanguage {
    Glsl,
    Hlsl,
    Wgsl,
    SpirV,
}

/// Shader representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shader {
    pub name: String,
    pub shader_type: ShaderType,
    pub language: ShaderLanguage,
    pub source: ShaderSource,
    pub entry_point: String,
}

/// Shader source data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShaderSource {
    Text(String),
    Binary(Vec<u8>),
}

/// Complete shader program combining multiple stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderProgram {
    pub name: String,
    pub shaders: Vec<ShaderHandle>,
    pub vertex_shader: Option<ShaderHandle>,
    pub fragment_shader: Option<ShaderHandle>,
    pub compute_shader: Option<ShaderHandle>,
}

/// Vertex attribute descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexAttribute {
    pub name: String,
    pub location: u32,
    pub format: VertexFormat,
    pub offset: u32,
}

/// Vertex buffer layout descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexBufferLayout {
    pub stride: u32,
    pub step_mode: VertexStepMode,
    pub attributes: Vec<VertexAttribute>,
}

/// Vertex attribute format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VertexFormat {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
    Uint32,
    Uint32x2,
    Uint32x3,
    Uint32x4,
    Sint32,
    Sint32x2,
    Sint32x3,
    Sint32x4,
    Float16x2,
    Float16x4,
    Uint8x2,
    Uint8x4,
    Sint8x2,
    Sint8x4,
    Uint16x2,
    Uint16x4,
    Sint16x2,
    Sint16x4,
}

/// Vertex step mode for instancing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VertexStepMode {
    Vertex,
    Instance,
}

/// Uniform/constant buffer descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniformBuffer {
    pub name: String,
    pub binding: u32,
    pub size: u32,
    pub fields: Vec<UniformField>,
}

/// Uniform field descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniformField {
    pub name: String,
    pub field_type: UniformType,
    pub offset: u32,
    pub array_size: Option<u32>,
}

/// Uniform data types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UniformType {
    Bool,
    Int,
    UInt,
    Float,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
    Texture2D,
    TextureCube,
    Sampler,
}

impl Shader {
    /// Create a new shader from text source
    pub fn from_text(
        name: &str,
        shader_type: ShaderType,
        language: ShaderLanguage,
        source: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            shader_type,
            language,
            source: ShaderSource::Text(source.to_string()),
            entry_point: Self::default_entry_point(shader_type, language),
        }
    }
    
    /// Create a new shader from binary data
    pub fn from_binary(
        name: &str,
        shader_type: ShaderType,
        language: ShaderLanguage,
        data: Vec<u8>,
    ) -> Self {
        Self {
            name: name.to_string(),
            shader_type,
            language,
            source: ShaderSource::Binary(data),
            entry_point: Self::default_entry_point(shader_type, language),
        }
    }
    
    /// Set custom entry point
    pub fn with_entry_point(mut self, entry_point: &str) -> Self {
        self.entry_point = entry_point.to_string();
        self
    }
    
    /// Get default entry point for shader type and language
    /// 
    /// Note: This provides common defaults. Implementation crates
    /// should override with language-specific logic if needed.
    fn default_entry_point(_shader_type: ShaderType, _language: ShaderLanguage) -> String {
        // Simple default - implementation crates handle language specifics
        "main".to_string()
    }
    
    /// Get the source as text (if available)
    pub fn source_text(&self) -> Option<&str> {
        match &self.source {
            ShaderSource::Text(text) => Some(text),
            ShaderSource::Binary(_) => None,
        }
    }
    
    /// Get the source as binary data (if available)
    pub fn source_binary(&self) -> Option<&[u8]> {
        match &self.source {
            ShaderSource::Text(_) => None,
            ShaderSource::Binary(data) => Some(data),
        }
    }
}

impl ShaderProgram {
    /// Create a new shader program
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            shaders: Vec::new(),
            vertex_shader: None,
            fragment_shader: None,
            compute_shader: None,
        }
    }
    
    /// Add a shader to the program
    pub fn add_shader(&mut self, shader: ShaderHandle, shader_type: ShaderType) {
        self.shaders.push(shader);
        
        match shader_type {
            ShaderType::Vertex => self.vertex_shader = Some(shader),
            ShaderType::Fragment => self.fragment_shader = Some(shader),
            ShaderType::Compute => self.compute_shader = Some(shader),
            _ => {} // Other shader types stored in general list
        }
    }
    
    /// Check if this is a graphics pipeline (has vertex + fragment)
    pub fn is_graphics_pipeline(&self) -> bool {
        self.vertex_shader.is_some() && self.fragment_shader.is_some()
    }
    
    /// Check if this is a compute pipeline
    pub fn is_compute_pipeline(&self) -> bool {
        self.compute_shader.is_some()
    }
}

impl VertexFormat {
    /// Get the size in bytes of this vertex format
    pub fn size(&self) -> u32 {
        match self {
            Self::Float32 => 4,
            Self::Float32x2 => 8,
            Self::Float32x3 => 12,
            Self::Float32x4 => 16,
            Self::Uint32 => 4,
            Self::Uint32x2 => 8,
            Self::Uint32x3 => 12,
            Self::Uint32x4 => 16,
            Self::Sint32 => 4,
            Self::Sint32x2 => 8,
            Self::Sint32x3 => 12,
            Self::Sint32x4 => 16,
            Self::Float16x2 => 4,
            Self::Float16x4 => 8,
            Self::Uint8x2 => 2,
            Self::Uint8x4 => 4,
            Self::Sint8x2 => 2,
            Self::Sint8x4 => 4,
            Self::Uint16x2 => 4,
            Self::Uint16x4 => 8,
            Self::Sint16x2 => 4,
            Self::Sint16x4 => 8,
        }
    }
    
    /// Get the component count of this vertex format
    pub fn component_count(&self) -> u32 {
        match self {
            Self::Float32 | Self::Uint32 | Self::Sint32 => 1,
            Self::Float32x2 | Self::Uint32x2 | Self::Sint32x2 
            | Self::Float16x2 | Self::Uint8x2 | Self::Sint8x2 
            | Self::Uint16x2 | Self::Sint16x2 => 2,
            Self::Float32x3 | Self::Uint32x3 | Self::Sint32x3 => 3,
            Self::Float32x4 | Self::Uint32x4 | Self::Sint32x4 
            | Self::Float16x4 | Self::Uint8x4 | Self::Sint8x4 
            | Self::Uint16x4 | Self::Sint16x4 => 4,
        }
    }
}