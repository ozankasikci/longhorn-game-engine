# Asset Browser Design

## Overview

A two-pane file browser for the `assets/` directory, integrated into the editor's docking system. Provides browsing, basic file operations, and opens files in appropriate editors.

## Architecture

### File Structure

```
crates/longhorn-editor/src/panels/asset_browser/
├── mod.rs           # AssetBrowserPanel - main panel, coordinates subcomponents
├── state.rs         # AssetBrowserState - selected folder, expanded nodes, etc.
├── tree_view.rs     # Folder tree (left pane)
├── grid_view.rs     # Icon grid (right pane)
├── file_ops.rs      # Create folder, rename, delete operations
└── preview.rs       # Image preview panel (separate dockable tab)
```

### Data Flow

```
Filesystem (assets/)
       ↓
  AssetBrowserState (caches directory structure, tracks selection)
       ↓
  ┌────┴────┐
  ↓         ↓
TreeView  GridView
  │         │
  └────┬────┘
       ↓
  User Actions (click, double-click, context menu)
       ↓
  file_ops.rs (filesystem mutations)
       ↓
  Refresh AssetBrowserState
```

## State Management

```rust
// state.rs
pub struct AssetBrowserState {
    /// Root path to assets directory
    assets_root: PathBuf,

    /// Cached directory tree (rebuilt on filesystem changes)
    directory_tree: DirectoryNode,

    /// Currently selected folder (shown in grid view)
    selected_folder: PathBuf,

    /// Expanded folders in tree view
    expanded_folders: HashSet<PathBuf>,

    /// Currently selected file (for context menu operations)
    selected_file: Option<PathBuf>,

    /// Active rename operation (inline editing)
    renaming: Option<PathBuf>,
}

pub struct DirectoryNode {
    pub path: PathBuf,
    pub name: String,
    pub children: Vec<DirectoryNode>,  // folders only
    pub files: Vec<FileEntry>,
}

pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub file_type: FileType,
}

pub enum FileType {
    Script,      // .ts
    Image,       // .png, .jpg, .webp
    Audio,       // .wav, .mp3, .ogg
    Scene,       // .scene.json
    Unknown,
}
```

### Refresh Strategy

- **Initial load:** Scan `assets/` recursively on editor startup
- **After file ops:** Re-scan affected directory only
- **Manual refresh:** Button in toolbar or F5 hotkey

## UI Components

### Tree View (left pane)

```rust
// tree_view.rs
pub fn show_tree_view(
    ui: &mut egui::Ui,
    state: &mut AssetBrowserState,
) -> TreeViewResponse {
    // Renders folder hierarchy with expand/collapse
    // - Click folder: select it (updates grid view)
    // - Right-click: context menu
    // - Folders only, no files in tree
}

pub struct TreeViewResponse {
    pub folder_selected: Option<PathBuf>,
    pub context_menu_action: Option<ContextAction>,
}
```

**Visual:** Indented rows with `▶`/`▼` expand icons, folder icon, folder name

### Grid View (right pane)

```rust
// grid_view.rs
pub fn show_grid_view(
    ui: &mut egui::Ui,
    state: &mut AssetBrowserState,
    folder: &DirectoryNode,
) -> GridViewResponse {
    // Renders files + subfolders as icon grid
    // - Single click: select
    // - Double-click folder: navigate into it
    // - Double-click file: open action
    // - Right-click: context menu
}

pub struct GridViewResponse {
    pub navigate_to: Option<PathBuf>,
    pub open_file: Option<PathBuf>,
    pub context_menu_action: Option<ContextAction>,
}
```

**Visual:** 64x64 icons in a wrapped grid
- Folders: folder icon
- Scripts: code/document icon
- Images: placeholder icon (thumbnails deferred to later version)
- Audio: speaker icon
- Unknown: generic file icon

Filename displayed below each icon (truncated if too long).

## File Operations

### Context Menu Actions

```rust
// file_ops.rs
pub enum ContextAction {
    CreateFolder,
    Rename,
    Delete,
    OpenExternal,  // "Show in Finder/Explorer"
}

pub fn create_folder(parent: &Path, name: &str) -> io::Result<PathBuf>;
pub fn rename(path: &Path, new_name: &str) -> io::Result<PathBuf>;
pub fn delete(path: &Path) -> io::Result<()>;  // moves to trash if possible
pub fn open_external(path: &Path) -> io::Result<()>;
```

### Double-Click Handlers

| File Type | Action |
|-----------|--------|
| `.ts` | Open in script editor panel (`ScriptEditorState::open_file`) |
| `.png/.jpg/.webp` | Open preview panel (new `ImagePreviewPanel`) |
| `.scene.json` | Load scene (existing scene loading logic) |
| Folder | Navigate into it (update `selected_folder`) |
| Unknown | `open::that(path)` - system default app |

### Delete Confirmation

Show modal dialog: "Delete [filename]? This cannot be undone."
- Uses `rfd` crate for native dialogs (already in use for file picker)

## Integration Points

### Editor Integration

**Docking system** (`docking.rs`):
```rust
pub enum PanelType {
    // ... existing panels
    AssetBrowser,
    ImagePreview,  // new panel for image viewing
}
```

**Default layout:** Asset browser docked as tab right of Game view

**State ownership:** `AssetBrowserState` stored in `EditorState`, passed to panel on render

### Script Editor Integration

When double-clicking a `.ts` file:
```rust
// Reuse existing script editor infrastructure
script_editor_state.open_file(&path);
```

This connects the asset browser to the already-working script editor.

### Future: Drag & Drop to Inspector

Not in v1, but the design supports it:
- Dragging an image from grid view onto a Sprite component's texture field
- Requires asset reference system (AssetId from path)

## Out of Scope (v1)

Explicitly **not** included in this version:

- **Image thumbnails** - Grid uses placeholder icons; real thumbnails require texture loading in UI thread
- **Drag & drop** - No dragging files onto scene/inspector yet
- **File watching** - No auto-refresh when external tools modify files; manual refresh only
- **Search/filter** - No search box to filter files by name
- **Multi-select** - Single selection only for now
- **Copy/paste/move** - Only create, rename, delete

These can be added incrementally after the foundation works.

## Summary

**What we're building:**
1. Two-pane asset browser (tree + grid) for `assets/` folder
2. Basic file operations (create folder, rename, delete)
3. Double-click opens files appropriately (scripts → editor, images → preview)
4. Dockable panel integrated with existing egui-dock system
5. Modular code structure (`panels/asset_browser/` submodule)
