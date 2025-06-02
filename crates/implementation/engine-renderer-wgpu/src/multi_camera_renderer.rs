//! Multi-camera renderer implementation for Phase 4
//! Integrates engine-camera system with wgpu rendering pipeline

use wgpu::{
    Device, Queue, Surface, SurfaceConfiguration, RenderPipeline, Buffer, 
    BindGroup, BindGroupLayout, Texture, TextureView,
};
use winit::window::Window;
use engine_ecs_core::{Transform, Mesh, EntityV2, WorldV2, Read};
use engine_camera::CameraComponent;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::collections::HashMap;

use crate::{Vertex, RenderError};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct MultiCameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub model: [[f32; 4]; 4],
}

/// Render target types supported by the multi-camera system
#[derive(Debug, Clone)]
pub enum RenderTarget {
    /// Render directly to the main surface
    Surface,
    /// Render to a texture (for render-to-texture effects)
    Texture { handle: u64, size: (u32, u32) },
}

/// Texture render target resource
pub struct TextureRenderTarget {
    pub texture: Texture,
    pub view: TextureView,
    pub size: (u32, u32),
}

/// Multi-camera renderer with support for render targets and priority sorting
pub struct MultiCameraRenderer {
    // Core wgpu resources
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
    
    // Rendering pipeline
    pub render_pipeline: RenderPipeline,
    pub uniform_bind_group_layout: BindGroupLayout,
    
    // Per-camera resources
    camera_uniforms: HashMap<EntityV2, Buffer>,
    camera_bind_groups: HashMap<EntityV2, BindGroup>,
    
    // Render targets
    texture_targets: HashMap<u64, TextureRenderTarget>,
    depth_buffers: HashMap<u64, Texture>,
    
    // Mesh buffers (shared across cameras)
    pub cube_vertex_buffer: Buffer,
    pub cube_index_buffer: Buffer,
    pub sphere_vertex_buffer: Buffer,
    pub sphere_index_buffer: Buffer,
    pub cube_index_count: u32,
    pub sphere_index_count: u32,
    
    // Frame tracking
    current_frame: u64,
}

