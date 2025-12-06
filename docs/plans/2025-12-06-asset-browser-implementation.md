# Asset Browser Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a two-pane asset browser panel for browsing and managing the `assets/` directory.

**Architecture:** State/Panel separation pattern (like ScriptEditorState + ScriptEditorPanel). AssetBrowserState handles filesystem caching and selection. Panel modules handle rendering. Integrates with existing egui-dock system.

**Tech Stack:** Rust, egui, egui-dock, std::fs

**Test Command:** `cargo test -p longhorn-editor --lib`

**Working Directory:** `/Users/ozan/Projects/longhorn-game-engine-v2/.worktrees/asset-browser`

---

## Task 1: Create AssetBrowserState with FileType enum

**Files:**
- Create: `crates/longhorn-editor/src/asset_browser_state.rs`
- Modify: `crates/longhorn-editor/src/lib.rs`

**Step 1: Write the failing test**

Add to `crates/longhorn-editor/src/asset_browser_state.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_type_from_extension() {
        assert_eq!(FileType::from_extension(Some("ts")), FileType::Script);
        assert_eq!(FileType::from_extension(Some("png")), FileType::Image);
        assert_eq!(FileType::from_extension(Some("jpg")), FileType::Image);
        assert_eq!(FileType::from_extension(Some("wav")), FileType::Audio);
        assert_eq!(FileType::from_extension(Some("xyz")), FileType::Unknown);
        assert_eq!(FileType::from_extension(None), FileType::Unknown);
    }

    #[test]
    fn test_asset_browser_state_new() {
        let state = AssetBrowserState::new();
        assert!(state.selected_folder.as_os_str().is_empty());
        assert!(state.expanded_folders.is_empty());
        assert!(state.selected_file.is_none());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p longhorn-editor --lib test_file_type`
Expected: FAIL with "cannot find type `FileType`"

**Step 3: Write minimal implementation**

Create `crates/longhorn-editor/src/asset_browser_state.rs`:

```rust
use std::collections::HashSet;
use std::path::PathBuf;

/// File type categorization for asset browser display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Script,
    Image,
    Audio,
    Scene,
    Unknown,
}

impl FileType {
    pub fn from_extension(ext: Option<&str>) -> Self {
        match ext {
            Some("ts") => FileType::Script,
            Some("png") | Some("jpg") | Some("jpeg") | Some("webp") => FileType::Image,
            Some("wav") | Some("mp3") | Some("ogg") => FileType::Audio,
            Some("scene.json") => FileType::Scene,
            _ => FileType::Unknown,
        }
    }
}

/// State for the asset browser panel
#[derive(Debug)]
pub struct AssetBrowserState {
    /// Currently selected folder (shown in grid view)
    pub selected_folder: PathBuf,
    /// Expanded folders in tree view
    pub expanded_folders: HashSet<PathBuf>,
    /// Currently selected file
    pub selected_file: Option<PathBuf>,
    /// Active rename operation
    pub renaming: Option<PathBuf>,
}

impl AssetBrowserState {
    pub fn new() -> Self {
        Self {
            selected_folder: PathBuf::new(),
            expanded_folders: HashSet::new(),
            selected_file: None,
            renaming: None,
        }
    }
}

impl Default for AssetBrowserState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_type_from_extension() {
        assert_eq!(FileType::from_extension(Some("ts")), FileType::Script);
        assert_eq!(FileType::from_extension(Some("png")), FileType::Image);
        assert_eq!(FileType::from_extension(Some("jpg")), FileType::Image);
        assert_eq!(FileType::from_extension(Some("wav")), FileType::Audio);
        assert_eq!(FileType::from_extension(Some("xyz")), FileType::Unknown);
        assert_eq!(FileType::from_extension(None), FileType::Unknown);
    }

    #[test]
    fn test_asset_browser_state_new() {
        let state = AssetBrowserState::new();
        assert!(state.selected_folder.as_os_str().is_empty());
        assert!(state.expanded_folders.is_empty());
        assert!(state.selected_file.is_none());
    }
}
```

**Step 4: Add module to lib.rs**

In `crates/longhorn-editor/src/lib.rs`, add after line 10:

```rust
mod asset_browser_state;
```

And add after line 24:

