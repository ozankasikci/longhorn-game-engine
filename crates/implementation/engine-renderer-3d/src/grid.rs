//! Grid rendering for spatial reference in 3D scenes
//!
//! Provides a configurable grid that can be rendered in the scene view
//! to help with spatial orientation and object placement.

use glam::{Mat4, Vec3};
use std::sync::Arc;
use wgpu::util::DeviceExt;

/// Grid configuration
#[derive(Debug, Clone)]
pub struct GridConfig {
    /// Grid size in world units (total size, not radius)
    pub size: f32,

    /// Spacing between grid lines
    pub spacing: f32,

    /// Major line interval (every N lines is a major line)
    pub major_interval: u32,

    /// Minor line color (RGBA)
    pub minor_color: [f32; 4],

    /// Major line color (RGBA)
    pub major_color: [f32; 4],

    /// Axis line colors (X=red, Z=blue)
    pub axis_x_color: [f32; 4],
    pub axis_z_color: [f32; 4],

    /// Whether to fade grid lines with distance
    pub fade_distance: bool,

    /// Maximum distance for grid visibility
    pub max_distance: f32,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            size: 100.0,
            spacing: 1.0,
            major_interval: 10,
            minor_color: [0.3, 0.3, 0.3, 0.3],
            major_color: [0.5, 0.5, 0.5, 0.5],
            axis_x_color: [1.0, 0.0, 0.0, 0.8],
            axis_z_color: [0.0, 0.0, 1.0, 0.8],
            fade_distance: true,
            max_distance: 50.0,
        }
    }
}

/// Grid vertex data
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl GridVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,  // position
        1 => Float32x4,  // color
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GridVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

/// Grid renderer
pub struct GridRenderer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    config: GridConfig,
}

/// Grid uniforms for shader
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GridUniforms {
    view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 4],
    fade_params: [f32; 4], // x: fade_enabled, y: max_distance, z: unused, w: unused
}

impl GridRenderer {
    /// Create a new grid renderer
    pub fn new(
        device: Arc<wgpu::Device>,
        format: wgpu::TextureFormat,
        config: GridConfig,
    ) -> Result<Self, anyhow::Error> {
        // Generate grid vertices
        let (vertices, indices) = generate_grid_mesh(&config);

        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create index buffer
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Grid Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/grid.wgsl").into()),
        });

        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grid Uniform Buffer"),
            size: std::mem::size_of::<GridUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Grid Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Grid Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Grid Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Grid Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[GridVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Ok(Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            pipeline,
            uniform_buffer,
            bind_group,
            config,
        })
    }

    /// Update grid configuration
    pub fn update_config(&mut self, device: Arc<wgpu::Device>, config: GridConfig) {
        // Regenerate mesh if needed
        let (vertices, indices) = generate_grid_mesh(&config);

        // Recreate buffers
        self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        self.index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        self.num_indices = indices.len() as u32;
        self.config = config;
    }

    /// Render the grid
    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        view_proj: &Mat4,
        camera_pos: Vec3,
        queue: &wgpu::Queue,
    ) {
        // Update uniforms
        let uniforms = GridUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: [camera_pos.x, camera_pos.y, camera_pos.z, 1.0],
            fade_params: [
                if self.config.fade_distance { 1.0 } else { 0.0 },
                self.config.max_distance,
                0.0,
                0.0,
            ],
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // Set pipeline and resources
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        // Draw
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}

/// Generate grid mesh vertices and indices
fn generate_grid_mesh(config: &GridConfig) -> (Vec<GridVertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let half_size = config.size / 2.0;
    let line_count = (config.size / config.spacing) as i32;
    let half_lines = line_count / 2;

    let mut vertex_index = 0u16;

    // Generate grid lines parallel to X axis (along Z)
    for i in -half_lines..=half_lines {
        let z = i as f32 * config.spacing;
        let is_major = i % config.major_interval as i32 == 0;
        let is_axis = i == 0;

        let color = if is_axis {
            config.axis_x_color
        } else if is_major {
            config.major_color
        } else {
            config.minor_color
        };

        // Start vertex
        vertices.push(GridVertex {
            position: [-half_size, 0.0, z],
            color,
        });

        // End vertex
        vertices.push(GridVertex {
            position: [half_size, 0.0, z],
            color,
        });

        // Add indices for this line
        indices.push(vertex_index);
        indices.push(vertex_index + 1);
        vertex_index += 2;
    }

    // Generate grid lines parallel to Z axis (along X)
    for i in -half_lines..=half_lines {
        let x = i as f32 * config.spacing;
        let is_major = i % config.major_interval as i32 == 0;
        let is_axis = i == 0;

        let color = if is_axis {
            config.axis_z_color
        } else if is_major {
            config.major_color
        } else {
            config.minor_color
        };

        // Start vertex
        vertices.push(GridVertex {
            position: [x, 0.0, -half_size],
            color,
        });

        // End vertex
        vertices.push(GridVertex {
            position: [x, 0.0, half_size],
            color,
        });

        // Add indices for this line
        indices.push(vertex_index);
        indices.push(vertex_index + 1);
        vertex_index += 2;
    }

    (vertices, indices)
}
