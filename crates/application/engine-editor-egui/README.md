# Engine Editor EGUI

Unity-style game editor built with EGUI and dockable panels.

## Module Structure

The editor is organized into the following modules:

### Core Modules
- `main.rs` (335 lines) - Main application entry point and UnityEditor struct
- `editor_state.rs` - Editor state management (GameObject, ConsoleMessage)
- `types.rs` - Common types (PlayState, SceneNavigation, GizmoSystem, etc.)
- `editor_coordinator.rs` - Coordinates play states and inter-panel communication

### UI Modules (`ui/`)
- `style.rs` - Unity-style theming and visual setup
- `toolbar.rs` - Play/pause controls and tool selection
- `menu_bar.rs` - File, Edit, View menus
- `tab_viewer.rs` - Docking system tab viewer implementation

### Panel Modules (`panels/`)
- `inspector/` - Component editing panel
- `hierarchy/` - Entity tree view
- `console/` - Log output panel
- `project/` - Asset browser panel
- `game_view/` - Runtime game preview
- `scene_view/` - 3D scene editor with gizmos
  - `mod.rs` - Main scene view panel
  - `rendering.rs` - Scene rendering logic
  - `navigation.rs` - WASD + mouse camera controls
  - `gizmos.rs` - 3D manipulation gizmos
  - `scene_input.rs` - Mouse/keyboard input handling
  - `scene_renderer.rs` - Rendering coordination
  - `object_renderer.rs` - Individual object rendering
  - `scene_view_impl.rs` - Scene view implementation

### Other Modules
- `assets/` - Asset management (textures, etc.)
- `world_setup.rs` - Default world initialization
- `play_state.rs` - Play/pause/stop state management
- `scene_renderer.rs` - Top-level scene rendering
- `bridge.rs` - ECS to renderer bridge
- `utils.rs` - Utility functions

## Architecture

The editor follows a modular architecture with clear separation of concerns:

1. **Main Application** (`main.rs`) - Orchestrates the editor UI and delegates to specialized modules
2. **Panels** - Each panel is self-contained with its own state and rendering logic
3. **Coordinator** - Manages high-level editor state transitions
4. **Type System** - Shared types enable communication between modules

## Running the Editor

```bash
cargo run --bin unity-editor-egui
```