//! Resource management abstractions and handle types

use crate::{RendererError, Result};
use serde::{Serialize, Deserialize};

/// Handle type for graphics resources
pub type Handle = u64;

/// Specific handle types for type safety
pub type TextureHandle = Handle;
pub type BufferHandle = Handle;
pub type ShaderHandle = Handle;
pub type PipelineHandle = Handle;
pub type SamplerHandle = Handle;

/// Resource manager trait for creating and managing graphics resources
pub trait ResourceManager: Send + Sync {
    /// Create a texture resource
    fn create_texture(&mut self, descriptor: &TextureDescriptor) -> Result<TextureHandle>;
    
    /// Create a buffer resource
    fn create_buffer(&mut self, descriptor: &BufferDescriptor) -> Result<BufferHandle>;
    
    /// Create a shader resource
    fn create_shader(&mut self, descriptor: &ShaderDescriptor) -> Result<ShaderHandle>;
    
    /// Create a sampler resource
    fn create_sampler(&mut self, descriptor: &SamplerDescriptor) -> Result<SamplerHandle>;
    
    /// Update texture data
    fn update_texture(&mut self, handle: TextureHandle, data: &[u8]) -> Result<()>;
    
    /// Update buffer data
    fn update_buffer(&mut self, handle: BufferHandle, data: &[u8], offset: u64) -> Result<()>;
    
    /// Destroy a texture resource
    fn destroy_texture(&mut self, handle: TextureHandle) -> Result<()>;
    
    /// Destroy a buffer resource
    fn destroy_buffer(&mut self, handle: BufferHandle) -> Result<()>;
    
    /// Destroy a shader resource
    fn destroy_shader(&mut self, handle: ShaderHandle) -> Result<()>;
    
    /// Destroy a sampler resource
    fn destroy_sampler(&mut self, handle: SamplerHandle) -> Result<()>;
    
    /// Get resource memory usage
    fn memory_usage(&self) -> ResourceMemoryUsage;
}

/// Texture descriptor for creating textures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureDescriptor {
    pub label: Option<String>,
    pub size: TextureSize,
    pub format: TextureFormat,
    pub usage: TextureUsage,
    pub mip_level_count: u32,
    pub sample_count: u32,
}

/// Texture size specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureSize {
    D1 { width: u32 },
    D2 { width: u32, height: u32 },
    D3 { width: u32, height: u32, depth: u32 },
}

/// Texture formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureFormat {
    // 8-bit formats
    R8Unorm,
    RG8Unorm,
    RGBA8Unorm,
    RGBA8UnormSrgb,
    
    // 16-bit formats
    R16Float,
    RG16Float,
    RGBA16Float,
    
    // 32-bit formats
    R32Float,
    RG32Float,
    RGBA32Float,
    
    // Depth formats
    Depth16Unorm,
    Depth24Plus,
    Depth32Float,
    Depth24PlusStencil8,
    
    // Compressed formats
    Bc1RgbaUnorm,
    Bc2RgbaUnorm,
    Bc3RgbaUnorm,
    Bc4RUnorm,
    Bc5RgUnorm,
    Bc6hRgbUfloat,
    Bc7RgbaUnorm,
}

/// Texture usage flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextureUsage {
    pub copy_src: bool,
    pub copy_dst: bool,
    pub texture_binding: bool,
    pub storage_binding: bool,
    pub render_attachment: bool,
}

/// Buffer descriptor for creating buffers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferDescriptor {
    pub label: Option<String>,
    pub size: u64,
    pub usage: BufferUsage,
    pub mapped_at_creation: bool,
}

/// Buffer usage flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BufferUsage {
    pub vertex: bool,
    pub index: bool,
    pub uniform: bool,
    pub storage: bool,
    pub indirect: bool,
    pub copy_src: bool,
    pub copy_dst: bool,
}

/// Shader descriptor for creating shaders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderDescriptor {
    pub label: Option<String>,
    pub source: ShaderSource,
    pub entry_point: String,
    pub stage: ShaderStage,
}

/// Shader source data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShaderSource {
    Wgsl(String),
    SpirV(Vec<u32>),
    Glsl(String),
    Hlsl(String),
}

/// Shader stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

