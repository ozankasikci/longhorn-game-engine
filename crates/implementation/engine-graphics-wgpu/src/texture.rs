use engine_graphics_traits::{
    GraphicsTexture, GraphicsTextureView, GraphicsSampler, 
    Extent3d, TextureFormat, FilterMode, AddressMode
};
use std::sync::Arc;

/// WGPU texture implementation
pub struct WgpuTexture {
    texture: Arc<wgpu::Texture>,
    dimensions: Extent3d,
    format: TextureFormat,
    mip_level_count: u32,
    sample_count: u32,
}

impl WgpuTexture {
    /// Create a new WGPU texture wrapper
    pub fn new(texture: wgpu::Texture, format: TextureFormat) -> Self {
        let size = texture.size();
        let dimensions = Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: size.depth_or_array_layers,
        };
        let mip_level_count = texture.mip_level_count();
        let sample_count = texture.sample_count();
        
        Self {
            texture: Arc::new(texture),
            dimensions,
            format,
            mip_level_count,
            sample_count,
        }
    }
    
    /// Get the underlying WGPU texture
    pub fn raw(&self) -> &wgpu::Texture {
        &self.texture
    }
}

impl GraphicsTexture for WgpuTexture {
    fn dimensions(&self) -> Extent3d {
        self.dimensions
    }
    
    fn format(&self) -> TextureFormat {
        self.format
    }
    
    fn mip_level_count(&self) -> u32 {
        self.mip_level_count
    }
    
    fn sample_count(&self) -> u32 {
        self.sample_count
    }
}

/// WGPU texture view implementation
pub struct WgpuTextureView {
    view: Arc<wgpu::TextureView>,
    texture: Arc<WgpuTexture>,
    format: TextureFormat,
}

impl WgpuTextureView {
    /// Create a new WGPU texture view wrapper
    pub fn new(view: wgpu::TextureView, texture: Arc<WgpuTexture>, format: TextureFormat) -> Self {
        Self {
            view: Arc::new(view),
            texture,
            format,
        }
    }
    
    /// Get the underlying WGPU texture view
    pub fn raw(&self) -> &wgpu::TextureView {
        &self.view
    }
}

impl GraphicsTextureView for WgpuTextureView {
    fn texture(&self) -> &dyn GraphicsTexture {
        &*self.texture
    }
    
    fn format(&self) -> TextureFormat {
        self.format
    }
}

/// WGPU sampler implementation
pub struct WgpuSampler {
    sampler: Arc<wgpu::Sampler>,
    min_filter: FilterMode,
    mag_filter: FilterMode,
    address_mode_u: AddressMode,
    address_mode_v: AddressMode,
    address_mode_w: AddressMode,
}

impl WgpuSampler {
    /// Create a new WGPU sampler wrapper
    pub fn new(
        sampler: wgpu::Sampler,
        min_filter: FilterMode,
        mag_filter: FilterMode,
        address_mode_u: AddressMode,
        address_mode_v: AddressMode,
        address_mode_w: AddressMode,
    ) -> Self {
        Self {
            sampler: Arc::new(sampler),
            min_filter,
            mag_filter,
            address_mode_u,
            address_mode_v,
            address_mode_w,
        }
    }
    
    /// Get the underlying WGPU sampler
    pub fn raw(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}

impl GraphicsSampler for WgpuSampler {
    fn min_filter(&self) -> FilterMode {
        self.min_filter
    }
    
    fn mag_filter(&self) -> FilterMode {
        self.mag_filter
    }
    
    fn address_mode_u(&self) -> AddressMode {
        self.address_mode_u
    }
    
    fn address_mode_v(&self) -> AddressMode {
        self.address_mode_v
    }
    
