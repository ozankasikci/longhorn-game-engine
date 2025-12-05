# Editor Scene & Play Modes Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Scene Mode and Play Mode to the Longhorn editor with render-to-texture viewport, play controls toolbar, and state snapshot/restore.

**Architecture:** Replace eframe with winit+wgpu+egui_wgpu for full render control. Game renders to off-screen texture displayed in egui. EditorMode state machine controls what runs each frame.

**Tech Stack:** winit 0.30, wgpu 23, egui 0.29, egui_wgpu 0.29

---

## Task 1: Add Editor Dependencies

**Files:**
- Modify: `editor/Cargo.toml`
- Modify: `crates/longhorn-editor/Cargo.toml`

**Step 1: Update editor/Cargo.toml**

Replace contents with:

```toml
[package]
name = "editor"
version.workspace = true
edition.workspace = true

[[bin]]
name = "longhorn-editor"
path = "src/main.rs"

[dependencies]
longhorn-core = { workspace = true }
longhorn-engine = { workspace = true }
longhorn-editor = { workspace = true }
longhorn-renderer = { workspace = true }
longhorn-assets = { workspace = true }

# Windowing and rendering
winit = { workspace = true }
wgpu = { workspace = true }

# UI
egui = { workspace = true }
egui-wgpu = "0.29"
egui-winit = "0.29"

# Utils
pollster = "0.4"
env_logger = { workspace = true }
log = { workspace = true }
glam = { workspace = true }
```

**Step 2: Update longhorn-editor/Cargo.toml**

Replace contents with:

```toml
[package]
name = "longhorn-editor"
version.workspace = true
edition.workspace = true

[dependencies]
longhorn-core = { workspace = true }
longhorn-engine = { workspace = true }
longhorn-renderer = { workspace = true }

egui = { workspace = true }
wgpu = { workspace = true }
glam = { workspace = true }
serde = { workspace = true }
log = { workspace = true }
```

**Step 3: Verify dependencies resolve**

Run: `cargo check -p editor -p longhorn-editor`
Expected: Dependencies download and resolve (may have warnings, no errors)

**Step 4: Commit**

```bash
git add editor/Cargo.toml crates/longhorn-editor/Cargo.toml
git commit -m "chore: add wgpu/egui-wgpu dependencies for editor viewport"
```

---

## Task 2: Add EditorMode and Update State

**Files:**
- Modify: `crates/longhorn-editor/src/state.rs`

**Step 1: Update state.rs with EditorMode**

Replace contents with:

```rust
use hecs::Entity;
use serde::{Deserialize, Serialize};

/// Editor operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorMode {
    #[default]
    Scene,  // Editing - no game logic runs
    Play,   // Running - game loop active
}

/// Editor state
#[derive(Debug, Default)]
pub struct EditorState {
    /// Current operating mode
    pub mode: EditorMode,
    /// Currently selected entity
    pub selected_entity: Option<Entity>,
    /// Whether game is paused (only relevant in Play mode)
    pub paused: bool,
    /// Path to loaded game
    pub game_path: Option<String>,
}

impl EditorState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn select(&mut self, entity: Option<Entity>) {
        self.selected_entity = entity;
    }

    pub fn is_selected(&self, entity: Entity) -> bool {
        self.selected_entity == Some(entity)
    }

    pub fn is_playing(&self) -> bool {
        self.mode == EditorMode::Play
    }

    pub fn is_scene_mode(&self) -> bool {
        self.mode == EditorMode::Scene
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mode_is_scene() {
        let state = EditorState::new();
        assert_eq!(state.mode, EditorMode::Scene);
        assert!(state.is_scene_mode());
        assert!(!state.is_playing());
    }

    #[test]
    fn test_mode_transitions() {
        let mut state = EditorState::new();

        state.mode = EditorMode::Play;
        assert!(state.is_playing());
        assert!(!state.is_scene_mode());

        state.mode = EditorMode::Scene;
        assert!(state.is_scene_mode());
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p longhorn-editor`
Expected: Tests pass

**Step 3: Commit**

