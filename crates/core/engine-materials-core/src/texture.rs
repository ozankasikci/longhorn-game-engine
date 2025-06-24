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
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl TextureDescriptor {
    pub fn new(width: u32, height: u32, format: TextureFormat) -> Self {
        Self {
            width,
            height,
            depth: 1,
            format,
            usage: TextureUsage::TEXTURE_BINDING,
            sample_count: 1,
        }
    }

    pub fn with_usage(mut self, usage: TextureUsage) -> Self {
        self.usage = usage;
        self
    }

    pub fn with_sample_count(mut self, sample_count: u32) -> Self {
        self.sample_count = sample_count;
        self
    }

    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }
}

impl TextureSize {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            depth: 1,
        }
    }

    pub fn new_3d(width: u32, height: u32, depth: u32) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }

    pub fn pixel_count(&self) -> u32 {
        self.width * self.height * self.depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_handle() {
        let handle1 = TextureHandle(1);
        let handle2 = TextureHandle(2);
        let handle1_copy = TextureHandle(1);

        assert_eq!(handle1, handle1_copy);
        assert_ne!(handle1, handle2);
        assert_eq!(handle1.0, 1);
    }

    #[test]
    fn test_texture_descriptor_creation() {
        let desc = TextureDescriptor::new(512, 512, TextureFormat::Rgba8Unorm);
        assert_eq!(desc.width, 512);
        assert_eq!(desc.height, 512);
        assert_eq!(desc.depth, 1);
        assert!(matches!(desc.format, TextureFormat::Rgba8Unorm));
        assert_eq!(desc.usage, TextureUsage::TEXTURE_BINDING);
        assert_eq!(desc.sample_count, 1);
    }

    #[test]
    fn test_texture_descriptor_builder() {
        let desc = TextureDescriptor::new(256, 256, TextureFormat::Depth32Float)
            .with_usage(TextureUsage::RENDER_ATTACHMENT | TextureUsage::TEXTURE_BINDING)
            .with_sample_count(4)
            .with_depth(6);

        assert_eq!(desc.width, 256);
        assert_eq!(desc.height, 256);
        assert_eq!(desc.depth, 6);
        assert!(desc.usage.contains(TextureUsage::RENDER_ATTACHMENT));
        assert!(desc.usage.contains(TextureUsage::TEXTURE_BINDING));
        assert_eq!(desc.sample_count, 4);
    }

    #[test]
    fn test_texture_usage_flags() {
        let usage = TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_SRC;
        assert!(usage.contains(TextureUsage::TEXTURE_BINDING));
        assert!(usage.contains(TextureUsage::COPY_SRC));
        assert!(!usage.contains(TextureUsage::STORAGE_BINDING));

        let all_usage = TextureUsage::TEXTURE_BINDING
            | TextureUsage::STORAGE_BINDING
            | TextureUsage::RENDER_ATTACHMENT
            | TextureUsage::COPY_SRC
            | TextureUsage::COPY_DST;
        assert!(all_usage.contains(TextureUsage::TEXTURE_BINDING));
        assert!(all_usage.contains(TextureUsage::STORAGE_BINDING));
        assert!(all_usage.contains(TextureUsage::RENDER_ATTACHMENT));
        assert!(all_usage.contains(TextureUsage::COPY_SRC));
        assert!(all_usage.contains(TextureUsage::COPY_DST));

        // Test equality
        assert_eq!(TextureUsage::TEXTURE_BINDING, TextureUsage::TEXTURE_BINDING);
        assert_ne!(TextureUsage::TEXTURE_BINDING, TextureUsage::STORAGE_BINDING);
        assert_eq!(
            usage,
            TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_SRC
        );
    }

    #[test]
    fn test_texture_size() {
        let size_2d = TextureSize::new(1920, 1080);
        assert_eq!(size_2d.width, 1920);
        assert_eq!(size_2d.height, 1080);
        assert_eq!(size_2d.depth, 1);
        assert_eq!(size_2d.pixel_count(), 1920 * 1080);

        let size_3d = TextureSize::new_3d(128, 128, 32);
        assert_eq!(size_3d.width, 128);
        assert_eq!(size_3d.height, 128);
        assert_eq!(size_3d.depth, 32);
        assert_eq!(size_3d.pixel_count(), 128 * 128 * 32);
    }

    #[test]
    fn test_texture_format_memory_sizes() {
        // Ensure format enum is properly defined
        let formats = [
            TextureFormat::Rgba8Unorm,
            TextureFormat::Rgba8Srgb,
            TextureFormat::Bgra8Unorm,
            TextureFormat::Bgra8Srgb,
            TextureFormat::Depth32Float,
            TextureFormat::Depth24PlusStencil8,
        ];

        for format in &formats {
            // Just ensure we can match on all variants
            match format {
                TextureFormat::Rgba8Unorm => {}
                TextureFormat::Rgba8Srgb => {}
                TextureFormat::Bgra8Unorm => {}
                TextureFormat::Bgra8Srgb => {}
                TextureFormat::Depth32Float => {}
                TextureFormat::Depth24PlusStencil8 => {}
            }
        }
    }
}
