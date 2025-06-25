use crate::{Extent3d, TextureFormat};

/// Trait for graphics textures
pub trait GraphicsTexture: Send + Sync {
    /// Get the dimensions of the texture
    fn dimensions(&self) -> Extent3d;
    
    /// Get the format of the texture
    fn format(&self) -> TextureFormat;
    
    /// Get the number of mip levels
    fn mip_level_count(&self) -> u32;
    
    /// Get the sample count (for multisampled textures)
    fn sample_count(&self) -> u32;
}

/// Trait for texture views (subresources of textures)
pub trait GraphicsTextureView: Send + Sync {
    /// Get the base texture this view references
    fn texture(&self) -> &dyn GraphicsTexture;
    
    /// Get the format of this view (may differ from base texture)
    fn format(&self) -> TextureFormat;
}

/// Trait for samplers (texture filtering and addressing modes)
pub trait GraphicsSampler: Send + Sync {
    /// Get the minification filter
    fn min_filter(&self) -> FilterMode;
    
    /// Get the magnification filter
    fn mag_filter(&self) -> FilterMode;
    
    /// Get the addressing mode for U coordinate
    fn address_mode_u(&self) -> AddressMode;
    
    /// Get the addressing mode for V coordinate
    fn address_mode_v(&self) -> AddressMode;
    
    /// Get the addressing mode for W coordinate
    fn address_mode_w(&self) -> AddressMode;
}

/// Texture filtering modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    /// Nearest neighbor filtering
    Nearest,
    /// Linear filtering
    Linear,
}

/// Texture addressing modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressMode {
    /// Clamp to edge
    ClampToEdge,
    /// Repeat
    Repeat,
    /// Mirror repeat
    MirrorRepeat,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Extent3d, TextureFormat};
    
    // Mock texture implementation for testing
    struct MockTexture {
        dimensions: Extent3d,
        format: TextureFormat,
        mip_levels: u32,
        samples: u32,
    }
    
    impl GraphicsTexture for MockTexture {
        fn dimensions(&self) -> Extent3d {
            self.dimensions
        }
        
        fn format(&self) -> TextureFormat {
            self.format
        }
        
        fn mip_level_count(&self) -> u32 {
            self.mip_levels
        }
        
        fn sample_count(&self) -> u32 {
            self.samples
        }
    }
    
    // Mock texture view
    struct MockTextureView<'a> {
        texture: &'a MockTexture,
        view_format: TextureFormat,
    }
    
    impl<'a> GraphicsTextureView for MockTextureView<'a> {
        fn texture(&self) -> &dyn GraphicsTexture {
            self.texture
        }
        
        fn format(&self) -> TextureFormat {
            self.view_format
        }
    }
    
    // Mock sampler
    struct MockSampler {
        min: FilterMode,
        mag: FilterMode,
        address_u: AddressMode,
        address_v: AddressMode,
        address_w: AddressMode,
    }
    
    impl GraphicsSampler for MockSampler {
        fn min_filter(&self) -> FilterMode {
            self.min
        }
        
        fn mag_filter(&self) -> FilterMode {
            self.mag
        }
        
        fn address_mode_u(&self) -> AddressMode {
            self.address_u
        }
        
        fn address_mode_v(&self) -> AddressMode {
            self.address_v
        }
        
        fn address_mode_w(&self) -> AddressMode {
            self.address_w
        }
    }
    
    #[test]
    fn test_texture_properties() {
        let texture = MockTexture {
            dimensions: Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            },
            format: TextureFormat::Rgba8Unorm,
            mip_levels: 8,
            samples: 1,
        };
        
        assert_eq!(texture.dimensions().width, 256);
        assert_eq!(texture.dimensions().height, 256);
        assert_eq!(texture.format(), TextureFormat::Rgba8Unorm);
        assert_eq!(texture.mip_level_count(), 8);
        assert_eq!(texture.sample_count(), 1);
    }
    
    #[test]
    fn test_texture_view() {
        let texture = MockTexture {
            dimensions: Extent3d {
                width: 512,
                height: 512,
                depth_or_array_layers: 1,
            },
            format: TextureFormat::Rgba8UnormSrgb,
            mip_levels: 1,
            samples: 1,
        };
        
        let view = MockTextureView {
            texture: &texture,
            view_format: TextureFormat::Rgba8Unorm, // Different format for view
        };
        
        assert_eq!(view.texture().dimensions().width, 512);
        assert_eq!(view.format(), TextureFormat::Rgba8Unorm);
    }
    
    #[test]
    fn test_sampler_properties() {
        let sampler = MockSampler {
            min: FilterMode::Linear,
            mag: FilterMode::Linear,
            address_u: AddressMode::ClampToEdge,
            address_v: AddressMode::ClampToEdge,
            address_w: AddressMode::Repeat,
        };
        
        assert_eq!(sampler.min_filter(), FilterMode::Linear);
        assert_eq!(sampler.mag_filter(), FilterMode::Linear);
        assert_eq!(sampler.address_mode_u(), AddressMode::ClampToEdge);
        assert_eq!(sampler.address_mode_v(), AddressMode::ClampToEdge);
        assert_eq!(sampler.address_mode_w(), AddressMode::Repeat);
    }
}