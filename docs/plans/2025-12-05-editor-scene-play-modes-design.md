# Editor Scene & Play Modes Design

## Overview

Add Scene Mode and Play Mode to the Longhorn editor, similar to Unity/Godot. The viewport will render the actual game scene, and users can switch between editing and playing.

## Key Decisions

| Decision | Choice |
|----------|--------|
| Rendering approach | Render-to-texture (wgpu → egui image) |
| Scene Mode behavior | Static view with click-to-select entities |
| Play Mode behavior | In-editor play with live inspection |
| State on stop | Full restore to pre-play state |
| Play controls | Toolbar below menu bar |

## Editor Modes

```rust
pub enum EditorMode {
    Scene,  // Editing - no game logic runs
    Play,   // Running - game loop active
}
```

### Mode Behavior

| Mode | Rendering | Game Scripts | Input to Game | Inspector |
|------|-----------|--------------|---------------|-----------|
| Scene | Yes | No | No | Edit values |
| Play | Yes | Yes | Yes | Read-only view |
| Play (paused) | Yes | No | No | Read-only view |

### Mode Transitions

- **Scene → Play**: Serialize current scene to snapshot, start game loop
- **Play → Scene**: Deserialize snapshot to restore original state
- **Pause/Resume**: Toggle paused flag, game loop checks this

## Render-to-Texture Architecture

### How It Works

1. Create an off-screen wgpu texture at viewport size
2. Renderer draws the game scene to this texture (not the screen)
3. Register texture with egui as an image
4. egui displays the image in the viewport panel

### ViewportRenderer

```rust
pub struct ViewportRenderer {
    render_texture: wgpu::Texture,
    render_view: wgpu::TextureView,
    texture_id: egui::TextureId,
    size: (u32, u32),
}

impl ViewportRenderer {
    pub fn new(
        device: &wgpu::Device,
        egui_renderer: &mut egui_wgpu::Renderer,
        width: u32,
        height: u32,
    ) -> Self;

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        egui_renderer: &mut egui_wgpu::Renderer,
        width: u32,
        height: u32,
    );

    pub fn texture_id(&self) -> egui::TextureId;
}
```

### Rendering Flow Per Frame

```
1. Begin frame
2. Renderer draws scene → ViewportRenderer's texture
3. egui builds UI, viewport panel uses texture_id as Image
4. egui renders to screen (including the viewport image)
5. Present
```

### Editor Binary Changes

Switch from eframe (glow backend) to egui + wgpu + winit directly for full control over the render pipeline.

## Toolbar & Play Controls

### Layout

```
┌─────────────────────────────────────────────────────────────┐
│ File  Edit  View                                            │  ← Menu bar
├─────────────────────────────────────────────────────────────┤
│              [▶ Play]  [⏸ Pause]  [⏹ Stop]                  │  ← Toolbar
├─────────────────────────────────────────────────────────────┤
│ Scene │           Viewport              │ Inspector         │
```

### Button States

| Mode | Play | Pause | Stop |
|------|------|-------|------|
| Scene | Enabled | Disabled | Disabled |
| Play (running) | Disabled | Enabled | Enabled |
| Play (paused) | Enabled (resumes) | Disabled | Enabled |

### Keyboard Shortcuts

- `Space` or `Cmd+P` — Play/Pause toggle
- `Escape` or `Cmd+.` — Stop (return to Scene)

### Visual Indicator

Toolbar shows mode label: "Scene Mode" / "Playing" / "Paused"

### Toolbar Struct

```rust
pub struct Toolbar;

impl Toolbar {
    pub fn show(&mut self, ui: &mut Ui, state: &mut EditorState) -> ToolbarAction;
}

pub enum ToolbarAction {
    None,
    Play,
    Pause,
    Resume,
    Stop,
}
```

## Scene Snapshot & Restore

### Data Structures

```rust
#[derive(Serialize, Deserialize)]
pub struct SceneSnapshot {
    entities: Vec<EntitySnapshot>,
}

#[derive(Serialize, Deserialize)]
pub struct EntitySnapshot {
    name: Option<Name>,
    transform: Option<Transform>,
    sprite: Option<Sprite>,
    enabled: Option<Enabled>,
}
```

### Operations