```bash
git add crates/longhorn-editor/src/state.rs
git commit -m "feat(editor): add EditorMode (Scene/Play) to state"
```

---

## Task 3: Add Toolbar Component

**Files:**
- Create: `crates/longhorn-editor/src/toolbar.rs`
- Modify: `crates/longhorn-editor/src/lib.rs`

**Step 1: Create toolbar.rs**

```rust
use egui::Ui;
use crate::state::{EditorMode, EditorState};

/// Actions that can be triggered from the toolbar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    None,
    Play,
    Pause,
    Resume,
    Stop,
}

/// Toolbar with play controls
pub struct Toolbar;

impl Toolbar {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut Ui, state: &EditorState) -> ToolbarAction {
        let mut action = ToolbarAction::None;

        ui.horizontal(|ui| {
            // Center the buttons
            ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                ui.set_max_width(300.0);

                match state.mode {
                    EditorMode::Scene => {
                        if ui.button("▶ Play").clicked() {
                            action = ToolbarAction::Play;
                        }
                        ui.add_enabled(false, egui::Button::new("⏸ Pause"));
                        ui.add_enabled(false, egui::Button::new("⏹ Stop"));
                    }
                    EditorMode::Play => {
                        if state.paused {
                            if ui.button("▶ Resume").clicked() {
                                action = ToolbarAction::Resume;
                            }
                        } else {
                            ui.add_enabled(false, egui::Button::new("▶ Play"));
                        }

                        if !state.paused {
                            if ui.button("⏸ Pause").clicked() {
                                action = ToolbarAction::Pause;
                            }
                        } else {
                            ui.add_enabled(false, egui::Button::new("⏸ Pause"));
                        }

                        if ui.button("⏹ Stop").clicked() {
                            action = ToolbarAction::Stop;
                        }
                    }
                }

                ui.separator();

                // Mode indicator
                let mode_text = match (state.mode, state.paused) {
                    (EditorMode::Scene, _) => "Scene Mode",
                    (EditorMode::Play, false) => "▶ Playing",
                    (EditorMode::Play, true) => "⏸ Paused",
                };
                ui.label(mode_text);
            });
        });

        action
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolbar_action_default() {
        assert_eq!(ToolbarAction::None, ToolbarAction::None);
    }
}
```

**Step 2: Update lib.rs to export toolbar**

Add to `crates/longhorn-editor/src/lib.rs`:

```rust
mod state;
mod toolbar;
mod panels;
mod editor;

pub use state::*;
pub use toolbar::*;
pub use panels::*;
pub use editor::*;
```

**Step 3: Verify compiles**

Run: `cargo check -p longhorn-editor`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add crates/longhorn-editor/src/toolbar.rs crates/longhorn-editor/src/lib.rs
git commit -m "feat(editor): add Toolbar with Play/Pause/Stop buttons"
```

---

## Task 4: Add Scene Snapshot

**Files:**
- Create: `crates/longhorn-editor/src/snapshot.rs`
- Modify: `crates/longhorn-editor/src/lib.rs`

**Step 1: Create snapshot.rs**

```rust
use longhorn_core::{World, Name, Transform, Sprite, Enabled};
use serde::{Deserialize, Serialize};

/// Snapshot of an entity's components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub name: Option<Name>,
    pub transform: Option<Transform>,
    pub sprite: Option<Sprite>,
    pub enabled: Option<Enabled>,
}

/// Snapshot of the entire scene for restore
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SceneSnapshot {
    pub entities: Vec<EntitySnapshot>,
}

impl SceneSnapshot {
    /// Capture current world state
    pub fn capture(world: &World) -> Self {
        let mut entities = Vec::new();

        // Query all entities and capture their components
        for (entity, _) in world.query::<()>().iter() {
            let snapshot = EntitySnapshot {
                name: world.get::<Name>(entity).ok().cloned(),
                transform: world.get::<Transform>(entity).ok().cloned(),
                sprite: world.get::<Sprite>(entity).ok().cloned(),
                enabled: world.get::<Enabled>(entity).ok().cloned(),
            };
            entities.push(snapshot);
        }

        SceneSnapshot { entities }
    }

