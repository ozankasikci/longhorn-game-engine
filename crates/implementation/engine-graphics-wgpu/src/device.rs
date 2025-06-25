use engine_graphics_traits::*;
use std::sync::Arc;

/// Convert engine buffer usage to WGPU buffer usage
fn convert_buffer_usage(usage: BufferUsage) -> wgpu::BufferUsages {
    let mut wgpu_usage = wgpu::BufferUsages::empty();

    if usage.contains(BufferUsage::VERTEX) {
        wgpu_usage |= wgpu::BufferUsages::VERTEX;
    }
    if usage.contains(BufferUsage::INDEX) {
        wgpu_usage |= wgpu::BufferUsages::INDEX;
    }
    if usage.contains(BufferUsage::UNIFORM) {
        wgpu_usage |= wgpu::BufferUsages::UNIFORM;
    }
    if usage.contains(BufferUsage::STORAGE) {
        wgpu_usage |= wgpu::BufferUsages::STORAGE;
    }
    if usage.contains(BufferUsage::COPY_SRC) {
        wgpu_usage |= wgpu::BufferUsages::COPY_SRC;
    }
    if usage.contains(BufferUsage::COPY_DST) {
        wgpu_usage |= wgpu::BufferUsages::COPY_DST;
    }

    wgpu_usage
}

/// WGPU device implementation
pub struct WgpuDevice {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    limits: DeviceLimits,
    features: DeviceFeatures,
}

impl WgpuDevice {
    /// Create a new WGPU device wrapper
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let limits = DeviceLimits {
            max_texture_dimension_2d: device.limits().max_texture_dimension_2d,
            max_texture_dimension_3d: device.limits().max_texture_dimension_3d,
            max_buffer_size: device.limits().max_buffer_size as u64,
            max_vertex_attributes: device.limits().max_vertex_attributes,
            max_bind_groups: device.limits().max_bind_groups,
        };

        let features = DeviceFeatures {
            depth_clamping: device
                .features()
                .contains(wgpu::Features::DEPTH_CLIP_CONTROL),
            texture_compression_bc: device
                .features()
                .contains(wgpu::Features::TEXTURE_COMPRESSION_BC),
            texture_compression_etc2: device
                .features()
                .contains(wgpu::Features::TEXTURE_COMPRESSION_ETC2),
            texture_compression_astc: device
                .features()
                .contains(wgpu::Features::TEXTURE_COMPRESSION_ASTC),
            anisotropic_filtering: true, // Generally available
        };

        Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            limits,
            features,
        }
    }

    /// Get the underlying WGPU device
    pub fn raw_device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get the underlying WGPU queue
    pub fn raw_queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

// Import texture types from texture module
pub use crate::texture::{WgpuSampler, WgpuTexture, WgpuTextureView};
// Import bind group types from bind_group module
pub use crate::bind_group::{WgpuBindGroup, WgpuBindGroupLayout};

/// Convert engine texture usage to WGPU texture usage
fn convert_texture_usage(usage: TextureUsage) -> wgpu::TextureUsages {
    let mut wgpu_usage = wgpu::TextureUsages::empty();

    if usage.contains(TextureUsage::COPY_SRC) {
        wgpu_usage |= wgpu::TextureUsages::COPY_SRC;
    }
    if usage.contains(TextureUsage::COPY_DST) {
        wgpu_usage |= wgpu::TextureUsages::COPY_DST;
    }
    if usage.contains(TextureUsage::TEXTURE_BINDING) {
        wgpu_usage |= wgpu::TextureUsages::TEXTURE_BINDING;
    }
    if usage.contains(TextureUsage::STORAGE_BINDING) {
        wgpu_usage |= wgpu::TextureUsages::STORAGE_BINDING;
    }
    if usage.contains(TextureUsage::RENDER_ATTACHMENT) {
        wgpu_usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
    }

    wgpu_usage
}

// Bind group types are now imported from bind_group module

