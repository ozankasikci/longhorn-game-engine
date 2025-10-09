# PHASE 33: TypeScript UI Integration Plan

## Overview

This document outlines the integration of TypeScript scripting support into the Longhorn Game Engine UI, making TypeScript the primary scripting language in the editor interface while maintaining Lua support in the background for compatibility.

## Current State Assessment

### Completed Infrastructure (Phase 32)
- ✅ TypeScript runtime with V8 engine
- ✅ SWC-based TypeScript compilation
- ✅ Complete ECS bindings (entity, component, world access)
- ✅ Input system bindings (keyboard, mouse input)
- ✅ Physics system bindings (basic physics API)
- ✅ Event system bindings (event listening/dispatching)
- ✅ Hot reload foundation (4/6 tests passing)
- ✅ Security model and sandboxing
- ✅ Function calling with proper type conversion

### Current UI State (Lua-focused)
- ❌ UI only shows Lua scripting options
- ❌ Script creation defaults to `.lua` files
- ❌ Code editor configured for Lua syntax
- ❌ No TypeScript project templates
- ❌ No TypeScript IntelliSense integration

## Goals

### Primary Objectives
1. **Make TypeScript the default scripting language** in the UI
2. **Integrate TypeScript editor support** with syntax highlighting and IntelliSense
3. **Add TypeScript project templates** and examples
4. **Update script creation workflows** to default to TypeScript
5. **Maintain Lua compatibility** without UI prominence

### Developer Experience Goals
- Zero-config TypeScript project setup
- Real-time type checking in the editor
- Auto-completion for engine APIs
- Integrated error reporting with source maps
- Hot reload with immediate feedback

## Technical Implementation Plan

### Phase 1: UI Framework Updates (1-2 weeks)

**Goal**: Update the editor UI to support TypeScript as the primary language

**Tasks:**
1. **Script File Type Management**
   - Update file creation dialogs to default to `.ts` files
   - Add TypeScript file icons and recognition
   - Update file browser to prioritize TypeScript files

2. **Script Template System**
   - Create TypeScript project templates
   - Add common script patterns (Entity behavior, System scripts, etc.)
   - Update "New Script" workflow to use TypeScript templates

3. **Runtime Selection UI**
   - Update script runtime dropdown to default to TypeScript
   - Move Lua option to "Advanced" or secondary position
   - Add runtime capability indicators

**Key Files to Modify:**
- `crates/application/engine-editor-egui/src/panels/project_view.rs`
- `crates/application/engine-editor-egui/src/dialogs/new_script_dialog.rs`
- `crates/application/engine-editor-egui/src/utils/file_types.rs`

### Phase 2: Code Editor Integration (2-3 weeks)

**Goal**: Integrate TypeScript-aware code editing capabilities

**Tasks:**
1. **Syntax Highlighting**
   - Configure egui_code_editor for TypeScript syntax
   - Add TypeScript keywords, operators, and patterns
   - Support for JSX/TSX if needed for UI scripts

2. **IntelliSense Integration**
   - Integrate with TypeScript Language Server (via rusty_v8 or external LSP)
   - Provide auto-completion for engine APIs
   - Real-time error checking and diagnostics

3. **Error Reporting**
   - Show TypeScript compilation errors in editor
   - Source map integration for runtime errors
   - Inline error markers and hover tooltips

**Dependencies to Add:**
```toml
# For TypeScript language server integration
tower-lsp = "0.20"
lsp-types = "0.94"

# For enhanced code editing
tree-sitter = "0.20"
tree-sitter-typescript = "0.20"
```

### Phase 3: Engine API Type Definitions (1-2 weeks)

**Goal**: Provide comprehensive TypeScript type definitions for all engine APIs

**Tasks:**
1. **Generate Type Definitions**
   - Auto-generate `.d.ts` files from Rust API bindings
   - Create comprehensive engine API typings
   - Document all available functions and properties