/// Sampler descriptor for creating samplers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplerDescriptor {
    pub label: Option<String>,
    pub address_mode_u: AddressMode,
    pub address_mode_v: AddressMode,
    pub address_mode_w: AddressMode,
    pub mag_filter: FilterMode,
    pub min_filter: FilterMode,
    pub mipmap_filter: FilterMode,
    pub lod_min_clamp: f32,
    pub lod_max_clamp: f32,
    pub compare: Option<CompareFunction>,
    pub anisotropy_clamp: u16,
    pub border_color: Option<SamplerBorderColor>,
}

/// Texture address modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressMode {
    ClampToEdge,
    Repeat,
    MirrorRepeat,
    ClampToBorder,
}

/// Texture filter modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterMode {
    Nearest,
    Linear,
}

/// Depth/stencil compare functions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareFunction {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

/// Sampler border colors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SamplerBorderColor {
    TransparentBlack,
    OpaqueBlack,
    OpaqueWhite,
}

/// Resource memory usage statistics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResourceMemoryUsage {
    pub textures_bytes: u64,
    pub buffers_bytes: u64,
    pub total_bytes: u64,
    pub texture_count: u32,
    pub buffer_count: u32,
    pub shader_count: u32,
    pub sampler_count: u32,
}

impl Default for TextureUsage {
    fn default() -> Self {
        Self {
            copy_src: false,
            copy_dst: true,
            texture_binding: true,
            storage_binding: false,
            render_attachment: false,
        }
    }
}

impl Default for BufferUsage {
    fn default() -> Self {
        Self {
            vertex: false,
            index: false,
            uniform: false,
            storage: false,
            indirect: false,
            copy_src: false,
            copy_dst: true,
        }
    }
}

impl Default for SamplerDescriptor {
    fn default() -> Self {
        Self {
            label: None,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        }
    }
}

impl TextureFormat {
    /// Get the size in bytes per pixel
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            Self::R8Unorm => 1,
            Self::RG8Unorm => 2,
            Self::RGBA8Unorm | Self::RGBA8UnormSrgb => 4,
            Self::R16Float => 2,
            Self::RG16Float => 4,
            Self::RGBA16Float => 8,
            Self::R32Float => 4,
            Self::RG32Float => 8,
            Self::RGBA32Float => 16,
            Self::Depth16Unorm => 2,
            Self::Depth24Plus => 4,
            Self::Depth32Float => 4,
            Self::Depth24PlusStencil8 => 4,
            Self::Bc1RgbaUnorm => 8, // Block compressed
            Self::Bc2RgbaUnorm => 16,
            Self::Bc3RgbaUnorm => 16,
            Self::Bc4RUnorm => 8,
            Self::Bc5RgUnorm => 16,
            Self::Bc6hRgbUfloat => 16,
            Self::Bc7RgbaUnorm => 16,
        }
    }
    
    /// Check if this is a depth format
    pub fn is_depth(&self) -> bool {
        matches!(
            self,
            Self::Depth16Unorm | Self::Depth24Plus | Self::Depth32Float | Self::Depth24PlusStencil8
        )
    }
    
    /// Check if this is a compressed format
    pub fn is_compressed(&self) -> bool {
        matches!(
            self,
            Self::Bc1RgbaUnorm
                | Self::Bc2RgbaUnorm
                | Self::Bc3RgbaUnorm
                | Self::Bc4RUnorm
                | Self::Bc5RgUnorm
                | Self::Bc6hRgbUfloat
                | Self::Bc7RgbaUnorm
        )
    }
}

impl TextureSize {
    /// Get the width
    pub fn width(&self) -> u32 {
        match self {
            Self::D1 { width } | Self::D2 { width, .. } | Self::D3 { width, .. } => *width,
        }
    }
    
    /// Get the height (1 for 1D textures)
    pub fn height(&self) -> u32 {
        match self {
            Self::D1 { .. } => 1,
            Self::D2 { height, .. } | Self::D3 { height, .. } => *height,
        }
    }
    
    /// Get the depth (1 for 1D and 2D textures)
    pub fn depth(&self) -> u32 {
        match self {
            Self::D1 { .. } | Self::D2 { .. } => 1,
            Self::D3 { depth, .. } => *depth,
        }
    }
    
    /// Calculate total pixel count
    pub fn pixel_count(&self) -> u32 {
        self.width() * self.height() * self.depth()
    }
}