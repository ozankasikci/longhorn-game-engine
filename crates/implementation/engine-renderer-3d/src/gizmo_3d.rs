//! 3D Gizmo rendering for transform manipulation
//!
//! Provides Unity-style gizmos for position, rotation, and scale manipulation
//! with proper 3D rendering and constant screen-space sizing.

use glam::{Vec3, Vec4, Mat4, Quat};
use wgpu::util::DeviceExt;
use std::sync::Arc;
use bytemuck::{Pod, Zeroable};

/// Gizmo mode determines which type of manipulation is active
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoMode {
    None,
    Translation,
    Rotation,
    Scale,
}

/// Gizmo component that can be interacted with
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoComponent {
    None,
    // Translation components
    AxisX,
    AxisY,
    AxisZ,
    PlaneXY,
    PlaneXZ,
    PlaneYZ,
    Center,
    // Rotation components
    RotationX,
    RotationY,
    RotationZ,
    RotationScreen,
    // Scale components
    ScaleX,
    ScaleY,
    ScaleZ,
    ScaleUniform,
}

/// Vertex data for gizmo rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GizmoVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl GizmoVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,  // position
        1 => Float32x4,  // color
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GizmoVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

/// Gizmo uniforms for shader
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GizmoUniforms {
    pub model: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub gizmo_position: [f32; 4],
    pub camera_position: [f32; 4],
    pub viewport_size: [f32; 4], // x: width, y: height, z: gizmo_size, w: unused
    pub highlight_color: [f32; 4],
}

/// Mesh data for a gizmo component
pub struct GizmoMesh {
    pub vertices: Vec<GizmoVertex>,
    pub indices: Vec<u16>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

/// 3D Gizmo renderer
pub struct GizmoRenderer3D {
    // Rendering pipeline
    pipeline: wgpu::RenderPipeline,
    
    // Meshes for different gizmo components
    arrow_x_mesh: GizmoMesh,  // Red arrow for X axis
    arrow_y_mesh: GizmoMesh,  // Green arrow for Y axis  
    arrow_z_mesh: GizmoMesh,  // Blue arrow for Z axis
    plane_mesh: GizmoMesh,
    circle_mesh: GizmoMesh,
    box_mesh: GizmoMesh,
    