pub struct WgpuCommandEncoder;
impl GraphicsCommandEncoder for WgpuCommandEncoder {
    type RenderPass<'a>
        = WgpuRenderPass<'a>
    where
        Self: 'a;
    type ComputePass<'a>
        = WgpuComputePass<'a>
    where
        Self: 'a;

    fn begin_render_pass<'a>(
        &'a mut self,
        _desc: &RenderPassDescriptor<'a>,
    ) -> Self::RenderPass<'a> {
        todo!()
    }

    fn begin_compute_pass<'a>(&'a mut self) -> Self::ComputePass<'a> {
        todo!()
    }

    fn copy_buffer_to_buffer(
        &mut self,
        _source: &dyn GraphicsBuffer,
        _source_offset: u64,
        _destination: &dyn GraphicsBuffer,
        _destination_offset: u64,
        _copy_size: u64,
    ) {
        todo!()
    }

    fn finish(self) -> Result<Box<dyn GraphicsCommandBuffer>> {
        todo!()
    }
}

pub struct WgpuRenderPass<'a> {
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> GraphicsRenderPass<'a> for WgpuRenderPass<'a> {
    fn set_pipeline(&mut self, _pipeline: &'a dyn GraphicsPipeline) {
        todo!()
    }
    fn set_bind_group(&mut self, _index: u32, _bind_group: &'a dyn GraphicsBindGroup) {
        todo!()
    }
    fn set_vertex_buffer(&mut self, _slot: u32, _buffer: &'a dyn GraphicsBuffer) {
        todo!()
    }
    fn set_index_buffer(&mut self, _buffer: &'a dyn GraphicsBuffer, _format: IndexFormat) {
        todo!()
    }
    fn set_viewport(
        &mut self,
        _x: f32,
        _y: f32,
        _width: f32,
        _height: f32,
        _min_depth: f32,
        _max_depth: f32,
    ) {
        todo!()
    }
    fn set_scissor_rect(&mut self, _x: u32, _y: u32, _width: u32, _height: u32) {
        todo!()
    }
    fn draw(&mut self, _vertices: u32, _instances: u32) {
        todo!()
    }
    fn draw_indexed(&mut self, _indices: u32, _instances: u32) {
        todo!()
    }
}

pub struct WgpuComputePass<'a> {
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> GraphicsComputePass<'a> for WgpuComputePass<'a> {
    fn set_pipeline(&mut self, _pipeline: &'a dyn ComputePipeline) {
        todo!()
    }
    fn set_bind_group(&mut self, _index: u32, _bind_group: &'a dyn GraphicsBindGroup) {
        todo!()
    }
    fn dispatch(&mut self, _x: u32, _y: u32, _z: u32) {
        todo!()
    }
}

impl GraphicsDevice for WgpuDevice {
    type Buffer = crate::buffer::WgpuBuffer;
    type Texture = WgpuTexture;
    type TextureView = WgpuTextureView;
    type Sampler = WgpuSampler;
    type BindGroup = WgpuBindGroup;
    type BindGroupLayout = WgpuBindGroupLayout;
    type CommandEncoder = WgpuCommandEncoder;

