# Editor Viewport Rendering Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Render sprites in the editor viewport using real wgpu rendering, so users can see their game scene.

**Architecture:** Downgrade workspace to wgpu 22 for egui compatibility, create EditorViewportRenderer that renders sprites to an off-screen texture, display texture in egui via ViewportPanel.

**Tech Stack:** wgpu 22, egui 0.29, egui-wgpu 0.29, glam, bytemuck

---

## Task 1: Downgrade wgpu to version 22

**Files:**
- Modify: `Cargo.toml:29`

**Step 1: Update workspace wgpu version**

Change line 29 from:
```toml
wgpu = "23"
```
to:
```toml
wgpu = "22"
```

**Step 2: Verify build passes**

Run: `cargo build --workspace`
Expected: Build succeeds (wgpu 22 and 23 APIs are nearly identical)

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: downgrade wgpu from 23 to 22 for egui compatibility"
```

---

## Task 2: Export pipeline module from longhorn-renderer

**Files:**
- Modify: `crates/longhorn-renderer/src/lib.rs:5`

**Step 1: Make pipeline module public**

Change line 5 from:
```rust
mod pipeline;
```
to:
```rust
pub mod pipeline;
```

**Step 2: Verify build passes**

Run: `cargo build -p longhorn-renderer`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add crates/longhorn-renderer/src/lib.rs
git commit -m "feat(renderer): export pipeline module for editor reuse"
```

---

## Task 3: Create embedded test sprite texture

**Files:**
- Create: `crates/longhorn-editor/assets/test_sprite.png`

**Step 1: Create assets directory**

```bash
mkdir -p crates/longhorn-editor/assets
```

**Step 2: Create a simple 32x32 white square PNG**

Use ImageMagick or similar:
```bash
convert -size 32x32 xc:white crates/longhorn-editor/assets/test_sprite.png
```

Or create programmatically in Rust later. For now, create a placeholder.

**Step 3: Commit**

```bash
git add crates/longhorn-editor/assets/
git commit -m "feat(editor): add test sprite texture"
```

---

## Task 4: Create EditorViewportRenderer struct

**Files:**
- Rewrite: `crates/longhorn-editor/src/viewport_renderer.rs`
- Modify: `crates/longhorn-editor/Cargo.toml`

**Step 1: Add dependencies to Cargo.toml**

Add `bytemuck` and `image` to `crates/longhorn-editor/Cargo.toml`:
```toml
[dependencies]
longhorn-core = { workspace = true }
longhorn-engine = { workspace = true }
longhorn-renderer = { workspace = true }

egui = { workspace = true }
wgpu = { workspace = true }
glam = { workspace = true }
serde = { workspace = true }
log = { workspace = true }
hecs = { workspace = true }
bytemuck = { version = "1.14", features = ["derive"] }
image = { workspace = true }
```

**Step 2: Rewrite viewport_renderer.rs**

Replace entire contents of `crates/longhorn-editor/src/viewport_renderer.rs`:

```rust
use bytemuck::{Pod, Zeroable};
use egui_wgpu::wgpu;
use glam::{Mat4, Vec2};
use longhorn_core::{Sprite, Transform, World};
use longhorn_renderer::{
    pipeline::{create_sprite_pipeline, CameraUniform, SPRITE_SHADER},
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
    max_vertices: usize,

    // Texture resources
    texture_bind_group_layout: wgpu::BindGroupLayout,
    test_texture_bind_group: wgpu::BindGroup,

    // Camera
    camera: Camera,

    // Clear color
    clear_color: Color,
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
        let max_vertices = 1000 * 6; // 1000 sprites
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Editor Vertex Buffer"),
            size: (max_vertices * std::mem::size_of::<SpriteVertex>()) as u64,
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
            max_vertices,
            texture_bind_group_layout,
            test_texture_bind_group,
            camera,
            clear_color: Color::from_rgba8(40, 44, 52, 255), // Dark background
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
}

use wgpu::util::DeviceExt;
```

**Step 3: Verify build passes**

Run: `cargo build -p longhorn-editor`
Expected: Build may fail due to missing egui_wgpu import. We'll fix in next task.

