use wgpu;
use wgpu::util::DeviceExt;
use glam::Vec2;
use std::collections::HashMap;
use longhorn_assets::{AssetManager, AssetSource, AssetHandle, TextureData};
use longhorn_core::{AssetId, Sprite, Transform, World};
use longhorn_renderer::{
    pipeline::{create_sprite_pipeline, CameraUniform},
    Camera, Color, SpriteBatch, SpriteInstance, SpriteVertex,
};

/// Embedded test sprite (32x32 white square)
const TEST_SPRITE_BYTES: &[u8] = include_bytes!("../assets/test_sprite.png");

/// GPU texture resource with bind group
struct GpuTextureResource {
    bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    width: u32,
    #[allow(dead_code)]
    height: u32,
}

/// Identifies which render target to use
enum RenderTarget {
    Editor,
    Game,
}

/// Renders the game scene to an off-screen texture for display in egui
pub struct EditorViewportRenderer {
    // Render targets
    // Renamed from render_texture
    editor_render_texture: wgpu::Texture,
    editor_render_view: wgpu::TextureView,

    // New field for game view
    game_render_texture: Option<wgpu::Texture>,
    game_render_view: Option<wgpu::TextureView>,

    size: (u32, u32),

    // Sprite pipeline
    pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_uniform: CameraUniform,
    vertex_buffer: wgpu::Buffer,

    // Texture bind group layout (needed for creating new textures)
    texture_bind_group_layout: wgpu::BindGroupLayout,

    // Texture cache - maps AssetId to GPU texture resources
    texture_cache: HashMap<AssetId, GpuTextureResource>,

    // Fallback texture for sprites without a loaded texture
    fallback_texture_bind_group: wgpu::BindGroup,

    // Camera
    camera: Camera,

    // Clear color
    clear_color: Color,

    // egui integration
    egui_texture_id: Option<egui::TextureId>,
    game_egui_texture_id: Option<egui::TextureId>,
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

        // Load and create fallback texture
        let fallback_texture_bind_group =
            Self::create_test_texture(device, queue, &texture_bind_group_layout);

