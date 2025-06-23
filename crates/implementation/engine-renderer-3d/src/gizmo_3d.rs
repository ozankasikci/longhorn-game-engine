//! 3D Gizmo rendering for transform manipulation
//!
//! Provides professional gizmos for position, rotation, and scale manipulation
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
    plane_xy_mesh: GizmoMesh, // Yellow plane for XY movement
    plane_xz_mesh: GizmoMesh, // Cyan plane for XZ movement
    plane_yz_mesh: GizmoMesh, // Magenta plane for YZ movement
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
                depth_write_enabled: false, // Don't write to depth buffer
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
        
        // Create plane meshes for constrained movement
        let plane_xy_mesh = create_plane_mesh_xy(&device); // Yellow plane
        let plane_xz_mesh = create_plane_mesh_xz(&device); // Cyan plane
        let plane_yz_mesh = create_plane_mesh_yz(&device); // Magenta plane
        
        let circle_mesh = create_circle_mesh(&device);
        let box_mesh = create_box_mesh(&device);
        
        Ok(Self {
            pipeline,
            arrow_x_mesh,
            arrow_y_mesh,
            arrow_z_mesh,
            plane_xy_mesh,
            plane_xz_mesh,
            plane_yz_mesh,
            circle_mesh,
            box_mesh,
            uniform_buffer,
            bind_group,
            mode: GizmoMode::None,
            highlighted_component: None,
            active_component: None,
            gizmo_size: 150.0, // Larger size for better visibility
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
        
        eprintln!("=== 3D Gizmo Rendering ===");
        eprintln!("Mode: {:?}", self.mode);
        eprintln!("Viewport size: {:?}", viewport_size);
        
        // Extract gizmo position from transform
        let gizmo_position = Vec3::new(
            gizmo_transform.col(3).x,
            gizmo_transform.col(3).y,
            gizmo_transform.col(3).z,
        );
        
        eprintln!("Gizmo position: {:?}", gizmo_position);
        eprintln!("Camera position: {:?}", camera_position);
        
        // Test if gizmo is in view
        let gizmo_pos_vec4 = gizmo_position.extend(1.0);
        let view_pos = *view_matrix * gizmo_pos_vec4;
        let gizmo_clip = *projection_matrix * view_pos;
        eprintln!("Gizmo clip space: {:?}", gizmo_clip);
        if gizmo_clip.w > 0.0 {
            let ndc = gizmo_clip.truncate() / gizmo_clip.w;
            eprintln!("Gizmo NDC: {:?}", ndc);
        }
        
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
        eprintln!("3D GIZMO: Rendering translation gizmo with 3 colored arrows and 3 plane handles");
        
        // Render plane handles first (so they appear behind arrows)
        
        // XY plane - Yellow
        render_pass.set_vertex_buffer(0, self.plane_xy_mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.plane_xy_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.plane_xy_mesh.index_count, 0, 0..1);
        
        // XZ plane - Cyan
        render_pass.set_vertex_buffer(0, self.plane_xz_mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.plane_xz_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.plane_xz_mesh.index_count, 0, 0..1);
        
        // YZ plane - Magenta
        render_pass.set_vertex_buffer(0, self.plane_yz_mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.plane_yz_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.plane_yz_mesh.index_count, 0, 0..1);
        
        // Render arrows on top
        
        // X axis - Red arrow
        render_pass.set_vertex_buffer(0, self.arrow_x_mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.arrow_x_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.arrow_x_mesh.index_count, 0, 0..1);
        eprintln!("3D GIZMO: Drew X axis arrow with {} indices", self.arrow_x_mesh.index_count);
        
        // Y axis - Green arrow  
        render_pass.set_vertex_buffer(0, self.arrow_y_mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.arrow_y_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.arrow_y_mesh.index_count, 0, 0..1);
        eprintln!("3D GIZMO: Drew Y axis arrow with {} indices", self.arrow_y_mesh.index_count);
        
        // Z axis - Blue arrow
        render_pass.set_vertex_buffer(0, self.arrow_z_mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.arrow_z_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.arrow_z_mesh.index_count, 0, 0..1);
        eprintln!("3D GIZMO: Drew Z axis arrow with {} indices", self.arrow_z_mesh.index_count);
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
    
    
    /// Helper to render a plane - not used anymore since we have specific plane meshes
    fn render_plane<'a>(
        &'a self,
        _render_pass: &mut wgpu::RenderPass<'a>,
        _normal: Vec3,
        _color: [f32; 4],
        _component: GizmoComponent,
    ) {
        // This method is deprecated - we now use specific plane meshes
        // (plane_xy_mesh, plane_xz_mesh, plane_yz_mesh) directly in render_translation_gizmo
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
    
    // Create a better-looking arrow pointing up (Y axis)
    let shaft_length = 0.7;
    let head_length = 0.3;
    let head_radius = 0.1;
    let shaft_radius = 0.03;
    let segments = 12; // More segments for smoother cylinder
    
    // Create cylindrical shaft
    let base_offset = 0;
    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let x = angle.cos() * shaft_radius;
        let z = angle.sin() * shaft_radius;
        
        // Bottom vertex
        vertices.push(GizmoVertex { position: [x, 0.0, z], color });
        // Top vertex
        vertices.push(GizmoVertex { position: [x, shaft_length, z], color });
    }
    
    // Add center vertices for end caps
    let bottom_center = vertices.len();
    vertices.push(GizmoVertex { position: [0.0, 0.0, 0.0], color });
    let top_center = vertices.len();
    vertices.push(GizmoVertex { position: [0.0, shaft_length, 0.0], color });
    
    // Create conical arrow head
    let head_base_offset = vertices.len();
    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let x = angle.cos() * head_radius;
        let z = angle.sin() * head_radius;
        vertices.push(GizmoVertex { position: [x, shaft_length, z], color });
    }
    
    // Arrow tip
    let tip_index = vertices.len();
    vertices.push(GizmoVertex { position: [0.0, shaft_length + head_length, 0.0], color });
    
    // Generate cylinder indices
    for i in 0..segments {
        let i1 = (i * 2) as u16;
        let i2 = (i * 2 + 1) as u16;
        let i3 = ((i + 1) % segments * 2) as u16;
        let i4 = ((i + 1) % segments * 2 + 1) as u16;
        
        // Side faces
        indices.extend_from_slice(&[i1, i3, i4, i1, i4, i2]);
        
        // Bottom cap
        indices.extend_from_slice(&[bottom_center as u16, i3, i1]);
        
        // Top cap (connecting to arrow base)
        indices.extend_from_slice(&[top_center as u16, i2, i4]);
    }
    
    // Generate cone indices for arrow head
    for i in 0..segments {
        let base = head_base_offset as u16 + i as u16;
        let next_base = head_base_offset as u16 + ((i + 1) % segments) as u16;
        
        // Cone sides
        indices.extend_from_slice(&[base, next_base, tip_index as u16]);
    }
    
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

