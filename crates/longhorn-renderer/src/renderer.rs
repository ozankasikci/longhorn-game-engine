use crate::{
    camera::Camera,
    pipeline::{create_sprite_pipeline, CameraUniform},
    sprite_batch::{SpriteBatch, SpriteVertex},
    texture::{GpuTexture, TextureCache},
    Color,
};
use longhorn_assets::{AssetHandle, AssetManager, AssetSource, TextureData};
use longhorn_core::{AssetId, Sprite, Transform, World};
use wgpu::{self, util::DeviceExt};

/// Main renderer for 2D sprites
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_uniform: CameraUniform,
    vertex_buffer: wgpu::Buffer,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_cache: TextureCache,
    clear_color: Color,
    max_vertices: usize,
}

impl Renderer {
    /// Create a new renderer
    pub async fn new(
        window: impl Into<wgpu::SurfaceTarget<'static>>,
        width: u32,
        height: u32,
    ) -> Result<Self, RendererError> {
        // Create instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = instance.create_surface(window)?;

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(RendererError::AdapterNotFound)?;

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: Some("Device"),
                    memory_hints: Default::default(),
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
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create camera uniform buffer
        let camera_uniform = CameraUniform::new();
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

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
                label: Some("Camera Bind Group Layout"),
            });

        // Create camera bind group
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("Camera Bind Group"),
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
                label: Some("Texture Bind Group Layout"),
            });

        // Create pipeline
        let pipeline = create_sprite_pipeline(
            &device,
            surface_format,
            &camera_bind_group_layout,
            &texture_bind_group_layout,
        );

        // Create vertex buffer (large enough for many sprites)
        let max_vertices = 10000 * 6; // 10000 sprites * 6 vertices each
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: (max_vertices * std::mem::size_of::<SpriteVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            camera_buffer,
            camera_bind_group,
            camera_uniform,
            vertex_buffer,
            texture_bind_group_layout,
            texture_cache: TextureCache::new(),
            clear_color: Color::BLACK,
            max_vertices,
        })
    }

    /// Resize the renderer
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Set the clear color
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    /// Upload a texture to the GPU
    pub fn upload_texture(
        &mut self,
        asset_id: AssetId,
        texture_data: &TextureData,
    ) -> Result<(), RendererError> {
        if !self.texture_cache.contains(asset_id) {
            let label = format!("Texture {:?}", asset_id);
            let gpu_texture = GpuTexture::from_texture_data(
                &self.device,
                &self.queue,
                &self.texture_bind_group_layout,
                texture_data,
                Some(&label),
            );
            self.texture_cache.insert(asset_id, gpu_texture);
        }
        Ok(())
    }

    /// Render the world
    pub fn render<S: AssetSource>(
        &mut self,
        world: &World,
        asset_manager: &AssetManager<S>,
        camera: &Camera,
    ) -> Result<(), RendererError> {
        // Update camera uniform
        self.camera_uniform.update(camera.view_projection());
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        // Collect sprites from the world
        let mut batch = SpriteBatch::new();
        for (entity_id, (sprite, transform)) in world.query::<(&Sprite, &Transform)>().iter() {
            // Upload texture if not already cached
            if !self.texture_cache.contains(sprite.texture) {
                let handle = AssetHandle::<TextureData>::new(sprite.texture);
                if let Some(texture_data) = asset_manager.get_texture(handle) {
                    self.upload_texture(sprite.texture, texture_data)?;
                } else {
                    log::warn!(
                        "Texture not found for entity {:?}: {:?}",
                        entity_id,
                        sprite.texture
                    );
                    continue;
                }
            }

            batch.add(crate::sprite_batch::SpriteInstance {
                position: transform.position,
                size: sprite.size,
                color: Color::new(sprite.color[0], sprite.color[1], sprite.color[2], sprite.color[3]),
                texture: sprite.texture,
                z_index: 0, // No z_index in Sprite component, default to 0
            });
        }

        // Sort sprites by z-index
        batch.sort();

        // Get current surface texture
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Begin render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            // Render sprites grouped by texture
            let mut current_texture: Option<AssetId> = None;
            let mut vertices = Vec::new();

            for sprite in batch.iter() {
                // If texture changed, flush previous batch
                if current_texture.is_some() && current_texture != Some(sprite.texture) {
                    self.render_vertices(&mut render_pass, &vertices, current_texture.unwrap())?;
                    vertices.clear();
                }

                current_texture = Some(sprite.texture);
                let sprite_vertices = SpriteBatch::generate_vertices(sprite);
                vertices.extend_from_slice(&sprite_vertices);
            }

            // Render remaining vertices
            if !vertices.is_empty() && current_texture.is_some() {
                self.render_vertices(&mut render_pass, &vertices, current_texture.unwrap())?;
            }
        }

        // Submit command buffer
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Render a batch of vertices
    fn render_vertices<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        vertices: &[SpriteVertex],
        asset_id: AssetId,
    ) -> Result<(), RendererError> {
        if vertices.is_empty() {
            return Ok(());
        }

        if vertices.len() > self.max_vertices {
            return Err(RendererError::TooManyVertices);
        }

        // Upload vertices to GPU
        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(vertices));

        // Get texture bind group
        let texture = self
            .texture_cache
            .get(asset_id)
            .ok_or_else(|| RendererError::TextureNotFound(asset_id))?;

        // Set texture and draw
        render_pass.set_bind_group(1, &texture.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..vertices.len() as u32, 0..1);

        Ok(())
    }
}

/// Renderer errors
#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("Adapter not found")]
    AdapterNotFound,

    #[error("Failed to request device: {0}")]
    RequestDevice(#[from] wgpu::RequestDeviceError),

    #[error("Failed to create surface: {0}")]
    CreateSurface(#[from] wgpu::CreateSurfaceError),

    #[error("Failed to get surface texture: {0}")]
    SurfaceTexture(#[from] wgpu::SurfaceError),

    #[error("Texture not found: {0:?}")]
    TextureNotFound(AssetId),

    #[error("Too many vertices")]
    TooManyVertices,
}