2. **API Documentation Integration**
   - Embed documentation in type definitions
   - Provide examples in hover tooltips
   - Link to comprehensive API documentation

3. **Type Definition Distribution**
   - Bundle type definitions with engine
   - Auto-update when APIs change
   - Version compatibility tracking

**Generated Type Definitions:**
```typescript
// engine.d.ts - Auto-generated from Rust bindings
declare global {
  namespace Engine {
    interface World {
      createEntity(components?: ComponentMap): Entity;
      removeEntity(entity: Entity): void;
      query<T extends Component>(type: ComponentType<T>): QueryResult<T>;
      getCurrentEntity(): Entity | null;
    }

    interface Input {
      isKeyPressed(key: string): boolean;
      getMousePosition(): Vector2;
      isMouseButtonPressed(button: MouseButton): boolean;
    }

    interface Math {
      vec2(x: number, y: number): Vector2;
      vec3(x: number, y: number, z: number): Vector3;
      lerp(a: number, b: number, t: number): number;
    }

    interface Time {
      readonly deltaTime: number;
      readonly totalTime: number;
      readonly frameCount: number;
    }
  }
}
```

### Phase 4: Project Template System (1-2 weeks)

**Goal**: Create comprehensive TypeScript project templates and examples

**Tasks:**
1. **Basic Templates**
   - Entity behavior script template
   - Game system script template
   - Utility script template
   - Event handler script template

2. **Example Projects**
   - Complete game examples in TypeScript
   - Common patterns and best practices
   - Migration examples from Lua patterns

3. **Template Integration**
   - Integrate templates into "New Project" workflow
   - Template selection UI with previews
   - Customizable template parameters

**Template Examples:**
```typescript
// entity_behavior_template.ts
interface EntityScript {
  init(): void;
  update(deltaTime: number): void;
}

export class PlayerController implements EntityScript {
  private entity: Entity;

  init(): void {
    this.entity = Engine.world.getCurrentEntity()!;
    console.log("Player controller initialized");
  }

  update(deltaTime: number): void {
    // Handle player input
    if (Engine.input.isKeyPressed("W")) {
      // Move forward
    }
  }
}
```

### Phase 5: Build and Hot Reload Integration (2-3 weeks)

**Goal**: Seamless TypeScript compilation and hot reload in the editor

**Tasks:**
1. **Background Compilation**
   - Automatic TypeScript compilation on file save
   - Incremental compilation for performance
   - Background error checking

2. **Hot Reload Enhancement**
   - Improve hot reload success rate from 67% to 90%+
   - Better state preservation during reload
   - Visual feedback for reload status

3. **Build Pipeline Integration**
   - Integrate with existing game build system
   - TypeScript bundling for distribution
   - Source map generation for debugging

**Hot Reload Improvements:**
- Fix variable redeclaration issues in V8 context
- Implement proper module scope isolation
- Add state preservation mechanisms

### Phase 6: Documentation and Migration Support (1-2 weeks)

**Goal**: Comprehensive documentation and Lua-to-TypeScript guidance

**Tasks:**
1. **TypeScript Scripting Guide**
   - Complete TypeScript scripting documentation
   - Best practices for game development
   - Performance optimization guide

2. **Lua Migration Documentation**
   - Lua to TypeScript conversion guide
   - API mapping reference
   - Common pattern translations

3. **Interactive Examples**
   - In-editor TypeScript tutorials
   - Interactive API explorer
   - Code snippet library

## UI Design Changes

### Script Creation Flow
```
Before (Lua-focused):
New Script → [Lua Script] → script.lua

After (TypeScript-first):
New Script → [TypeScript Script] [Other...] → script.ts
                               ↳ [Lua Script] → script.lua
```

### File Browser Enhancements
- TypeScript files (`.ts`, `.tsx`) prominently displayed
- TypeScript project folders with special icons
- Type definition files (`.d.ts`) grouped separately
- Lua files still visible but de-emphasized

