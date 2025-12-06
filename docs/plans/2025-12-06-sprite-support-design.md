# Sprite Support Design

## Overview

End-to-end sprite workflow: importing images into a project, adding them to entities as components via the editor, serializing scenes, and rendering sprites in the game.

## Asset Import Flow

**When user imports an image:**

1. User right-clicks in Project panel → "Import Asset" → file picker opens
2. User selects an image file (PNG, JPEG)
3. Image is copied to the currently-selected folder in the Project panel (or project root if nothing selected)
4. Asset is registered in the `AssetManager` and assigned an `AssetId`
5. Project panel refreshes to show the new file

**Alternative import method:**
- Drag a file from the OS file explorer into the Project panel → same copy+register behavior

**Asset registry for ID stability:**
- A simple `assets.json` at project root maps paths → IDs:
```json
{
  "sprites/player.png": 1,
  "sprites/enemy.png": 2
}
```
- Updated whenever a new asset is imported
- Ensures the same file always gets the same ID across sessions

## Adding Sprites to Entities

### Method A: "Add Component" in Inspector

1. User selects an entity in the Scene Tree
2. Inspector panel shows existing components + an "Add Component" button at the bottom
3. Clicking "Add Component" shows a dropdown/popup with available components:
   - `Sprite`
   - `Script`
   - (future: more component types)
4. User selects "Sprite"
5. A texture picker popup appears showing all images in the project (grid view with thumbnails)
6. User clicks an image → Sprite component is added with that texture
7. Default values: `size` from image dimensions, `color` white (1,1,1,1), `flip_x/y` false

### Method B: Drag-drop from Project panel

1. User drags an image file from the Project panel
2. Drops onto an entity in the Scene Tree
3. If entity has no Sprite → adds Sprite component with that texture
4. If entity already has Sprite → replaces the texture (with undo support later)

### Inspector editing

Once a Sprite component exists, the Inspector shows:
- Texture: thumbnail + name + "Change" button (opens picker)
- Size: editable width/height fields
- Color: RGBA color picker
- Flip X / Flip Y: checkboxes

## Scene Serialization

**Scene file format:** `scenes/main.scene.json` (or user-chosen name)

```json
{
  "name": "Main Scene",
  "entities": [
    {
      "id": 1,
      "components": {
        "Name": "Player",
        "Transform": {
          "position": [100, 200],
          "rotation": 0,
          "scale": [1, 1]
        },
        "Sprite": {
          "texture_path": "sprites/player.png",
          "texture_id": 1,
          "size": [32, 32],
          "color": [1, 1, 1, 1],
          "flip_x": false,
          "flip_y": false
        },
        "Enabled": true
      }
    }
  ]
}
```

**Loading behavior:**
1. Parse JSON, iterate entities
2. For each Sprite, try loading by `texture_path` first
3. Validate that loaded asset's ID matches `texture_id` — if mismatch, warn in console (file may have been re-imported)
4. If path fails but ID exists in registry, try loading by ID (handles renames)
5. If both fail, log error and skip the Sprite component (entity still loads)

**Saving behavior:**
1. Iterate all entities in the World
2. Serialize each component to JSON
3. For Sprite, look up the texture's path from asset registry using its ID
4. Write to scene file

## Rendering Integration

**Current state:** The renderer already supports sprites — it batches by texture and renders via wgpu. The `Sprite` component already exists with `texture: AssetId`.

**What we need to connect:**

1. Engine render loop queries all entities with `(Transform, Sprite, Enabled)` components
2. For each sprite, look up texture in `AssetManager` by `AssetId`
3. Pass to renderer's `SpriteBatch` for drawing
4. Renderer handles the rest (z-sorting, batching, GPU upload)

**Texture loading:**
- Textures are lazy-loaded — first time a sprite is rendered, its texture is uploaded to GPU
- `TextureCache` in renderer prevents duplicate uploads
- Already implemented, just needs to be wired up to the component data

## Implementation Plan

### Files to create/modify

| Area | File | Changes |
|------|------|---------|
| Asset registry | `longhorn-assets/src/registry.rs` | New: `AssetRegistry` struct, loads/saves `assets.json`, maps path↔ID |
| Asset manager | `longhorn-assets/src/asset_manager.rs` | Integrate registry, add `import_asset()` method that copies + registers |
| Scene serialization | `longhorn-core/src/scene/` | New module: `Scene` struct, `save_scene()`, `load_scene()` |
| Inspector | `longhorn-editor/src/panels/inspector.rs` | Add "Add Component" button, Sprite editor UI, texture picker |
| Project panel | `longhorn-editor/src/panels/project.rs` | Add import context menu, drag-drop source for assets |
| Scene tree | `longhorn-editor/src/panels/scene_tree.rs` | Accept drag-drop of images onto entities |
| Editor state | `longhorn-editor/src/editor.rs` | Track drag-drop state, coordinate between panels |

### Implementation order

1. Asset registry (`assets.json` read/write)
2. Asset import (copy file + register)
3. Scene serialization (save/load JSON)
4. Inspector "Add Component" + Sprite editor
5. Texture picker popup
6. Drag-drop from Project panel to Scene Tree

## Scope (v1)

**Included:**
- Basic sprite: texture reference, size, color tint, flip x/y
- PNG and JPEG image formats
- Single scene file serialization

**Deferred:**
- Pivot/anchor points
- Sprite sheets / UV regions
- Animation support
- Metadata sidecar files