    fn create_buffer(&self, desc: &BufferDescriptor) -> Result<Self::Buffer> {
        let usage = convert_buffer_usage(desc.usage);

        let wgpu_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: desc.size,
            usage,
            mapped_at_creation: desc.mapped_at_creation,
        });

        Ok(crate::buffer::WgpuBuffer::new(wgpu_buffer))
    }

    fn create_texture(&self, desc: &TextureDescriptor) -> Result<Self::Texture> {
        let format = crate::texture::convert_texture_format(desc.format);
        let usage = convert_texture_usage(desc.usage);

        let wgpu_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: desc.size.width,
                height: desc.size.height,
                depth_or_array_layers: desc.size.depth_or_array_layers,
            },
            mip_level_count: desc.mip_level_count,
            sample_count: desc.sample_count,
            dimension: wgpu::TextureDimension::D2, // Default to 2D for now
            format,
            usage,
            view_formats: &[],
        });

        Ok(WgpuTexture::new(wgpu_texture, desc.format))
    }

    fn create_texture_view(&self, _texture: &Self::Texture) -> Result<Self::TextureView> {
        // For now, we'll skip this implementation as it requires texture cloning
        // This will be improved when we implement bind groups
        Err(GraphicsError::InvalidOperation(
            "Texture view creation not yet implemented".to_string(),
        ))
    }

    fn create_sampler(&self, desc: &SamplerDescriptor) -> Result<Self::Sampler> {
        let min_filter = crate::texture::convert_filter_mode(desc.min_filter);
        let mag_filter = crate::texture::convert_filter_mode(desc.mag_filter);
        let address_mode_u = crate::texture::convert_address_mode(desc.address_mode_u);
        let address_mode_v = crate::texture::convert_address_mode(desc.address_mode_v);
        let address_mode_w = crate::texture::convert_address_mode(desc.address_mode_w);

        let wgpu_sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u,
            address_mode_v,
            address_mode_w,
            mag_filter,
            min_filter,
            ..Default::default()
        });

        Ok(WgpuSampler::new(
            wgpu_sampler,
            desc.min_filter,
            desc.mag_filter,
            desc.address_mode_u,
            desc.address_mode_v,
            desc.address_mode_w,
        ))
    }

    fn create_bind_group_layout(
        &self,
        desc: &BindGroupLayoutDescriptor,
    ) -> Result<Self::BindGroupLayout> {
        let entries: Vec<wgpu::BindGroupLayoutEntry> = desc
            .entries
            .iter()
            .map(|entry| wgpu::BindGroupLayoutEntry {
                binding: entry.binding,
                visibility: crate::bind_group::convert_shader_stages(entry.visibility),
                ty: crate::bind_group::convert_binding_type(entry.ty.clone()),
                count: entry.count.map(|c| std::num::NonZeroU32::new(c).unwrap()),
            })
            .collect();

        let wgpu_layout = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &entries,
            });

        Ok(WgpuBindGroupLayout::new(
            wgpu_layout,
            desc.entries.len() as u32,
        ))
    }

    fn create_bind_group(&self, desc: &BindGroupDescriptor) -> Result<Self::BindGroup> {
        // Convert binding resources
        let entries: Result<Vec<wgpu::BindGroupEntry>> = desc.entries.iter().map(|entry| {
            let resource = match &entry.resource {
                BindingResource::Buffer(buffer_binding) => {
                    // We need to convert from trait object to concrete type
                    // This is a limitation of the current design - we'll handle it for now
                    return Err(GraphicsError::InvalidOperation(
                        "Buffer binding not yet implemented - requires concrete buffer access".to_string()
                    ));
                },
                BindingResource::TextureView(_view) => {
                    return Err(GraphicsError::InvalidOperation(
                        "Texture view binding not yet implemented - requires concrete view access".to_string()
                    ));
                },
                BindingResource::Sampler(_sampler) => {
                    return Err(GraphicsError::InvalidOperation(
                        "Sampler binding not yet implemented - requires concrete sampler access".to_string()
                    ));
                },
            };
        }).collect();

        // For now, return an error until we implement proper resource binding
        Err(GraphicsError::InvalidOperation(
            "Bind group creation not fully implemented yet".to_string(),
        ))
    }

    fn create_command_encoder(&self) -> Result<Self::CommandEncoder> {
        todo!()
    }

    fn limits(&self) -> DeviceLimits {
        self.limits.clone()
    }

    fn features(&self) -> DeviceFeatures {
        self.features.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_device() -> WgpuDevice {
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

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Test Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Failed to request device");

        WgpuDevice::new(device, queue)
    }

    #[test]
    fn test_device_creation() {
        pollster::block_on(async {
            let device = create_test_device().await;

            // Check limits
            let limits = device.limits();
            assert!(limits.max_texture_dimension_2d > 0);
            assert!(limits.max_buffer_size > 0);

            // Check features
            let features = device.features();
            // Just check it returns valid features
            let _ = features.anisotropic_filtering;
        });
    }

    #[test]
    fn test_device_create_buffer() {
        pollster::block_on(async {
            let device = create_test_device().await;

            let desc = BufferDescriptor {
                size: 1024,
                usage: BufferUsage::VERTEX.union(BufferUsage::COPY_DST),
                mapped_at_creation: false,
            };

            let buffer = device
                .create_buffer(&desc)
                .expect("Failed to create buffer");
            assert_eq!(buffer.size(), 1024);
        });
    }

    #[test]
    fn test_buffer_usage_conversion() {
        let test_cases = vec![
            (BufferUsage::VERTEX, wgpu::BufferUsages::VERTEX),
            (BufferUsage::INDEX, wgpu::BufferUsages::INDEX),
            (BufferUsage::UNIFORM, wgpu::BufferUsages::UNIFORM),
            (BufferUsage::STORAGE, wgpu::BufferUsages::STORAGE),
            (BufferUsage::COPY_SRC, wgpu::BufferUsages::COPY_SRC),
            (BufferUsage::COPY_DST, wgpu::BufferUsages::COPY_DST),
        ];

        for (engine_usage, expected_wgpu) in test_cases {
            let converted = convert_buffer_usage(engine_usage);
            assert_eq!(converted, expected_wgpu);
        }

        // Test combined usage
        let combined = BufferUsage::VERTEX.union(BufferUsage::COPY_DST);
        let converted = convert_buffer_usage(combined);
        assert!(converted.contains(wgpu::BufferUsages::VERTEX));
        assert!(converted.contains(wgpu::BufferUsages::COPY_DST));
    }

    #[test]
    fn test_create_various_buffer_types() {
        pollster::block_on(async {
            let device = create_test_device().await;

            // Vertex buffer
            let vertex_desc = BufferDescriptor {
                size: 256,
                usage: BufferUsage::VERTEX,
                mapped_at_creation: false,
            };
            let vertex_buffer = device.create_buffer(&vertex_desc);
            assert!(vertex_buffer.is_ok());

            // Index buffer
            let index_desc = BufferDescriptor {
                size: 128,
                usage: BufferUsage::INDEX,
                mapped_at_creation: false,
            };
            let index_buffer = device.create_buffer(&index_desc);
            assert!(index_buffer.is_ok());

            // Uniform buffer
            let uniform_desc = BufferDescriptor {
                size: 64,
                usage: BufferUsage::UNIFORM.union(BufferUsage::COPY_DST),
                mapped_at_creation: false,
            };
            let uniform_buffer = device.create_buffer(&uniform_desc);
            assert!(uniform_buffer.is_ok());
        });
    }

    #[test]
    fn test_device_create_texture() {
        pollster::block_on(async {
            let device = create_test_device().await;

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

            let texture = device
                .create_texture(&desc)
                .expect("Failed to create texture");
            assert_eq!(texture.dimensions().width, 256);
            assert_eq!(texture.dimensions().height, 256);
            assert_eq!(texture.format(), TextureFormat::Rgba8Unorm);
            assert_eq!(texture.mip_level_count(), 1);
            assert_eq!(texture.sample_count(), 1);
        });
    }

    #[test]
    fn test_device_create_sampler() {
        pollster::block_on(async {
            let device = create_test_device().await;

            let desc = SamplerDescriptor {
                min_filter: FilterMode::Linear,
                mag_filter: FilterMode::Nearest,
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::MirrorRepeat,
            };

            let sampler = device
                .create_sampler(&desc)
                .expect("Failed to create sampler");
            assert_eq!(sampler.min_filter(), FilterMode::Linear);
            assert_eq!(sampler.mag_filter(), FilterMode::Nearest);
            assert_eq!(sampler.address_mode_u(), AddressMode::Repeat);
            assert_eq!(sampler.address_mode_v(), AddressMode::ClampToEdge);
            assert_eq!(sampler.address_mode_w(), AddressMode::MirrorRepeat);
        });
    }

    #[test]
    fn test_texture_usage_conversion() {
        let test_cases = vec![
            (TextureUsage::COPY_SRC, wgpu::TextureUsages::COPY_SRC),
            (TextureUsage::COPY_DST, wgpu::TextureUsages::COPY_DST),
            (
                TextureUsage::TEXTURE_BINDING,
                wgpu::TextureUsages::TEXTURE_BINDING,
            ),
            (
                TextureUsage::STORAGE_BINDING,
                wgpu::TextureUsages::STORAGE_BINDING,
            ),
            (
                TextureUsage::RENDER_ATTACHMENT,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            ),
        ];

        for (engine_usage, expected_wgpu) in test_cases {
            let converted = convert_texture_usage(engine_usage);
            assert_eq!(converted, expected_wgpu);
        }

        // Test combined usage
        let combined = TextureUsage::TEXTURE_BINDING.union(TextureUsage::COPY_DST);
        let converted = convert_texture_usage(combined);
        assert!(converted.contains(wgpu::TextureUsages::TEXTURE_BINDING));
        assert!(converted.contains(wgpu::TextureUsages::COPY_DST));
    }

    #[test]
    fn test_create_various_textures() {
        pollster::block_on(async {
            let device = create_test_device().await;

            // Color texture
            let color_desc = TextureDescriptor {
                size: Extent3d {
                    width: 128,
                    height: 128,
                    depth_or_array_layers: 1,
                },
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsage::TEXTURE_BINDING.union(TextureUsage::RENDER_ATTACHMENT),
                mip_level_count: 1,
                sample_count: 1,
            };
            let color_texture = device.create_texture(&color_desc);
            assert!(color_texture.is_ok());

            // Depth texture
            let depth_desc = TextureDescriptor {
                size: Extent3d {
                    width: 256,
                    height: 256,
                    depth_or_array_layers: 1,
                },
                format: TextureFormat::Depth32Float,
                usage: TextureUsage::RENDER_ATTACHMENT,
                mip_level_count: 1,
                sample_count: 1,
            };
            let depth_texture = device.create_texture(&depth_desc);
            assert!(depth_texture.is_ok());

            // HDR texture
            let hdr_desc = TextureDescriptor {
                size: Extent3d {
                    width: 512,
                    height: 512,
                    depth_or_array_layers: 1,
                },
                format: TextureFormat::Rgba32Float,
                usage: TextureUsage::STORAGE_BINDING.union(TextureUsage::TEXTURE_BINDING),
                mip_level_count: 3,
                sample_count: 1,
            };
            let hdr_texture = device.create_texture(&hdr_desc);
            assert!(hdr_texture.is_ok());
        });
    }

    #[test]
    fn test_device_create_bind_group_layout() {
        pollster::block_on(async {
            let device = create_test_device().await;

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
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler { filtering: true },
                        count: None,
                    },
                ],
            };

            let layout = device
                .create_bind_group_layout(&desc)
                .expect("Failed to create bind group layout");
            assert_eq!(layout.binding_count(), 3);
        });
    }

    #[test]
    fn test_bind_group_layout_with_storage_buffer() {
        pollster::block_on(async {
            let device = create_test_device().await;

            let desc = BindGroupLayoutDescriptor {
                entries: vec![
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Storage,
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageReadOnly,
                        count: None,
                    },
                ],
            };

            let layout = device
                .create_bind_group_layout(&desc)
                .expect("Failed to create storage layout");
            assert_eq!(layout.binding_count(), 2);
        });
    }

    #[test]
    fn test_bind_group_creation_returns_error() {
        pollster::block_on(async {
            let device = create_test_device().await;

            // Create a simple layout
            let layout_desc = BindGroupLayoutDescriptor {
                entries: vec![BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Uniform,
                    count: None,
                }],
            };

            let layout = device
                .create_bind_group_layout(&layout_desc)
                .expect("Failed to create layout");

            // Try to create bind group - should return error for now
            let bind_group_desc = BindGroupDescriptor {
                layout: &layout,
                entries: vec![], // Empty for now since we can't create proper resources yet
            };

            let result = device.create_bind_group(&bind_group_desc);
            assert!(result.is_err());
        });
    }
}