```rust
pub use asset_browser_state::*;
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p longhorn-editor --lib test_file_type`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/longhorn-editor/src/asset_browser_state.rs crates/longhorn-editor/src/lib.rs
git commit -m "feat(editor): add AssetBrowserState and FileType enum"
```

---

## Task 2: Add FileEntry and DirectoryNode structs

**Files:**
- Modify: `crates/longhorn-editor/src/asset_browser_state.rs`

**Step 1: Write the failing test**

Add to the tests module in `asset_browser_state.rs`:

```rust
    #[test]
    fn test_file_entry_creation() {
        let entry = FileEntry::new(
            PathBuf::from("assets/scripts/player.ts"),
            "player.ts".to_string(),
        );
        assert_eq!(entry.name, "player.ts");
        assert_eq!(entry.file_type, FileType::Script);
        assert_eq!(entry.extension, Some("ts".to_string()));
    }

    #[test]
    fn test_directory_node_creation() {
        let node = DirectoryNode::new(
            PathBuf::from("assets/scripts"),
            "scripts".to_string(),
        );
        assert_eq!(node.name, "scripts");
        assert!(node.children.is_empty());
        assert!(node.files.is_empty());
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p longhorn-editor --lib test_file_entry`
Expected: FAIL with "cannot find struct `FileEntry`"

**Step 3: Write minimal implementation**

Add to `asset_browser_state.rs` after the `AssetBrowserState` struct:

```rust
/// A file entry in the asset browser
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub file_type: FileType,
}

impl FileEntry {
    pub fn new(path: PathBuf, name: String) -> Self {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());
        let file_type = FileType::from_extension(extension.as_deref());
        Self {
            path,
            name,
            extension,
            file_type,
        }
    }
}

/// A directory node in the asset browser tree
#[derive(Debug, Clone)]
pub struct DirectoryNode {
    pub path: PathBuf,
    pub name: String,
    pub children: Vec<DirectoryNode>,
    pub files: Vec<FileEntry>,
}