    // Uniforms
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    
    // State
    mode: GizmoMode,
    highlighted_component: Option<GizmoComponent>,
    active_component: Option<GizmoComponent>,
    gizmo_size: f32,
}

impl GizmoRenderer3D {
    /// Create a new gizmo renderer
    pub fn new(
        device: Arc<wgpu::Device>,
        format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Result<Self, anyhow::Error> {
        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Gizmo 3D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/gizmo_3d.wgsl").into()),
        });
        
        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gizmo Uniform Buffer"),
            size: std::mem::size_of::<GizmoUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Gizmo Bind Group Layout"),
            entries: &[
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
            ],
        });
        
        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Gizmo Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Gizmo Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Gizmo Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[GizmoVertex::desc()],
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
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // No culling for gizmos
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: false, // Don't write to depth
                depth_compare: wgpu::CompareFunction::Always, // Always render on top
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
        
        // Create gizmo meshes with axis-specific colors
        let arrow_x_mesh = create_arrow_mesh_with_color(&device, [1.0, 0.2, 0.2, 1.0]); // Red
        let arrow_y_mesh = create_arrow_mesh_with_color(&device, [0.2, 1.0, 0.2, 1.0]); // Green  
        let arrow_z_mesh = create_arrow_mesh_with_color(&device, [0.2, 0.2, 1.0, 1.0]); // Blue
        let plane_mesh = create_plane_mesh(&device);
        let circle_mesh = create_circle_mesh(&device);
        let box_mesh = create_box_mesh(&device);
        
        Ok(Self {
            pipeline,
            arrow_x_mesh,
            arrow_y_mesh,
            arrow_z_mesh,
            plane_mesh,
            circle_mesh,
            box_mesh,
            uniform_buffer,
            bind_group,
            mode: GizmoMode::None,
            highlighted_component: None,
            active_component: None,
            gizmo_size: 100.0, // Default size in pixels
        })
    }
    
    /// Set the current gizmo mode
    pub fn set_mode(&mut self, mode: GizmoMode) {
        self.mode = mode;
        self.highlighted_component = None;
        self.active_component = None;
    }
    
    /// Set highlighted component
    pub fn set_highlighted(&mut self, component: Option<GizmoComponent>) {
        self.highlighted_component = component;
    }
    
    /// Set active (dragging) component
    pub fn set_active(&mut self, component: Option<GizmoComponent>) {
        self.active_component = component;
    }
    
    /// Render the gizmo
    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        gizmo_transform: &Mat4,
        view_matrix: &Mat4,
        projection_matrix: &Mat4,
        camera_position: Vec3,
        viewport_size: (u32, u32),
        queue: &wgpu::Queue,
    ) {
        if self.mode == GizmoMode::None {
            return;
        }
        
        log::info!("=== Rendering Gizmo ===");
        log::info!("Mode: {:?}", self.mode);
        log::info!("Viewport size: {:?}", viewport_size);
        
        // Extract gizmo position from transform
        let gizmo_position = Vec3::new(
            gizmo_transform.col(3).x,
            gizmo_transform.col(3).y,
            gizmo_transform.col(3).z,
        );
        
        log::info!("Gizmo position: {:?}", gizmo_position);
        log::info!("Camera position: {:?}", camera_position);
        
        // Update uniforms
        let uniforms = GizmoUniforms {
            model: gizmo_transform.to_cols_array_2d(),
            view: view_matrix.to_cols_array_2d(),
            projection: projection_matrix.to_cols_array_2d(),
            gizmo_position: [gizmo_position.x, gizmo_position.y, gizmo_position.z, 1.0],
            camera_position: [camera_position.x, camera_position.y, camera_position.z, 1.0],
            viewport_size: [viewport_size.0 as f32, viewport_size.1 as f32, self.gizmo_size, 0.0],
            highlight_color: [1.0, 1.0, 0.0, 1.0], // Yellow for highlight
        };
        
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
        
        // Set pipeline and bind group
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        
        // Render appropriate gizmo based on mode
        match self.mode {
            GizmoMode::Translation => self.render_translation_gizmo(render_pass),
            GizmoMode::Rotation => self.render_rotation_gizmo(render_pass),
            GizmoMode::Scale => self.render_scale_gizmo(render_pass),
            GizmoMode::None => {}
        }
    }
    
    /// Render translation gizmo (arrows and planes)
    fn render_translation_gizmo<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        log::info!("Rendering translation gizmo with 3 colored arrows");
        
        // Render each axis with its specific mesh (which has the correct color)
        
        // X axis - Red arrow
        render_pass.set_vertex_buffer(0, self.arrow_x_mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.arrow_x_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.arrow_x_mesh.index_count, 0, 0..1);
        log::info!("Drew X axis arrow with {} indices", self.arrow_x_mesh.index_count);
        
        // Y axis - Green arrow  
        render_pass.set_vertex_buffer(0, self.arrow_y_mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.arrow_y_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.arrow_y_mesh.index_count, 0, 0..1);
        log::info!("Drew Y axis arrow with {} indices", self.arrow_y_mesh.index_count);
        
        // Z axis - Blue arrow
        render_pass.set_vertex_buffer(0, self.arrow_z_mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.arrow_z_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.arrow_z_mesh.index_count, 0, 0..1);
        log::info!("Drew Z axis arrow with {} indices", self.arrow_z_mesh.index_count);
    }
    
    /// Render rotation gizmo (circles)
    fn render_rotation_gizmo<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        // Render circles for each axis
        // X rotation - Red
        self.render_circle(render_pass, Vec3::X, [1.0, 0.2, 0.2, 1.0], GizmoComponent::RotationX);
        
        // Y rotation - Green
        self.render_circle(render_pass, Vec3::Y, [0.2, 1.0, 0.2, 1.0], GizmoComponent::RotationY);
        
        // Z rotation - Blue
        self.render_circle(render_pass, Vec3::Z, [0.2, 0.2, 1.0, 1.0], GizmoComponent::RotationZ);
    }
    
    /// Render scale gizmo (lines with boxes)
    fn render_scale_gizmo<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        // Similar to translation but with boxes at ends
        // Implementation similar to arrows but with box mesh
    }
    
    
    /// Helper to render a plane
    fn render_plane<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        _normal: Vec3,
        _color: [f32; 4],
        _component: GizmoComponent,
    ) {
        // Set vertex and index buffers
        render_pass.set_vertex_buffer(0, self.plane_mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.plane_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        
        // Draw
        render_pass.draw_indexed(0..self.plane_mesh.index_count, 0, 0..1);
    }
    
    /// Helper to render a circle
    fn render_circle<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        _axis: Vec3,
        _color: [f32; 4],
        _component: GizmoComponent,
    ) {
        // Set vertex and index buffers
        render_pass.set_vertex_buffer(0, self.circle_mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.circle_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        
        // Draw
        render_pass.draw_indexed(0..self.circle_mesh.index_count, 0, 0..1);
    }
}