impl MultiCameraRenderer {
    pub async fn new(window: &Window) -> Result<Self, Box<dyn std::error::Error>> {
        let size = window.inner_size();
        
        // Create wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = unsafe { instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(window)?) }?;

        // Get adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or("Failed to find adapter")?;

        // Get device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Multi-Camera Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("basic.wgsl").into()),
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
            label: Some("uniform_bind_group_layout"),
        });

        // Create render pipeline
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Multi-Camera Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Multi-Camera Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create mesh buffers
        let (cube_vertices, cube_indices) = Self::create_cube_mesh();
        let cube_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(&cube_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let cube_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Index Buffer"),
            contents: bytemuck::cast_slice(&cube_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let (sphere_vertices, sphere_indices) = Self::create_sphere_mesh();
        let sphere_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Vertex Buffer"),
            contents: bytemuck::cast_slice(&sphere_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let sphere_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Index Buffer"),
            contents: bytemuck::cast_slice(&sphere_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            uniform_bind_group_layout,
            camera_uniforms: HashMap::new(),
            camera_bind_groups: HashMap::new(),
            texture_targets: HashMap::new(),
            depth_buffers: HashMap::new(),
            cube_vertex_buffer,
            cube_index_buffer,
            sphere_vertex_buffer,
            sphere_index_buffer,
            cube_index_count: cube_indices.len() as u32,
            sphere_index_count: sphere_indices.len() as u32,
            current_frame: 0,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Main render function - renders all active cameras in priority order
    pub fn render(&mut self, world: &WorldV2) -> Result<(), wgpu::SurfaceError> {
        self.current_frame += 1;
        
        // Get surface texture
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Collect and sort active cameras by render order
        let mut active_cameras = Vec::new();
        for (entity, camera_comp) in world.query::<Read<CameraComponent>>().iter() {
            if camera_comp.camera.enabled() {
                active_cameras.push((entity, camera_comp.camera.render_order()));
            }
        }
        
        // Sort by render order (lower numbers render first)
        active_cameras.sort_by_key(|(_, order)| *order);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Multi-Camera Render Encoder"),
        });

        // Render each camera
        for (camera_entity, _) in active_cameras {
            if let Err(e) = self.render_camera(camera_entity, world, &view, &mut encoder) {
                eprintln!("Error rendering camera {:?}: {:?}", camera_entity, e);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Render a single camera view
    fn render_camera(
        &mut self,
        camera_entity: EntityV2,
        world: &WorldV2,
        surface_view: &TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<(), RenderError> {
        let camera_comp = world.get_component::<CameraComponent>(camera_entity)
            .ok_or(RenderError::CameraNotFound)?;
        let transform = world.get_component::<Transform>(camera_entity)
            .ok_or(RenderError::TransformNotFound)?;

        // Update camera view matrix and derived data
        let mut camera_comp_mut = camera_comp.clone();
        camera_comp_mut.update(transform, self.current_frame).map_err(|e| RenderError::WgpuError(e.to_string()))?;

        // Ensure camera has up-to-date uniform buffer
        self.ensure_camera_uniform(camera_entity, &camera_comp_mut, transform)?;

        // Determine render target (for now, only surface rendering)
        let target_view = match camera_comp_mut.target_texture {
            Some(_texture_handle) => {
                // TODO: Implement render-to-texture support
                surface_view
            }
            None => surface_view,
        };

        // Create render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Camera Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: if camera_comp_mut.camera.render_order() == 0 {
                            // First camera clears the screen
                            let clear_color = camera_comp_mut.camera.clear_color();
                            wgpu::LoadOp::Clear(wgpu::Color {
                                r: clear_color[0] as f64,
                                g: clear_color[1] as f64,
                                b: clear_color[2] as f64,
                                a: clear_color[3] as f64,
                            })
                        } else {
                            // Subsequent cameras render on top
                            wgpu::LoadOp::Load
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            
            // Use camera-specific bind group
            if let Some(bind_group) = self.camera_bind_groups.get(&camera_entity) {
                render_pass.set_bind_group(0, bind_group, &[]);
            }

            // Render all entities with Mesh components
            // TODO: Add frustum culling here
            let view_proj = camera_comp_mut.camera.view_projection_matrix();
            for (entity, mesh) in world.query::<Read<Mesh>>().iter() {
                if let Some(mesh_transform) = world.get_component::<Transform>(entity) {
                    // Update model matrix for this entity
                    let uniform = self.update_entity_uniform(camera_entity, view_proj, mesh_transform);
                    
                    // Update buffer
                    if let Some(buffer) = self.camera_uniforms.get(&camera_entity) {
                        self.queue.write_buffer(
                            buffer,
                            0,
                            bytemuck::cast_slice(&[uniform]),
                        );
                    }
                    
                    // Render the mesh
                    Self::render_mesh_static(&mut render_pass, mesh, &self.cube_vertex_buffer, &self.cube_index_buffer, &self.sphere_vertex_buffer, &self.sphere_index_buffer, self.cube_index_count, self.sphere_index_count)?;
                }
            }
        }

        Ok(())
    }

    /// Ensure camera has an up-to-date uniform buffer
    fn ensure_camera_uniform(
        &mut self,
        camera_entity: EntityV2,
        camera_comp: &CameraComponent,
        transform: &Transform,
    ) -> Result<(), RenderError> {
        // Create uniform buffer if it doesn't exist
        if !self.camera_uniforms.contains_key(&camera_entity) {
            let uniform_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Camera Uniform Buffer"),
                size: std::mem::size_of::<MultiCameraUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
                label: Some("Camera Bind Group"),
            });

            self.camera_uniforms.insert(camera_entity, uniform_buffer);
            self.camera_bind_groups.insert(camera_entity, bind_group);
        }

        Ok(())
    }

    /// Update entity-specific uniform data (model matrix)
    fn update_entity_uniform(
        &self,
        camera_entity: EntityV2,
        view_proj: Mat4,
        entity_transform: &Transform,
    ) -> MultiCameraUniform {
        // Calculate model matrix from entity transform
        let translation = Mat4::from_translation(Vec3::from(entity_transform.position));
        let rotation = Mat4::from_euler(
            glam::EulerRot::XYZ,
            entity_transform.rotation[0].to_radians(),
            entity_transform.rotation[1].to_radians(),
            entity_transform.rotation[2].to_radians(),
        );
        let scale = Mat4::from_scale(Vec3::from(entity_transform.scale));
        let model = translation * rotation * scale;

        // Create uniform data
        MultiCameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            model: model.to_cols_array_2d(),
        }
    }

    /// Render a mesh using the current render pass (static to avoid borrowing issues)
    fn render_mesh_static<'a>(
        render_pass: &mut wgpu::RenderPass<'a>,
        mesh: &Mesh,
        cube_vertex_buffer: &'a Buffer,
        cube_index_buffer: &'a Buffer,
        sphere_vertex_buffer: &'a Buffer,
        sphere_index_buffer: &'a Buffer,
        cube_index_count: u32,
        sphere_index_count: u32,
    ) -> Result<(), RenderError> {
        match mesh.mesh_type {
            engine_ecs_core::MeshType::Cube => {
                render_pass.set_vertex_buffer(0, cube_vertex_buffer.slice(..));
                render_pass.set_index_buffer(cube_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..cube_index_count, 0, 0..1);
            }
            engine_ecs_core::MeshType::Sphere => {
                render_pass.set_vertex_buffer(0, sphere_vertex_buffer.slice(..));
                render_pass.set_index_buffer(sphere_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..sphere_index_count, 0, 0..1);
            }
            _ => {
                // Default to cube for other mesh types
                render_pass.set_vertex_buffer(0, cube_vertex_buffer.slice(..));
                render_pass.set_index_buffer(cube_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..cube_index_count, 0, 0..1);
            }
        }
        Ok(())
    }

    // Mesh generation functions (copied from original renderer)
    fn create_cube_mesh() -> (Vec<Vertex>, Vec<u16>) {
        let vertices = vec![
            // Front face
            Vertex { position: [-0.5, -0.5,  0.5], normal: [0.0, 0.0, 1.0], color: [1.0, 0.0, 0.0] },
            Vertex { position: [ 0.5, -0.5,  0.5], normal: [0.0, 0.0, 1.0], color: [0.0, 1.0, 0.0] },
            Vertex { position: [ 0.5,  0.5,  0.5], normal: [0.0, 0.0, 1.0], color: [0.0, 0.0, 1.0] },
            Vertex { position: [-0.5,  0.5,  0.5], normal: [0.0, 0.0, 1.0], color: [1.0, 1.0, 0.0] },
            
            // Back face
            Vertex { position: [-0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0], color: [1.0, 0.0, 1.0] },
            Vertex { position: [ 0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0], color: [0.0, 1.0, 1.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], normal: [0.0, 0.0, -1.0], color: [0.5, 0.5, 0.5] },
            Vertex { position: [-0.5,  0.5, -0.5], normal: [0.0, 0.0, -1.0], color: [1.0, 0.5, 0.0] },
        ];

        let indices = vec![
            // Front
            0, 1, 2, 2, 3, 0,
            // Back
            4, 6, 5, 6, 4, 7,
            // Left
            4, 0, 3, 3, 7, 4,
            // Right
            1, 5, 6, 6, 2, 1,
            // Top
            3, 2, 6, 6, 7, 3,
            // Bottom
            4, 5, 1, 1, 0, 4,
        ];

        (vertices, indices)
    }

    fn create_sphere_mesh() -> (Vec<Vertex>, Vec<u16>) {
        // Simple octahedron as sphere approximation
        let vertices = vec![
            Vertex { position: [ 0.0,  1.0,  0.0], normal: [0.0, 1.0, 0.0], color: [1.0, 0.0, 0.0] }, // Top
            Vertex { position: [ 0.0, -1.0,  0.0], normal: [0.0, -1.0, 0.0], color: [0.0, 1.0, 0.0] }, // Bottom
            Vertex { position: [ 1.0,  0.0,  0.0], normal: [1.0, 0.0, 0.0], color: [0.0, 0.0, 1.0] }, // Right
            Vertex { position: [-1.0,  0.0,  0.0], normal: [-1.0, 0.0, 0.0], color: [1.0, 1.0, 0.0] }, // Left
            Vertex { position: [ 0.0,  0.0,  1.0], normal: [0.0, 0.0, 1.0], color: [1.0, 0.0, 1.0] }, // Front
            Vertex { position: [ 0.0,  0.0, -1.0], normal: [0.0, 0.0, -1.0], color: [0.0, 1.0, 1.0] }, // Back
        ];

        let indices = vec![
            // Top triangles
            0, 4, 2,  0, 2, 5,  0, 5, 3,  0, 3, 4,
            // Bottom triangles
            1, 2, 4,  1, 5, 2,  1, 3, 5,  1, 4, 3,
        ];

        (vertices, indices)
    }
}

use wgpu::util::DeviceExt;