    /// Restore world to this snapshot
    pub fn restore(self, world: &mut World) {
        // Clear all existing entities
        world.clear();

        // Recreate entities from snapshot
        for entity_data in self.entities {
            let mut builder = world.spawn();

            if let Some(name) = entity_data.name {
                builder = builder.with(name);
            }
            if let Some(transform) = entity_data.transform {
                builder = builder.with(transform);
            }
            if let Some(sprite) = entity_data.sprite {
                builder = builder.with(sprite);
            }
            if let Some(enabled) = entity_data.enabled {
                builder = builder.with(enabled);
            }

            builder.build();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn test_capture_and_restore() {
        let mut world = World::new();

        // Create test entity
        world.spawn()
            .with(Name::new("TestEntity"))
            .with(Transform::from_position(Vec2::new(100.0, 200.0)))
            .with(Enabled::default())
            .build();

        assert_eq!(world.len(), 1);

        // Capture snapshot
        let snapshot = SceneSnapshot::capture(&world);
        assert_eq!(snapshot.entities.len(), 1);

        // Modify world
        world.spawn()
            .with(Name::new("NewEntity"))
            .build();
        assert_eq!(world.len(), 2);

        // Restore snapshot
        snapshot.restore(&mut world);
        assert_eq!(world.len(), 1);

        // Verify restored entity
        let entity = world.find("TestEntity");
        assert!(entity.is_some());
    }

    #[test]
    fn test_capture_empty_world() {
        let world = World::new();
        let snapshot = SceneSnapshot::capture(&world);
        assert_eq!(snapshot.entities.len(), 0);
    }
}
```

**Step 2: Update lib.rs**

Update `crates/longhorn-editor/src/lib.rs`:

```rust
mod state;
mod toolbar;
mod snapshot;
mod panels;
mod editor;

pub use state::*;
pub use toolbar::*;
pub use snapshot::*;
pub use panels::*;
pub use editor::*;
```

**Step 3: Run tests**

Run: `cargo test -p longhorn-editor`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/longhorn-editor/src/snapshot.rs crates/longhorn-editor/src/lib.rs
git commit -m "feat(editor): add SceneSnapshot for capture/restore"
```

---

## Task 5: Add ViewportRenderer

**Files:**
- Create: `crates/longhorn-editor/src/viewport_renderer.rs`
- Modify: `crates/longhorn-editor/src/lib.rs`

**Step 1: Create viewport_renderer.rs**

```rust
use wgpu;
use longhorn_core::World;
use longhorn_renderer::{Camera, Color};

/// Renders the game scene to an off-screen texture for display in egui
pub struct ViewportRenderer {
    render_texture: wgpu::Texture,
    render_view: wgpu::TextureView,
    size: (u32, u32),
    clear_color: Color,
}

impl ViewportRenderer {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let (render_texture, render_view) = Self::create_texture(device, width, height);

        Self {
            render_texture,
            render_view,
            size: (width, height),
            clear_color: Color::from_rgba8(40, 40, 50, 255),
        }
    }

    fn create_texture(device: &wgpu::Device, width: u32, height: u32) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Viewport Render Texture"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
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

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        if self.size == (width, height) {
            return;
        }

        let (texture, view) = Self::create_texture(device, width, height);
        self.render_texture = texture;
        self.render_view = view;
        self.size = (width, height);
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

    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    /// Render a clear frame (no sprites, just background)
    pub fn render_clear(&self, encoder: &mut wgpu::CommandEncoder) {
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Viewport Clear Pass"),
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
        // Pass ends when dropped
    }
}
```

**Step 2: Update lib.rs**

Update `crates/longhorn-editor/src/lib.rs`:

```rust
mod state;
mod toolbar;
mod snapshot;
mod viewport_renderer;
mod panels;
mod editor;

pub use state::*;
pub use toolbar::*;
pub use snapshot::*;
pub use viewport_renderer::*;
pub use panels::*;
pub use editor::*;
```

**Step 3: Verify compiles**

Run: `cargo check -p longhorn-editor`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add crates/longhorn-editor/src/viewport_renderer.rs crates/longhorn-editor/src/lib.rs
git commit -m "feat(editor): add ViewportRenderer for render-to-texture"
```

---

## Task 6: Update Editor to Use Toolbar

**Files:**
- Modify: `crates/longhorn-editor/src/editor.rs`

**Step 1: Update editor.rs**

Replace contents with:

```rust
use egui::Context;
use longhorn_engine::Engine;
use crate::{EditorState, EditorMode, SceneTreePanel, InspectorPanel, ViewportPanel, Toolbar, ToolbarAction, SceneSnapshot};

pub struct Editor {
    state: EditorState,
    scene_tree: SceneTreePanel,
    inspector: InspectorPanel,
    viewport: ViewportPanel,
    toolbar: Toolbar,
    scene_snapshot: Option<SceneSnapshot>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            state: EditorState::new(),
            scene_tree: SceneTreePanel::new(),
            inspector: InspectorPanel::new(),
            viewport: ViewportPanel::new(),
            toolbar: Toolbar::new(),
            scene_snapshot: None,
        }
    }

