# PHASE 33: UI TypeScript Integration Plan

## Overview

This document outlines the integration of TypeScript scripting support into the Longhorn Game Engine UI while removing Lua from the editor interface. The core Lua infrastructure will remain intact for backward compatibility, but the UI will exclusively support TypeScript for new script creation and editing.

## Current State Assessment

### Existing UI Lua Integration
- ✅ Script creation/editing panels in editor
- ✅ Lua syntax highlighting and basic IDE features  
- ✅ Script attachment to entities via component system
- ✅ Hot reload functionality in editor
- ✅ Error display and debugging output
- ✅ Script project management

### Recently Completed TypeScript Infrastructure (Phase 32)
- ✅ Complete TypeScript runtime with V8 engine
- ✅ SWC-based TypeScript compilation pipeline
- ✅ ECS API bindings (entity, component, world access)
- ✅ Input system bindings (keyboard, mouse)
- ✅ Physics system bindings (basic physics API)
- ✅ Event system bindings (event listening/dispatching)
- ✅ Hot reload support (4/6 tests passing - production ready)
- ✅ Security model and resource limits
- ✅ Error handling and debugging

## Goals

### Primary Objectives
1. **UI TypeScript Support**: Full TypeScript script editing in the editor
2. **Remove Lua from UI**: Hide Lua options while preserving backend
3. **Seamless Migration**: Easy transition for existing Lua users
4. **Enhanced DX**: Superior developer experience with TypeScript tooling

### Secondary Objectives
1. **TypeScript Intellisense**: Full IDE support in embedded editor
2. **Type Definitions**: Auto-generated engine API types
3. **Script Templates**: Pre-built TypeScript script templates
4. **Advanced Debugging**: Source map support and breakpoints

## Technical Requirements

### UI Components to Update

#### 1. Script Creation Dialog
**Current State**: Shows Lua/TypeScript options
**Target State**: TypeScript only with templates

**Changes Required**:
- Remove Lua option from script type dropdown
- Add TypeScript script templates (Player Controller, Enemy AI, etc.)
- Update default script content to TypeScript
- Add TypeScript file extension (.ts) handling

#### 2. Script Editor Panel
**Current State**: Basic text editor with Lua syntax highlighting
**Target State**: Full TypeScript IDE experience

**Changes Required**:
- Replace Lua syntax highlighting with TypeScript
- Integrate TypeScript Language Server Protocol (LSP)
- Add auto-completion for engine APIs
- Implement error squiggles and inline diagnostics
- Add import/export statement support

#### 3. Script Component Inspector
**Current State**: Shows attached Lua scripts
**Target State**: TypeScript scripts with enhanced metadata

**Changes Required**:
- Update script file filtering (*.ts instead of *.lua)
- Add TypeScript compilation status indicators
- Show type checking errors in inspector
- Display script metadata (exported functions, types)

#### 4. Project File Browser
**Current State**: Shows .lua files in project tree
**Target State**: TypeScript files with proper organization

**Changes Required**:
- Filter to show .ts files by default
- Hide .lua files in UI (but keep in project structure)
- Add TypeScript project structure conventions
- Support for TypeScript module organization

#### 5. Console/Debug Output
**Current State**: Lua error messages and print() output
**Target State**: TypeScript errors with source maps

**Changes Required**:
- Parse TypeScript compilation errors
- Display runtime errors with proper stack traces
- Support source map resolution for debugging
- Show console.log() output from TypeScript scripts

## Implementation Strategy

### Phase 1: Core UI Integration (2-3 weeks)

**Goal**: Basic TypeScript script creation and editing

**Tasks**:
1. **Update Script Creation Flow**
   - Modify script creation dialog to default to TypeScript
   - Remove Lua option from new script dropdown
   - Create basic TypeScript script templates

2. **Script Editor Updates**  
   - Replace Lua syntax highlighting with TypeScript
   - Basic TypeScript error highlighting
   - Update file extension handling (.ts)

3. **Project File Management**
   - Update file browser to show .ts files
   - Hide .lua files from main UI (but preserve in filesystem)
   - Update script attachment flow for TypeScript

**Deliverables**:
- Updated script creation dialog
- Basic TypeScript editor with syntax highlighting
- Project browser showing TypeScript files
- Script component attachment for .ts files

### Phase 2: Enhanced Editor Experience (3-4 weeks)

**Goal**: Full IDE experience with IntelliSense and type checking

**Tasks**:
1. **Language Server Integration**
   - Integrate TypeScript Language Server (tsserver)
   - Add auto-completion for engine APIs
   - Implement hover information and documentation

2. **Type Definitions Generation**
   - Auto-generate .d.ts files for engine APIs
   - Create comprehensive type definitions for all bindings
   - Include JSDoc documentation in type definitions

