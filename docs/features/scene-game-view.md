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