    pub fn state(&self) -> &EditorState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut EditorState {
        &mut self.state
    }

    /// Handle toolbar action and update state
    pub fn handle_toolbar_action(&mut self, action: ToolbarAction, engine: &mut Engine) {
        match action {
            ToolbarAction::None => {}
            ToolbarAction::Play => {
                // Capture scene state before playing
                self.scene_snapshot = Some(SceneSnapshot::capture(engine.world()));
                self.state.mode = EditorMode::Play;
                self.state.paused = false;
                log::info!("Entering Play mode");
            }
            ToolbarAction::Pause => {
                self.state.paused = true;
                log::info!("Game paused");
            }
            ToolbarAction::Resume => {
                self.state.paused = false;
                log::info!("Game resumed");
            }
            ToolbarAction::Stop => {
                // Restore scene state
                if let Some(snapshot) = self.scene_snapshot.take() {
                    snapshot.restore(engine.world_mut());
                    log::info!("Scene restored");
                }
                self.state.mode = EditorMode::Scene;
                self.state.paused = false;
                log::info!("Entering Scene mode");
            }
        }
    }

    pub fn show(&mut self, ctx: &Context, engine: &mut Engine) -> bool {
        let mut should_exit = false;
        let mut toolbar_action = ToolbarAction::None;

        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Game").clicked() {
                        log::info!("Open Game clicked (not implemented)");
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        should_exit = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // Toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            toolbar_action = self.toolbar.show(ui, &self.state);
        });

        // Handle toolbar action
        self.handle_toolbar_action(toolbar_action, engine);

        // Left panel - Scene Tree
        egui::SidePanel::left("scene_tree")
            .default_width(200.0)
            .show(ctx, |ui| {
                self.scene_tree.show(ui, engine.world(), &mut self.state);
            });

        // Right panel - Inspector
        egui::SidePanel::right("inspector")
            .default_width(250.0)
            .show(ctx, |ui| {
                // In play mode, show read-only indicator
                if self.state.is_playing() {
                    ui.label("(Read-only during play)");
                    ui.separator();
                }
                self.inspector.show(ui, engine.world_mut(), &self.state);
            });

        // Center panel - Viewport
        egui::CentralPanel::default().show(ctx, |ui| {
            self.viewport.show(ui);
        });

        should_exit
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: Verify compiles**

Run: `cargo check -p longhorn-editor`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add crates/longhorn-editor/src/editor.rs
git commit -m "feat(editor): integrate toolbar with mode switching and snapshot"
```

---

## Task 7: Rewrite Editor Binary with winit + wgpu + egui

**Files:**
- Modify: `editor/src/main.rs`

**Step 1: Rewrite main.rs**

Replace entire contents with:

```rust
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};
use longhorn_editor::{Editor, EditorMode, ViewportRenderer};
use longhorn_engine::Engine;
use longhorn_core::{Name, Transform, Sprite, Enabled, AssetId};
use glam::Vec2;

