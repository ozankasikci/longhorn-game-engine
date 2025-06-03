// Scene renderer integration for WGPU with EGUI
// This module handles 3D scene rendering within the editor's Scene View

use wgpu::{Device, Queue, TextureView, CommandEncoder};
use engine_ecs_core::World;
use engine_components_3d::Transform;
use engine_components_3d::{Mesh, Material, Light};
use engine_camera::Camera;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SceneUniform {
    pub view_proj: [[f32; 4]; 4],
    pub model: [[f32; 4]; 4],
}

/// Scene renderer that integrates with EGUI's wgpu context
pub struct SceneRenderer {
    // Render pipeline and resources
    render_pipeline: Option<wgpu::RenderPipeline>,
    uniform_bind_group_layout: Option<wgpu::BindGroupLayout>,
    
    // Per-frame uniform buffer
    uniform_buffer: Option<wgpu::Buffer>,
    uniform_bind_group: Option<wgpu::BindGroup>,
    
    // Mesh resources
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    num_indices: u32,
    
    // Cached view/projection matrix
    view_proj_matrix: Mat4,
    
    // Initialization flag
    initialized: bool,
}

impl SceneRenderer {
    pub fn new() -> Self {
        Self {
            render_pipeline: None,
            uniform_bind_group_layout: None,
            uniform_buffer: None,
            uniform_bind_group: None,
            vertex_buffer: None,
            index_buffer: None,
            num_indices: 0,
            view_proj_matrix: Mat4::IDENTITY,
            initialized: false,
        }
    }
    
    /// Initialize renderer resources with wgpu device
    pub fn initialize(&mut self, device: &Device, surface_format: wgpu::TextureFormat) {
        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Scene Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("scene_shader.wgsl").into()),
        });
        
        // Create uniform bind group layout
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("scene_uniform_bind_group_layout"),
        });
        
        // Create render pipeline
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Scene Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Scene Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
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
        
        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Scene Uniform Buffer"),
            size: std::mem::size_of::<SceneUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create bind group
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scene_uniform_bind_group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        
        // Create a simple cube mesh for testing
        let (vertices, indices) = create_cube_mesh();
        
        use wgpu::util::DeviceExt;
        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Scene Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Scene Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        self.render_pipeline = Some(render_pipeline);
        self.uniform_bind_group_layout = Some(uniform_bind_group_layout);
        self.uniform_buffer = Some(uniform_buffer);
        self.uniform_bind_group = Some(uniform_bind_group);
        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        self.num_indices = indices.len() as u32;
        self.initialized = true;
    }
    
    /// Update camera view/projection matrix
    pub fn update_camera(&mut self, camera_transform: &Transform, camera: &Camera, aspect_ratio: f32) {
        // Calculate view matrix
        let eye = Vec3::from(camera_transform.position);
        
        // Calculate forward vector from rotation (assuming Y-up, -Z forward)
        // rotation values are already in radians
        let pitch = camera_transform.rotation[0];
        let yaw = camera_transform.rotation[1];
        
        let forward = Vec3::new(
            -yaw.sin() * pitch.cos(),
            pitch.sin(),
            -yaw.cos() * pitch.cos(),
        );
        
        let target = eye + forward;
        let up = Vec3::Y;
        
        let view = Mat4::look_at_rh(eye, target, up);
        
        // Calculate projection matrix
        let projection = Mat4::perspective_rh(
            camera.fov.to_radians(),
            aspect_ratio,
            camera.near,
            camera.far,
        );
        
        self.view_proj_matrix = projection * view;
    }
    
    /// Render the scene
    pub fn render(
        &self,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        depth_view: &TextureView,
        queue: &Queue,
        world: &World,
    ) {
        if !self.initialized {
            return;
        }
        
        let Some(render_pipeline) = &self.render_pipeline else { return };
        let Some(uniform_buffer) = &self.uniform_buffer else { return };
        let Some(uniform_bind_group) = &self.uniform_bind_group else { return };
        let Some(vertex_buffer) = &self.vertex_buffer else { return };
        let Some(index_buffer) = &self.index_buffer else { return };
        
        // Begin render pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Scene Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.1,
                        b: 0.15,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        
        render_pass.set_pipeline(render_pipeline);
        render_pass.set_bind_group(0, uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        
        // Query and render all entities with Transform + Mesh components
        let entities: Vec<_> = world.query_legacy::<Transform>().map(|(entity, _)| entity).collect();
        for entity in entities {
            if let Some(transform) = world.get_component::<Transform>(entity) {
                if let Some(_mesh) = world.get_component::<Mesh>(entity) {
                    // Calculate model matrix
                    let model = transform.matrix();
                    
                    // Update uniform buffer
                    let uniform_data = SceneUniform {
                        view_proj: self.view_proj_matrix.to_cols_array_2d(),
                        model: model.to_cols_array_2d(),
                    };
                    
                    queue.write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&[uniform_data]));
                    
                    // Draw mesh
                    render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
                }
            }
        }
    }
}

