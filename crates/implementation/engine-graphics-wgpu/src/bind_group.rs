use engine_graphics_traits::{
    BindingType, GraphicsBindGroup, GraphicsBindGroupLayout, ShaderStages, TextureSampleType,
    TextureViewDimension,
};
use std::sync::Arc;

/// WGPU bind group layout implementation
pub struct WgpuBindGroupLayout {
    layout: Arc<wgpu::BindGroupLayout>,
    binding_count: u32,
}

impl WgpuBindGroupLayout {
    /// Create a new WGPU bind group layout wrapper
    pub fn new(layout: wgpu::BindGroupLayout, binding_count: u32) -> Self {
        Self {
            layout: Arc::new(layout),
            binding_count,
        }
    }

    /// Get the underlying WGPU bind group layout
    pub fn raw(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }
}

impl GraphicsBindGroupLayout for WgpuBindGroupLayout {
    fn binding_count(&self) -> u32 {
        self.binding_count
    }
}

/// WGPU bind group implementation
pub struct WgpuBindGroup {
    bind_group: Arc<wgpu::BindGroup>,
    layout: Arc<WgpuBindGroupLayout>,
}

impl WgpuBindGroup {
    /// Create a new WGPU bind group wrapper
    pub fn new(bind_group: wgpu::BindGroup, layout: Arc<WgpuBindGroupLayout>) -> Self {
        Self {
            bind_group: Arc::new(bind_group),
            layout,
        }
    }

    /// Get the underlying WGPU bind group
    pub fn raw(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

impl GraphicsBindGroup for WgpuBindGroup {
    fn layout(&self) -> &dyn GraphicsBindGroupLayout {
        &*self.layout
    }
}

/// Convert engine shader stages to WGPU shader stages
pub fn convert_shader_stages(stages: ShaderStages) -> wgpu::ShaderStages {
    let mut wgpu_stages = wgpu::ShaderStages::empty();

    if stages.contains(ShaderStages::VERTEX) {
        wgpu_stages |= wgpu::ShaderStages::VERTEX;
    }
    if stages.contains(ShaderStages::FRAGMENT) {
        wgpu_stages |= wgpu::ShaderStages::FRAGMENT;
    }
    if stages.contains(ShaderStages::COMPUTE) {
        wgpu_stages |= wgpu::ShaderStages::COMPUTE;
    }

    wgpu_stages
}

/// Convert engine binding type to WGPU binding type
pub fn convert_binding_type(binding_type: BindingType) -> wgpu::BindingType {
    match binding_type {
        BindingType::Uniform => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        BindingType::StorageReadOnly => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        BindingType::Storage => wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        BindingType::Texture {
            sample_type,
            view_dimension,
            multisampled,
        } => wgpu::BindingType::Texture {
            sample_type: convert_texture_sample_type(sample_type),
            view_dimension: convert_texture_view_dimension(view_dimension),
            multisampled,
        },
        BindingType::Sampler { filtering } => wgpu::BindingType::Sampler(if filtering {
            wgpu::SamplerBindingType::Filtering
        } else {
            wgpu::SamplerBindingType::NonFiltering
        }),
    }
}

/// Convert engine texture sample type to WGPU texture sample type
fn convert_texture_sample_type(sample_type: TextureSampleType) -> wgpu::TextureSampleType {
    match sample_type {
        TextureSampleType::Float { filterable } => {
            if filterable {
                wgpu::TextureSampleType::Float { filterable: true }
            } else {
                wgpu::TextureSampleType::Float { filterable: false }
            }
        }
        TextureSampleType::Sint => wgpu::TextureSampleType::Sint,
        TextureSampleType::Uint => wgpu::TextureSampleType::Uint,
        TextureSampleType::Depth => wgpu::TextureSampleType::Depth,
    }
}

/// Convert engine texture view dimension to WGPU texture view dimension
fn convert_texture_view_dimension(dimension: TextureViewDimension) -> wgpu::TextureViewDimension {
    match dimension {
        TextureViewDimension::D1 => wgpu::TextureViewDimension::D1,
        TextureViewDimension::D2 => wgpu::TextureViewDimension::D2,
        TextureViewDimension::D2Array => wgpu::TextureViewDimension::D2Array,
        TextureViewDimension::Cube => wgpu::TextureViewDimension::Cube,
        TextureViewDimension::CubeArray => wgpu::TextureViewDimension::CubeArray,
        TextureViewDimension::D3 => wgpu::TextureViewDimension::D3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn should_skip_graphics_tests() -> bool {
        // Skip graphics tests in CI environments where GPU might not be available
        std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok()
    }

    async fn create_test_device() -> (wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: true,
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
    fn test_bind_group_layout_creation() {
        if should_skip_graphics_tests() {
            return;
        }
        pollster::block_on(async {
            let (device, _queue) = create_test_device().await;

            let wgpu_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Test Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

            let layout = WgpuBindGroupLayout::new(wgpu_layout, 2);
            assert_eq!(layout.binding_count(), 2);
        });
    }

    #[test]
    fn test_bind_group_creation() {
        if should_skip_graphics_tests() {
            return;
        }
        pollster::block_on(async {
            let (device, _queue) = create_test_device().await;

            // Create layout first
            let wgpu_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Test Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

            let _layout = Arc::new(WgpuBindGroupLayout::new(wgpu_layout, 1));

            // Create buffer for binding
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Test Buffer"),
                size: 64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            // Create a second layout for the bind group since we can't clone the first one
            let wgpu_layout2 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Test Layout 2"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

            // Create bind group
            let wgpu_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Test Bind Group"),
                layout: &wgpu_layout2,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

            let layout2 = Arc::new(WgpuBindGroupLayout::new(wgpu_layout2, 1));
            let bind_group = WgpuBindGroup::new(wgpu_bind_group, layout2);
            assert_eq!(bind_group.layout().binding_count(), 1);
        });
    }

    #[test]
    fn test_shader_stages_conversion() {
        let test_cases = vec![
            (ShaderStages::VERTEX, wgpu::ShaderStages::VERTEX),
            (ShaderStages::FRAGMENT, wgpu::ShaderStages::FRAGMENT),
            (ShaderStages::COMPUTE, wgpu::ShaderStages::COMPUTE),
        ];

        for (engine_stages, expected_wgpu) in test_cases {
            let converted = convert_shader_stages(engine_stages);
            assert_eq!(converted, expected_wgpu);
        }

        // Test combined stages
        let combined = ShaderStages::VERTEX_FRAGMENT;
        let converted = convert_shader_stages(combined);
        assert!(converted.contains(wgpu::ShaderStages::VERTEX));
        assert!(converted.contains(wgpu::ShaderStages::FRAGMENT));
    }

    #[test]
    fn test_binding_type_conversions() {
        // Test buffer bindings
        let uniform = convert_binding_type(BindingType::Uniform);
        match uniform {
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                ..
            } => {}
            _ => panic!("Wrong binding type for uniform"),
        }

        let storage = convert_binding_type(BindingType::Storage);
        match storage {
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                ..
            } => {}
            _ => panic!("Wrong binding type for storage"),
        }

        let storage_ro = convert_binding_type(BindingType::StorageReadOnly);
        match storage_ro {
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                ..
            } => {}
            _ => panic!("Wrong binding type for read-only storage"),
        }

        // Test texture binding
        let texture = convert_binding_type(BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        });
        match texture {
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            } => {}
            _ => panic!("Wrong binding type for texture"),
        }