struct EditorApp {
    window: Option<Arc<Window>>,
    gpu_state: Option<GpuState>,
    egui_state: Option<EguiState>,
    engine: Engine,
    editor: Editor,
    viewport_renderer: Option<ViewportRenderer>,
}

struct GpuState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
}

struct EguiState {
    ctx: egui::Context,
    winit_state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

impl EditorApp {
    fn new() -> Self {
        let mut engine = Engine::new_headless();

        // Spawn test entities
        engine.world_mut()
            .spawn()
            .with(Name::new("Player"))
            .with(Transform::from_position(Vec2::new(100.0, 200.0)))
            .with(Sprite::new(AssetId::new(1), Vec2::new(32.0, 32.0)))
            .with(Enabled::default())
            .build();

        engine.world_mut()
            .spawn()
            .with(Name::new("Enemy"))
            .with(Transform::from_position(Vec2::new(300.0, 150.0)))
            .with(Sprite::new(AssetId::new(2), Vec2::new(64.0, 64.0)))
            .with(Enabled::default())
            .build();

        Self {
            window: None,
            gpu_state: None,
            egui_state: None,
            engine,
            editor: Editor::new(),
            viewport_renderer: None,
        }
    }

    fn init_gpu(&mut self, window: Arc<Window>) {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: Some("Editor Device"),
                memory_hints: Default::default(),
            },
            None,
        ))
        .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // Create viewport renderer
        let viewport_renderer = ViewportRenderer::new(&device, 800, 600);

        // Create egui state
        let ctx = egui::Context::default();
        let winit_state = egui_winit::State::new(
            ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1, false);

        self.gpu_state = Some(GpuState {
            device,
            queue,
            surface,
            surface_config,
        });

        self.egui_state = Some(EguiState {
            ctx,
            winit_state,
            renderer,
        });

        self.viewport_renderer = Some(viewport_renderer);
        self.window = Some(window);
    }

    fn render(&mut self) {
        let Some(window) = &self.window else { return };
        let Some(gpu) = &mut self.gpu_state else { return };
        let Some(egui_state) = &mut self.egui_state else { return };
        let Some(viewport_renderer) = &mut self.viewport_renderer else { return };

        // Update game if in play mode and not paused
        let editor_state = self.editor.state();
        if editor_state.mode == EditorMode::Play && !editor_state.paused {
            let _ = self.engine.update();
        }

        // Get surface texture
        let output = match gpu.surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost) => {
                gpu.surface.configure(&gpu.device, &gpu.surface_config);
                return;
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                log::error!("Out of memory");
                return;
            }
            Err(e) => {
                log::warn!("Surface error: {:?}", e);
                return;
            }
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Begin egui frame
        let raw_input = egui_state.winit_state.take_egui_input(window);
        egui_state.ctx.begin_pass(raw_input);

        // Run editor UI
        let should_exit = self.editor.show(&egui_state.ctx, &mut self.engine);
        if should_exit {
            std::process::exit(0);
        }

        // End egui frame
        let full_output = egui_state.ctx.end_pass();

        // Handle platform output
        egui_state.winit_state.handle_platform_output(window, full_output.platform_output);

        // Render viewport to texture
        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Editor Encoder"),
        });

        viewport_renderer.render_clear(&mut encoder);

        // Render egui
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [gpu.surface_config.width, gpu.surface_config.height],
            pixels_per_point: window.scale_factor() as f32,
        };

        let tris = egui_state.ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, delta) in &full_output.textures_delta.set {
            egui_state.renderer.update_texture(&gpu.device, &gpu.queue, *id, delta);
        }

        egui_state.renderer.update_buffers(&gpu.device, &gpu.queue, &mut encoder, &tris, &screen_descriptor);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            egui_state.renderer.render(&mut render_pass, &tris, &screen_descriptor);
        }

        for id in &full_output.textures_delta.free {
            egui_state.renderer.free_texture(id);
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // Request redraw
        window.request_redraw();
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            if let Some(gpu) = &mut self.gpu_state {
                gpu.surface_config.width = size.width;
                gpu.surface_config.height = size.height;
                gpu.surface.configure(&gpu.device, &gpu.surface_config);
            }
        }
    }
}