/// Create arrow mesh for translation gizmo - pointing up in Y  
fn create_arrow_mesh(device: &wgpu::Device) -> GizmoMesh {
    // Default white color for backward compatibility
    create_arrow_mesh_with_color(device, [1.0, 1.0, 1.0, 1.0])
}

/// Create arrow mesh for translation gizmo - pointing up in Y with specified color
fn create_arrow_mesh_with_color(device: &wgpu::Device, color: [f32; 4]) -> GizmoMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    // Create a simple arrow pointing up (Y axis)
    let shaft_length = 0.8;
    let head_length = 0.2;
    let head_width = 0.15;
    
    // Shaft (simple box)
    let shaft_width = 0.04;
    
    // Front face of shaft
    vertices.push(GizmoVertex { position: [-shaft_width, 0.0, -shaft_width], color });
    vertices.push(GizmoVertex { position: [shaft_width, 0.0, -shaft_width], color });
    vertices.push(GizmoVertex { position: [shaft_width, shaft_length, -shaft_width], color });
    vertices.push(GizmoVertex { position: [-shaft_width, shaft_length, -shaft_width], color });
    
    // Back face of shaft
    vertices.push(GizmoVertex { position: [-shaft_width, 0.0, shaft_width], color });
    vertices.push(GizmoVertex { position: [shaft_width, 0.0, shaft_width], color });
    vertices.push(GizmoVertex { position: [shaft_width, shaft_length, shaft_width], color });
    vertices.push(GizmoVertex { position: [-shaft_width, shaft_length, shaft_width], color });
    
    // Arrow head (pyramid)
    let head_base_y = shaft_length;
    let head_tip_y = shaft_length + head_length;
    
    // Base of pyramid
    vertices.push(GizmoVertex { position: [-head_width, head_base_y, -head_width], color });
    vertices.push(GizmoVertex { position: [head_width, head_base_y, -head_width], color });
    vertices.push(GizmoVertex { position: [head_width, head_base_y, head_width], color });
    vertices.push(GizmoVertex { position: [-head_width, head_base_y, head_width], color });
    
    // Tip of pyramid
    vertices.push(GizmoVertex { position: [0.0, head_tip_y, 0.0], color });
    
    // Shaft indices
    indices.extend_from_slice(&[
        // Front face
        0, 1, 2, 0, 2, 3,
        // Back face
        5, 4, 7, 5, 7, 6,
        // Left face
        4, 0, 3, 4, 3, 7,
        // Right face
        1, 5, 6, 1, 6, 2,
        // Bottom face
        4, 5, 1, 4, 1, 0,
        // Top face (connects to arrow head)
        3, 2, 6, 3, 6, 7,
    ]);
    
    // Arrow head indices (pyramid)
    let base = 8;
    indices.extend_from_slice(&[
        // Four triangular faces
        base, base + 1, 12, // Front
        base + 1, base + 2, 12, // Right
        base + 2, base + 3, 12, // Back
        base + 3, base, 12, // Left
        // Base
        base, base + 2, base + 1,
        base, base + 3, base + 2,
    ]);
    
    log::info!("Created arrow mesh with {} vertices, {} indices, color: {:?}", vertices.len(), indices.len(), color);
    
    // Create buffers
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Arrow Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Arrow Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    
    GizmoMesh {
        vertices,
        indices: indices.clone(),
        vertex_buffer,
        index_buffer,
        index_count: indices.len() as u32,
    }
}