    fn address_mode_w(&self) -> AddressMode {
        self.address_mode_w
    }
}

/// Convert engine texture format to WGPU format
pub fn convert_texture_format(format: TextureFormat) -> wgpu::TextureFormat {
    match format {
        TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
        TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
        TextureFormat::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
        TextureFormat::Depth32Float => wgpu::TextureFormat::Depth32Float,
        TextureFormat::Depth24PlusStencil8 => wgpu::TextureFormat::Depth24PlusStencil8,
    }
}

/// Convert engine filter mode to WGPU filter mode
pub fn convert_filter_mode(mode: FilterMode) -> wgpu::FilterMode {
    match mode {
        FilterMode::Nearest => wgpu::FilterMode::Nearest,
        FilterMode::Linear => wgpu::FilterMode::Linear,
    }
}

/// Convert engine address mode to WGPU address mode
pub fn convert_address_mode(mode: AddressMode) -> wgpu::AddressMode {
    match mode {
        AddressMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
        AddressMode::Repeat => wgpu::AddressMode::Repeat,
        AddressMode::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_graphics_traits::{TextureDescriptor, TextureUsage, SamplerDescriptor};
    
    #[allow(unused_imports)]
    use super::*;
    
    async fn create_test_device() -> (wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to request adapter");
        
        adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Test Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Failed to request device")
    }
    
    #[test]
    fn test_texture_creation() {
        pollster::block_on(async {
            let (device, _queue) = create_test_device().await;
            
            let wgpu_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Test Texture"),
                size: wgpu::Extent3d {
                    width: 256,
                    height: 256,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            
            let texture = WgpuTexture::new(wgpu_texture, TextureFormat::Rgba8Unorm);
            
            assert_eq!(texture.dimensions().width, 256);
            assert_eq!(texture.dimensions().height, 256);
            assert_eq!(texture.dimensions().depth_or_array_layers, 1);
            assert_eq!(texture.format(), TextureFormat::Rgba8Unorm);
            assert_eq!(texture.mip_level_count(), 1);
            assert_eq!(texture.sample_count(), 1);
        });
    }
    
    #[test]
    fn test_texture_view_creation() {
        pollster::block_on(async {
            let (device, _queue) = create_test_device().await;
            
            let wgpu_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Test Texture"),
                size: wgpu::Extent3d {
                    width: 512,
                    height: 512,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            
            // Create a separate texture for the view test since we can't clone wgpu::Texture
            let texture2 = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Test Texture 2"),
                size: wgpu::Extent3d {
                    width: 512,
                    height: 512,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            
            let texture = Arc::new(WgpuTexture::new(texture2, TextureFormat::Rgba8UnormSrgb));
            let wgpu_view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
            let view = WgpuTextureView::new(wgpu_view, texture.clone(), TextureFormat::Rgba8UnormSrgb);
            
            assert_eq!(view.format(), TextureFormat::Rgba8UnormSrgb);
            assert_eq!(view.texture().dimensions().width, 512);
        });
    }
    
    #[test]
    fn test_sampler_creation() {
        pollster::block_on(async {
            let (device, _queue) = create_test_device().await;
            
            let wgpu_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Test Sampler"),
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::MirrorRepeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });
            
            let sampler = WgpuSampler::new(
                wgpu_sampler,
                FilterMode::Nearest,
                FilterMode::Linear,
                AddressMode::Repeat,
                AddressMode::ClampToEdge,
                AddressMode::MirrorRepeat,
            );
            
            assert_eq!(sampler.min_filter(), FilterMode::Nearest);
            assert_eq!(sampler.mag_filter(), FilterMode::Linear);
            assert_eq!(sampler.address_mode_u(), AddressMode::Repeat);
            assert_eq!(sampler.address_mode_v(), AddressMode::ClampToEdge);
            assert_eq!(sampler.address_mode_w(), AddressMode::MirrorRepeat);
        });
    }
    
    #[test]
    fn test_format_conversions() {
        let test_cases = vec![
            (TextureFormat::Rgba8UnormSrgb, wgpu::TextureFormat::Rgba8UnormSrgb),
            (TextureFormat::Rgba8Unorm, wgpu::TextureFormat::Rgba8Unorm),
            (TextureFormat::Rgba32Float, wgpu::TextureFormat::Rgba32Float),
            (TextureFormat::Depth32Float, wgpu::TextureFormat::Depth32Float),
            (TextureFormat::Depth24PlusStencil8, wgpu::TextureFormat::Depth24PlusStencil8),
        ];
        
        for (engine_format, expected_wgpu) in test_cases {
            let converted = convert_texture_format(engine_format);
            assert_eq!(converted, expected_wgpu);
        }
    }
    
    #[test]
    fn test_filter_mode_conversions() {
        assert_eq!(convert_filter_mode(FilterMode::Nearest), wgpu::FilterMode::Nearest);
        assert_eq!(convert_filter_mode(FilterMode::Linear), wgpu::FilterMode::Linear);
    }
    
    #[test]
    fn test_address_mode_conversions() {
        assert_eq!(convert_address_mode(AddressMode::ClampToEdge), wgpu::AddressMode::ClampToEdge);
        assert_eq!(convert_address_mode(AddressMode::Repeat), wgpu::AddressMode::Repeat);
        assert_eq!(convert_address_mode(AddressMode::MirrorRepeat), wgpu::AddressMode::MirrorRepeat);
    }
    
    #[test]
    fn test_texture_with_mipmaps() {
        pollster::block_on(async {
            let (device, _queue) = create_test_device().await;
            
            let mip_level_count = 5;
            let wgpu_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Test Texture with Mipmaps"),
                size: wgpu::Extent3d {
                    width: 256,
                    height: 256,
                    depth_or_array_layers: 1,
                },
                mip_level_count,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            
            let texture = WgpuTexture::new(wgpu_texture, TextureFormat::Rgba8Unorm);
            assert_eq!(texture.mip_level_count(), mip_level_count);
        });
    }
}