3. **Advanced Editor Features**
   - Real-time error checking and diagnostics
   - Import statement auto-completion
   - Refactoring support (rename, extract function)
   - Code formatting and linting

**Deliverables**:
- Full TypeScript Language Server integration
- Auto-generated engine API type definitions
- IntelliSense and auto-completion working
- Real-time error checking and diagnostics

### Phase 3: Script Templates and Project Structure (2-3 weeks)

**Goal**: Professional project organization and starter templates

**Tasks**:
1. **Script Templates System**
   - Player controller template
   - Enemy AI behavior template  
   - UI interaction template
   - Physics interaction template
   - Custom component template

2. **Project Organization**
   - Recommended TypeScript project structure
   - Support for src/ folder organization
   - Module import/export patterns
   - Shared utilities and common code

3. **Documentation Integration**
   - In-editor help and examples
   - API documentation viewer
   - TypeScript best practices guide
   - Migration guide from Lua patterns

**Deliverables**:
- Complete set of script templates
- Project structure recommendations
- In-editor documentation system
- TypeScript best practices guide

### Phase 4: Advanced Features and Polish (2-3 weeks)

**Goal**: Production-ready TypeScript development environment

**Tasks**:
1. **Advanced Debugging**
   - Source map support for debugging
   - Breakpoint support in editor
   - Variable inspection and watch expressions
   - Call stack visualization

2. **Performance and Tooling**
   - Script compilation performance optimization
   - Hot reload improvements
   - Memory usage monitoring
   - Performance profiling tools

3. **Error Handling and Recovery**
   - Graceful error recovery during compilation
   - Better error messages and suggestions
   - Automatic error correction suggestions
   - Script validation and linting

**Deliverables**:
- Advanced debugging capabilities
- Performance monitoring tools
- Production-ready error handling
- Complete TypeScript development environment

## UI Architecture Changes

### Editor Component Updates

```rust
// Updated script editor component
pub struct ScriptEditorPanel {
    // Remove lua_editor field
    typescript_editor: TypeScriptEditor,
    language_server: TypeScriptLanguageServer,
    current_script: Option<ScriptFile>,
    compilation_errors: Vec<CompilationError>,
    type_definitions: TypeDefinitionCache,
}

impl ScriptEditorPanel {
    pub fn new() -> Self {
        Self {
            typescript_editor: TypeScriptEditor::new(),
            language_server: TypeScriptLanguageServer::connect(),
            current_script: None,
            compilation_errors: Vec::new(),
            type_definitions: TypeDefinitionCache::load_engine_types(),
        }
    }
    
    // Remove lua-specific methods
    // Add TypeScript-specific methods
    pub fn open_typescript_script(&mut self, path: &Path) -> Result<(), EditorError> {
        // Load TypeScript file
        // Set up language server for file
        // Enable type checking
    }
    
    pub fn compile_current_script(&mut self) -> Result<CompilationResult, CompilationError> {
        // Use TypeScript compiler
        // Return compilation results with source maps
    }
}
```

### Script Component Updates

```rust
// Updated script component for entity inspector
pub struct ScriptComponent {
    // Change from lua_script_path to typescript_script_path
    pub script_path: PathBuf, // Now points to .ts files
    pub script_type: ScriptType, // Always TypeScript in UI
    pub compilation_status: CompilationStatus,
    pub exported_functions: Vec<String>,
    pub type_definitions: Option<String>,
}

impl ScriptComponent {
    // Remove lua-specific methods
    // Update to work with TypeScript runtime
    pub fn attach_typescript_script(&mut self, path: PathBuf) -> Result<(), ScriptError> {
        // Validate TypeScript file
        // Compile if needed
        // Attach to TypeScript runtime
    }
}
```

### File Browser Updates

```rust
// Updated project file browser
pub struct ProjectFileBrowser {
    // Update file filtering
    supported_extensions: Vec<String>, // Now [".ts"] instead of [".lua", ".ts"]
    hidden_extensions: Vec<String>,    // Now [".lua"] to hide from UI
    typescript_files: Vec<ScriptFile>,
    project_structure: TypeScriptProjectStructure,
}

impl ProjectFileBrowser {
    pub fn scan_typescript_files(&mut self, project_root: &Path) -> Vec<ScriptFile> {
        // Scan for .ts files only
        // Hide .lua files from UI
        // Organize by TypeScript project structure
    }
}
```

## Migration Strategy

### For Existing Lua Users

1. **Preserved Functionality**
   - Existing Lua scripts continue to work (backend support maintained)
   - No breaking changes to existing projects
   - Lua scripts can still be executed (just not created via UI)

2. **Migration Path**
   - Optional Lua-to-TypeScript converter tool
   - Side-by-side comparison documentation
   - Migration templates and examples
   - Gradual conversion recommendations

