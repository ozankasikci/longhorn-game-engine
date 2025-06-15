//! Core 3D renderer implementation using wgpu
//! 
//! This is the main renderer struct that manages GPU resources and rendering.

use std::sync::Arc;
use std::collections::HashMap;
use wgpu::util::DeviceExt;
use glam::Mat4;
use bytemuck::{Pod, Zeroable};

use crate::scene::RenderScene;
use crate::resources::{ResourceManager, ResourceStats};
use crate::{Mesh, Material};
use crate::grid::{GridRenderer, GridConfig};
use crate::gizmo_3d::{GizmoRenderer3D, GizmoMode};

/// Vertex data structure for the renderer
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,  // position
        1 => Float32x3,  // color
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

/// Camera uniform data for shaders
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

/// Model uniform data for shaders (per-object transform)
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ModelUniform {
    pub model: [[f32; 4]; 4],
}

/// Main 3D renderer struct
pub struct Renderer3D {
    // Core WGPU resources
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    
    // Resource management
    resource_manager: ResourceManager,
    default_meshes: HashMap<String, u32>,
    default_materials: HashMap<String, u32>,
    default_textures: HashMap<String, u32>,
    
    // Render targets
    render_texture: wgpu::Texture,
    render_view: wgpu::TextureView,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    
    // Rendering pipeline
    render_pipeline: wgpu::RenderPipeline,
    
    // Uniforms and bind groups
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    model_bind_group_layout: wgpu::BindGroupLayout,
    material_bind_group_layout: wgpu::BindGroupLayout,
    
    // Test geometry (triangle for now)
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    
    // Render dimensions
    width: u32,
    height: u32,
    
    // Grid renderer
    grid_renderer: Option<GridRenderer>,
    grid_enabled: bool,
    
    // Gizmo renderer
    gizmo_renderer: Option<GizmoRenderer3D>,
    gizmo_enabled: bool,
    gizmo_transform: Option<glam::Mat4>,
}