impl ApplicationHandler for EditorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Longhorn Editor")
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            self.init_gpu(window);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // Let egui handle events first
        if let Some(egui_state) = &mut self.egui_state {
            if let Some(window) = &self.window {
                let _ = egui_state.winit_state.on_window_event(window, &event);
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.resize(size);
            }
            WindowEvent::RedrawRequested => {
                self.render();
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = EditorApp::new();
    event_loop.run_app(&mut app).unwrap();
}
```

**Step 2: Verify compiles**

Run: `cargo build -p editor`
Expected: Compiles without errors (warnings OK)

**Step 3: Test run**

Run: `cargo run -p editor`
Expected: Editor window opens with toolbar showing Play/Pause/Stop buttons

**Step 4: Commit**

```bash
git add editor/src/main.rs
git commit -m "feat(editor): rewrite with winit+wgpu+egui for render control"
```

---

## Task 8: Add Viewport Texture Display

**Files:**
- Modify: `crates/longhorn-editor/src/panels/viewport.rs`
- Modify: `editor/src/main.rs`

**Step 1: Update viewport.rs to accept texture ID**

Replace contents with:

```rust
use egui::{Ui, TextureId, Sense, Vec2 as EguiVec2};

pub struct ViewportPanel {
    texture_id: Option<TextureId>,
}

impl ViewportPanel {
    pub fn new() -> Self {
        Self { texture_id: None }
    }

    pub fn set_texture(&mut self, texture_id: TextureId) {
        self.texture_id = Some(texture_id);
    }

    pub fn show(&mut self, ui: &mut Ui) -> egui::Response {
        ui.heading("Viewport");
        ui.separator();

        let available = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available, Sense::click_and_drag());

        if let Some(texture_id) = self.texture_id {
            // Draw the rendered game texture
            ui.painter().image(
                texture_id,
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        } else {
            // Placeholder when no texture
            ui.painter().rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgb(30, 30, 30),
            );

            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Viewport (No Texture)",
                egui::FontId::proportional(20.0),
                egui::Color32::from_rgb(150, 150, 150),
            );
        }

        response
    }

    pub fn texture_id(&self) -> Option<TextureId> {
        self.texture_id
    }
}

impl Default for ViewportPanel {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: Update Editor to expose viewport**

Add to `crates/longhorn-editor/src/editor.rs` a method:

```rust
    pub fn viewport_mut(&mut self) -> &mut ViewportPanel {
        &mut self.viewport
    }
```

**Step 3: Update main.rs to register viewport texture with egui**

This requires modifying the render function to register the viewport texture. Add after creating viewport_renderer in init_gpu:

The texture registration happens dynamically. Update the render function to include texture registration.

**Step 4: Verify compiles**

Run: `cargo check -p editor`
Expected: Compiles

**Step 5: Commit**

```bash
git add crates/longhorn-editor/src/panels/viewport.rs crates/longhorn-editor/src/editor.rs
git commit -m "feat(editor): viewport panel accepts texture for display"
```

---

## Summary

**Implementation order:**
1. Add dependencies (egui-wgpu, egui-winit, pollster)
2. Add EditorMode to state
3. Add Toolbar component
4. Add SceneSnapshot
5. Add ViewportRenderer
6. Update Editor to use toolbar
7. Rewrite editor binary with winit+wgpu+egui
8. Connect viewport texture display

**Key files created:**
- `crates/longhorn-editor/src/toolbar.rs`
- `crates/longhorn-editor/src/snapshot.rs`
- `crates/longhorn-editor/src/viewport_renderer.rs`

**Key files modified:**
- `editor/Cargo.toml`
- `crates/longhorn-editor/Cargo.toml`
- `crates/longhorn-editor/src/state.rs`
- `crates/longhorn-editor/src/editor.rs`
- `crates/longhorn-editor/src/panels/viewport.rs`
- `editor/src/main.rs`
