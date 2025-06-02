//! Renderer capabilities and feature detection

use serde::{Serialize, Deserialize};

/// Renderer capabilities and limitations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererCapabilities {
    /// Maximum texture size in pixels
    pub max_texture_size: u32,
    /// Maximum 3D texture size
    pub max_texture_3d_size: u32,
    /// Maximum texture array layers
    pub max_texture_array_layers: u32,
    /// Maximum vertex attributes
    pub max_vertex_attributes: u32,
    /// Maximum vertex buffers
    pub max_vertex_buffers: u32,
    /// Maximum uniform buffer size
    pub max_uniform_buffer_size: u64,
    /// Maximum storage buffer size
    pub max_storage_buffer_size: u64,
    /// Maximum bind groups
    pub max_bind_groups: u32,
    /// Maximum bindings per bind group
    pub max_bindings_per_bind_group: u32,
    /// Supported texture compression formats
    pub texture_compression_formats: Vec<TextureCompressionFormat>,
    /// Support for compute shaders
    pub compute_shaders: bool,
    /// Support for geometry shaders
    pub geometry_shaders: bool,
    /// Support for tessellation
    pub tessellation: bool,
    /// Support for multi-sampling
    pub multi_sampling: bool,
    /// Maximum sample count for multi-sampling
    pub max_sample_count: u32,
    /// Support for anisotropic filtering
    pub anisotropic_filtering: bool,
    /// Maximum anisotropy level
    pub max_anisotropy: u32,
    /// Support for depth textures
    pub depth_textures: bool,
    /// Support for stencil textures
    pub stencil_textures: bool,
    /// Support for render to texture arrays
    pub render_to_texture_arrays: bool,
    /// Support for instanced rendering
    pub instanced_rendering: bool,
    /// Support for indirect rendering
    pub indirect_rendering: bool,
    /// Support for conservative rasterization
    pub conservative_rasterization: bool,
}

/// Texture compression formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureCompressionFormat {
    // Block Compression (BC) - DirectX
    Bc1,
    Bc2,
    Bc3,
    Bc4,
    Bc5,
    Bc6h,
    Bc7,
    
    // Ericsson Texture Compression (ETC) - OpenGL ES
    Etc2Rgb8,
    Etc2Rgba8,
    
    // Adaptive Scalable Texture Compression (ASTC) - Modern mobile
    Astc4x4,
    Astc5x4,
    Astc5x5,
    Astc6x5,
    Astc6x6,
    Astc8x5,
    Astc8x6,
    Astc8x8,
    Astc10x5,
    Astc10x6,
    Astc10x8,
    Astc10x10,
    Astc12x10,
    Astc12x12,
}

impl Default for RendererCapabilities {
    fn default() -> Self {
        Self {
            max_texture_size: 4096,
            max_texture_3d_size: 256,
            max_texture_array_layers: 256,
            max_vertex_attributes: 16,
            max_vertex_buffers: 8,
            max_uniform_buffer_size: 65536,
            max_storage_buffer_size: 134217728, // 128MB
            max_bind_groups: 4,
            max_bindings_per_bind_group: 640,
            texture_compression_formats: vec![
                TextureCompressionFormat::Bc1,
                TextureCompressionFormat::Bc3,
                TextureCompressionFormat::Etc2Rgb8,
            ],
            compute_shaders: true,
            geometry_shaders: false,
            tessellation: false,
            multi_sampling: true,
            max_sample_count: 4,
            anisotropic_filtering: true,
            max_anisotropy: 16,
            depth_textures: true,
            stencil_textures: true,
            render_to_texture_arrays: true,
            instanced_rendering: true,
            indirect_rendering: true,
            conservative_rasterization: false,
        }
    }
}

impl RendererCapabilities {
    /// Check if a texture compression format is supported
    pub fn supports_compression(&self, format: TextureCompressionFormat) -> bool {
        self.texture_compression_formats.contains(&format)
    }
    
    /// Check if large textures are supported
    pub fn supports_large_textures(&self) -> bool {
        self.max_texture_size >= 8192
    }
    
    /// Check if high-end features are supported
    pub fn is_high_end(&self) -> bool {
        self.compute_shaders
            && self.max_texture_size >= 8192
            && self.max_anisotropy >= 16
            && self.supports_compression(TextureCompressionFormat::Bc7)
    }
    
    /// Check if this is likely a mobile GPU
    pub fn is_mobile(&self) -> bool {
        self.max_texture_size <= 4096
            && !self.geometry_shaders
            && !self.tessellation
            && self.supports_compression(TextureCompressionFormat::Astc4x4)
    }
    
    /// Get recommended texture compression format for this platform
    pub fn recommended_compression(&self) -> Option<TextureCompressionFormat> {
        if self.supports_compression(TextureCompressionFormat::Bc7) {
            Some(TextureCompressionFormat::Bc7)
        } else if self.supports_compression(TextureCompressionFormat::Astc4x4) {
            Some(TextureCompressionFormat::Astc4x4)
        } else if self.supports_compression(TextureCompressionFormat::Bc3) {
            Some(TextureCompressionFormat::Bc3)
        } else if self.supports_compression(TextureCompressionFormat::Etc2Rgba8) {
            Some(TextureCompressionFormat::Etc2Rgba8)
        } else {
            None
        }
    }
}