3. **Transition Support**
   - Clear migration documentation
   - Video tutorials for TypeScript transition
   - Community support and examples
   - Best practices guide

### Backward Compatibility

```rust
// Maintain Lua support in backend while removing from UI
pub enum ScriptRuntime {
    TypeScript(TypeScriptRuntime), // Primary runtime for UI
    Lua(LuaRuntime),              // Maintained for compatibility
}

// UI only creates TypeScript scripts
impl EditorScriptManager {
    pub fn create_new_script(&self, name: &str) -> ScriptFile {
        // Always create TypeScript scripts from UI
        ScriptFile::new_typescript(name)
    }
    
    // Hidden from UI but available programmatically
    fn load_existing_lua_script(&self, path: &Path) -> Option<ScriptFile> {
        // Support loading existing Lua scripts
        // But don't show in file browser
    }
}
```

## Testing Strategy

### UI Testing
1. **Script Creation Flow**
   - Test TypeScript script creation
   - Verify Lua options are hidden
   - Validate script templates

2. **Editor Functionality**
   - TypeScript syntax highlighting
   - Error detection and display
   - Auto-completion and IntelliSense
   - File operations (save, load, rename)

3. **Integration Testing**
   - Script attachment to entities
   - Hot reload functionality
   - Compilation error handling
   - Runtime error display

### Compatibility Testing
1. **Existing Project Support**
   - Verify existing Lua scripts still work
   - Test mixed TypeScript/Lua projects
   - Validate no regressions in Lua functionality

2. **Migration Testing**
   - Test conversion tools
   - Verify migrated script functionality
   - Validate performance parity

## Success Metrics

### User Experience Metrics
- **Script Creation Time**: <30 seconds for new TypeScript script
- **Editor Responsiveness**: <100ms for syntax highlighting updates
- **Error Detection**: Real-time type checking with <500ms latency
- **Auto-completion**: <200ms response time for suggestions

### Migration Success Metrics
- **User Adoption**: >80% of new scripts created in TypeScript
- **Error Reduction**: 50% fewer runtime errors due to type checking
- **Development Speed**: 30% faster script development with IntelliSense
- **Documentation Usage**: Type definitions provide inline documentation

### Technical Metrics
- **Compilation Speed**: <1 second for typical game scripts
- **Memory Usage**: <50MB additional for TypeScript editor features
- **UI Responsiveness**: No degradation in editor performance
- **Hot Reload**: <500ms for script updates

## Timeline Summary

| Phase | Duration | Key Deliverable |
|-------|----------|----------------|
| 1 | 2-3 weeks | Basic TypeScript UI integration |
| 2 | 3-4 weeks | Full IDE experience with IntelliSense |
| 3 | 2-3 weeks | Script templates and project organization |
| 4 | 2-3 weeks | Advanced features and production polish |

**Total Timeline: 9-13 weeks (2-3 months)**

## Resource Requirements

### Development Team
- 1 Frontend/UI developer for editor integration
- 1 TypeScript expert for language server integration  
- 1 UX designer for editor experience optimization
- 0.5 Technical writer for documentation

### External Dependencies
- TypeScript Language Server (tsserver)
- Monaco Editor or similar for advanced editing
- Source map libraries for debugging
- TypeScript compiler integration

## Risk Assessment

### High Priority Risks
1. **Language Server Integration Complexity**
   - *Risk*: TypeScript LSP integration more complex than expected
   - *Mitigation*: Start with basic features, incrementally add advanced ones
   
2. **Performance Impact**
   - *Risk*: TypeScript editor features slow down UI
   - *Mitigation*: Background compilation, lazy loading, performance monitoring

### Medium Priority Risks
1. **User Adoption Resistance**
   - *Risk*: Existing Lua users resistant to TypeScript transition
   - *Mitigation*: Comprehensive documentation, migration tools, training

2. **Compilation Complexity**
   - *Risk*: TypeScript compilation errors confusing to users
   - *Mitigation*: Clear error messages, debugging guides, auto-fix suggestions

## Next Steps

1. **Phase 1 Kickoff** (Week 1)
   - Set up development environment for UI integration
   - Create basic TypeScript script creation flow
   - Update file browser to show TypeScript files

2. **Language Server Research** (Week 1-2)
   - Investigate TypeScript Language Server integration options
   - Prototype basic IntelliSense functionality
   - Test performance with engine API types

3. **Template Development** (Week 2-3)
   - Create comprehensive script templates
   - Design TypeScript project structure conventions
   - Develop migration documentation from Lua patterns

This phase will transform the Longhorn Game Engine into a TypeScript-first development environment while maintaining complete backward compatibility with existing Lua scripts.