```rust
impl SceneSnapshot {
    /// Capture current world state
    pub fn capture(world: &World) -> Self {
        let mut entities = Vec::new();
        for (entity, ()) in world.query::<()>().iter() {
            entities.push(EntitySnapshot {
                name: world.get::<Name>(entity).ok(),
                transform: world.get::<Transform>(entity).ok(),
                sprite: world.get::<Sprite>(entity).ok(),
                enabled: world.get::<Enabled>(entity).ok(),
            });
        }
        SceneSnapshot { entities }
    }

    /// Restore world to this snapshot
    pub fn restore(&self, world: &mut World) {
        world.clear();
        for entity_data in &self.entities {
            let mut builder = world.spawn();
            if let Some(name) = &entity_data.name {
                builder = builder.with(name.clone());
            }
            if let Some(transform) = &entity_data.transform {
                builder = builder.with(transform.clone());
            }
            if let Some(sprite) = &entity_data.sprite {
                builder = builder.with(sprite.clone());
            }
            if let Some(enabled) = &entity_data.enabled {
                builder = builder.with(*enabled);
            }
            builder.build();
        }
    }
}
```

### Usage

- Play clicked → `state.scene_snapshot = Some(SceneSnapshot::capture(&world))`
- Stop clicked → `state.scene_snapshot.take().unwrap().restore(&mut world)`

## Viewport Click Selection (Scene Mode Only)

### Selection Logic

```rust
impl ViewportPanel {
    pub fn handle_click(
        &self,
        click_pos: egui::Pos2,
        viewport_rect: egui::Rect,
        camera: &Camera,
        world: &World,
    ) -> Option<Entity> {
        // Convert to normalized viewport coords (0..1)
        let normalized = (click_pos - viewport_rect.min) / viewport_rect.size();

        // Convert to world coords
        let world_pos = camera.viewport_to_world(normalized);

        // Find entities under click, sorted by z-index (top first)
        let mut hits: Vec<(Entity, i32)> = vec![];
        for (entity, (transform, sprite)) in world.query::<(&Transform, &Sprite)>().iter() {
            let bounds = sprite_bounds(transform, sprite);
            if bounds.contains(world_pos) {
                hits.push((entity, sprite.z_index));
            }
        }

        hits.sort_by(|a, b| b.1.cmp(&a.1));
        hits.first().map(|(e, _)| *e)
    }
}
```

### Visual Feedback

Selected entity gets a highlight rectangle drawn around it in the viewport (Scene Mode only).

## Updated Editor Flow

### Main Loop

```rust
fn main_loop() {
    // Setup: winit window, wgpu device, egui_wgpu renderer
    // Create: Engine, Editor, ViewportRenderer

    event_loop.run(|event| {
        match event {
            WindowEvent::RedrawRequested => {
                // 1. Handle toolbar actions
                match toolbar_action {
                    Play => {
                        state.scene_snapshot = Some(SceneSnapshot::capture(&world));
                        state.mode = EditorMode::Play;
                        engine.start();
                    }
                    Stop => {
                        state.scene_snapshot.take().unwrap().restore(&mut world);
                        state.mode = EditorMode::Scene;
                    }
                    Pause => state.paused = true,
                    Resume => state.paused = false,
                }

                // 2. Update game (only in Play mode, not paused)
                if state.mode == EditorMode::Play && !state.paused {
                    engine.update();
                }

                // 3. Render scene to viewport texture
                viewport_renderer.render(&engine, &camera);

                // 4. Render egui (with viewport as image)
                egui_renderer.render(ui);
            }
        }
    });
}
```

## File Structure

### New Files

```
crates/longhorn-editor/src/
├── state.rs              # Add EditorMode, SceneSnapshot
├── toolbar.rs            # NEW: Play/Pause/Stop toolbar
├── viewport_renderer.rs  # NEW: Render-to-texture logic
├── snapshot.rs           # NEW: SceneSnapshot implementation
├── panels/
│   └── viewport.rs       # Update: display texture, handle clicks

editor/src/
└── main.rs               # Rewrite: winit + wgpu + egui_wgpu
```

### Dependencies to Add

```toml
# editor/Cargo.toml
egui_wgpu = "0.29"
winit = { workspace = true }
wgpu = { workspace = true }
```

## Implementation Order

1. Switch editor to winit + wgpu + egui_wgpu (remove eframe)
2. Add ViewportRenderer with render-to-texture
3. Display rendered texture in viewport panel
4. Add EditorMode and Toolbar
5. Implement SceneSnapshot capture/restore
6. Add viewport click selection
7. Add selection highlight rendering
