# Phase 21: Implementation Guide - Customizable Project View

## Quick Start Implementation

### Step 1: Update Project Asset Structure

First, we need to replace the current limited `ProjectAsset` structure with a more flexible one:

```rust
// In engine-editor-assets/src/types.rs

use std::path::PathBuf;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectItem {
  Folder(ProjectFolder),
  Asset(ProjectAsset),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectFolder {
  pub name: String,
  pub path: PathBuf,
  pub children: Vec<ProjectItem>,
  pub expanded: bool,
  pub created_at: DateTime<Utc>,
  pub modified_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectAsset {
  pub name: String,
  pub path: PathBuf,
  pub asset_type: AssetType,
  pub size_bytes: u64,
  pub guid: Uuid,
  pub import_time: DateTime<Utc>,
  pub meta_path: Option<PathBuf>, // .meta file for import settings
}
```

### Step 2: Create File System Watcher

```rust
// In engine-editor-egui/src/project/file_system.rs

use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use std::path::PathBuf;

pub struct ProjectFileSystem {
  root_path: PathBuf,
  watcher: notify::RecommendedWatcher,
  event_receiver: Receiver<DebouncedEvent>,
}

impl ProjectFileSystem {
  pub fn new(root_path: PathBuf) -> Result<Self> {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_millis(500))?;
    
    // Watch the entire project directory
    watcher.watch(&root_path, RecursiveMode::Recursive)?;
    
    Ok(Self {
      root_path,
      watcher,
      event_receiver: rx,
    })
  }
  
  pub fn scan_directory(&self) -> ProjectFolder {
    self.scan_directory_recursive(&self.root_path)
  }
  
  fn scan_directory_recursive(&self, path: &Path) -> ProjectFolder {
    let mut folder = ProjectFolder {
      name: path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string(),
      path: path.to_path_buf(),
      children: Vec::new(),
      expanded: false,
      created_at: Utc::now(),
      modified_at: Utc::now(),
    };
    
    if let Ok(entries) = fs::read_dir(path) {
      for entry in entries.flatten() {
        let path = entry.path();
        
        if path.is_dir() {
          // Skip hidden folders and common ignore patterns
          if !should_ignore_folder(&path) {
            folder.children.push(
              ProjectItem::Folder(self.scan_directory_recursive(&path))
            );
          }
        } else if let Some(asset) = self.create_asset_from_path(&path) {
          folder.children.push(ProjectItem::Asset(asset));
        }
      }
    }
    
    // Sort children: folders first, then files
    folder.children.sort_by(|a, b| {
      match (a, b) {
        (ProjectItem::Folder(_), ProjectItem::Asset(_)) => Ordering::Less,
        (ProjectItem::Asset(_), ProjectItem::Folder(_)) => Ordering::Greater,
        _ => a.name().cmp(b.name()),
      }
    });
    
    folder
  }
}
```

### Step 3: Create New Project Panel UI

