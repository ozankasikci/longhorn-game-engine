# Scene View and Game View Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement separate Scene View (editor camera) and Game View (game camera) panels with independent rendering pipelines.

**Architecture:** Two-camera system with Bevy-inspired render targets. EditorCamera (persistent, editor-managed) renders to editor_render_texture for Scene View panel. MainCamera (scene entity with marker component) renders to game_render_texture for Game View panel during Play mode only.

**Tech Stack:** Rust, WGPU, egui, egui_dock, longhorn_engine ECS

---

## Task 1: Add MainCamera Component

**Files:**
- Modify: `crates/longhorn-engine/src/components/camera.rs`
- Modify: `crates/longhorn-engine/src/components/mod.rs`

**Step 1: Write test for MainCamera component registration**

In `crates/longhorn-engine/src/components/camera.rs`, add test at bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::World;

    #[test]
    fn test_main_camera_component_registration() {
        let mut world = World::new();
        world.register_component::<MainCamera>();

        let entity = world.spawn();
        world.add_component(entity, MainCamera);

        assert!(world.has_component::<MainCamera>(entity));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p longhorn-engine test_main_camera_component_registration`

Expected: FAIL with "MainCamera not found" or compilation error

**Step 3: Implement MainCamera component**

In `crates/longhorn-engine/src/components/camera.rs`, add after existing Camera component:

```rust
/// Marker component indicating this camera is the main game camera
/// Only one MainCamera should exist per scene
#[derive(Component, Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MainCamera;
```

**Step 4: Export MainCamera in components module**

In `crates/longhorn-engine/src/components/mod.rs`, add to exports:

```rust
pub use camera::{Camera, MainCamera};
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p longhorn-engine test_main_camera_component_registration`

Expected: PASS

**Step 6: Commit**

```bash
git add crates/longhorn-engine/src/components/camera.rs crates/longhorn-engine/src/components/mod.rs
git commit -m "feat(engine): add MainCamera marker component"
```

---

## Task 2: Add EditorCamera Struct

**Files:**
- Create: `crates/longhorn-editor/src/camera.rs`
- Modify: `crates/longhorn-editor/src/lib.rs`

**Step 1: Write test for EditorCamera creation and input handling**

Create `crates/longhorn-editor/src/camera.rs`:

```rust
use longhorn_engine::math::{Vec2, Vec3};
use longhorn_engine::Transform;

#[derive(Debug, Clone)]
pub struct EditorCamera {
    pub transform: Transform,
    pub zoom: f32,
}

#[derive(Debug, Default)]
pub struct CameraInput {
    pub mmb_held: bool,
    pub rmb_held: bool,
    pub mouse_delta: Vec2,
    pub scroll_delta: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_camera_default() {
        let camera = EditorCamera::default();
        assert_eq!(camera.zoom, 1.0);
        assert_eq!(camera.transform.position, Vec3::new(0.0, 0.0, 10.0));
    }

    #[test]
    fn test_camera_pan() {
        let mut camera = EditorCamera::default();
        let input = CameraInput {
            mmb_held: true,
            mouse_delta: Vec2::new(10.0, 5.0),
            ..Default::default()
        };

        camera.handle_input(&input);

        // Camera should move opposite to mouse delta
        assert!(camera.transform.position.x < 0.0);
        assert!(camera.transform.position.y > 0.0);
    }

    #[test]
    fn test_camera_zoom() {
        let mut camera = EditorCamera::default();
        let input = CameraInput {
            scroll_delta: 1.0,
            ..Default::default()
        };

        camera.handle_input(&input);

        assert!(camera.zoom > 1.0);
        assert!(camera.zoom <= 10.0); // Clamped
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p longhorn-editor test_editor_camera`

Expected: FAIL - methods not implemented

**Step 3: Implement EditorCamera**

In `crates/longhorn-editor/src/camera.rs`, add implementations before tests:

```rust
impl Default for EditorCamera {
    fn default() -> Self {
        Self {
            transform: Transform {
                position: Vec3::new(0.0, 0.0, 10.0),
                rotation: Vec3::ZERO,
                scale: Vec3::ONE,
            },
            zoom: 1.0,
        }
    }
}

impl EditorCamera {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_input(&mut self, input: &CameraInput) {
        // Middle mouse button - Pan
        if input.mmb_held {
            let pan_speed = self.pan_speed();
            self.transform.position.x -= input.mouse_delta.x * pan_speed;
            self.transform.position.y += input.mouse_delta.y * pan_speed;
        }

        // Scroll - Zoom
        if input.scroll_delta != 0.0 {
            self.zoom *= 1.0 + input.scroll_delta * 0.1;
            self.zoom = self.zoom.clamp(0.1, 10.0);
        }
    }

    fn pan_speed(&self) -> f32 {
        // Pan speed scales with zoom level
        self.zoom * 0.01
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p longhorn-editor test_editor_camera`

Expected: PASS (all 3 tests)

**Step 5: Export camera module**

In `crates/longhorn-editor/src/lib.rs`, add:

```rust
mod camera;
pub use camera::{EditorCamera, CameraInput};
```

**Step 6: Commit**

```bash
git add crates/longhorn-editor/src/camera.rs crates/longhorn-editor/src/lib.rs
git commit -m "feat(editor): add EditorCamera with pan/zoom input handling"
```

---

## Task 3: Add Dual Texture Support to ViewportRenderer

**Files:**
- Modify: `crates/longhorn-editor/src/viewport_renderer.rs`

**Step 1: Write test for dual texture creation**

In `crates/longhorn-editor/src/viewport_renderer.rs`, add test module at bottom:

```rust
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
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p longhorn-editor test_viewport_renderer_has_editor_texture`

Expected: FAIL - field names don't exist

**Step 3: Rename and add texture fields**

In `crates/longhorn-editor/src/viewport_renderer.rs`, find the `EditorViewportRenderer` struct and modify:

```rust
pub struct EditorViewportRenderer {
    // Renamed from viewport_texture
    editor_render_texture: GpuTextureResource,

    // New field for game view
    game_render_texture: Option<GpuTextureResource>,

    // ... keep all existing fields (texture_cache, etc.)
}
```

**Step 4: Update constructor to use new field names**

In `EditorViewportRenderer::new()`, change:

```rust
// OLD:
// viewport_texture: GpuTextureResource::new(...)

// NEW:
editor_render_texture: GpuTextureResource::new(...),
game_render_texture: None,
```

**Step 5: Update all references from viewport_texture to editor_render_texture**

Search and replace in `viewport_renderer.rs`:
- `self.viewport_texture` → `self.editor_render_texture`
- Update `register_with_egui()` call
- Update `resize()` call

**Step 6: Run test to verify it passes**

Run: `cargo test -p longhorn-editor test_viewport_renderer_has_editor_texture`

Expected: PASS

**Step 7: Verify editor still compiles and runs**

Run: `cargo build -p longhorn-editor`

Expected: SUCCESS

**Step 8: Commit**

```bash
git add crates/longhorn-editor/src/viewport_renderer.rs
git commit -m "refactor(editor): rename viewport_texture to editor_render_texture, add game_render_texture"
```

---

## Task 4: Add Texture ID Getters

**Files:**
- Modify: `crates/longhorn-editor/src/viewport_renderer.rs`

**Step 1: Write tests for texture ID getters**

In `crates/longhorn-editor/src/viewport_renderer.rs` test module:

```rust
#[test]
fn test_editor_texture_id_getter() {
    let _test_compile = |renderer: &EditorViewportRenderer| {
        let _id: egui::TextureId = renderer.editor_texture_id();
    };
}

#[test]
fn test_game_texture_id_getter_when_none() {
    let _test_compile = |renderer: &EditorViewportRenderer| {
        let id: Option<egui::TextureId> = renderer.game_texture_id();
        assert!(id.is_none());
    };
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p longhorn-editor editor_texture_id`

Expected: FAIL - methods don't exist

**Step 3: Implement texture ID getters**

In `crates/longhorn-editor/src/viewport_renderer.rs`, add methods to `EditorViewportRenderer` impl:

```rust
impl EditorViewportRenderer {
    // ... existing methods

    pub fn editor_texture_id(&self) -> egui::TextureId {
        self.editor_render_texture.egui_texture_id
    }

    pub fn game_texture_id(&self) -> Option<egui::TextureId> {
        self.game_render_texture.as_ref().map(|tex| tex.egui_texture_id)
    }

    pub fn editor_texture_size(&self) -> (u32, u32) {
        (self.editor_render_texture.width, self.editor_render_texture.height)
    }

    pub fn game_texture_size(&self) -> Option<(u32, u32)> {
        self.game_render_texture.as_ref().map(|tex| (tex.width, tex.height))
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p longhorn-editor editor_texture_id`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/longhorn-editor/src/viewport_renderer.rs
git commit -m "feat(editor): add texture ID and size getters for editor and game textures"
```

---

## Task 5: Split Rendering into Scene and Game Methods

**Files:**
- Modify: `crates/longhorn-editor/src/viewport_renderer.rs`

**Step 1: Write structural test for new render methods**

In `crates/longhorn-editor/src/viewport_renderer.rs` tests:

```rust
#[test]
fn test_render_methods_exist() {
    let _test_compile = |renderer: &mut EditorViewportRenderer,
                         world: &longhorn_engine::ecs::World,
                         asset_manager: &longhorn_engine::AssetManager,
                         camera: &crate::EditorCamera| {
        let _r1 = renderer.render_scene_view(world, asset_manager, camera);
        let _r2 = renderer.render_game_view(world, asset_manager);
    };
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p longhorn-editor test_render_methods_exist`

Expected: FAIL - methods don't exist

**Step 3: Create render_to_texture helper method**

In `crates/longhorn-editor/src/viewport_renderer.rs`, add private helper:

```rust
impl EditorViewportRenderer {
    // ... existing methods

    fn render_to_texture(
        &mut self,
        world: &World,
        asset_manager: &AssetManager,
        camera_transform: &Transform,
        target_texture: &mut GpuTextureResource,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Copy existing render_with_assets logic here
        // Replace self.viewport_texture references with target_texture parameter
        // Use camera_transform for view matrix calculation

        // For now, just copy the entire existing implementation
        // This will be the core rendering logic

        // TODO: Extract this from current render_with_assets method
        todo!("Extract rendering logic from render_with_assets")
    }
}
```

**Step 4: Implement render_scene_view**

In `crates/longhorn-editor/src/viewport_renderer.rs`:

```rust
impl EditorViewportRenderer {
    pub fn render_scene_view(
        &mut self,
        world: &World,
        asset_manager: &AssetManager,
        editor_camera: &EditorCamera,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.render_to_texture(
            world,
            asset_manager,
            &editor_camera.transform,
            &mut self.editor_render_texture,
        )
    }
}
```

**Step 5: Implement render_game_view**

In `crates/longhorn-editor/src/viewport_renderer.rs`:

```rust
impl EditorViewportRenderer {
    pub fn render_game_view(
        &mut self,
        world: &World,
        asset_manager: &AssetManager,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use longhorn_engine::components::{Camera, MainCamera};

        // Find MainCamera in scene
        let main_camera_data = world
            .query::<(&Transform, &Camera, &MainCamera)>()
            .iter()
            .next();

        if let Some((transform, _camera, _)) = main_camera_data {
            // Lazy allocate game texture if needed
            if self.game_render_texture.is_none() {
                let (width, height) = self.editor_texture_size();
                self.game_render_texture = Some(
                    GpuTextureResource::new(
                        &self.device,
                        &self.queue,
                        width,
                        height,
                    )?
                );
                // Register with egui
                if let Some(game_tex) = &self.game_render_texture {
                    self.egui_renderer.register_native_texture(
                        &self.device,
                        &game_tex.view,
                        wgpu::FilterMode::Linear,
                    );
                }
            }

            let game_texture = self.game_render_texture.as_mut().unwrap();
            self.render_to_texture(world, asset_manager, transform, game_texture)
        } else {
            // No main camera found, skip rendering
            Ok(())
        }
    }
}
```

**Step 6: Extract render_to_texture logic**

Copy the body of the existing `render_with_assets()` method into `render_to_texture()`, replacing:
- `&self.viewport_texture` → `target_texture`
- Hardcoded camera transform → `camera_transform` parameter

**Step 7: Update existing render_with_assets to use new method**

```rust
pub fn render_with_assets(
    &mut self,
    world: &World,
    asset_manager: &AssetManager,
) -> Result<(), Box<dyn std::error::Error>> {
    // Temporary: use default camera for backwards compatibility
    let default_camera = EditorCamera::default();
    self.render_scene_view(world, asset_manager, &default_camera)
}
```

**Step 8: Build to verify compilation**

Run: `cargo build -p longhorn-editor`

Expected: SUCCESS

**Step 9: Commit**

```bash
git add crates/longhorn-editor/src/viewport_renderer.rs
git commit -m "feat(editor): split rendering into render_scene_view and render_game_view methods"
```

---

## Task 6: Add EditorCamera to Editor Struct

**Files:**
- Modify: `crates/longhorn-editor/src/editor.rs`

**Step 1: Add editor_camera field to Editor struct**

In `crates/longhorn-editor/src/editor.rs`, find the `Editor` struct and add:

```rust
pub struct Editor {
    // Add this field
    editor_camera: EditorCamera,

    // ... existing fields (state, panels, dock_state, etc.)
}
```

**Step 2: Initialize in Editor::new()**

In `Editor::new()` constructor:

```rust
pub fn new(/* existing params */) -> Self {
    Self {
        editor_camera: EditorCamera::new(),
        // ... existing field initialization
    }
}
```

**Step 3: Build to verify**

Run: `cargo build -p longhorn-editor`

Expected: SUCCESS

**Step 4: Commit**

```bash
git add crates/longhorn-editor/src/editor.rs
git commit -m "feat(editor): add EditorCamera to Editor struct"
```

---

## Task 7: Update Editor Render Loop

**Files:**
- Modify: `crates/longhorn-editor/src/editor.rs`

**Step 1: Find existing render call**

Locate the line in `editor.rs` that calls `viewport_renderer.render_with_assets()`

**Step 2: Replace with dual render calls**

Replace:
```rust
// OLD:
viewport_renderer.render_with_assets(&engine.world, &asset_manager)?;
```

With:
```rust
// NEW: Always render scene view
viewport_renderer.render_scene_view(
    &engine.world,
    &asset_manager,
    &self.editor_camera,
)?;

// Conditionally render game view in Play mode
if self.state.mode == EditorMode::Play {
    viewport_renderer.render_game_view(
        &engine.world,
        &asset_manager,
    )?;
}
```

**Step 3: Build to verify**

Run: `cargo build -p longhorn-editor`

Expected: SUCCESS

**Step 4: Test manually**

Run: `cargo run`

Expected: Editor opens, Scene View shows scene (same as before)

**Step 5: Commit**

```bash
git add crates/longhorn-editor/src/editor.rs
git commit -m "feat(editor): update render loop to use dual render calls"
```

---

## Task 8: Update Scene View Panel

**Files:**
- Modify: `crates/longhorn-editor/src/panels/viewport.rs`

**Step 1: Update texture reference in ViewportPanel::show()**

Find the line that displays the viewport texture in `viewport.rs`

**Step 2: Change to use editor_texture_id()**

Replace:
```rust
// OLD: (something like)
// ui.painter().image(viewport_renderer.viewport_texture_id(), ...)
```

With:
```rust
// NEW:
let texture_id = viewport_renderer.editor_texture_id();
ui.painter().image(
    texture_id,
    rect,
    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
    Color32::WHITE,
);
```

**Step 3: Build and test**

Run: `cargo run`

Expected: Scene View panel shows editor render texture

**Step 4: Commit**

```bash
git add crates/longhorn-editor/src/panels/viewport.rs
git commit -m "refactor(editor): update Scene View panel to use editor_texture_id()"
```

---

## Task 9: Add Camera Input Handling to Scene View Panel

**Files:**
- Modify: `crates/longhorn-editor/src/panels/viewport.rs`
- Modify: `crates/longhorn-editor/src/editor.rs`

**Step 1: Capture input in ViewportPanel**

In `viewport.rs`, in the `show()` method, after allocating the viewport rect:

```rust
// Allocate interactive rect
let response = ui.allocate_rect(viewport_rect, egui::Sense::click_and_drag());

// Capture camera input when hovered
let mut camera_input = CameraInput::default();
if response.hovered() {
    camera_input.mmb_held = ui.input(|i| {
        i.pointer.button_down(egui::PointerButton::Middle)
    });
    camera_input.mouse_delta = response.drag_delta().into(); // Convert to Vec2
    camera_input.scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
}
```

**Step 2: Return camera_input from ViewportPanel::show()**

Change method signature from:
```rust
pub fn show(&mut self, ...) -> ()
```

To:
```rust
pub fn show(&mut self, ...) -> CameraInput
```

Return `camera_input` at the end.

**Step 3: Update Editor to handle camera input**

In `editor.rs`, find where `ViewportPanel::show()` is called:

```rust
// In the panel rendering loop
if let Some(viewport_panel) = self.panels.get_viewport_panel() {
    let camera_input = viewport_panel.show(/* params */);
    self.editor_camera.handle_input(&camera_input);
}
```

**Step 4: Build and test**

Run: `cargo run`

Test:
- Middle mouse drag should pan camera
- Scroll wheel should zoom

Expected: Camera moves in Scene View

**Step 5: Commit**

```bash
git add crates/longhorn-editor/src/panels/viewport.rs crates/longhorn-editor/src/editor.rs
git commit -m "feat(editor): add camera pan/zoom input handling to Scene View panel"
```

---

## Task 10: Add Play Mode Snapshot System

**Files:**
- Modify: `crates/longhorn-editor/src/state.rs`

**Step 1: Write test for snapshot save/restore**

In `crates/longhorn-editor/src/state.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use longhorn_engine::ecs::World;

    #[test]
    fn test_enter_play_mode_saves_snapshot() {
        let mut state = EditorState::new();
        let world = World::new();

        assert_eq!(state.mode, EditorMode::Scene);

        state.enter_play_mode(&world).unwrap();

        assert_eq!(state.mode, EditorMode::Play);
        assert!(state.play_mode_snapshot.is_some());
    }

    #[test]
    fn test_exit_play_mode_clears_snapshot() {
        let mut state = EditorState::new();
        let mut world = World::new();

        state.enter_play_mode(&world).unwrap();
        state.exit_play_mode(&mut world).unwrap();

        assert_eq!(state.mode, EditorMode::Scene);
        assert!(state.play_mode_snapshot.is_none());
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p longhorn-editor test_enter_play_mode`

Expected: FAIL - methods/fields don't exist

**Step 3: Add play_mode_snapshot field to EditorState**

In `crates/longhorn-editor/src/state.rs`:

```rust
pub struct EditorState {
    pub mode: EditorMode,
    pub selected_entity: Option<Entity>,

    // NEW:
    play_mode_snapshot: Option<Vec<u8>>,
}
```

**Step 4: Initialize in constructor**

```rust
impl EditorState {
    pub fn new() -> Self {
        Self {
            mode: EditorMode::Scene,
            selected_entity: None,
            play_mode_snapshot: None,
        }
    }
}
```

**Step 5: Implement enter_play_mode()**

```rust
impl EditorState {
    pub fn enter_play_mode(&mut self, world: &World) -> Result<(), Box<dyn std::error::Error>> {
        // Serialize world to snapshot
        let snapshot = bincode::serialize(world)?;
        self.play_mode_snapshot = Some(snapshot);
        self.mode = EditorMode::Play;
        Ok(())
    }
}
```

**Step 6: Implement exit_play_mode()**

```rust
impl EditorState {
    pub fn exit_play_mode(&mut self, world: &mut World) -> Result<(), Box<dyn std::error::Error>> {
        // Restore world from snapshot
        if let Some(snapshot) = &self.play_mode_snapshot {
            *world = bincode::deserialize(snapshot)?;
        }

        self.mode = EditorMode::Scene;
        self.play_mode_snapshot = None;
        Ok(())
    }
}
```

**Step 7: Add bincode dependency if not present**

In `crates/longhorn-editor/Cargo.toml`, ensure:

```toml
[dependencies]
bincode = "1.3"
```

**Step 8: Run tests to verify they pass**

Run: `cargo test -p longhorn-editor test_enter_play_mode`

Expected: PASS (both tests)

**Step 9: Commit**

```bash
git add crates/longhorn-editor/src/state.rs crates/longhorn-editor/Cargo.toml
git commit -m "feat(editor): add play mode snapshot save/restore system"
```

---

## Task 11: Wire Up Play/Stop Buttons

**Files:**
- Modify: `crates/longhorn-editor/src/toolbar.rs` (or wherever toolbar is rendered)

**Step 1: Find Play button click handler**

Locate the Play button UI code (likely in `toolbar.rs` or `editor.rs`)

**Step 2: Add Play button handler**

```rust
// Play button
if ui.add_enabled(
    state.mode == EditorMode::Scene,
    egui::Button::new("▶ Play"),
).clicked() {
    if let Err(e) = state.enter_play_mode(&engine.world) {
        eprintln!("Failed to enter play mode: {}", e);
    }
}
```

**Step 3: Add Stop button handler**

```rust
// Stop button
if ui.add_enabled(
    state.mode == EditorMode::Play,
    egui::Button::new("⏹ Stop"),
).clicked() {
    if let Err(e) = state.exit_play_mode(&mut engine.world) {
        eprintln!("Failed to exit play mode: {}", e);
    }
}
```

**Step 4: Add mode indicator**

```rust
// Show current mode
let mode_text = match state.mode {
    EditorMode::Scene => "✏ Editing",
    EditorMode::Play => "▶ Playing",
};
ui.label(mode_text);
```

**Step 5: Build and test**

Run: `cargo run`

Test:
1. Click Play button
2. Verify mode changes to "▶ Playing"
3. Modify entity position
4. Click Stop button
5. Verify entity returns to original position

Expected: Scene state restores on Stop

**Step 6: Commit**

```bash
git add crates/longhorn-editor/src/toolbar.rs
git commit -m "feat(editor): wire up Play/Stop buttons with snapshot system"
```

---

## Task 12: Create Game View Panel

**Files:**
- Create: `crates/longhorn-editor/src/panels/game_view.rs`
- Modify: `crates/longhorn-editor/src/panels/mod.rs`
- Modify: `crates/longhorn-editor/src/docking.rs`

**Step 1: Create GameViewPanel struct**

Create `crates/longhorn-editor/src/panels/game_view.rs`:

```rust
use egui::{Color32, Pos2, Rect, Ui, pos2};
use crate::viewport_renderer::EditorViewportRenderer;
use crate::state::EditorState;

pub struct GameViewPanel;

impl GameViewPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        viewport_renderer: &EditorViewportRenderer,
        editor_state: &EditorState,
    ) {
        let available_size = ui.available_size();
        let rect = Rect::from_min_size(ui.cursor().min, available_size);

        // Check if in Play mode and has game texture
        if editor_state.mode == crate::state::EditorMode::Play {
            if let Some(texture_id) = viewport_renderer.game_texture_id() {
                // Render game texture
                ui.painter().image(
                    texture_id,
                    rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                // No game camera found
                self.show_placeholder(ui, "⚠ No MainCamera in scene");
            }
        } else {
            // Not in Play mode
            self.show_placeholder(ui, "Press ▶ Play to start");
        }
    }

    fn show_placeholder(&self, ui: &mut Ui, message: &str) {
        ui.centered_and_justified(|ui| {
            ui.label(message);
        });
    }
}
```

**Step 2: Export in panels module**

In `crates/longhorn-editor/src/panels/mod.rs`:

```rust
mod game_view;
pub use game_view::GameViewPanel;
```

**Step 3: Add to Editor struct**

In `crates/longhorn-editor/src/editor.rs`:

```rust
pub struct Editor {
    // ... existing fields
    game_view_panel: GameViewPanel,
}
```

Initialize in `Editor::new()`:
```rust
game_view_panel: GameViewPanel::new(),
```

**Step 4: Ensure Game View panel in dock layout**

In `crates/longhorn-editor/src/docking.rs`, verify `PanelType::GameView` exists in the default layout (tabbed with Scene View).

If not present, add it to center tabs.

**Step 5: Render Game View panel in dock system**

In the panel rendering match statement (in `editor.rs` or `docking.rs`):

```rust
PanelType::GameView => {
    self.game_view_panel.show(ui, viewport_renderer, &self.state);
}
```

**Step 6: Build and test**

Run: `cargo run`

Test:
1. Open Game View tab
2. Should show "Press ▶ Play to start"
3. Click Play
4. Should show game render texture (or "No MainCamera" if no camera in scene)

Expected: Game View panel displays correctly

**Step 7: Commit**

```bash
git add crates/longhorn-editor/src/panels/game_view.rs crates/longhorn-editor/src/panels/mod.rs crates/longhorn-editor/src/editor.rs crates/longhorn-editor/src/docking.rs
git commit -m "feat(editor): add Game View panel with placeholder and texture display"
```

---

## Task 13: Add MainCamera Helper in Inspector

**Files:**
- Modify: `crates/longhorn-editor/src/panels/inspector.rs`

**Step 1: Add MainCamera to component list**

In the Inspector panel where components are listed, ensure `MainCamera` appears as an addable component.

Find the component type dropdown or list and add:
```rust
use longhorn_engine::components::MainCamera;

// In component add UI:
if ui.button("Add MainCamera").clicked() {
    world.add_component(selected_entity, MainCamera);
}
```

**Step 2: Display MainCamera component**

In component display loop:
```rust
if world.has_component::<MainCamera>(entity) {
    ui.label("MainCamera ✓");
    if ui.button("Remove").clicked() {
        world.remove_component::<MainCamera>(entity);
    }
}
```

**Step 3: Build and test**

Run: `cargo run`

Test:
1. Select entity
2. Add Camera component
3. Add MainCamera component
4. Enter Play mode
5. Game View should show this camera's perspective

Expected: MainCamera can be added/removed via Inspector

**Step 4: Commit**

```bash
git add crates/longhorn-editor/src/panels/inspector.rs
git commit -m "feat(editor): add MainCamera component to Inspector panel"
```

---

## Task 14: Handle Multiple MainCamera Warning

**Files:**
- Modify: `crates/longhorn-editor/src/viewport_renderer.rs`
- Modify: `crates/longhorn-editor/src/panels/console.rs` (if console exists)

**Step 1: Add warning detection in render_game_view**

In `viewport_renderer.rs`, modify `render_game_view()`:

```rust
pub fn render_game_view(
    &mut self,
    world: &World,
    asset_manager: &AssetManager,
) -> Result<(), Box<dyn std::error::Error>> {
    use longhorn_engine::components::{Camera, MainCamera};

    // Collect all main cameras
    let main_cameras: Vec<_> = world
        .query::<(Entity, &Transform, &Camera, &MainCamera)>()
        .iter()
        .collect();

    // Warn if multiple found
    if main_cameras.len() > 1 {
        eprintln!("Warning: {} MainCamera components found. Using first one.", main_cameras.len());
    }

    // Use first camera
    if let Some((_entity, transform, _camera, _)) = main_cameras.first() {
        // ... existing render logic
    }

    Ok(())
}
```

**Step 2: Build and test**

Run: `cargo run`

Test:
1. Add MainCamera to two different entities
2. Enter Play mode
3. Check console output

Expected: Warning printed to stderr

**Step 3: Commit**

```bash
git add crates/longhorn-editor/src/viewport_renderer.rs
git commit -m "feat(editor): warn when multiple MainCamera components detected"
```

---

## Task 15: Add Texture Resize Handling

**Files:**
- Modify: `crates/longhorn-editor/src/viewport_renderer.rs`
- Modify: `crates/longhorn-editor/src/panels/viewport.rs`
- Modify: `crates/longhorn-editor/src/panels/game_view.rs`

**Step 1: Add resize methods to ViewportRenderer**

In `viewport_renderer.rs`:

```rust
impl EditorViewportRenderer {
    pub fn resize_editor_texture(&mut self, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.editor_render_texture = GpuTextureResource::new(
            &self.device,
            &self.queue,
            width,
            height,
        )?;

        // Re-register with egui
        self.egui_renderer.update_egui_texture_from_wgpu_texture(
            &self.device,
            &self.editor_render_texture.view,
            wgpu::FilterMode::Linear,
            self.editor_render_texture.egui_texture_id,
        );

        Ok(())
    }

    pub fn resize_game_texture(&mut self, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(game_tex) = &mut self.game_render_texture {
            *game_tex = GpuTextureResource::new(
                &self.device,
                &self.queue,
                width,
                height,
            )?;

            // Re-register with egui
            self.egui_renderer.update_egui_texture_from_wgpu_texture(
                &self.device,
                &game_tex.view,
                wgpu::FilterMode::Linear,
                game_tex.egui_texture_id,
            );
        }

        Ok(())
    }
}
```

**Step 2: Add resize check in Scene View panel**

In `viewport.rs`, before rendering:

```rust
let current_size = (viewport_rect.width() as u32, viewport_rect.height() as u32);
let texture_size = viewport_renderer.editor_texture_size();

if current_size.0 != texture_size.0 || current_size.1 != texture_size.1 {
    viewport_renderer.resize_editor_texture(current_size.0, current_size.1)?;
}
```

**Step 3: Add resize check in Game View panel**

In `game_view.rs`, before rendering game texture:

```rust
if let Some(texture_size) = viewport_renderer.game_texture_size() {
    let current_size = (rect.width() as u32, rect.height() as u32);
    if current_size.0 != texture_size.0 || current_size.1 != texture_size.1 {
        viewport_renderer.resize_game_texture(current_size.0, current_size.1)?;
    }
}
```

**Step 4: Build and test**

Run: `cargo run`

Test: Resize editor window and panels

Expected: Textures resize without artifacts

**Step 5: Commit**

```bash
git add crates/longhorn-editor/src/viewport_renderer.rs crates/longhorn-editor/src/panels/viewport.rs crates/longhorn-editor/src/panels/game_view.rs
git commit -m "feat(editor): add dynamic texture resizing for viewport panels"
```

---

## Task 16: Add Frame Selected Entity Feature

**Files:**
- Modify: `crates/longhorn-editor/src/camera.rs`
- Modify: `crates/longhorn-editor/src/editor.rs`

**Step 1: Write test for frame_entity**

In `camera.rs` tests:

```rust
#[test]
fn test_frame_entity() {
    use longhorn_engine::math::Vec3;

    let mut camera = EditorCamera::default();
    let entity_position = Vec3::new(100.0, 50.0, 0.0);
    let entity_size = Vec2::new(20.0, 20.0);

    camera.frame_entity(entity_position, entity_size);

    // Camera should center on entity
    assert!((camera.transform.position.x - 100.0).abs() < 0.01);
    assert!((camera.transform.position.y - 50.0).abs() < 0.01);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p longhorn-editor test_frame_entity`

Expected: FAIL - method doesn't exist

**Step 3: Implement frame_entity**

In `camera.rs`:

```rust
impl EditorCamera {
    pub fn frame_entity(&mut self, entity_position: Vec3, entity_size: Vec2) {
        // Center camera on entity
        self.transform.position.x = entity_position.x;
        self.transform.position.y = entity_position.y;

        // Adjust zoom to fit entity in view
        // Assume viewport size of 1000x1000 for now (can be refined)
        let viewport_size = 1000.0;
        let max_dimension = entity_size.x.max(entity_size.y);

        if max_dimension > 0.0 {
            self.zoom = viewport_size / (max_dimension * 2.0);
            self.zoom = self.zoom.clamp(0.1, 10.0);
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p longhorn-editor test_frame_entity`

Expected: PASS

**Step 5: Add 'F' key handler in editor**

In `editor.rs`, add keyboard input handling:

```rust
// When Scene View is focused
if ui.input(|i| i.key_pressed(egui::Key::F)) {
    if let Some(selected) = self.state.selected_entity {
        if let Ok(transform) = engine.world.get_component::<Transform>(selected) {
            // Calculate entity bounds (simplified - use sprite size if available)
            let entity_size = Vec2::new(64.0, 64.0); // Default size
            self.editor_camera.frame_entity(transform.position, entity_size);
        }
    }
}
```

**Step 6: Build and test**

Run: `cargo run`

Test:
1. Select entity
2. Press F key
3. Camera should center on entity

Expected: Camera frames selected entity

**Step 7: Commit**

```bash
git add crates/longhorn-editor/src/camera.rs crates/longhorn-editor/src/editor.rs
git commit -m "feat(editor): add Frame Selected Entity (F key) feature"
```

---

## Task 17: Documentation and Final Testing

**Files:**
- Create: `docs/features/scene-game-view.md`
- Modify: `README.md` (if user-facing)

**Step 1: Create feature documentation**

Create `docs/features/scene-game-view.md`:

```markdown
# Scene View and Game View

## Overview

Longhorn Editor provides two viewport panels for different purposes:

- **Scene View**: Editor-controlled camera for positioning and framing scenes
- **Game View**: Shows the game from the MainCamera's perspective during Play mode

## Usage

### Scene View Controls

- **Middle Mouse Drag**: Pan camera
- **Scroll Wheel**: Zoom in/out
- **F Key**: Frame selected entity

### Setting Up Main Camera

1. Create an entity in your scene
2. Add `Camera` component
3. Add `MainCamera` component

Only one entity should have the `MainCamera` component.

### Play Mode

- Click **▶ Play** to enter Play mode
  - Game View activates and shows MainCamera perspective
  - Game scripts begin executing
  - Scene View remains active for inspection

- Click **⏹ Stop** to exit Play mode
  - Scene restores to pre-Play state
  - Game View shows placeholder

## Architecture

- **EditorCamera**: Persistent editor-managed camera (not serialized)
- **MainCamera**: Scene entity with marker component
- Each camera renders to independent GPU textures
- Bevy-inspired RenderTarget pattern

## Implementation Files

- `crates/longhorn-engine/src/components/camera.rs` - MainCamera component
- `crates/longhorn-editor/src/camera.rs` - EditorCamera
- `crates/longhorn-editor/src/viewport_renderer.rs` - Dual texture rendering
- `crates/longhorn-editor/src/panels/viewport.rs` - Scene View panel
- `crates/longhorn-editor/src/panels/game_view.rs` - Game View panel
```

**Step 2: Comprehensive manual test**

Run: `cargo run`

Test checklist:
- [ ] Scene View displays scene
- [ ] Middle mouse drag pans camera
- [ ] Scroll wheel zooms
- [ ] Game View shows "Press Play" placeholder
- [ ] Add Camera + MainCamera to entity
- [ ] Press Play button
- [ ] Game View shows MainCamera perspective
- [ ] Scene View still functional during Play
- [ ] Modify entity position
- [ ] Press Stop button
- [ ] Entity returns to original position
- [ ] Game View shows placeholder again
- [ ] Resize panels - textures resize correctly
- [ ] Add MainCamera to second entity - warning appears
- [ ] Press F to frame entity

**Step 3: Fix any bugs found**

Address issues discovered during testing.

**Step 4: Commit documentation**

```bash
git add docs/features/scene-game-view.md
git commit -m "docs: add Scene View and Game View feature documentation"
```

---

## Final Verification

**Build release binary:**
```bash
cargo build --release
```

**Run full test suite:**
```bash
cargo test
```

**Expected:** All tests pass, release build succeeds

---

## Implementation Complete

This plan implements:
✅ Two-camera system (EditorCamera + MainCamera)
✅ Dual render targets (Bevy-inspired)
✅ Scene View with pan/zoom controls
✅ Game View with Play mode activation
✅ Play/Stop with snapshot save/restore
✅ MainCamera component and Inspector integration
✅ Texture resize handling
✅ Frame selected entity feature
✅ Error handling for edge cases
✅ Comprehensive documentation

**Total estimated time:** 4-6 hours for experienced developer with zero context
**Tasks:** 17 bite-sized tasks
**Commits:** 17+ commits (following DRY, YAGNI, TDD principles)