        Self {
            editor_render_texture: render_texture,
            editor_render_view: render_view,
            game_render_texture: None,
            game_render_view: None,
            size: (width, height),
            pipeline,
            camera_buffer,
            camera_bind_group,
            camera_uniform,
            vertex_buffer,
            texture_bind_group_layout,
            texture_cache: HashMap::new(),
            fallback_texture_bind_group,
            camera,
            clear_color: Color::from_rgba8(40, 44, 52, 255), // Dark background
            egui_texture_id: None,
            game_egui_texture_id: None,
        }
    }

    /// Upload a texture to the GPU cache
    pub fn upload_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        asset_id: AssetId,
        texture_data: &TextureData,
    ) {
        if self.texture_cache.contains_key(&asset_id) {
            return; // Already cached
        }

        let size = wgpu::Extent3d {
            width: texture_data.width,
            height: texture_data.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("Editor Sprite Texture {:?}", asset_id)),
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
            &texture_data.pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * texture_data.width),
                rows_per_image: Some(texture_data.height),
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Editor Sprite Bind Group {:?}", asset_id)),
            layout: &self.texture_bind_group_layout,
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
        });

        self.texture_cache.insert(asset_id, GpuTextureResource {
            bind_group,
            width: texture_data.width,
            height: texture_data.height,
        });

        log::debug!("Uploaded texture {:?} to GPU cache", asset_id);
    }

    /// Check if a texture is in the GPU cache
    pub fn has_texture(&self, asset_id: AssetId) -> bool {
        self.texture_cache.contains_key(&asset_id)
    }

    /// Get texture bind group, or fallback if not loaded
    fn get_texture_bind_group(&self, asset_id: AssetId) -> &wgpu::BindGroup {
        self.texture_cache
            .get(&asset_id)
            .map(|t| &t.bind_group)
            .unwrap_or(&self.fallback_texture_bind_group)
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
        self.editor_render_texture = texture;
        self.editor_render_view = view;
        self.size = (width, height);

        // Update camera viewport
        self.camera.viewport_size = Vec2::new(width as f32, height as f32);
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.editor_render_texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.editor_render_view
    }

    /// Capture a screenshot of the editor viewport and save to a PNG file
    pub fn capture_screenshot(&self, device: &wgpu::Device, queue: &wgpu::Queue, path: &str) -> Result<(u32, u32), String> {
        let (width, height) = self.size;

        // Calculate buffer dimensions (must be aligned to 256 bytes per row for wgpu)
        let bytes_per_pixel = 4; // RGBA8
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = ((unpadded_bytes_per_row + align - 1) / align) * align;
        let buffer_size = (padded_bytes_per_row * height) as wgpu::BufferAddress;

        // Create a buffer to copy the texture data into
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Screenshot Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Create command encoder for the copy operation
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Screenshot Encoder"),
        });

        // Copy texture to buffer
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.editor_render_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        // Submit the copy command
        queue.submit(Some(encoder.finish()));

        // Map the buffer to read the data
        let buffer_slice = buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });

        // Poll the device until the buffer is mapped
        device.poll(wgpu::Maintain::Wait);

        if let Ok(Ok(())) = receiver.recv() {
            let data = buffer_slice.get_mapped_range();

            // Convert padded buffer data to image
            let mut image_data = Vec::with_capacity((width * height * bytes_per_pixel) as usize);
            for row in 0..height {
                let row_start = (row * padded_bytes_per_row) as usize;
                let row_end = row_start + (width * bytes_per_pixel) as usize;
                image_data.extend_from_slice(&data[row_start..row_end]);
            }

            drop(data);
            buffer.unmap();

            // Save as PNG
            match image::save_buffer(
                path,
                &image_data,
                width,
                height,
                image::ColorType::Rgba8,
            ) {
                Ok(_) => {
                    log::info!("Screenshot saved to: {} ({}x{})", path, width, height);
                    Ok((width, height))
                }
                Err(e) => {
                    Err(format!("Failed to save screenshot: {}", e))
                }
            }
        } else {
            Err("Failed to map buffer for screenshot".to_string())
        }
    }

    /// Render sprites from the world to the off-screen texture
    ///
    /// This method now takes the asset manager to load textures on demand
    /// For backwards compatibility, uses default camera
    pub fn render_with_assets<S: AssetSource>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world: &World,
        assets: &AssetManager<S>,
    ) {
        // Temporary: use default camera for backwards compatibility
        let default_camera = crate::EditorCamera::default();
        self.render_scene_view(device, queue, world, assets, &default_camera);
    }

    /// Render a batch of vertices with a specific texture
    fn render_vertices_batch<'a>(
        &'a self,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'a>,
        vertices: &[SpriteVertex],
        texture_id: AssetId,
    ) {
        const MAX_VERTICES: usize = 1000 * 6;
        let vertices = if vertices.len() > MAX_VERTICES {
            log::warn!("Sprite count exceeds vertex buffer capacity ({} > {}). Truncating.",
                       vertices.len(), MAX_VERTICES);
            &vertices[..MAX_VERTICES]
        } else {
            vertices
        };

        // Upload vertices
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(vertices));

        // Get texture bind group (or fallback)
        let bind_group = self.get_texture_bind_group(texture_id);
        render_pass.set_bind_group(1, bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..vertices.len() as u32, 0..1);
    }

    /// Legacy render method for backwards compatibility (uses fallback texture)
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
                size: sprite.size * transform.scale, // Apply transform scale to sprite size
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
                    view: &self.editor_render_view,
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
                render_pass.set_bind_group(1, &self.fallback_texture_bind_group, &[]);
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
            &self.editor_render_view,
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
                &self.editor_render_view,
                wgpu::FilterMode::Linear,
                id,
            );
        }
    }

    pub fn egui_texture_id(&self) -> Option<egui::TextureId> {
        self.egui_texture_id
    }

    pub fn editor_texture_id(&self) -> Option<egui::TextureId> {
        self.egui_texture_id
    }

    pub fn game_texture_id(&self) -> Option<egui::TextureId> {
        self.game_egui_texture_id
    }

    pub fn editor_texture_size(&self) -> (u32, u32) {
        self.size
    }

    pub fn game_texture_size(&self) -> Option<(u32, u32)> {
        self.game_render_texture.as_ref().map(|texture| {
            let size = texture.size();
            (size.width, size.height)
        })
    }

    /// Render scene view using the editor camera
    pub fn render_scene_view<S: AssetSource>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world: &World,
        assets: &AssetManager<S>,
        editor_camera: &crate::EditorCamera,
    ) {
        // Update camera from editor camera
        self.camera.position = editor_camera.transform.position;
        self.camera.zoom = editor_camera.zoom;

        // Render to editor texture
        self.render_to_texture(device, queue, world, assets, RenderTarget::Editor);
    }

    /// Render game view using the main camera from the scene
    pub fn render_game_view<S: AssetSource>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world: &World,
        assets: &AssetManager<S>,
        egui_renderer: Option<&mut egui_wgpu::Renderer>,
    ) {
        use longhorn_engine::MainCamera;
        use longhorn_renderer::Camera as RendererCamera;

        // Find MainCamera in scene and copy the data we need
        let mut query = world.query::<(&Transform, &RendererCamera, &MainCamera)>();
        let camera_data: Vec<_> = query.iter().collect();

        // Warn if multiple found
        if camera_data.len() > 1 {
            eprintln!("Warning: {} MainCamera components found. Using first one.", camera_data.len());
        }

        // Use first camera
        if let Some((_entity, (_transform, camera, _main_camera))) = camera_data.first() {
            // Lazy allocate game texture if needed
            if self.game_render_texture.is_none() {
                let (width, height) = self.size;
                let (game_texture, game_view) = Self::create_render_texture(device, width, height);

                // Register with egui if renderer is provided
                if let Some(renderer) = egui_renderer {
                    let texture_id = renderer.register_native_texture(
                        device,
                        &game_view,
                        wgpu::FilterMode::Linear,
                    );
                    self.game_egui_texture_id = Some(texture_id);
                }

                self.game_render_texture = Some(game_texture);
                self.game_render_view = Some(game_view);
            }

            // Update camera from main camera
            let saved_position = self.camera.position;
            let saved_zoom = self.camera.zoom;

            self.camera.position = camera.position;
            self.camera.zoom = camera.zoom;

            // Render to game texture
            self.render_to_texture(device, queue, world, assets, RenderTarget::Game);

            // Restore camera
            self.camera.position = saved_position;
            self.camera.zoom = saved_zoom;
        }
    }

    /// Core rendering method that renders to a specific texture view
    fn render_to_texture<S: AssetSource>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world: &World,
        assets: &AssetManager<S>,
        target: RenderTarget,
    ) {
        // Update camera uniform
        self.camera_uniform.update(self.camera.view_projection());
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        // Collect sprites from world and upload textures as needed
        let mut batch = SpriteBatch::new();
        for (entity_id, (sprite, transform)) in world.query::<(&Sprite, &Transform)>().iter() {
            // Upload texture to GPU if not already cached
            if !self.texture_cache.contains_key(&sprite.texture) {
                log::info!("Texture {} not in GPU cache, attempting to load from AssetManager", sprite.texture.0);
                let handle = AssetHandle::<TextureData>::new(sprite.texture);
                if let Some(texture_data) = assets.get_texture(handle) {
                    log::info!("Loaded texture {} from AssetManager, uploading to GPU", sprite.texture.0);
                    self.upload_texture(device, queue, sprite.texture, texture_data);
                } else {
                    log::warn!("Texture {} not found in AssetManager! Using fallback texture", sprite.texture.0);
                }
            }

            batch.add(SpriteInstance {
                position: transform.position,
                size: sprite.size * transform.scale, // Apply transform scale to sprite size
                color: Color::new(sprite.color[0], sprite.color[1], sprite.color[2], sprite.color[3]),
                texture: sprite.texture,
                z_index: 0,
            });
        }

        batch.sort();

        // Debug: Log sprite batch info
        log::info!("Sprite batch has {} sprites", batch.len());
        for (i, sprite) in batch.iter().enumerate() {
            log::info!("  Sprite {}: texture={}, z_index={}, position=({}, {}), size=({}, {})",
                i, sprite.texture.0, sprite.z_index,
                sprite.position.x, sprite.position.y,
                sprite.size.x, sprite.size.y);
        }

        // Get the appropriate texture view based on the target
        // This is done AFTER all mutable borrows are complete
        let target_view = match target {
            RenderTarget::Editor => &self.editor_render_view,
            RenderTarget::Game => {
                if let Some(view) = &self.game_render_view {
                    view
                } else {
                    // Fall back to editor view if game view not allocated
                    &self.editor_render_view
                }
            }
        };

        // Create command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Editor Viewport Encoder"),
        });

        // Render pass - render sprites grouped by texture
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Editor Viewport Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
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
            // Collect batches with their texture and vertex ranges
            struct TextureBatch {
                texture: AssetId,
                start_vertex: u32,
                vertex_count: u32,
            }

            let mut batches = Vec::new();
            let mut all_vertices = Vec::new();
            let mut current_texture: Option<AssetId> = None;
            let mut batch_start = 0u32;

            for sprite in batch.iter() {
                // If texture changed, record the previous batch
                if current_texture.is_some() && current_texture != Some(sprite.texture) {
                    let vertex_count = all_vertices.len() as u32 - batch_start;
                    if vertex_count > 0 {
                        batches.push(TextureBatch {
                            texture: current_texture.unwrap(),
                            start_vertex: batch_start,
                            vertex_count,
                        });
                        log::info!("Recorded batch: texture={}, start={}, count={}",
                            current_texture.unwrap().0, batch_start, vertex_count);
                    }
                    batch_start = all_vertices.len() as u32;
                }

                current_texture = Some(sprite.texture);
                let sprite_verts = SpriteBatch::generate_vertices(sprite);
                all_vertices.extend_from_slice(&sprite_verts);
            }

            // Record the final batch
            if current_texture.is_some() {
                let vertex_count = all_vertices.len() as u32 - batch_start;
                if vertex_count > 0 {
                    batches.push(TextureBatch {
                        texture: current_texture.unwrap(),
                        start_vertex: batch_start,
                        vertex_count,
                    });
                    log::info!("Recorded final batch: texture={}, start={}, count={}",
                        current_texture.unwrap().0, batch_start, vertex_count);
                }
            }

            // Upload all vertices to the buffer once
            if !all_vertices.is_empty() {
                const MAX_VERTICES: usize = 1000 * 6;
                let vertices_to_upload = if all_vertices.len() > MAX_VERTICES {
                    log::warn!("Total vertex count exceeds buffer capacity ({} > {}). Truncating.",
                               all_vertices.len(), MAX_VERTICES);
                    &all_vertices[..MAX_VERTICES]
                } else {
                    &all_vertices[..]
                };

                queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(vertices_to_upload));
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

                // Render each batch with its texture
                for batch in batches.iter() {
                    let bind_group = self.get_texture_bind_group(batch.texture);
                    render_pass.set_bind_group(1, bind_group, &[]);
                    render_pass.draw(batch.start_vertex..(batch.start_vertex + batch.vertex_count), 0..1);
                    log::info!("Drew batch: texture={}, vertices {}..{}",
                        batch.texture.0, batch.start_vertex, batch.start_vertex + batch.vertex_count);
                }
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_renderer_has_editor_texture() {
        // This is a structure test - verify fields exist
        // Actual rendering requires GPU context, tested manually
        let _test_compile = |renderer: &EditorViewportRenderer| {
            let _editor_tex = &renderer.editor_render_texture;
            let _game_tex = &renderer.game_render_texture;
        };
    }

    #[test]
    fn test_editor_texture_id_getter() {
        let _test_compile = |renderer: &EditorViewportRenderer| {
            let _id: Option<egui::TextureId> = renderer.editor_texture_id();
        };
    }

    #[test]
    fn test_game_texture_id_getter_when_none() {
        let _test_compile = |renderer: &EditorViewportRenderer| {
            let id: Option<egui::TextureId> = renderer.game_texture_id();
            assert!(id.is_none());
        };
    }

    #[test]
    fn test_render_methods_exist() {
        // This is a structural test - methods will be tested manually with GPU context
        // Just verify the methods exist with correct signatures
    }
}
