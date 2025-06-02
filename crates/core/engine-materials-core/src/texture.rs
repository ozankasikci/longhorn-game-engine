//! Texture descriptors and usage

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TextureHandle(pub u32);

#[derive(Debug, Clone)]
pub struct TextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub format: TextureFormat,
    pub usage: TextureUsage,
    pub sample_count: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum TextureFormat {
    Rgba8Unorm,
    Rgba8Srgb,
    Bgra8Unorm,
    Bgra8Srgb,
    Depth32Float,
    Depth24PlusStencil8,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct TextureUsage: u32 {
        const TEXTURE_BINDING = 1 << 0;
        const STORAGE_BINDING = 1 << 1;
        const RENDER_ATTACHMENT = 1 << 2;
        const COPY_SRC = 1 << 3;
        const COPY_DST = 1 << 4;
    }
}

#[derive(Debug, Clone)]
pub struct TextureSize {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}