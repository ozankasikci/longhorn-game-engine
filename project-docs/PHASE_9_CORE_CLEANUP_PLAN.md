# Phase 9: Core Folder Cleanup and Reorganization - Implementation Plan

## Phase Overview
Comprehensive cleanup and reorganization of the core folder structure to eliminate code duplication, improve separation of concerns, and establish proper 4-tier architecture boundaries.

## Current Issues Identified

### Major Problems
1. **engine-ecs-core is severely bloated** - contains ECS, math, components, Transform, GameObject, and EditorState (belongs in application tier)
2. **Duplicate Camera Systems** - 3 different camera implementations across engine-camera-core, engine-scene-core, and engine-ecs-core
3. **Unused Core Crates** - engine-audio-core and engine-physics-core aren't used anywhere but consume workspace space
4. **Math utilities scattered** - only basic glam re-exports in ecs-core instead of dedicated math crate
5. **Poor separation** - EditorState and GameObject are application-specific but sitting in core tier

### Current Problematic Structure
```
crates/core/
├── engine-audio-core/     # UNUSED - No references found
├── engine-camera-core/     # Camera system #1
├── engine-ecs-core/      # BLOATED - ECS + math + components + editor state
├── engine-geometry-core/    # OK - Pure geometric data
├── engine-materials-core/   # OK - Material definitions
├── engine-physics-core/    # UNUSED - No references found
├── engine-renderer-core/    # MINIMAL - Mostly empty traits
└── engine-scene-core/     # Camera system #2 + scene hierarchy
```

## Implementation Plan

### Phase 9.1: Extract & Organize (1-2 hours)
**Objective**: Remove application-specific code from core and organize math utilities

#### Task 9.1.1: Create engine-math-core (30 minutes)
- Create new `crates/core/engine-math-core/` directory
- Move math utilities from `engine-ecs-core/src/math.rs`
- Centralize glam re-exports and mathematical functions
- Add workspace dependency configuration

#### Task 9.1.2: Move EditorState to Application Tier (45 minutes)
- Create `crates/application/engine-editor-egui/src/editor_state.rs`
- Move EditorState, GameObject, ConsoleMessage from `engine-ecs-core/src/lib.rs`
- Update imports in main.rs
- Remove editor-specific code from core

#### Task 9.1.3: Rename and Clean engine-ecs-core (30 minutes)
- Rename `engine-ecs-core` → `engine-entity-core`
- Remove non-ECS code: math.rs, GameObject, EditorState
- Keep only: ecs.rs, ecs_v2.rs, memory.rs, time.rs, basic Transform
- Update all workspace dependencies and imports

### Phase 9.2: Consolidate Camera Systems (2-3 hours)
**Objective**: Merge 3 camera systems into single unified camera-core

#### Task 9.2.1: Analyze Camera Implementations (30 minutes)
- Document camera features in each system:
 - engine-camera-core: Viewport, projection, culling, optimization
 - engine-scene-core: Basic camera component
 - engine-ecs-core: Camera, Camera2D components
- Identify best features from each

#### Task 9.2.2: Merge Camera Systems (90 minutes)
- Keep `engine-camera-core` as the single camera crate
- Move Camera, Camera2D components from engine-ecs-core
- Move camera.rs from engine-scene-core to engine-camera-core
- Create unified Camera trait and component system
- Preserve advanced features: viewport, projection, culling

#### Task 9.2.3: Update Dependencies (30 minutes)
- Remove camera code from engine-scene-core and engine-entity-core
- Update all imports to use unified engine-camera-core
- Test camera functionality in editor

### Phase 9.3: Component Organization (1-2 hours)
**Objective**: Create centralized component system and standardize Transform

#### Task 9.3.1: Create engine-components-core (45 minutes)
- Create new `crates/core/engine-components-core/` directory
- Move components from `engine-entity-core/src/components.rs`
- Include: Mesh, Material, Name, Visibility, Sprite, SpriteRenderer, Canvas
- Standardize Transform implementation (single source of truth)

#### Task 9.3.2: Standardize Transform Component (30 minutes)
- Choose best Transform implementation from current duplicates
- Ensure compatibility with both ECS systems
- Update all Transform references across codebase

