use crate::{
    BufferDescriptor, GraphicsBuffer, GraphicsSampler, GraphicsTexture, GraphicsTextureView,
    Result, TextureDescriptor, AddressMode, FilterMode, GraphicsBindGroup, GraphicsBindGroupLayout,
    BindGroupDescriptor, BindGroupLayoutDescriptor, GraphicsCommandEncoder,
};

/// Main trait for graphics devices that create resources
pub trait GraphicsDevice: Send + Sync {
    /// Associated buffer type
    type Buffer: GraphicsBuffer;
    /// Associated texture type
    type Texture: GraphicsTexture;
    /// Associated texture view type
    type TextureView: GraphicsTextureView;
    /// Associated sampler type
    type Sampler: GraphicsSampler;
    /// Associated bind group type
    type BindGroup: GraphicsBindGroup;
    /// Associated bind group layout type
    type BindGroupLayout: GraphicsBindGroupLayout;
    /// Associated command encoder type
    type CommandEncoder: GraphicsCommandEncoder;
    
    /// Create a new buffer
    fn create_buffer(&self, desc: &BufferDescriptor) -> Result<Self::Buffer>;
    
    /// Create a new texture
    fn create_texture(&self, desc: &TextureDescriptor) -> Result<Self::Texture>;
    
    /// Create a view of a texture
    fn create_texture_view(&self, texture: &Self::Texture) -> Result<Self::TextureView>;
    
    /// Create a sampler
    fn create_sampler(&self, desc: &SamplerDescriptor) -> Result<Self::Sampler>;
    
    /// Create a bind group layout
    fn create_bind_group_layout(&self, desc: &BindGroupLayoutDescriptor) -> Result<Self::BindGroupLayout>;
    
    /// Create a bind group
    fn create_bind_group(&self, desc: &BindGroupDescriptor) -> Result<Self::BindGroup>;
    
    /// Create a command encoder
    fn create_command_encoder(&self) -> Result<Self::CommandEncoder>;
    
    /// Get device limits
    fn limits(&self) -> DeviceLimits;
    
    /// Get device features
    fn features(&self) -> DeviceFeatures;
}

/// Sampler descriptor for creation
#[derive(Debug, Clone)]
pub struct SamplerDescriptor {
    /// Minification filter
    pub min_filter: FilterMode,
    /// Magnification filter
    pub mag_filter: FilterMode,
    /// Address mode for U coordinate
    pub address_mode_u: AddressMode,
    /// Address mode for V coordinate
    pub address_mode_v: AddressMode,
    /// Address mode for W coordinate
    pub address_mode_w: AddressMode,
}

impl Default for SamplerDescriptor {
    fn default() -> Self {
        Self {
            min_filter: FilterMode::Linear,
            mag_filter: FilterMode::Linear,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
        }
    }
}

/// Device limits
#[derive(Debug, Clone)]
pub struct DeviceLimits {
    /// Maximum texture dimension (1D and 2D)
    pub max_texture_dimension_2d: u32,
    /// Maximum texture dimension (3D)
    pub max_texture_dimension_3d: u32,
    /// Maximum buffer size
    pub max_buffer_size: u64,
    /// Maximum vertex attributes
    pub max_vertex_attributes: u32,
    /// Maximum bind groups
    pub max_bind_groups: u32,
}

impl Default for DeviceLimits {
    fn default() -> Self {
        Self {
            max_texture_dimension_2d: 8192,
            max_texture_dimension_3d: 2048,
            max_buffer_size: 256 * 1024 * 1024, // 256 MB
            max_vertex_attributes: 16,
            max_bind_groups: 4,
        }
    }
}

