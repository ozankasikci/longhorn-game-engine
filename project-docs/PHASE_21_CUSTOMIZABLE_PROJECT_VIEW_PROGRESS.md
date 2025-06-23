# Phase 21: Customizable Project View Progress

## Overview
Implementing a professional customizable project folder structure with file system integration, drag & drop support, and flexible organization.

## Progress Tracking

### Phase 21.1: File System Integration
- [ ] Create file system watcher for project directory
- [ ] Map project folder structure to in-memory representation
- [ ] Handle file system events (create, delete, move, rename)
- [ ] Implement asset path resolution
- [ ] Add project root directory configuration

**Status**: Not Started
**Blockers**: None
**Notes**: Will use `notify` crate for cross-platform file watching

### Phase 21.2: Data Model
- [ ] Design flexible folder/asset tree structure
- [ ] Implement folder metadata (creation date, size, item count)
- [ ] Add asset metadata (type, size, import settings, guid)
- [ ] Create asset database for quick lookups
- [ ] Implement asset dependency tracking

**Status**: Not Started
**Blockers**: None
**Notes**: Need to replace current simple ProjectAsset structure

### Phase 21.3: UI Components
- [ ] Two-column layout with resizable splitter
- [ ] Folder tree view with expand/collapse
- [ ] Asset grid/list view with sorting
- [ ] Context menus for folders and assets
- [ ] Breadcrumb navigation bar
- [ ] Search bar with filters
- [ ] Asset preview panel

**Status**: Not Started
**Blockers**: None
**Notes**: Will build on existing egui components

### Phase 21.4: Drag & Drop
- [ ] Implement drag source for assets
- [ ] Implement drop targets for folders
- [ ] Visual feedback during drag
- [ ] Multi-selection support
- [ ] External file drop support

**Status**: Not Started
**Blockers**: Need Phase 21.3 UI components first
**Notes**: Egui has drag & drop support we can leverage

### Phase 21.5: Asset Operations
- [ ] Create folder functionality
- [ ] Rename assets/folders
- [ ] Delete with confirmation
- [ ] Move/copy operations
- [ ] Duplicate assets
- [ ] Show in file explorer

**Status**: Not Started
**Blockers**: Need Phase 21.1 file system integration
**Notes**: Must handle file system operations safely

### Phase 21.6: Search & Filtering
- [ ] Text search by name
- [ ] Filter by asset type
- [ ] Filter by labels/tags
- [ ] Recent assets view
- [ ] Save search queries

**Status**: Not Started
**Blockers**: Need Phase 21.2 data model
**Notes**: Consider fuzzy search for better UX

### Phase 21.7: Asset Preview
- [ ] Thumbnail generation for images
- [ ] 3D model preview
- [ ] Audio waveform preview
- [ ] Text file preview
- [ ] Asset info panel

**Status**: Not Started
**Blockers**: Need Phase 21.3 UI components
**Notes**: Can reuse existing texture loading code

### Phase 21.8: Persistence & Settings
- [ ] Save folder expansion state
- [ ] Remember view preferences
- [ ] Store custom folder colors/icons
- [ ] Project-specific settings
- [ ] Import settings per folder

**Status**: Not Started
**Blockers**: None
**Notes**: Store in project metadata file

## Implementation Log

### Date: [Phase Start Date]
- Created Phase 21 plan and progress documents
- Analyzed current project view limitations
- Researched industry-standard project window implementation

## Current Issues
1. Current ProjectAsset structure is too simple - only has name and children
2. No file system integration - assets only exist in memory
3. Predefined folder structure is too restrictive
4. No drag & drop support
5. No asset preview or metadata

## Next Steps
1. Create new ProjectItem enum with Folder/Asset variants
2. Implement basic file system watcher
3. Build folder tree from project directory
4. Create two-column UI layout
5. Add basic folder navigation

## Code Changes
- Need to update `engine-editor-assets/src/types.rs` with new structures
- Create new `engine-editor-egui/src/project/` module
- Add `notify` and `chrono` dependencies
- Create `ProjectPanelV2` to replace current `ProjectPanel`

## Testing Plan
1. Unit tests for file system operations
2. Integration tests for folder tree building
3. UI tests for drag & drop interactions
4. Performance tests with large folder structures
5. Cross-platform testing (Windows/macOS/Linux)

## Risk Assessment
- **File System Complexity**: Different OS behaviors need careful handling
- **Performance**: Large projects might need virtualization
- **Backwards Compatibility**: Need migration path from old system
- **Data Loss**: File operations must be safe with undo support

## Dependencies Added
- [ ] `notify = "6.0"` - File system watching
- [ ] `chrono = "0.4"` - Date/time handling (if not already present)
- [ ] Existing: `uuid`, `rfd`

## Migration Notes
- Keep both old and new project panels initially
- Add feature flag in settings
- Provide data migration tool
- Document breaking changes
- Phase out old system gradually

## Performance Targets
- Load 10,000 assets in < 1 second
- Smooth scrolling with 1000+ visible items 
- Instant search results
- < 100ms response to file system changes
- < 16ms frame time during drag operations

## Success Metrics
- [ ] Users can create custom folder structures
- [ ] File system changes reflect immediately
- [ ] Drag & drop works smoothly
- [ ] Search finds assets quickly
- [ ] No data loss during operations
- [ ] Performance meets targets

## References
- standard project Window: https://industry references/Manual/ProjectView.html
- Phase 20 Asset Import System (for integration)
- Current ProjectPanel implementation
- egui drag & drop examples