impl DirectoryNode {
    pub fn new(path: PathBuf, name: String) -> Self {
        Self {
            path,
            name,
            children: Vec::new(),
            files: Vec::new(),
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p longhorn-editor --lib test_file_entry`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/longhorn-editor/src/asset_browser_state.rs
git commit -m "feat(editor): add FileEntry and DirectoryNode structs"
```

---

## Task 3: Add directory scanning functionality

**Files:**
- Modify: `crates/longhorn-editor/src/asset_browser_state.rs`

**Step 1: Write the failing test**

Add to tests module:

```rust
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_scan_directory() {
        // Create a temp directory structure
        let temp = tempdir().unwrap();
        let assets = temp.path().join("assets");
        fs::create_dir_all(assets.join("scripts")).unwrap();
        fs::write(assets.join("scripts/test.ts"), "// test").unwrap();
        fs::write(assets.join("image.png"), &[0u8; 10]).unwrap();

        let node = DirectoryNode::scan(&assets).unwrap();

        assert_eq!(node.name, "assets");
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].name, "scripts");
        assert_eq!(node.files.len(), 1);
        assert_eq!(node.files[0].name, "image.png");
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p longhorn-editor --lib test_scan_directory`
Expected: FAIL with "no function or associated item named `scan`"

**Step 3: Add tempfile to dev-dependencies**

In `crates/longhorn-editor/Cargo.toml`, add under `[dev-dependencies]`:

```toml
tempfile = "3"
```

**Step 4: Write minimal implementation**

Add to `DirectoryNode` impl:

```rust
    /// Scan a directory and build the tree structure
    pub fn scan(path: &std::path::Path) -> std::io::Result<Self> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let mut node = Self::new(path.to_path_buf(), name);

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let entry_name = entry
                .file_name()
                .to_str()
                .unwrap_or("")
                .to_string();

            // Skip hidden files
            if entry_name.starts_with('.') {
                continue;
            }

            if entry_path.is_dir() {
                if let Ok(child) = Self::scan(&entry_path) {
                    node.children.push(child);
                }
            } else {
                node.files.push(FileEntry::new(entry_path, entry_name));
            }
        }

        // Sort children and files alphabetically
        node.children.sort_by(|a, b| a.name.cmp(&b.name));
        node.files.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(node)
    }
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p longhorn-editor --lib test_scan_directory`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/longhorn-editor/Cargo.toml crates/longhorn-editor/src/asset_browser_state.rs
git commit -m "feat(editor): add directory scanning for asset browser"
```

---

## Task 4: Add AssetBrowser PanelType variant

**Files:**
- Modify: `crates/longhorn-editor/src/docking.rs`

**Step 1: Add AssetBrowser to PanelType enum**

In `crates/longhorn-editor/src/docking.rs`, modify `PanelType` enum (around line 8):

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelType {
    Hierarchy,
    Inspector,
    SceneView,
    GameView,
    Console,
    Project,
    ScriptEditor,
    AssetBrowser,  // Add this line
}
```

**Step 2: Add title match arm**

In the `title()` method (around line 27), add:

```rust
            PanelType::AssetBrowser => "Assets",
```

**Step 3: Verify it compiles**

Run: `cargo build -p longhorn-editor`
Expected: Builds successfully (may have warnings about non-exhaustive match - fix those in editor.rs)

**Step 4: Commit**

```bash
git add crates/longhorn-editor/src/docking.rs
git commit -m "feat(editor): add AssetBrowser panel type"
```

---

## Task 5: Create asset_browser panel module structure

**Files:**
- Create: `crates/longhorn-editor/src/panels/asset_browser/mod.rs`
- Create: `crates/longhorn-editor/src/panels/asset_browser/tree_view.rs`
- Create: `crates/longhorn-editor/src/panels/asset_browser/grid_view.rs`
- Modify: `crates/longhorn-editor/src/panels/mod.rs`

**Step 1: Create the module directory and files**

Create `crates/longhorn-editor/src/panels/asset_browser/mod.rs`:

```rust
mod tree_view;
mod grid_view;

pub use tree_view::*;
pub use grid_view::*;

use egui::Ui;
use crate::asset_browser_state::{AssetBrowserState, DirectoryNode};

/// Asset browser panel with tree and grid views
pub struct AssetBrowserPanel;

impl AssetBrowserPanel {
    pub fn new() -> Self {
        Self
    }

    /// Show the asset browser panel
    /// Returns an action if the user triggered one (e.g., open file)
    pub fn show(
        &mut self,
        ui: &mut Ui,
        state: &mut AssetBrowserState,
        root: Option<&DirectoryNode>,
    ) -> Option<AssetBrowserAction> {
        if root.is_none() {
            ui.centered_and_justified(|ui| {
                ui.label("No project loaded");
            });
            return None;
        }

        let root = root.unwrap();
        let mut action = None;

        // Two-pane layout: tree on left, grid on right
        ui.columns(2, |columns| {
            // Left pane: Tree view
            columns[0].push_id("asset_tree", |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if let Some(tree_action) = show_tree_view(ui, state, root) {
                        action = Some(tree_action);
                    }
                });
            });

            // Right pane: Grid view
            columns[1].push_id("asset_grid", |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if let Some(grid_action) = show_grid_view(ui, state, root) {
                        action = Some(grid_action);
                    }
                });
            });
        });

        action
    }
}

impl Default for AssetBrowserPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Actions that can be triggered from the asset browser
#[derive(Debug, Clone)]
pub enum AssetBrowserAction {
    OpenScript(std::path::PathBuf),
    OpenImage(std::path::PathBuf),
    OpenExternal(std::path::PathBuf),
}
```

**Step 2: Create tree_view.rs stub**

Create `crates/longhorn-editor/src/panels/asset_browser/tree_view.rs`:

```rust
use egui::Ui;
use crate::asset_browser_state::{AssetBrowserState, DirectoryNode};
use super::AssetBrowserAction;

/// Render the folder tree view
pub fn show_tree_view(
    ui: &mut Ui,
    state: &mut AssetBrowserState,
    root: &DirectoryNode,
) -> Option<AssetBrowserAction> {
    show_tree_node(ui, state, root, 0)
}

fn show_tree_node(
    ui: &mut Ui,
    state: &mut AssetBrowserState,
    node: &DirectoryNode,
    depth: usize,
) -> Option<AssetBrowserAction> {
    let mut action = None;

    let is_expanded = state.expanded_folders.contains(&node.path);
    let is_selected = state.selected_folder == node.path;

    // Indent based on depth
    ui.horizontal(|ui| {
        ui.add_space(depth as f32 * 16.0);

        // Expand/collapse button for folders with children
        let icon = if node.children.is_empty() {
            "  "
        } else if is_expanded {
            "v "
        } else {
            "> "
        };

        let folder_icon = "[D]";
        let label = format!("{}{} {}", icon, folder_icon, node.name);

        let response = ui.selectable_label(is_selected, label);

        if response.clicked() {
            state.selected_folder = node.path.clone();
            if !node.children.is_empty() {
                if is_expanded {
                    state.expanded_folders.remove(&node.path);
                } else {
                    state.expanded_folders.insert(node.path.clone());
                }
            }
        }
    });

    // Show children if expanded
    if is_expanded {
        for child in &node.children {
            if let Some(child_action) = show_tree_node(ui, state, child, depth + 1) {
                action = Some(child_action);
            }
        }
    }

    action
}
```

**Step 3: Create grid_view.rs stub**

Create `crates/longhorn-editor/src/panels/asset_browser/grid_view.rs`:

```rust
use egui::Ui;
use crate::asset_browser_state::{AssetBrowserState, DirectoryNode, FileType};
use super::AssetBrowserAction;

/// Render the grid view of the selected folder's contents
pub fn show_grid_view(
    ui: &mut Ui,
    state: &mut AssetBrowserState,
    root: &DirectoryNode,
) -> Option<AssetBrowserAction> {
    // Find the selected folder in the tree
    let folder = find_folder(root, &state.selected_folder).unwrap_or(root);

    let mut action = None;

    // Show breadcrumb path
    ui.horizontal(|ui| {
        ui.label(folder.path.display().to_string());
    });
    ui.separator();

    // Grid layout for files and subfolders
    let available_width = ui.available_width();
    let item_size = 80.0;
    let columns = ((available_width / item_size) as usize).max(1);

    egui::Grid::new("asset_grid_items")
        .num_columns(columns)
        .spacing([8.0, 8.0])
        .show(ui, |ui| {
            let mut col = 0;

            // Show subfolders first
            for child in &folder.children {
                if show_grid_item(ui, state, &child.path, &child.name, true) {
                    state.selected_folder = child.path.clone();
                    state.expanded_folders.insert(child.path.clone());
                }
                col += 1;
                if col >= columns {
                    ui.end_row();
                    col = 0;
                }
            }

            // Then show files
            for file in &folder.files {
                let is_selected = state.selected_file.as_ref() == Some(&file.path);
                if show_file_grid_item(ui, state, file, is_selected) {
                    // Double-click handling
                    action = Some(match file.file_type {
                        FileType::Script => AssetBrowserAction::OpenScript(file.path.clone()),
                        FileType::Image => AssetBrowserAction::OpenImage(file.path.clone()),
                        _ => AssetBrowserAction::OpenExternal(file.path.clone()),
                    });
                }
                col += 1;
                if col >= columns {
                    ui.end_row();
                    col = 0;
                }
            }
        });

    action
}

fn find_folder<'a>(root: &'a DirectoryNode, path: &std::path::Path) -> Option<&'a DirectoryNode> {
    if root.path == path {
        return Some(root);
    }
    for child in &root.children {
        if let Some(found) = find_folder(child, path) {
            return Some(found);
        }
    }
    None
}

fn show_grid_item(
    ui: &mut Ui,
    _state: &mut AssetBrowserState,
    _path: &std::path::Path,
    name: &str,
    is_folder: bool,
) -> bool {
    let icon = if is_folder { "[D]" } else { "[F]" };

    ui.vertical(|ui| {
        ui.set_width(72.0);
        ui.set_height(72.0);

        let response = ui.button(format!("{}\n{}", icon, truncate_name(name, 10)));
        response.double_clicked()
    }).inner
}

fn show_file_grid_item(
    ui: &mut Ui,
    state: &mut AssetBrowserState,
    file: &crate::asset_browser_state::FileEntry,
    is_selected: bool,
) -> bool {
    let icon = match file.file_type {
        FileType::Script => "[S]",
        FileType::Image => "[I]",
        FileType::Audio => "[A]",
        FileType::Scene => "[C]",
        FileType::Unknown => "[?]",
    };

    ui.vertical(|ui| {
        ui.set_width(72.0);
        ui.set_height(72.0);

        let text = format!("{}\n{}", icon, truncate_name(&file.name, 10));
        let response = ui.selectable_label(is_selected, text);

        if response.clicked() {
            state.selected_file = Some(file.path.clone());
        }

        response.double_clicked()
    }).inner
}

fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}...", &name[..max_len.saturating_sub(3)])
    }
}
```

**Step 4: Update panels/mod.rs**

Modify `crates/longhorn-editor/src/panels/mod.rs`:

```rust
mod scene_tree;
mod inspector;
mod viewport;
mod console;
mod script_editor;
mod asset_browser;

pub use scene_tree::*;
pub use inspector::*;
pub use viewport::*;
pub use console::*;
pub use script_editor::*;
pub use asset_browser::*;
```

**Step 5: Verify it compiles**

Run: `cargo build -p longhorn-editor`
Expected: Builds successfully

**Step 6: Commit**

```bash
git add crates/longhorn-editor/src/panels/
git commit -m "feat(editor): add asset browser panel module structure"
```

---

## Task 6: Integrate AssetBrowser into Editor

**Files:**
- Modify: `crates/longhorn-editor/src/editor.rs`

**Step 1: Add state and panel fields to Editor struct**

In `editor.rs`, add to the `Editor` struct (around line 12-26):

```rust
    asset_browser_state: AssetBrowserState,
    asset_browser_panel: AssetBrowserPanel,
    asset_tree: Option<DirectoryNode>,
```

**Step 2: Add imports at top of file**

Add to the imports (around line 6):

```rust
use crate::{AssetBrowserState, AssetBrowserPanel, AssetBrowserAction, DirectoryNode};
```

**Step 3: Initialize in Editor::new()**

Add to the `Self { ... }` block in `new()`:

```rust
            asset_browser_state: AssetBrowserState::new(),
            asset_browser_panel: AssetBrowserPanel::new(),
            asset_tree: None,
```

**Step 4: Add PanelType::AssetBrowser match arm in show_panel**

Find the `impl PanelRenderer for Editor` block and add to the match in `show_panel`:

```rust
            PanelType::AssetBrowser => {
                if let Some(action) = self.asset_browser_panel.show(
                    ui,
                    &mut self.asset_browser_state,
                    self.asset_tree.as_ref(),
                ) {
                    match action {
                        AssetBrowserAction::OpenScript(path) => {
                            if let Some(game_path) = &self.state.game_path {
                                let project_path = std::path::Path::new(game_path);
                                if let Ok(relative) = path.strip_prefix(project_path.join("assets")) {
                                    let script_path = std::path::PathBuf::from("assets").join(relative);
                                    if let Err(e) = self.script_editor_state.open(script_path, project_path) {
                                        log::error!("Failed to open script: {}", e);
                                    }
                                }
                            }
                        }
                        AssetBrowserAction::OpenImage(path) => {
                            log::info!("TODO: Open image preview: {:?}", path);
                        }
                        AssetBrowserAction::OpenExternal(path) => {
                            if let Err(e) = open::that(&path) {
                                log::error!("Failed to open external: {}", e);
                            }
                        }
                    }
                }
            }
```

**Step 5: Add method to refresh asset tree**

Add to `impl Editor`:

```rust
    /// Refresh the asset tree from disk
    pub fn refresh_asset_tree(&mut self) {
        if let Some(game_path) = &self.state.game_path {
            let assets_path = std::path::Path::new(game_path).join("assets");
            if assets_path.exists() {
                match DirectoryNode::scan(&assets_path) {
                    Ok(tree) => {
                        // Set selected folder to root if not set
                        if self.asset_browser_state.selected_folder.as_os_str().is_empty() {
                            self.asset_browser_state.selected_folder = tree.path.clone();
                        }
                        self.asset_tree = Some(tree);
                    }
                    Err(e) => {
                        log::error!("Failed to scan assets directory: {}", e);
                    }
                }
            }
        }
    }
```

**Step 6: Call refresh when game is loaded**

Find where `game_path` is set (likely in a load/open game function) and add:

```rust
self.refresh_asset_tree();
```

**Step 7: Verify it compiles**

Run: `cargo build -p longhorn-editor`
Expected: Builds successfully

**Step 8: Commit**

```bash
git add crates/longhorn-editor/src/editor.rs
git commit -m "feat(editor): integrate asset browser into editor"
```

---

## Task 7: Add AssetBrowser to default dock layout

**Files:**
- Modify: `crates/longhorn-editor/src/docking.rs`

**Step 1: Modify create_default_dock_state**

In `docking.rs`, update `create_default_dock_state()` to include AssetBrowser:

Change line 80 from:
```rust
        vec![PanelType::Console, PanelType::Project],
```
To:
```rust
        vec![PanelType::Console, PanelType::Project, PanelType::AssetBrowser],
```

**Step 2: Verify it compiles and runs**

Run: `cargo build -p editor && cargo run -p editor`
Expected: Editor launches with Assets tab in bottom panel

**Step 3: Commit**

```bash
git add crates/longhorn-editor/src/docking.rs
git commit -m "feat(editor): add AssetBrowser to default dock layout"
```

---

## Task 8: Add file operations (create folder, rename, delete)

**Files:**
- Create: `crates/longhorn-editor/src/panels/asset_browser/file_ops.rs`
- Modify: `crates/longhorn-editor/src/panels/asset_browser/mod.rs`

**Step 1: Write the failing test**

Create `crates/longhorn-editor/src/panels/asset_browser/file_ops.rs`:

```rust
use std::fs;
use std::io;
use std::path::Path;

/// Create a new folder
pub fn create_folder(parent: &Path, name: &str) -> io::Result<()> {
    let path = parent.join(name);
    fs::create_dir(&path)
}

/// Rename a file or folder
pub fn rename(path: &Path, new_name: &str) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "No parent directory")
    })?;
    let new_path = parent.join(new_name);
    fs::rename(path, new_path)
}

/// Delete a file or folder
pub fn delete(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_folder() {
        let temp = tempdir().unwrap();
        create_folder(temp.path(), "new_folder").unwrap();
        assert!(temp.path().join("new_folder").exists());
    }

    #[test]
    fn test_rename() {
        let temp = tempdir().unwrap();
        let original = temp.path().join("original.txt");
        fs::write(&original, "test").unwrap();

        rename(&original, "renamed.txt").unwrap();

        assert!(!original.exists());
        assert!(temp.path().join("renamed.txt").exists());
    }

    #[test]
    fn test_delete_file() {
        let temp = tempdir().unwrap();
        let file = temp.path().join("to_delete.txt");
        fs::write(&file, "test").unwrap();

        delete(&file).unwrap();
        assert!(!file.exists());
    }

    #[test]
    fn test_delete_folder() {
        let temp = tempdir().unwrap();
        let folder = temp.path().join("to_delete");
        fs::create_dir(&folder).unwrap();
        fs::write(folder.join("file.txt"), "test").unwrap();

        delete(&folder).unwrap();
        assert!(!folder.exists());
    }
}
```

**Step 2: Update mod.rs**

In `crates/longhorn-editor/src/panels/asset_browser/mod.rs`, add:

```rust
mod file_ops;
pub use file_ops::*;
```

**Step 3: Run tests**

Run: `cargo test -p longhorn-editor --lib test_create_folder test_rename test_delete`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/longhorn-editor/src/panels/asset_browser/
git commit -m "feat(editor): add file operations for asset browser"
```

---

## Task 9: Add context menu to grid view

**Files:**
- Modify: `crates/longhorn-editor/src/panels/asset_browser/grid_view.rs`
- Modify: `crates/longhorn-editor/src/panels/asset_browser/mod.rs`

**Step 1: Add ContextAction enum to mod.rs**

In `mod.rs`, add:

```rust
/// Context menu actions
#[derive(Debug, Clone)]
pub enum ContextAction {
    CreateFolder,
    Rename(std::path::PathBuf),
    Delete(std::path::PathBuf),
    Refresh,
}
```

**Step 2: Update AssetBrowserAction to include context actions**

Modify `AssetBrowserAction`:

```rust
#[derive(Debug, Clone)]
pub enum AssetBrowserAction {
    OpenScript(std::path::PathBuf),
    OpenImage(std::path::PathBuf),
    OpenExternal(std::path::PathBuf),
    Context(ContextAction),
}
```

**Step 3: Add context menu to grid_view.rs**

Update `show_file_grid_item` to show context menu on right-click:

```rust
fn show_file_grid_item(
    ui: &mut Ui,
    state: &mut AssetBrowserState,
    file: &crate::asset_browser_state::FileEntry,
    is_selected: bool,
) -> Option<AssetBrowserAction> {
    let icon = match file.file_type {
        FileType::Script => "[S]",
        FileType::Image => "[I]",
        FileType::Audio => "[A]",
        FileType::Scene => "[C]",
        FileType::Unknown => "[?]",
    };

    let mut action = None;

    ui.vertical(|ui| {
        ui.set_width(72.0);
        ui.set_height(72.0);

        let text = format!("{}\n{}", icon, truncate_name(&file.name, 10));
        let response = ui.selectable_label(is_selected, text);

        if response.clicked() {
            state.selected_file = Some(file.path.clone());
        }

        if response.double_clicked() {
            action = Some(match file.file_type {
                FileType::Script => AssetBrowserAction::OpenScript(file.path.clone()),
                FileType::Image => AssetBrowserAction::OpenImage(file.path.clone()),
                _ => AssetBrowserAction::OpenExternal(file.path.clone()),
            });
        }

        // Context menu
        response.context_menu(|ui| {
            if ui.button("Rename").clicked() {
                action = Some(AssetBrowserAction::Context(
                    super::ContextAction::Rename(file.path.clone())
                ));
                ui.close_menu();
            }
            if ui.button("Delete").clicked() {
                action = Some(AssetBrowserAction::Context(
                    super::ContextAction::Delete(file.path.clone())
                ));
                ui.close_menu();
            }
        });
    });

    action
}
```

**Step 4: Update show_grid_view to return actions**

Update the function signature and body to collect and return actions.

**Step 5: Verify it compiles**

Run: `cargo build -p longhorn-editor`
Expected: Builds successfully

**Step 6: Commit**

```bash
git add crates/longhorn-editor/src/panels/asset_browser/
git commit -m "feat(editor): add context menu to asset browser"
```

---

## Task 10: Handle context actions in Editor

**Files:**
- Modify: `crates/longhorn-editor/src/editor.rs`

**Step 1: Update the AssetBrowser match arm**

In the `PanelType::AssetBrowser` match arm, add handling for context actions:

```rust
                        AssetBrowserAction::Context(ctx_action) => {
                            match ctx_action {
                                ContextAction::CreateFolder => {
                                    // TODO: Show dialog for folder name
                                }
                                ContextAction::Rename(path) => {
                                    self.asset_browser_state.renaming = Some(path);
                                }
                                ContextAction::Delete(path) => {
                                    if let Err(e) = crate::panels::asset_browser::delete(&path) {
                                        log::error!("Failed to delete: {}", e);
                                    }
                                    self.refresh_asset_tree();
                                }
                                ContextAction::Refresh => {
                                    self.refresh_asset_tree();
                                }
                            }
                        }
```

**Step 2: Add import for ContextAction**

Add to imports:

```rust
use crate::ContextAction;
```

**Step 3: Verify it compiles**

Run: `cargo build -p longhorn-editor`
Expected: Builds successfully

**Step 4: Commit**

```bash
git add crates/longhorn-editor/src/editor.rs
git commit -m "feat(editor): handle asset browser context actions"
```

---

## Task 11: Final integration test

**Step 1: Run full test suite**

Run: `cargo test --workspace --exclude longhorn-scripting`
Expected: All tests pass

**Step 2: Manual testing**

Run: `cargo run -p editor`

Test checklist:
- [ ] Asset browser panel appears in dock
- [ ] Tree view shows folder structure
- [ ] Clicking folder in tree updates grid view
- [ ] Double-clicking .ts file opens script editor
- [ ] Right-click shows context menu
- [ ] Delete removes file and refreshes

**Step 3: Final commit**

```bash
git add .
git commit -m "feat(editor): complete asset browser implementation"
```

---

## Summary

This plan implements the asset browser in 11 tasks:

1. **AssetBrowserState** - Core state with FileType enum
2. **FileEntry/DirectoryNode** - Data structures for file tree
3. **Directory scanning** - Filesystem reading
4. **PanelType variant** - Dock integration
5. **Panel module structure** - Tree/grid view files
6. **Editor integration** - Wire up state and panel
7. **Default layout** - Add to dock tabs
8. **File operations** - Create, rename, delete
9. **Context menu** - Right-click actions
10. **Action handling** - Process user actions
11. **Final testing** - Verify everything works
