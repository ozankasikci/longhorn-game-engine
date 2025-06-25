use bytemuck::{Pod, Zeroable};

/// Vertex attribute formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VertexFormat {
    /// One 32-bit float
    Float32,
    /// Two 32-bit floats
    Float32x2,
    /// Three 32-bit floats
    Float32x3,
    /// Four 32-bit floats
    Float32x4,
    /// One 32-bit unsigned integer
    Uint32,
    /// Two 32-bit unsigned integers
    Uint32x2,
    /// Three 32-bit unsigned integers
    Uint32x3,
    /// Four 32-bit unsigned integers
    Uint32x4,
}

/// Buffer usage flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferUsage(u32);

impl BufferUsage {
    /// Buffer can be used as a vertex buffer
    pub const VERTEX: Self = Self(1 << 0);
    /// Buffer can be used as an index buffer
    pub const INDEX: Self = Self(1 << 1);
    /// Buffer can be used as a uniform buffer
    pub const UNIFORM: Self = Self(1 << 2);
    /// Buffer can be used as a storage buffer
    pub const STORAGE: Self = Self(1 << 3);
    /// Buffer can be copied from
    pub const COPY_SRC: Self = Self(1 << 4);
    /// Buffer can be copied to
    pub const COPY_DST: Self = Self(1 << 5);
    
    /// Combine usage flags
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    
    /// Check if usage contains specific flags
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

/// Buffer descriptor for creation
#[derive(Debug, Clone)]
pub struct BufferDescriptor {
    /// Size of the buffer in bytes
    pub size: u64,
    /// How the buffer will be used
    pub usage: BufferUsage,
    /// Whether the buffer should be mapped at creation
    pub mapped_at_creation: bool,
}

/// Texture dimensions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Extent3d {
    /// Width of the texture
    pub width: u32,
    /// Height of the texture
    pub height: u32,
    /// Depth or array layer count
    pub depth_or_array_layers: u32,
}

/// Texture format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFormat {
    /// 8-bit red, green, blue, alpha - sRGB
    Rgba8UnormSrgb,
    /// 8-bit red, green, blue, alpha - linear
    Rgba8Unorm,
    /// 32-bit red, green, blue, alpha - float
    Rgba32Float,
    /// 32-bit depth
    Depth32Float,
    /// 24-bit depth, 8-bit stencil
    Depth24PlusStencil8,
}

/// Texture usage flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureUsage(u32);

impl TextureUsage {
    /// Texture can be copied from
    pub const COPY_SRC: Self = Self(1 << 0);
    /// Texture can be copied to
    pub const COPY_DST: Self = Self(1 << 1);
    /// Texture can be sampled
    pub const TEXTURE_BINDING: Self = Self(1 << 2);
    /// Texture can be used as storage
    pub const STORAGE_BINDING: Self = Self(1 << 3);
    /// Texture can be rendered to
    pub const RENDER_ATTACHMENT: Self = Self(1 << 4);
    
    /// Combine usage flags
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    
    /// Check if usage contains specific flags
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

/// Texture descriptor for creation
#[derive(Debug, Clone)]
pub struct TextureDescriptor {
    /// Size of the texture
    pub size: Extent3d,
    /// Format of the texture
    pub format: TextureFormat,
    /// How the texture will be used
    pub usage: TextureUsage,
    /// Number of mip levels
    pub mip_level_count: u32,
    /// Number of samples for multisampling
    pub sample_count: u32,
}

/// Color value
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct Color {
    /// Red component (0.0 - 1.0)
    pub r: f32,
    /// Green component (0.0 - 1.0)
    pub g: f32,
    /// Blue component (0.0 - 1.0)
    pub b: f32,
    /// Alpha component (0.0 - 1.0)
    pub a: f32,
}

impl Color {
    /// Create a new color
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    
    /// Black color
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    /// White color
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    /// Transparent
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_buffer_usage_flags() {
        let usage = BufferUsage::VERTEX.union(BufferUsage::COPY_DST);
        assert!(usage.contains(BufferUsage::VERTEX));
        assert!(usage.contains(BufferUsage::COPY_DST));
        assert!(!usage.contains(BufferUsage::INDEX));
    }
    
    #[test]
    fn test_texture_usage_flags() {
        let usage = TextureUsage::TEXTURE_BINDING.union(TextureUsage::COPY_DST);
        assert!(usage.contains(TextureUsage::TEXTURE_BINDING));
        assert!(usage.contains(TextureUsage::COPY_DST));
        assert!(!usage.contains(TextureUsage::RENDER_ATTACHMENT));
    }
    
    #[test]
    fn test_color_constants() {
        assert_eq!(Color::BLACK.r, 0.0);
        assert_eq!(Color::WHITE.r, 1.0);
        assert_eq!(Color::TRANSPARENT.a, 0.0);
    }
}