**Step 4: Commit (partial)**

```bash
git add crates/longhorn-editor/
git commit -m "feat(editor): implement EditorViewportRenderer for sprite rendering"
```

---

## Task 5: Update editor binary to use EditorViewportRenderer

**Files:**
- Modify: `editor/Cargo.toml`
- Modify: `editor/src/main.rs`

**Step 1: Ensure editor/Cargo.toml has correct dependencies**

The file should already have wgpu and egui-wgpu. Verify it includes:
```toml
wgpu = { version = "22.1", default-features = true }
egui-wgpu = "0.29"
```

**Step 2: Update editor/src/main.rs**

Add `viewport_renderer` field to `EditorApp` struct (around line 16):

```rust
struct EditorApp {
    window: Option<Arc<Window>>,
    gpu_state: Option<GpuState>,
    egui_state: Option<EguiState>,
    engine: Engine,
    editor: Editor,
    viewport_renderer: Option<EditorViewportRenderer>,
}
```

Update `EditorApp::new()` (around line 38):
```rust
fn new() -> Self {
    // ... existing engine setup code ...

    Self {
        window: None,
        gpu_state: None,
        egui_state: None,
        engine,
        editor: Editor::new(),
        viewport_renderer: None,
    }
}
```

Add import at top of file:
```rust
use longhorn_editor::{Editor, EditorMode, EditorViewportRenderer};
```

Update `init_gpu()` to create viewport renderer (after line 149, before `self.window = Some(window.clone())`):

```rust
// Create viewport renderer
let viewport_renderer = EditorViewportRenderer::new(&device, &queue, size.width, size.height);
self.viewport_renderer = Some(viewport_renderer);
```

Update `render()` to render viewport before egui (after line 166, before `// Get surface texture`):

```rust
// Render game viewport
if let Some(viewport_renderer) = &mut self.viewport_renderer {
    viewport_renderer.render(&gpu.device, &gpu.queue, self.engine.world());
}
```

**Step 3: Verify build passes**

Run: `cargo build -p editor`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add editor/
git commit -m "feat(editor): integrate EditorViewportRenderer in editor binary"
```

---

## Task 6: Register viewport texture with egui

**Files:**
- Modify: `editor/src/main.rs`
- Modify: `crates/longhorn-editor/src/viewport_renderer.rs`

**Step 1: Add egui texture registration to EditorViewportRenderer**

Add field to EditorViewportRenderer struct:
```rust
egui_texture_id: Option<egui::TextureId>,
```

Add method to register texture with egui:
```rust
pub fn register_with_egui(
    &mut self,
    egui_renderer: &mut egui_wgpu::Renderer,
    device: &wgpu::Device,
) {
    // Create sampler for egui
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

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
```

**Step 2: Call registration in editor binary**

In `init_gpu()`, after creating viewport_renderer:
```rust
viewport_renderer.register_with_egui(&mut renderer, &device);
```

**Step 3: Update texture after resize**

In `resize()`:
```rust
fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
    if size.width > 0 && size.height > 0 {
        if let Some(gpu) = &mut self.gpu_state {
            gpu.surface_config.width = size.width;
            gpu.surface_config.height = size.height;
            gpu.surface.configure(&gpu.device, &gpu.surface_config);
        }
        // Update viewport renderer
        if let (Some(gpu), Some(vr), Some(egui_state)) =
            (&self.gpu_state, &mut self.viewport_renderer, &mut self.egui_state)
        {
            vr.resize(&gpu.device, size.width, size.height);
            vr.update_egui_texture(&mut egui_state.renderer, &gpu.device);
        }
    }
}
```

**Step 4: Verify build passes**

Run: `cargo build -p editor`
Expected: Build succeeds

**Step 5: Commit**

```bash
git add editor/ crates/longhorn-editor/
git commit -m "feat(editor): register viewport texture with egui"
```

---

## Task 7: Update ViewportPanel to display texture

**Files:**
- Modify: `crates/longhorn-editor/src/panels/viewport.rs`

**Step 1: Simplify ViewportPanel**

Replace contents of `crates/longhorn-editor/src/panels/viewport.rs`:

```rust
use egui::{Ui, TextureId, Sense, Vec2 as EguiVec2};