/// Create plane mesh for constrained movement
fn create_plane_mesh(device: &wgpu::Device) -> GizmoMesh {
    let size = 0.3;
    let vertices = vec![
        GizmoVertex { position: [0.0, 0.0, 0.0], color: [1.0, 1.0, 1.0, 0.3] },
        GizmoVertex { position: [size, 0.0, 0.0], color: [1.0, 1.0, 1.0, 0.3] },
        GizmoVertex { position: [size, size, 0.0], color: [1.0, 1.0, 1.0, 0.3] },
        GizmoVertex { position: [0.0, size, 0.0], color: [1.0, 1.0, 1.0, 0.3] },
    ];
    
    let indices = vec![0, 1, 2, 0, 2, 3];
    
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Plane Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Plane Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    
    GizmoMesh {
        vertices,
        indices: indices.clone(),
        vertex_buffer,
        index_buffer,
        index_count: indices.len() as u32,
    }
}

/// Create circle mesh for rotation gizmo
fn create_circle_mesh(device: &wgpu::Device) -> GizmoMesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    let radius = 1.0;
    let segments = 64;
    
    // Create vertices
    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let x = angle.cos() * radius;
        let y = angle.sin() * radius;
        
        vertices.push(GizmoVertex {
            position: [x, y, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
        });
    }
    
    // Create line indices
    for i in 0..segments {
        indices.push(i as u16);
        indices.push((i + 1) as u16);
    }
    
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Circle Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Circle Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    
    GizmoMesh {
        vertices,
        indices: indices.clone(),
        vertex_buffer,
        index_buffer,
        index_count: indices.len() as u32,
    }
}

/// Create box mesh for scale gizmo
fn create_box_mesh(device: &wgpu::Device) -> GizmoMesh {
    let size = 0.1;
    let vertices = vec![
        // Front face
        GizmoVertex { position: [-size, -size, size], color: [1.0, 1.0, 1.0, 1.0] },
        GizmoVertex { position: [size, -size, size], color: [1.0, 1.0, 1.0, 1.0] },
        GizmoVertex { position: [size, size, size], color: [1.0, 1.0, 1.0, 1.0] },
        GizmoVertex { position: [-size, size, size], color: [1.0, 1.0, 1.0, 1.0] },
        // Back face
        GizmoVertex { position: [-size, -size, -size], color: [1.0, 1.0, 1.0, 1.0] },
        GizmoVertex { position: [size, -size, -size], color: [1.0, 1.0, 1.0, 1.0] },
        GizmoVertex { position: [size, size, -size], color: [1.0, 1.0, 1.0, 1.0] },
        GizmoVertex { position: [-size, size, -size], color: [1.0, 1.0, 1.0, 1.0] },
    ];
    
    let indices = vec![
        // Front face
        0, 1, 2, 0, 2, 3,
        // Back face
        5, 4, 7, 5, 7, 6,
        // Top face
        3, 2, 6, 3, 6, 7,
        // Bottom face
        4, 5, 1, 4, 1, 0,
        // Right face
        1, 5, 6, 1, 6, 2,
        // Left face
        4, 0, 3, 4, 3, 7,
    ];
    
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Box Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Box Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    
    GizmoMesh {
        vertices,
        indices: indices.clone(),
        vertex_buffer,
        index_buffer,
        index_count: indices.len() as u32,
    }
}