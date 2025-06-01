// Basic wgpu renderer for the game engine

use wgpu::{
    Device, Queue, Surface, SurfaceConfiguration, RenderPipeline, Buffer, 
    BindGroup,
};
use winit::window::Window;
use engine_core::{World, Transform, Mesh, Camera};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x3, // position
        1 => Float32x3, // normal
        2 => Float32x3, // color
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Uniform {
    pub view_proj: [[f32; 4]; 4],
    pub model: [[f32; 4]; 4],
}

pub struct Renderer {
    pub surface: wgpu::Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pub render_pipeline: RenderPipeline,
    pub uniform_buffer: Buffer,
    pub uniform_bind_group: BindGroup,
    pub cube_vertex_buffer: Buffer,
    pub cube_index_buffer: Buffer,
    pub sphere_vertex_buffer: Buffer,
    pub sphere_index_buffer: Buffer,
    pub cube_index_count: u32,
    pub sphere_index_count: u32,
}

impl Renderer {
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
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("basic.wgsl").into()),
        });

        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
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

        // Create bind group
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        // Create render pipeline
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
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

        // Create cube mesh
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

        // Create sphere mesh (simplified - just use cube for now)
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
            uniform_buffer,
            uniform_bind_group,
            cube_vertex_buffer,
            cube_index_buffer,
            sphere_vertex_buffer,
            sphere_index_buffer,
            cube_index_count: cube_indices.len() as u32,
            sphere_index_count: sphere_indices.len() as u32,
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self, world: &World) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Find the main camera
        let camera_entity = world.query::<Camera>()
            .find(|(_, camera)| camera.is_main)
            .map(|(entity, _)| entity);

        let view_proj = if let Some(camera_entity) = camera_entity {
            let camera = world.get_component::<Camera>(camera_entity).unwrap();
            let default_transform = Transform::default();
            let transform = world.get_component::<Transform>(camera_entity)
                .unwrap_or(&default_transform);
            
            let eye = Vec3::from(transform.position);
            let target = eye + Vec3::new(0.0, 0.0, -1.0); // Look forward
            let up = Vec3::Y;
            
            let view = Mat4::look_at_rh(eye, target, up);
            let proj = Mat4::perspective_rh(
                camera.fov.to_radians(),
                self.config.width as f32 / self.config.height as f32,
                camera.near,
                camera.far,
            );
            
            proj * view
        } else {
            // Default camera
            let view = Mat4::look_at_rh(
                Vec3::new(0.0, 0.0, 5.0),
                Vec3::ZERO,
                Vec3::Y,
            );
            let proj = Mat4::perspective_rh(
                60.0_f32.to_radians(),
                self.config.width as f32 / self.config.height as f32,
                0.1,
                100.0,
            );
            proj * view
        };

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

            // Render all entities with Mesh and Transform components
            for (entity, mesh) in world.query::<Mesh>() {
                if let Some(transform) = world.get_component::<Transform>(entity) {
                    // Calculate model matrix
                    let translation = Mat4::from_translation(Vec3::from(transform.position));
                    let rotation = Mat4::from_euler(
                        glam::EulerRot::XYZ,
                        transform.rotation[0].to_radians(),
                        transform.rotation[1].to_radians(),
                        transform.rotation[2].to_radians(),
                    );
                    let scale = Mat4::from_scale(Vec3::from(transform.scale));
                    let model = translation * rotation * scale;

                    // Update uniforms
                    let uniform = Uniform {
                        view_proj: view_proj.to_cols_array_2d(),
                        model: model.to_cols_array_2d(),
                    };

                    self.queue.write_buffer(
                        &self.uniform_buffer,
                        0,
                        bytemuck::cast_slice(&[uniform]),
                    );

                    // Choose mesh type and render
                    match mesh.mesh_type {
                        engine_core::MeshType::Cube => {
                            render_pass.set_vertex_buffer(0, self.cube_vertex_buffer.slice(..));
                            render_pass.set_index_buffer(self.cube_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                            render_pass.draw_indexed(0..self.cube_index_count, 0, 0..1);
                        }
                        engine_core::MeshType::Sphere => {
                            render_pass.set_vertex_buffer(0, self.sphere_vertex_buffer.slice(..));
                            render_pass.set_index_buffer(self.sphere_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                            render_pass.draw_indexed(0..self.sphere_index_count, 0, 0..1);
                        }
                        _ => {
                            // For now, default to cube for other mesh types
                            render_pass.set_vertex_buffer(0, self.cube_vertex_buffer.slice(..));
                            render_pass.set_index_buffer(self.cube_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                            render_pass.draw_indexed(0..self.cube_index_count, 0, 0..1);
                        }
                    }
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

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
        // For now, create a simple octahedron as a sphere approximation
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