### Script Editor Interface
```
┌─ Script Editor: player_controller.ts ────────────────┐
│ TypeScript ▼  [●] Auto-compile  [⚡] Hot Reload     │
├──────────────────────────────────────────────────────┤
│ export class PlayerController {                       │
│   init(): void {                                     │
│     const entity = Engine.world.getCurrentEntity(); │
│           ~~~~~~ ✓ Type: Entity | null               │
│   }                                                  │
│ }                                                    │
├──────────────────────────────────────────────────────┤
│ ✓ No errors  ⚡ Hot reload ready  📝 Auto-save on    │
└──────────────────────────────────────────────────────┘
```

## Backward Compatibility Strategy

### Lua Support Maintenance
- Keep all existing Lua runtime functionality
- Maintain Lua API bindings
- Support mixed TypeScript/Lua projects
- Lua accessible via "Advanced" runtime options

### Migration Path
1. **Immediate**: New projects default to TypeScript
2. **Gradual**: Existing Lua projects continue working
3. **Optional**: Provide migration tools for Lua→TypeScript conversion
4. **Long-term**: Lua remains available but not promoted in UI

## Success Metrics

### Developer Experience
- **Script Creation**: 90% of new scripts created as TypeScript
- **Compilation Speed**: <500ms for typical game scripts
- **Hot Reload Success**: >90% successful hot reloads
- **Error Detection**: Real-time error detection within 1 second

### Technical Performance
- **UI Responsiveness**: No noticeable lag in TypeScript editor
- **Memory Usage**: <50MB additional memory for TypeScript support
- **Build Time**: TypeScript compilation adds <10% to build time

### Adoption Metrics
- **Developer Preference**: Survey indicates TypeScript preference
- **Error Reduction**: 30% fewer runtime errors due to type checking
- **Development Speed**: Faster iteration due to better tooling

## Risk Assessment

### Technical Risks
1. **Editor Performance**: Large TypeScript files may slow down editor
   - *Mitigation*: Lazy loading, syntax highlighting optimization
2. **Hot Reload Complexity**: Variable scoping issues in V8
   - *Mitigation*: Context isolation improvements, better state management
3. **Type Definition Maintenance**: Keeping types in sync with Rust APIs
   - *Mitigation*: Automated generation from Rust code

### User Experience Risks
1. **Learning Curve**: Developers unfamiliar with TypeScript
   - *Mitigation*: Comprehensive documentation, examples, gradual introduction
2. **Compilation Complexity**: Build process becomes more complex
   - *Mitigation*: Zero-config defaults, clear error messages

## Timeline Summary

| Phase | Duration | Key Deliverable |
|-------|----------|----------------|
| 1 | 1-2 weeks | TypeScript-first UI |
| 2 | 2-3 weeks | Code editor integration |
| 3 | 1-2 weeks | Type definitions |
| 4 | 1-2 weeks | Project templates |
| 5 | 2-3 weeks | Build & hot reload |
| 6 | 1-2 weeks | Documentation |

**Total Timeline: 8-14 weeks (2-3.5 months)**

## Implementation Priority

### High Priority (Must Have)
- TypeScript-first script creation
- Basic syntax highlighting
- Auto-completion for engine APIs
- Hot reload functionality

### Medium Priority (Should Have)
- Advanced error reporting
- Project templates
- Migration documentation

### Low Priority (Nice to Have)
- Advanced debugging integration
- Performance profiling
- Package manager integration

## Next Steps

1. **Week 1**: Begin Phase 1 - UI framework updates
2. **Week 2**: Continue UI work, start code editor research
3. **Week 3**: Begin Phase 2 - code editor integration
4. **Week 4**: Complete syntax highlighting and basic IntelliSense

This phase will transform the developer experience by making TypeScript the primary scripting language while maintaining full backward compatibility with existing Lua scripts.