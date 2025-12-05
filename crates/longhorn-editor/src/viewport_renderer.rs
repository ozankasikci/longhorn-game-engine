use wgpu;
use wgpu::util::DeviceExt;
use glam::Vec2;
use longhorn_core::{Sprite, Transform, World};
use longhorn_renderer::{
    pipeline::{create_sprite_pipeline, CameraUniform},
    Camera, Color, SpriteBatch, SpriteInstance, SpriteVertex,
};

/// Embedded test sprite (32x32 white square)
const TEST_SPRITE_BYTES: &[u8] = include_bytes!("../assets/test_sprite.png");

/// Renders the game scene to an off-screen texture for display in egui
pub struct EditorViewportRenderer {
    // Render target
    render_texture: wgpu::Texture,
    render_view: wgpu::TextureView,
    size: (u32, u32),

    // Sprite pipeline
    pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_uniform: CameraUniform,
    vertex_buffer: wgpu::Buffer,

    // Texture resources
    test_texture_bind_group: wgpu::BindGroup,

    // Camera
    camera: Camera,

    // Clear color
    clear_color: Color,

    // egui integration
    egui_texture_id: Option<egui::TextureId>,
}

impl EditorViewportRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> Self {
        let width = width.max(1);
        let height = height.max(1);

        // Create render texture
        let (render_texture, render_view) = Self::create_render_texture(device, width, height);

        // Create camera bind group layout
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                label: Some("Editor Camera Bind Group Layout"),
            });

        // Create texture bind group layout
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("Editor Texture Bind Group Layout"),
            });

        // Create pipeline
        let pipeline = create_sprite_pipeline(
            device,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            &camera_bind_group_layout,
            &texture_bind_group_layout,
        );

        // Create camera uniform and buffer
        let camera = Camera::new(width as f32, height as f32);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update(camera.view_projection());

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Editor Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("Editor Camera Bind Group"),
        });

        // Create vertex buffer
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Editor Vertex Buffer"),
            size: (1000 * 6 * std::mem::size_of::<SpriteVertex>()) as u64, // 1000 sprites
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Load and create test texture
        let test_texture_bind_group =
            Self::create_test_texture(device, queue, &texture_bind_group_layout);

        Self {
            render_texture,
            render_view,
            size: (width, height),
            pipeline,
            camera_buffer,
            camera_bind_group,
            camera_uniform,
            vertex_buffer,
            test_texture_bind_group,
            camera,
            clear_color: Color::from_rgba8(40, 44, 52, 255), // Dark background
            egui_texture_id: None,
        }
    }

    fn create_render_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Editor Viewport Render Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    fn create_test_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        // Decode embedded PNG
        let img = image::load_from_memory(TEST_SPRITE_BYTES)
            .expect("Failed to decode test sprite")
            .to_rgba8();
        let (width, height) = img.dimensions();
        let pixels = img.into_raw();

        // Create texture
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Test Sprite Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Test Sprite Bind Group"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        })
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);

        if self.size == (width, height) {
            return;
        }

        let (texture, view) = Self::create_render_texture(device, width, height);
        self.render_texture = texture;
        self.render_view = view;
        self.size = (width, height);

        // Update camera viewport
        self.camera.viewport_size = Vec2::new(width as f32, height as f32);
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.render_texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.render_view
    }

    /// Render sprites from the world to the off-screen texture
    pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, world: &World) {
        // Update camera uniform
        self.camera_uniform.update(self.camera.view_projection());
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        // Collect sprites from world
        let mut batch = SpriteBatch::new();
        for (_entity_id, (sprite, transform)) in world.query::<(&Sprite, &Transform)>().iter() {
            batch.add(SpriteInstance {
                position: transform.position,
                size: sprite.size,
                color: Color::new(sprite.color[0], sprite.color[1], sprite.color[2], sprite.color[3]),
                texture: sprite.texture,
                z_index: 0,
            });
        }

        batch.sort();

        // Generate vertices
        let mut vertices = Vec::new();
        for sprite in batch.iter() {
            let sprite_verts = SpriteBatch::generate_vertices(sprite);
            vertices.extend_from_slice(&sprite_verts);
        }

        // Create command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Editor Viewport Encoder"),
        });

        // Render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Editor Viewport Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color.to_wgpu()),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            if !vertices.is_empty() {
                // Before writing to buffer, check capacity
                const MAX_VERTICES: usize = 1000 * 6;
                if vertices.len() > MAX_VERTICES {
                    log::warn!("Sprite count exceeds vertex buffer capacity ({} > {}). Truncating.",
                               vertices.len(), MAX_VERTICES);
                    vertices.truncate(MAX_VERTICES);
                }

                // Upload vertices
                queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_bind_group(1, &self.test_texture_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.draw(0..vertices.len() as u32, 0..1);
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn register_with_egui(
        &mut self,
        egui_renderer: &mut egui_wgpu::Renderer,
        device: &wgpu::Device,
    ) {
        let id = egui_renderer.register_native_texture(
            device,
            &self.render_view,
            wgpu::FilterMode::Linear,
        );
        self.egui_texture_id = Some(id);
    }

    pub fn update_egui_texture(
        &mut self,
        egui_renderer: &mut egui_wgpu::Renderer,
        device: &wgpu::Device,
    ) {
        if let Some(id) = self.egui_texture_id {
            egui_renderer.update_egui_texture_from_wgpu_texture(
                device,
                &self.render_view,
                wgpu::FilterMode::Linear,
                id,
            );
        }
    }

    pub fn egui_texture_id(&self) -> Option<egui::TextureId> {
        self.egui_texture_id
    }
}
