use engine_geometry_core::{MeshHandle, BufferHandle};
use engine_materials_core::{texture::TextureHandle, shader::ShaderHandle, TextureUsage};

pub trait ResourceManager {
    fn create_buffer(&mut self, descriptor: &BufferDescriptor) -> BufferHandle;
    fn create_texture(&mut self, descriptor: &TextureDescriptor) -> TextureHandle;
    fn update_buffer(&mut self, handle: &BufferHandle, data: &[u8]);
    fn update_texture(&mut self, handle: &TextureHandle, data: &[u8]);
}

#[derive(Debug, Clone)]
pub struct BufferDescriptor {
    pub size: u64,
    pub usage: BufferUsage,
    pub memory_type: MemoryType,
}

#[derive(Debug, Clone)]
pub struct TextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub format: TextureFormat,
    pub usage: TextureUsage,
    pub sample_count: u32,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct BufferUsage: u32 {
        const VERTEX = 1 << 0;
        const INDEX = 1 << 1;
        const UNIFORM = 1 << 2;
        const STORAGE = 1 << 3;
        const COPY_SRC = 1 << 4;
        const COPY_DST = 1 << 5;
    }
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

#[derive(Debug, Clone, Copy)]
pub enum MemoryType {
    GpuOnly,
    CpuToGpu,
    GpuToCpu,
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