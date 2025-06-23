# Phase 21: Customizable Project View Plan

## Overview
Implement a professional customizable project folder structure that allows users to organize their assets freely, create custom folders, move assets between folders, and have a file-system based project structure.

## Motivation
The current project view has predefined folders (Scripts, Materials, Textures, Models, Audio) which is too restrictive. Users need the flexibility to organize their project assets according to their needs, similar to industry-standard Project window.

## standard project View Reference Features
1. **File System Based**: Mirrors actual file system structure
2. **Custom Folders**: Users can create any folder structure they want
3. **Drag & Drop**: Move assets between folders via drag & drop
4. **Context Menus**: Right-click to create folders, rename, delete
5. **Search & Filter**: Search bar and type filters
6. **Two-Column Layout**: Folder tree on left, folder contents on right
7. **Thumbnails/List View**: Toggle between thumbnail and list view
8. **Asset Preview**: Preview selected assets
9. **Breadcrumb Navigation**: Shows current folder path
10. **Favorites**: Pin frequently used folders

## Technical Requirements

### Phase 21.1: File System Integration
- [ ] Create file system watcher for project directory
- [ ] Map project folder structure to in-memory representation
- [ ] Handle file system events (create, delete, move, rename)
- [ ] Implement asset path resolution
- [ ] Add project root directory configuration

### Phase 21.2: Data Model
- [ ] Design flexible folder/asset tree structure
- [ ] Implement folder metadata (creation date, size, item count)
- [ ] Add asset metadata (type, size, import settings, guid)
- [ ] Create asset database for quick lookups
- [ ] Implement asset dependency tracking

### Phase 21.3: UI Components
- [ ] Two-column layout with resizable splitter
- [ ] Folder tree view with expand/collapse
- [ ] Asset grid/list view with sorting
- [ ] Context menus for folders and assets
- [ ] Breadcrumb navigation bar
- [ ] Search bar with filters
- [ ] Asset preview panel

### Phase 21.4: Drag & Drop
- [ ] Implement drag source for assets
- [ ] Implement drop targets for folders
- [ ] Visual feedback during drag
- [ ] Multi-selection support
- [ ] External file drop support

### Phase 21.5: Asset Operations
- [ ] Create folder functionality
- [ ] Rename assets/folders
- [ ] Delete with confirmation
- [ ] Move/copy operations
- [ ] Duplicate assets
- [ ] Show in file explorer

### Phase 21.6: Search & Filtering
- [ ] Text search by name
- [ ] Filter by asset type
- [ ] Filter by labels/tags
- [ ] Recent assets view
- [ ] Save search queries

### Phase 21.7: Asset Preview
- [ ] Thumbnail generation for images
- [ ] 3D model preview
- [ ] Audio waveform preview
- [ ] Text file preview
- [ ] Asset info panel

### Phase 21.8: Persistence & Settings
- [ ] Save folder expansion state
- [ ] Remember view preferences
- [ ] Store custom folder colors/icons
- [ ] Project-specific settings
- [ ] Import settings per folder

## Implementation Plan

### Week 1: File System Foundation
1. Create `ProjectFileSystem` struct to watch project directory
2. Implement file system event handling
3. Build in-memory folder tree from disk
4. Add asset discovery and type detection

### Week 2: Data Model & Database
1. Design `ProjectFolder` and `ProjectAsset` structures
2. Implement `AssetDatabase` with indexing
3. Add GUID system for assets
4. Create asset metadata storage

### Week 3: Basic UI
1. Implement two-column layout
2. Create folder tree view
3. Add asset list/grid view
4. Implement selection system

### Week 4: Interactions
1. Add context menus
2. Implement drag & drop
3. Add folder/asset operations
4. Create keyboard shortcuts

### Week 5: Advanced Features
1. Implement search and filtering
2. Add asset preview system
3. Create thumbnail generation
4. Add breadcrumb navigation

### Week 6: Polish & Integration
1. Add animations and transitions
2. Implement undo/redo
3. Add error handling
4. Performance optimization
5. Integration testing

## Key Components

### ProjectFileSystem
```rust
pub struct ProjectFileSystem {
  root_path: PathBuf,
  watcher: FileWatcher,
  asset_database: AssetDatabase,
}
```

### ProjectFolder
```rust
pub struct ProjectFolder {
  path: PathBuf,
  name: String,
  children: Vec<ProjectItem>,
  expanded: bool,
  metadata: FolderMetadata,
}
```

### ProjectAsset
```rust
pub struct ProjectAsset {
  path: PathBuf,
  name: String,
  asset_type: AssetType,
  guid: Guid,
  metadata: AssetMetadata,
  thumbnail: Option<TextureHandle>,
}
```

### AssetDatabase
```rust
pub struct AssetDatabase {
  assets: HashMap<Guid, ProjectAsset>,
  path_index: HashMap<PathBuf, Guid>,
  type_index: HashMap<AssetType, Vec<Guid>>,
}
```

## Success Criteria
1. Users can create any folder structure they want
2. Assets automatically appear when added to project folder
3. Drag & drop works smoothly for reorganizing
4. Search and filtering help find assets quickly
5. Performance remains good with thousands of assets
6. File system and editor stay in sync

## Testing Strategy
1. Unit tests for file system operations
2. Integration tests for asset database
3. UI tests for drag & drop
4. Performance tests with large projects
5. Cross-platform file system tests

## Dependencies
- `notify` crate for file watching
- `uuid` crate for GUIDs 
- `image` crate for thumbnails
- `rfd` for native dialogs
- Existing asset import system

## Risks & Mitigations
1. **File System Differences**: Test on Windows, macOS, Linux
2. **Performance with Large Projects**: Implement virtualization and lazy loading
3. **File Conflicts**: Add proper locking and conflict resolution
4. **Data Loss**: Implement trash/recycle bin integration

## Future Enhancements
1. Version control integration (git status indicators)
2. Cloud storage support
3. Asset packages/bundles
4. Smart folders with rules
5. Asset tagging system
6. Collaborative features

## References
- standard project Window: https://industry references/Manual/ProjectView.html
- Unreal Content Browser: https://docs.unrealengine.com/5.0/en-US/content-browser-interface-in-unreal-engine/
- Godot FileSystem Dock: https://docs.godotengine.org/en/stable/tutorials/editor/project_manager.html