```rust
// In engine-editor-panels/src/project_v2.rs

pub struct ProjectPanelV2 {
  selected_item: Option<PathBuf>,
  current_folder: PathBuf,
  view_mode: ViewMode,
  search_query: String,
  show_create_menu: bool,
  drag_drop_state: DragDropState,
}

impl ProjectPanelV2 {
  pub fn show(&mut self, ui: &mut egui::Ui, project_root: &ProjectFolder) {
    // Top toolbar
    ui.horizontal(|ui| {
      // Breadcrumb navigation
      self.show_breadcrumbs(ui);
      
      ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        // View mode toggle
        if ui.button("🔲").clicked() {
          self.view_mode = ViewMode::Grid;
        }
        if ui.button("☰").clicked() {
          self.view_mode = ViewMode::List;
        }
        
        // Search bar
        ui.add(egui::TextEdit::singleline(&mut self.search_query)
          .hint_text("Search...")
          .desired_width(200.0));
      });
    });
    
    ui.separator();
    
    // Two-column layout
    egui::SidePanel::left("project_folders")
      .default_width(200.0)
      .resizable(true)
      .show_inside(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
          self.show_folder_tree(ui, project_root);
        });
      });
    
    egui::CentralPanel::default().show_inside(ui, |ui| {
      // Context menu
      if ui.input(|i| i.pointer.secondary_clicked()) {
        self.show_create_menu = true;
      }
      
      if self.show_create_menu {
        self.show_context_menu(ui);
      }
      
      // Asset grid/list
      egui::ScrollArea::vertical().show(ui, |ui| {
        match self.view_mode {
          ViewMode::Grid => self.show_asset_grid(ui, &current_folder_contents),
          ViewMode::List => self.show_asset_list(ui, &current_folder_contents),
        }
      });
    });
  }
  
  fn show_folder_tree(&mut self, ui: &mut egui::Ui, folder: &ProjectFolder) {
    let response = ui.collapsing(&folder.name, |ui| {
      for child in &folder.children {
        match child {
          ProjectItem::Folder(subfolder) => {
            self.show_folder_tree(ui, subfolder);
          }
          ProjectItem::Asset(_) => {
            // Don't show assets in folder tree
          }
        }
      }
    });
    
    // Handle folder selection
    if response.header_response.clicked() {
      self.current_folder = folder.path.clone();
    }
    
    // Handle drag & drop
    if response.header_response.drag_started() {
      self.drag_drop_state = DragDropState::Dragging(folder.path.clone());
    }
  }
}
```

### Step 4: Implement Drag & Drop

```rust
// Drag & drop handling
fn handle_drag_drop(&mut self, ui: &mut egui::Ui, target_folder: &Path) {
  let response = ui.interact(rect, id, egui::Sense::drag());
  
  // Visual feedback
  if ui.memory(|mem| mem.is_being_dragged(id)) {
    ui.painter().rect_filled(
      rect,
      0.0,
      egui::Color32::from_rgba_premultiplied(100, 100, 200, 50),
    );
  }
  
  // Drop target
  if response.hovered() && ui.input(|i| i.pointer.any_released()) {
    if let DragDropState::Dragging(source_path) = &self.drag_drop_state {
      // Move the asset/folder
      self.move_item(source_path, target_folder);
      self.drag_drop_state = DragDropState::None;
    }
  }
}
```

### Step 5: Context Menu Implementation

```rust
fn show_context_menu(&mut self, ui: &mut egui::Ui) {
  egui::Window::new("Create")
    .fixed_pos(ui.input(|i| i.pointer.hover_pos().unwrap_or_default()))
    .movable(false)
    .show(ui.ctx(), |ui| {
      if ui.button("📁 Create Folder").clicked() {
        self.create_folder_dialog = true;
        self.show_create_menu = false;
      }
      
      ui.separator();
      
      if ui.button("📄 Create Script").clicked() {
        self.create_asset(AssetType::Script);
        self.show_create_menu = false;
      }
      
      if ui.button("🎨 Create Material").clicked() {
        self.create_asset(AssetType::Material);
        self.show_create_menu = false;
      }
      
      ui.separator();
      
      if ui.button("Show in Explorer").clicked() {
        self.show_in_file_explorer();
        self.show_create_menu = false;
      }
    });
  
  // Close menu if clicked outside
  if ui.input(|i| i.pointer.any_click()) {
    self.show_create_menu = false;
  }
}
```

## Migration Strategy

1. **Keep Both Systems**: Initially keep both the old and new project panels
2. **Feature Flag**: Add a setting to switch between old and new project view
3. **Gradual Migration**: Move features one by one
4. **Data Migration**: Convert existing project structure to new format
5. **Testing Period**: Let users test and provide feedback
6. **Final Switch**: Remove old system once new one is stable

## Next Steps

1. Start with Phase 21.1: File System Integration
2. Create basic file watcher
3. Build folder tree from disk
4. Display in simple tree view
5. Add folder selection
6. Iterate and add features incrementally

This approach allows us to build the new project view incrementally while keeping the existing functionality intact.