pub struct ViewportPanel {
    last_size: (f32, f32),
}

impl ViewportPanel {
    pub fn new() -> Self {
        Self {
            last_size: (0.0, 0.0),
        }
    }

    /// Show the viewport panel, returns (response, new_size_if_changed)
    pub fn show(&mut self, ui: &mut Ui, texture_id: Option<TextureId>) -> (egui::Response, Option<(u32, u32)>) {
        ui.heading("Viewport");
        ui.separator();

        let available = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available, Sense::click_and_drag());

        // Check if size changed
        let size_changed = if (available.x, available.y) != self.last_size {
            self.last_size = (available.x, available.y);
            Some((available.x as u32, available.y as u32))
        } else {
            None
        };

        if let Some(tex_id) = texture_id {
            // Draw the rendered game texture
            ui.painter().image(
                tex_id,
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        } else {
            // Placeholder when no texture is set
            ui.painter().rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgb(30, 30, 30),
            );

            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Game Viewport",
                egui::FontId::proportional(20.0),
                egui::Color32::from_rgb(150, 150, 150),
            );
        }

        (response, size_changed)
    }
}

impl Default for ViewportPanel {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: Verify build passes**

Run: `cargo build -p longhorn-editor`
Expected: Build succeeds

**Step 3: Commit**

```bash
git add crates/longhorn-editor/src/panels/viewport.rs
git commit -m "feat(editor): update ViewportPanel to display texture"
```

---

## Task 8: Wire up Editor.show() to pass texture ID

**Files:**
- Modify: `crates/longhorn-editor/src/editor.rs`

**Step 1: Update Editor.show() signature**

Change the `show` method to accept optional texture ID:

```rust
pub fn show(&mut self, ctx: &Context, engine: &mut Engine, viewport_texture: Option<egui::TextureId>) -> bool {
```

**Step 2: Pass texture to viewport panel**

In the CentralPanel section (around line 118):
```rust
// Center panel - Viewport
egui::CentralPanel::default().show(ctx, |ui| {
    let (_response, _size_changed) = self.viewport.show(ui, viewport_texture);
});
```

**Step 3: Verify build passes**

Run: `cargo build -p longhorn-editor`
Expected: Build succeeds (but editor binary will fail)

**Step 4: Commit**

```bash
git add crates/longhorn-editor/src/editor.rs
git commit -m "feat(editor): pass viewport texture ID through Editor.show()"
```

---

## Task 9: Update editor binary to pass texture ID

**Files:**
- Modify: `editor/src/main.rs`

**Step 1: Update render() to pass texture ID**

In the `render()` function, update the egui frame to pass texture ID:

```rust
let viewport_texture = self.viewport_renderer
    .as_ref()
    .and_then(|vr| vr.egui_texture_id());

let full_output = egui_state.ctx.run(raw_input, |ctx| {
    should_exit = self.editor.show(ctx, &mut self.engine, viewport_texture);
});
```

**Step 2: Verify build passes**

Run: `cargo build -p editor`
Expected: Build succeeds

**Step 3: Run the editor**

Run: `cargo run -p editor`
Expected: Editor window opens with sprites visible in viewport (colored rectangles at their positions)

**Step 4: Commit**

```bash
git add editor/src/main.rs
git commit -m "feat(editor): pass viewport texture to editor UI"
```

---

## Task 10: Final testing and cleanup

**Step 1: Run editor and verify sprites render**

Run: `cargo run -p editor`
Expected:
- Editor window opens
- Viewport shows two colored rectangles (Player at 100,200 and Enemy at 300,150)
- Play/Pause/Stop buttons work

**Step 2: Test resize**

Resize the window. Sprites should still render correctly.

**Step 3: Final commit**

```bash
git add .
git commit -m "feat(editor): complete viewport sprite rendering"
```

---

## Summary

After completing all tasks:
1. wgpu downgraded to 22 for egui compatibility
2. EditorViewportRenderer renders sprites to off-screen texture
3. Texture registered with egui and displayed in ViewportPanel
4. Sprites from World are visible in editor viewport