impl Renderer3D {
    /// Create a new 3D renderer
    pub async fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        width: u32,
        height: u32,
    ) -> Result<Self, anyhow::Error> {
        log::info!("Creating Renderer3D with dimensions {}x{}", width, height);
        
        // Create render texture
        let render_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Render Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        
        let render_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Create depth texture
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Basic 3D Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/basic.wgsl").into()),
        });
        
        // Create camera uniform buffer
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create camera bind group layout
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
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
            ],
        });
        
        // Create camera bind group
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Create model bind group layout for per-object transforms
        let model_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Model Bind Group Layout"),
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
            ],
        });
        
        // Create material bind group layout
        let material_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Material Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        // Create pipeline layout - include both camera and model bind groups
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &model_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back), // Enable backface culling
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
        
        // Create a cube with proper vertices (24 vertices for 6 faces with unique normals/colors)
        let vertices = vec![
            // Front face (z = 0.5) - Red
            Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 0.0, 0.0] }, // 0
            Vertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 0.0, 0.0] }, // 1
            Vertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 0.0, 0.0] }, // 2
            Vertex { position: [-0.5,  0.5,  0.5], color: [1.0, 0.0, 0.0] }, // 3
            
            // Back face (z = -0.5) - Green
            Vertex { position: [ 0.5, -0.5, -0.5], color: [0.0, 1.0, 0.0] }, // 4
            Vertex { position: [-0.5, -0.5, -0.5], color: [0.0, 1.0, 0.0] }, // 5
            Vertex { position: [-0.5,  0.5, -0.5], color: [0.0, 1.0, 0.0] }, // 6
            Vertex { position: [ 0.5,  0.5, -0.5], color: [0.0, 1.0, 0.0] }, // 7
            
            // Top face (y = 0.5) - Blue
            Vertex { position: [-0.5,  0.5,  0.5], color: [0.0, 0.0, 1.0] }, // 8
            Vertex { position: [ 0.5,  0.5,  0.5], color: [0.0, 0.0, 1.0] }, // 9
            Vertex { position: [ 0.5,  0.5, -0.5], color: [0.0, 0.0, 1.0] }, // 10
            Vertex { position: [-0.5,  0.5, -0.5], color: [0.0, 0.0, 1.0] }, // 11
            
            // Bottom face (y = -0.5) - Yellow
            Vertex { position: [-0.5, -0.5, -0.5], color: [1.0, 1.0, 0.0] }, // 12
            Vertex { position: [ 0.5, -0.5, -0.5], color: [1.0, 1.0, 0.0] }, // 13
            Vertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 1.0, 0.0] }, // 14
            Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 1.0, 0.0] }, // 15
            
            // Right face (x = 0.5) - Magenta
            Vertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 0.0, 1.0] }, // 16
            Vertex { position: [ 0.5, -0.5, -0.5], color: [1.0, 0.0, 1.0] }, // 17
            Vertex { position: [ 0.5,  0.5, -0.5], color: [1.0, 0.0, 1.0] }, // 18
            Vertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 0.0, 1.0] }, // 19
            
            // Left face (x = -0.5) - Cyan
            Vertex { position: [-0.5, -0.5, -0.5], color: [0.0, 1.0, 1.0] }, // 20
            Vertex { position: [-0.5, -0.5,  0.5], color: [0.0, 1.0, 1.0] }, // 21
            Vertex { position: [-0.5,  0.5,  0.5], color: [0.0, 1.0, 1.0] }, // 22
            Vertex { position: [-0.5,  0.5, -0.5], color: [0.0, 1.0, 1.0] }, // 23
        ];
        
        let indices: Vec<u16> = vec![
            // Front face (CCW when viewed from front)
            0, 1, 2,    0, 2, 3,
            // Back face (CCW when viewed from back)
            4, 5, 6,    4, 6, 7,
            // Top face (CCW when viewed from top)
            8, 9, 10,   8, 10, 11,
            // Bottom face (CCW when viewed from bottom)
            12, 13, 14, 12, 14, 15,
            // Right face (CCW when viewed from right)
            16, 17, 18, 16, 18, 19,
            // Left face (CCW when viewed from left)
            20, 21, 22, 20, 22, 23,
        ];
        
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        // Create resource manager and load default resources
        let resource_manager = ResourceManager::new(device.clone(), queue.clone());
        let default_meshes = resource_manager.load_default_meshes()?;
        let default_materials = resource_manager.load_default_materials(&material_bind_group_layout)?;
        let default_textures = resource_manager.load_default_textures()?;
        
        // Create grid renderer
        let grid_renderer = match GridRenderer::new(
            device.clone(),
            wgpu::TextureFormat::Rgba8UnormSrgb,
            GridConfig::default(),
        ) {
            Ok(renderer) => Some(renderer),
            Err(e) => {
                log::warn!("Failed to create grid renderer: {}", e);
                None
            }
        };
        
        // Create gizmo renderer
        let gizmo_renderer = match GizmoRenderer3D::new(
            device.clone(),
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Depth32Float,
        ) {
            Ok(renderer) => Some(renderer),
            Err(e) => {
                log::warn!("Failed to create gizmo renderer: {}", e);
                None
            }
        };
        
        log::info!("Renderer3D created successfully");
        log::info!("Loaded {} default meshes, {} default materials, {} default textures", 
                   default_meshes.len(), default_materials.len(), default_textures.len());
        
        Ok(Self {
            device,
            queue,
            resource_manager,
            default_meshes,
            default_materials,
            default_textures,
            render_texture,
            render_view,
            depth_texture,
            depth_view,
            render_pipeline,
            camera_buffer,
            camera_bind_group,
            model_bind_group_layout,
            material_bind_group_layout,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            width,
            height,
            grid_renderer,
            grid_enabled: true,
            gizmo_renderer,
            gizmo_enabled: true,
            gizmo_transform: None,
        })
    }
    
    /// Render a scene to the internal texture
    pub fn render(&mut self, scene: &RenderScene) -> Result<(), anyhow::Error> {
        log::info!("=== Renderer3D::render called ===");
        log::info!("Scene has {} objects", scene.objects.len());
        log::info!("Render texture size: {}x{}", self.width, self.height);
        log::info!("Camera position: {:?}", scene.camera.position);
        log::info!("Camera target: {:?}", scene.camera.target);
        // Update camera uniform
        let view_matrix = scene.camera.view_matrix();
        let proj_matrix = scene.camera.projection_matrix();
        let view_proj_matrix = scene.camera.view_proj_matrix();
        log::info!("View matrix: {:?}", view_matrix);
        log::info!("Projection matrix: {:?}", proj_matrix);
        log::info!("Combined view_proj matrix: {:?}", view_proj_matrix);
        
        let camera_uniform = CameraUniform {
            view_proj: view_proj_matrix.to_cols_array_2d(),
        };
        
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
        
        // Create command encoder
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        // Create model buffers and bind groups for all objects first
        let mut model_bind_groups = Vec::new();
        
        for (idx, render_object) in scene.objects.iter().enumerate() {
            // Create model uniform from object transform
            let model_uniform = ModelUniform {
                model: render_object.transform.to_cols_array_2d(),
            };
            
            // Create temporary buffer for this object's model transform
            let model_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Model Buffer {}", idx)),
                contents: bytemuck::cast_slice(&[model_uniform]),
                usage: wgpu::BufferUsages::UNIFORM,
            });
            
            // Create bind group for this object
            let model_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Model Bind Group {}", idx)),
                layout: &self.model_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: model_buffer.as_entire_binding(),
                    },
                ],
            });
            
            model_bind_groups.push(model_bind_group);
        }
        
        // If no objects, create a default bind group
        if scene.objects.is_empty() {
            let model_uniform = ModelUniform {
                model: Mat4::IDENTITY.to_cols_array_2d(),
            };
            
            let model_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Default Model Buffer"),
                contents: bytemuck::cast_slice(&[model_uniform]),
                usage: wgpu::BufferUsages::UNIFORM,
            });
            
            let model_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Default Model Bind Group"),
                layout: &self.model_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: model_buffer.as_entire_binding(),
                    },
                ],
            });
            
            model_bind_groups.push(model_bind_group);
        }
        
        // Begin render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.2,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            
            // Draw each object in the scene
            if !scene.objects.is_empty() {
                for (idx, render_object) in scene.objects.iter().enumerate() {
                    log::info!("Drawing object {} with mesh_id: {}, material_id: {}", idx, render_object.mesh_id, render_object.material_id);
                    log::info!("Object transform: {:?}", render_object.transform);
                    
                    // Set the model bind group
                    render_pass.set_bind_group(1, &model_bind_groups[idx], &[]);
                    
                    // For now, always use the default cube to verify rendering works
                    // TODO: Fix mesh resource lifetime issue
                    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    log::info!("Drawing {} indices (should be 36 for a cube)", self.num_indices);
                    render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
                }
            } else {
                // Draw default cube
                log::warn!("No objects in scene, drawing default cube");
                render_pass.set_bind_group(1, &model_bind_groups[0], &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            }
            
            // Render grid if enabled
            if self.grid_enabled {
                if let Some(grid) = &self.grid_renderer {
                    grid.render(
                        &mut render_pass,
                        &view_proj_matrix,
                        scene.camera.position,
                        &self.queue,
                    );
                }
            }
            
            // Render gizmo if enabled and transform is set
            if self.gizmo_enabled {
                if let (Some(gizmo), Some(transform)) = (&self.gizmo_renderer, &self.gizmo_transform) {
                    gizmo.render(
                        &mut render_pass,
                        transform,
                        &view_matrix,
                        &proj_matrix,
                        scene.camera.position,
                        (self.width, self.height),
                        &self.queue,
                    );
                }
            }
        }
        
        // Submit commands
        let command_buffer = encoder.finish();
        log::info!("Submitting render commands");
        self.queue.submit(std::iter::once(command_buffer));
        
        log::info!("=== Render complete ===");
        Ok(())
    }
    
    /// Get the render texture for display in egui
    pub fn get_render_texture(&self) -> &wgpu::Texture {
        &self.render_texture
    }
    
    /// Get the render texture view
    pub fn get_render_view(&self) -> &wgpu::TextureView {
        &self.render_view
    }
    
    /// Resize the render targets
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), anyhow::Error> {
        if self.width == width && self.height == height {
            return Ok(());
        }
        
        log::info!("Resizing renderer from {}x{} to {}x{}", self.width, self.height, width, height);
        
        self.width = width;
        self.height = height;
        
        // Recreate render texture
        self.render_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Render Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        
        self.render_view = self.render_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        // Recreate depth texture
        self.depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        
        self.depth_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        Ok(())
    }
    
    /// Upload a new mesh and return its ID
    pub fn upload_mesh(&self, mesh: Mesh) -> Result<u32, anyhow::Error> {
        self.resource_manager.upload_mesh(mesh)
    }
    
    /// Upload a new material and return its ID
    pub fn upload_material(&self, material: Material) -> Result<u32, anyhow::Error> {
        self.resource_manager.upload_material(material, &self.material_bind_group_layout)
    }
    
    /// Get resource statistics
    pub fn get_resource_stats(&self) -> ResourceStats {
        self.resource_manager.get_stats()
    }
    
    /// Get default mesh ID by name
    pub fn get_default_mesh_id(&self, name: &str) -> Option<u32> {
        self.default_meshes.get(name).copied()
    }
    
    /// Get default material ID by name
    pub fn get_default_material_id(&self, name: &str) -> Option<u32> {
        self.default_materials.get(name).copied()
    }
    
    /// Update material properties
    pub fn update_material(&self, material_id: u32, material: Material) -> Result<(), anyhow::Error> {
        self.resource_manager.update_material(material_id, material)
    }
    
    /// Create a texture from descriptor
    pub fn create_texture(&self, desc: crate::texture::TextureDescriptor) -> Result<u32, anyhow::Error> {
        self.resource_manager.create_texture(desc)
    }
    
    /// Create a solid color texture
    pub fn create_solid_color_texture(&self, color: [u8; 4], size: u32) -> Result<u32, anyhow::Error> {
        self.resource_manager.create_solid_color_texture(color, size)
    }
    
    /// Get default texture ID by name
    pub fn get_default_texture_id(&self, name: &str) -> Option<u32> {
        self.default_textures.get(name).copied()
    }
    
    /// Enable or disable grid rendering
    pub fn set_grid_enabled(&mut self, enabled: bool) {
        self.grid_enabled = enabled;
    }
    
    /// Update grid configuration
    pub fn set_grid_config(&mut self, config: GridConfig) {
        if let Some(grid) = &mut self.grid_renderer {
            grid.update_config(self.device.clone(), config);
        }
    }
    
    /// Get whether grid is enabled
    pub fn is_grid_enabled(&self) -> bool {
        self.grid_enabled
    }
    
    /// Enable or disable gizmo rendering
    pub fn set_gizmo_enabled(&mut self, enabled: bool) {
        self.gizmo_enabled = enabled;
    }
    
    /// Set gizmo transform (position/rotation/scale of the selected object)
    pub fn set_gizmo_transform(&mut self, transform: Option<glam::Mat4>) {
        self.gizmo_transform = transform;
    }
    
    /// Set gizmo mode (Translation, Rotation, Scale, None)
    pub fn set_gizmo_mode(&mut self, mode: GizmoMode) {
        if let Some(gizmo) = &mut self.gizmo_renderer {
            gizmo.set_mode(mode);
        }
    }
    
    /// Get the render texture data as RGBA bytes
    pub fn get_texture_data(&self) -> Result<Vec<u8>, anyhow::Error> {
        // Need to update render texture format to include COPY_SRC usage
        let bytes_per_row = self.width * 4; // RGBA8
        let padded_bytes_per_row = (bytes_per_row + 255) & !255; // Align to 256 bytes
        let buffer_size = padded_bytes_per_row * self.height;
        
        // Create a buffer to copy the texture data into
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Texture Copy Buffer"),
            size: buffer_size as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        
        // Create command encoder
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Texture Copy Encoder"),
        });
        
        // Copy texture to buffer
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.render_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        
        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));
        
        // Map the buffer and read the data
        let buffer_slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap()?;
        
        // Read the data
        let data = buffer_slice.get_mapped_range();
        let mut result = Vec::with_capacity((self.width * self.height * 4) as usize);
        
        // Copy data row by row to remove padding
        for y in 0..self.height {
            let start = (y * padded_bytes_per_row) as usize;
            let end = start + (self.width * 4) as usize;
            result.extend_from_slice(&data[start..end]);
        }
        
        drop(data);
        output_buffer.unmap();
        
        Ok(result)
    }
}