/// Device features
#[derive(Debug, Clone, Default)]
pub struct DeviceFeatures {
    /// Depth clamping is supported
    pub depth_clamping: bool,
    /// Texture compression BC formats
    pub texture_compression_bc: bool,
    /// Texture compression ETC2 formats
    pub texture_compression_etc2: bool,
    /// Texture compression ASTC formats
    pub texture_compression_astc: bool,
    /// Anisotropic filtering
    pub anisotropic_filtering: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BufferUsage, Extent3d, TextureFormat, TextureUsage, GraphicsError,
        BindGroupEntry, BindGroupLayoutEntry, BindingResource, BindingType, BufferBinding,
        ShaderStages, TextureSampleType, TextureViewDimension,
    };
    use std::sync::Arc;
    
    // Mock implementations for testing
    struct MockBuffer {
        size: u64,
    }
    
    impl GraphicsBuffer for MockBuffer {
        fn write(&self, _offset: u64, _data: &[u8]) -> Result<()> {
            Ok(())
        }
        
        fn read(&self) -> Result<Vec<u8>> {
            Ok(vec![0; self.size as usize])
        }
        
        fn size(&self) -> u64 {
            self.size
        }
        
        fn map_write(&self) -> Result<crate::BufferMappedRange> {
            Err(GraphicsError::InvalidOperation("Not implemented".to_string()))
        }
        
        fn unmap(&self) {}
    }
    
    struct MockTexture {
        dimensions: Extent3d,
        format: TextureFormat,
    }
    
    impl GraphicsTexture for MockTexture {
        fn dimensions(&self) -> Extent3d {
            self.dimensions
        }
        
        fn format(&self) -> TextureFormat {
            self.format
        }
        
        fn mip_level_count(&self) -> u32 {
            1
        }
        
        fn sample_count(&self) -> u32 {
            1
        }
    }
    
    struct MockTextureView {
        texture: Arc<MockTexture>,
    }
    
    impl GraphicsTextureView for MockTextureView {
        fn texture(&self) -> &dyn GraphicsTexture {
            &*self.texture
        }
        
        fn format(&self) -> TextureFormat {
            self.texture.format
        }
    }
    
    struct MockSampler {
        desc: SamplerDescriptor,
    }
    
    impl GraphicsSampler for MockSampler {
        fn min_filter(&self) -> FilterMode {
            self.desc.min_filter
        }
        
        fn mag_filter(&self) -> FilterMode {
            self.desc.mag_filter
        }
        
        fn address_mode_u(&self) -> AddressMode {
            self.desc.address_mode_u
        }
        
        fn address_mode_v(&self) -> AddressMode {
            self.desc.address_mode_v
        }
        
        fn address_mode_w(&self) -> AddressMode {
            self.desc.address_mode_w
        }
    }
    
    struct MockBindGroupLayout {
        binding_count: u32,
    }
    
    impl GraphicsBindGroupLayout for MockBindGroupLayout {
        fn binding_count(&self) -> u32 {
            self.binding_count
        }
    }
    
    struct MockBindGroup {
        layout: Arc<MockBindGroupLayout>,
    }
    
    impl GraphicsBindGroup for MockBindGroup {
        fn layout(&self) -> &dyn GraphicsBindGroupLayout {
            &*self.layout
        }
    }
    
    // Mock command encoder
    struct MockCommandEncoder;
    
    impl GraphicsCommandEncoder for MockCommandEncoder {
        type RenderPass<'a> = MockRenderPass<'a> where Self: 'a;
        type ComputePass<'a> = MockComputePass<'a> where Self: 'a;
        
        fn begin_render_pass<'a>(&'a mut self, _desc: &crate::RenderPassDescriptor<'a>) -> Self::RenderPass<'a> {
            MockRenderPass {
                _phantom: std::marker::PhantomData,
            }
        }
        
        fn begin_compute_pass<'a>(&'a mut self) -> Self::ComputePass<'a> {
            MockComputePass {
                _phantom: std::marker::PhantomData,
            }
        }
        
        fn copy_buffer_to_buffer(&mut self, _: &dyn GraphicsBuffer, _: u64, _: &dyn GraphicsBuffer, _: u64, _: u64) {}
        
        fn finish(self) -> Result<Box<dyn crate::GraphicsCommandBuffer>> {
            Ok(Box::new(MockCommandBuffer))
        }
    }
    
    struct MockRenderPass<'a> {
        _phantom: std::marker::PhantomData<&'a ()>,
    }
    
    impl<'a> crate::GraphicsRenderPass<'a> for MockRenderPass<'a> {
        fn set_pipeline(&mut self, _: &'a dyn crate::GraphicsPipeline) {}
        fn set_bind_group(&mut self, _: u32, _: &'a dyn GraphicsBindGroup) {}
        fn set_vertex_buffer(&mut self, _: u32, _: &'a dyn GraphicsBuffer) {}
        fn set_index_buffer(&mut self, _: &'a dyn GraphicsBuffer, _: crate::IndexFormat) {}
        fn set_viewport(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {}
        fn set_scissor_rect(&mut self, _: u32, _: u32, _: u32, _: u32) {}
        fn draw(&mut self, _: u32, _: u32) {}
        fn draw_indexed(&mut self, _: u32, _: u32) {}
    }
    
    struct MockComputePass<'a> {
        _phantom: std::marker::PhantomData<&'a ()>,
    }
    
    impl<'a> crate::GraphicsComputePass<'a> for MockComputePass<'a> {
        fn set_pipeline(&mut self, _: &'a dyn crate::ComputePipeline) {}
        fn set_bind_group(&mut self, _: u32, _: &'a dyn GraphicsBindGroup) {}
        fn dispatch(&mut self, _: u32, _: u32, _: u32) {}
    }
    
    struct MockCommandBuffer;
    impl crate::GraphicsCommandBuffer for MockCommandBuffer {}
    
    struct MockDevice {
        limits: DeviceLimits,
        features: DeviceFeatures,
    }
    
    impl GraphicsDevice for MockDevice {
        type Buffer = MockBuffer;
        type Texture = MockTexture;
        type TextureView = MockTextureView;
        type Sampler = MockSampler;
        type BindGroup = MockBindGroup;
        type BindGroupLayout = MockBindGroupLayout;
        type CommandEncoder = MockCommandEncoder;
        
        fn create_buffer(&self, desc: &BufferDescriptor) -> Result<Self::Buffer> {
            if desc.size > self.limits.max_buffer_size {
                return Err(GraphicsError::InvalidOperation(
                    "Buffer size exceeds limit".to_string()
                ));
            }
            Ok(MockBuffer { size: desc.size })
        }
        
        fn create_texture(&self, desc: &TextureDescriptor) -> Result<Self::Texture> {
            if desc.size.width > self.limits.max_texture_dimension_2d {
                return Err(GraphicsError::InvalidOperation(
                    "Texture width exceeds limit".to_string()
                ));
            }
            Ok(MockTexture {
                dimensions: desc.size,
                format: desc.format,
            })
        }
        
        fn create_texture_view(&self, texture: &Self::Texture) -> Result<Self::TextureView> {
            Ok(MockTextureView {
                texture: Arc::new(MockTexture {
                    dimensions: texture.dimensions,
                    format: texture.format,
                }),
            })
        }
        
        fn create_sampler(&self, desc: &SamplerDescriptor) -> Result<Self::Sampler> {
            Ok(MockSampler {
                desc: desc.clone(),
            })
        }
        
        fn create_bind_group_layout(&self, desc: &BindGroupLayoutDescriptor) -> Result<Self::BindGroupLayout> {
            Ok(MockBindGroupLayout {
                binding_count: desc.entries.len() as u32,
            })
        }
        
        fn create_bind_group(&self, desc: &BindGroupDescriptor) -> Result<Self::BindGroup> {
            Ok(MockBindGroup {
                layout: Arc::new(MockBindGroupLayout {
                    binding_count: desc.layout.binding_count(),
                }),
            })
        }
        
        fn create_command_encoder(&self) -> Result<Self::CommandEncoder> {
            Ok(MockCommandEncoder)
        }
        
        fn limits(&self) -> DeviceLimits {
            self.limits.clone()
        }
        
        fn features(&self) -> DeviceFeatures {
            self.features.clone()
        }
    }
    
    #[test]
    fn test_device_create_buffer() {
        let device = MockDevice {
            limits: DeviceLimits::default(),
            features: DeviceFeatures::default(),
        };
        
        let desc = BufferDescriptor {
            size: 1024,
            usage: BufferUsage::VERTEX.union(BufferUsage::COPY_DST),
            mapped_at_creation: false,
        };
        
        let buffer = device.create_buffer(&desc).expect("Failed to create buffer");
        assert_eq!(buffer.size(), 1024);
    }
    
    #[test]
    fn test_device_create_buffer_exceeds_limit() {
        let device = MockDevice {
            limits: DeviceLimits {
                max_buffer_size: 1024,
                ..Default::default()
            },
            features: DeviceFeatures::default(),
        };
        
        let desc = BufferDescriptor {
            size: 2048,
            usage: BufferUsage::VERTEX,
            mapped_at_creation: false,
        };
        
        let result = device.create_buffer(&desc);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_device_create_texture() {
        let device = MockDevice {
            limits: DeviceLimits::default(),
            features: DeviceFeatures::default(),
        };
        
        let desc = TextureDescriptor {
            size: Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            },
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsage::TEXTURE_BINDING.union(TextureUsage::COPY_DST),
            mip_level_count: 1,
            sample_count: 1,
        };
        
        let texture = device.create_texture(&desc).expect("Failed to create texture");
        assert_eq!(texture.dimensions().width, 256);
        assert_eq!(texture.format(), TextureFormat::Rgba8Unorm);
    }
    
    #[test]
    fn test_device_create_sampler() {
        let device = MockDevice {
            limits: DeviceLimits::default(),
            features: DeviceFeatures::default(),
        };
        
        let desc = SamplerDescriptor {
            min_filter: FilterMode::Nearest,
            mag_filter: FilterMode::Linear,
            ..Default::default()
        };
        
        let sampler = device.create_sampler(&desc).expect("Failed to create sampler");
        assert_eq!(sampler.min_filter(), FilterMode::Nearest);
        assert_eq!(sampler.mag_filter(), FilterMode::Linear);
    }
    
    #[test]
    fn test_device_limits_and_features() {
        let device = MockDevice {
            limits: DeviceLimits {
                max_texture_dimension_2d: 4096,
                ..Default::default()
            },
            features: DeviceFeatures {
                anisotropic_filtering: true,
                ..Default::default()
            },
        };
        
        assert_eq!(device.limits().max_texture_dimension_2d, 4096);
        assert!(device.features().anisotropic_filtering);
    }
    
    #[test]
    fn test_device_create_bind_group_layout() {
        let device = MockDevice {
            limits: DeviceLimits::default(),
            features: DeviceFeatures::default(),
        };
        
        let desc = BindGroupLayoutDescriptor {
            entries: vec![
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Uniform,
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        };
        
        let layout = device.create_bind_group_layout(&desc).expect("Failed to create layout");
        assert_eq!(layout.binding_count(), 2);
    }
    
    #[test]
    fn test_device_create_bind_group() {
        let device = MockDevice {
            limits: DeviceLimits::default(),
            features: DeviceFeatures::default(),
        };
        
        let layout = MockBindGroupLayout { binding_count: 1 };
        let buffer = MockBuffer { size: 256 };
        
        let desc = BindGroupDescriptor {
            layout: &layout,
            entries: vec![
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        };
        
        let bind_group = device.create_bind_group(&desc).expect("Failed to create bind group");
        assert_eq!(bind_group.layout().binding_count(), 1);
    }
    
    #[test]
    fn test_device_create_command_encoder() {
        let device = MockDevice {
            limits: DeviceLimits::default(),
            features: DeviceFeatures::default(),
        };
        
        let encoder = device.create_command_encoder().expect("Failed to create command encoder");
        let _command_buffer = encoder.finish().expect("Failed to finish encoding");
    }
}