/// Create a simple cube mesh for testing
fn create_cube_mesh() -> (Vec<[f32; 8]>, Vec<u16>) {
    let vertices = vec![
        // Front face
        [-0.5, -0.5,  0.5,  0.0,  0.0,  1.0,  0.0, 0.0],
        [ 0.5, -0.5,  0.5,  0.0,  0.0,  1.0,  1.0, 0.0],
        [ 0.5,  0.5,  0.5,  0.0,  0.0,  1.0,  1.0, 1.0],
        [-0.5,  0.5,  0.5,  0.0,  0.0,  1.0,  0.0, 1.0],
        
        // Back face
        [-0.5, -0.5, -0.5,  0.0,  0.0, -1.0,  1.0, 0.0],
        [-0.5,  0.5, -0.5,  0.0,  0.0, -1.0,  1.0, 1.0],
        [ 0.5,  0.5, -0.5,  0.0,  0.0, -1.0,  0.0, 1.0],
        [ 0.5, -0.5, -0.5,  0.0,  0.0, -1.0,  0.0, 0.0],
        
        // Top face
        [-0.5,  0.5, -0.5,  0.0,  1.0,  0.0,  0.0, 1.0],
        [-0.5,  0.5,  0.5,  0.0,  1.0,  0.0,  0.0, 0.0],
        [ 0.5,  0.5,  0.5,  0.0,  1.0,  0.0,  1.0, 0.0],
        [ 0.5,  0.5, -0.5,  0.0,  1.0,  0.0,  1.0, 1.0],
        
        // Bottom face
        [-0.5, -0.5, -0.5,  0.0, -1.0,  0.0,  1.0, 1.0],
        [ 0.5, -0.5, -0.5,  0.0, -1.0,  0.0,  0.0, 1.0],
        [ 0.5, -0.5,  0.5,  0.0, -1.0,  0.0,  0.0, 0.0],
        [-0.5, -0.5,  0.5,  0.0, -1.0,  0.0,  1.0, 0.0],
        
        // Right face
        [ 0.5, -0.5, -0.5,  1.0,  0.0,  0.0,  1.0, 0.0],
        [ 0.5,  0.5, -0.5,  1.0,  0.0,  0.0,  1.0, 1.0],
        [ 0.5,  0.5,  0.5,  1.0,  0.0,  0.0,  0.0, 1.0],
        [ 0.5, -0.5,  0.5,  1.0,  0.0,  0.0,  0.0, 0.0],
        
        // Left face
        [-0.5, -0.5, -0.5, -1.0,  0.0,  0.0,  0.0, 0.0],
        [-0.5, -0.5,  0.5, -1.0,  0.0,  0.0,  1.0, 0.0],
        [-0.5,  0.5,  0.5, -1.0,  0.0,  0.0,  1.0, 1.0],
        [-0.5,  0.5, -0.5, -1.0,  0.0,  0.0,  0.0, 1.0],
    ];
    
    let indices = vec![
        0,  1,  2,  2,  3,  0,  // front
        4,  5,  6,  6,  7,  4,  // back
        8,  9, 10, 10, 11,  8,  // top
        12, 13, 14, 14, 15, 12, // bottom
        16, 17, 18, 18, 19, 16, // right
        20, 21, 22, 22, 23, 20, // left
    ];
    
    (vertices, indices)
}