#### Task 9.3.3: Update Component Dependencies (15 minutes)
- Update engine-entity-core to depend on engine-components-core
- Update editor and other consumers to import from engine-components-core
- Remove component definitions from entity core

### Phase 9.4: Delete Unused Crates (30 minutes)
**Objective**: Remove dead code and clean workspace

#### Task 9.4.1: Remove Unused Audio and Physics Crates (15 minutes)
- Delete `crates/core/engine-audio-core/` directory
- Delete `crates/core/engine-physics-core/` directory
- These can be recreated later when actually needed

#### Task 9.4.2: Clean Workspace Configuration (15 minutes)
- Remove deleted crates from `Cargo.toml` workspace members
- Remove workspace dependency entries for deleted crates
- Verify no remaining references to deleted crates

## Expected Final Structure

### New Clean Core Structure
```
crates/core/
├── engine-math-core/     # Mathematical utilities (NEW)
├── engine-entity-core/    # Pure ECS systems (RENAMED/CLEANED)
├── engine-components-core/  # Standard game components (NEW)
├── engine-camera-core/    # Unified camera system (CONSOLIDATED)
├── engine-geometry-core/   # Pure geometric data (UNCHANGED)
├── engine-materials-core/   # Material definitions (UNCHANGED)
└── engine-scene-core/     # Scene hierarchy only (CLEANED)
```

### Moved to Application Tier
```
crates/application/engine-editor-egui/
└── src/
  ├── main.rs
  ├── editor_state.rs    # EditorState, GameObject, ConsoleMessage (MOVED)
  └── ...
```

## Success Criteria

### Phase 9.1 Success Indicators:
- ✅ engine-math-core exists with centralized math utilities
- ✅ EditorState moved to application tier, editor still functions
- ✅ engine-entity-core contains only ECS systems and core utilities
- ✅ No application-specific code remains in core tier

### Phase 9.2 Success Indicators:
- ✅ Single unified camera system in engine-camera-core
- ✅ No duplicate camera implementations across crates
- ✅ All camera features preserved and functional
- ✅ Editor camera navigation still works

### Phase 9.3 Success Indicators:
- ✅ engine-components-core contains all standard components
- ✅ Single standardized Transform implementation
- ✅ All component references updated and functional
- ✅ ECS systems work with new component organization

### Phase 9.4 Success Indicators:
- ✅ Unused crates completely removed from workspace
- ✅ No broken references to deleted crates
- ✅ Workspace builds successfully
- ✅ Editor application functions normally

## Architecture Benefits

### Improved Separation of Concerns
- **Core**: Pure domain logic and abstractions
- **Application**: Editor-specific state and functionality
- **Clean Dependencies**: No circular or inappropriate dependencies

### Eliminated Code Duplication
- **Single Camera System**: Unified camera with all features
- **Single Transform**: One standardized transform implementation
- **Single Math Library**: Centralized mathematical utilities

### Better Maintainability
- **Smaller Focused Crates**: Each crate has single responsibility
- **Clear Boundaries**: Obvious separation between core and application
- **Easier Testing**: Pure core logic can be tested independently

## Risk Assessment

### Technical Risks
1. **Import Breakage**: Moving code will break existing imports
  - Mitigation: Systematic import updates and testing after each phase
2. **Camera System Conflicts**: Merging camera systems may lose functionality
  - Mitigation: Careful analysis and feature preservation
3. **Transform Compatibility**: Standardizing Transform may break ECS integration
  - Mitigation: Ensure new Transform implements both Component traits

### Mitigation Strategies
- Work in phases with testing after each phase
- Keep backup of working editor state
- Update dependencies incrementally
- Test camera navigation and component systems thoroughly

## Timeline

**Total Estimated Time**: 4.5-6.5 hours

**Recommended Approach**: Complete one phase fully before starting the next to maintain working state throughout reorganization.

**Priority**: Medium - This cleanup will significantly improve codebase maintainability and prepare for future engine development.

## Integration Notes

- This cleanup builds on the 4-tier architecture established in Phase 5
- Prepares clean foundation for future rendering and physics integration
- Maintains all existing editor functionality while improving code organization
- Sets stage for Phase 10+ feature development with clean architecture