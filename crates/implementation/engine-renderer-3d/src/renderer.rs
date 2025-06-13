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
    material_bind_group_layout: wgpu::BindGroupLayout,
    
    // Test geometry (triangle for now)
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    
    // Render dimensions
    width: u32,
    height: u32,
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
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
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
        
        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &material_bind_group_layout],
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
        
        // Create test triangle geometry
        let vertices = vec![
            Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },   // Top - Red
            Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] }, // Bottom left - Green
            Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },  // Bottom right - Blue
        ];
        
        let indices = vec![0u16, 1, 2];
        
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
            material_bind_group_layout,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            width,
            height,
        })
    }
    
    /// Render a scene to the internal texture
    pub fn render(&mut self, scene: &RenderScene) -> Result<(), anyhow::Error> {
        // Update camera uniform
        let camera_uniform = CameraUniform {
            view_proj: scene.camera.view_proj_matrix().to_cols_array_2d(),
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
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
        
        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));
        
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
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
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
}