        // Test sampler binding
        let sampler = convert_binding_type(BindingType::Sampler { filtering: true });
        match sampler {
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering) => {}
            _ => panic!("Wrong binding type for sampler"),
        }
    }

    #[test]
    fn test_texture_sample_type_conversions() {
        assert_eq!(
            convert_texture_sample_type(TextureSampleType::Float { filterable: true }),
            wgpu::TextureSampleType::Float { filterable: true }
        );

        assert_eq!(
            convert_texture_sample_type(TextureSampleType::Float { filterable: false }),
            wgpu::TextureSampleType::Float { filterable: false }
        );

        assert_eq!(
            convert_texture_sample_type(TextureSampleType::Sint),
            wgpu::TextureSampleType::Sint
        );

        assert_eq!(
            convert_texture_sample_type(TextureSampleType::Uint),
            wgpu::TextureSampleType::Uint
        );

        assert_eq!(
            convert_texture_sample_type(TextureSampleType::Depth),
            wgpu::TextureSampleType::Depth
        );
    }

    #[test]
    fn test_texture_view_dimension_conversions() {
        let test_cases = vec![
            (TextureViewDimension::D1, wgpu::TextureViewDimension::D1),
            (TextureViewDimension::D2, wgpu::TextureViewDimension::D2),
            (
                TextureViewDimension::D2Array,
                wgpu::TextureViewDimension::D2Array,
            ),
            (TextureViewDimension::Cube, wgpu::TextureViewDimension::Cube),
            (
                TextureViewDimension::CubeArray,
                wgpu::TextureViewDimension::CubeArray,
            ),
            (TextureViewDimension::D3, wgpu::TextureViewDimension::D3),
        ];

        for (engine_dim, expected_wgpu) in test_cases {
            let converted = convert_texture_view_dimension(engine_dim);
            assert_eq!(converted, expected_wgpu);
        }
    }

    #[test]
    fn test_complex_bind_group_layout() {
        if should_skip_graphics_tests() {
            return;
        }
        pollster::block_on(async {
            let (device, _queue) = create_test_device().await;

            // Create a complex layout with multiple binding types
            let wgpu_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Complex Layout"),
                entries: &[
                    // Uniform buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // Storage buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

            let layout = WgpuBindGroupLayout::new(wgpu_layout, 4);
            assert_eq!(layout.binding_count(), 4);
        });
    }
}