/// Create XY plane mesh (yellow) - for moving in X and Y
fn create_plane_mesh_xy(device: &wgpu::Device) -> GizmoMesh {
    let size = 0.3;  // Larger size for better visibility
    let offset = 0.0; // Start at the pivot point
    let color = [1.0, 1.0, 0.0, 0.7]; // Yellow with good opacity
    
    let vertices = vec![
        GizmoVertex { position: [offset, offset, 0.0], color },
        GizmoVertex { position: [offset + size, offset, 0.0], color },
        GizmoVertex { position: [offset + size, offset + size, 0.0], color },
        GizmoVertex { position: [offset, offset + size, 0.0], color },
    ];
    
    let indices = vec![0, 1, 2, 0, 2, 3];
    
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("XY Plane Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("XY Plane Index Buffer"),
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

/// Create XZ plane mesh (cyan) - for moving in X and Z
fn create_plane_mesh_xz(device: &wgpu::Device) -> GizmoMesh {
    let size = 0.3;
    let offset = 0.0;
    let color = [0.0, 1.0, 1.0, 0.7]; // Cyan with good opacity
    
    let vertices = vec![
        GizmoVertex { position: [offset, 0.0, offset], color },
        GizmoVertex { position: [offset + size, 0.0, offset], color },
        GizmoVertex { position: [offset + size, 0.0, offset + size], color },
        GizmoVertex { position: [offset, 0.0, offset + size], color },
    ];
    
    let indices = vec![0, 1, 2, 0, 2, 3];
    
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("XZ Plane Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("XZ Plane Index Buffer"),
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

/// Create YZ plane mesh (magenta) - for moving in Y and Z
fn create_plane_mesh_yz(device: &wgpu::Device) -> GizmoMesh {
    let size = 0.3;
    let offset = 0.0;
    let color = [1.0, 0.0, 1.0, 0.7]; // Magenta with good opacity
    
    let vertices = vec![
        GizmoVertex { position: [0.0, offset, offset], color },
        GizmoVertex { position: [0.0, offset + size, offset], color },
        GizmoVertex { position: [0.0, offset + size, offset + size], color },
        GizmoVertex { position: [0.0, offset, offset + size], color },
    ];
    
    let indices = vec![0, 1, 2, 0, 2, 3];
    
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("YZ Plane Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("YZ Plane Index Buffer"),
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