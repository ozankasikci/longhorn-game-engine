# Phase 26: Lua Scripting Support Implementation Plan

## Overview
This phase introduces Lua scripting support to the Longhorn game engine, enabling developers to write game logic, components, and behaviors in Lua. The implementation leverages the existing `engine-scripting` infrastructure while providing seamless integration with the ECS system, events, and engine APIs through the mlua crate.

## Objectives
- Integrate Lua scripting runtime using mlua into the existing `engine-scripting` crate
- Provide safe Lua/Rust interop for ECS components and engine APIs
- Enable hot-reloading of Lua scripts during development
- Support Lua script debugging through the editor
- Maintain high performance with minimal overhead (~245KB runtime)

## Technical Approach

### Runtime Selection: Lua 5.4 with mlua
Based on research, we'll use mlua for Lua integration:
- **Advantages**: Minimal overhead, excellent performance (especially with LuaJIT), native hot-reload support
- **Safety**: mlua provides safe, high-level Rust bindings to Lua
- **Features**: Supports Lua 5.4/5.3/5.2/5.1, LuaJIT, async/await, and serde integration

### Architecture Integration Points
1. **ECS Integration**: Lua tables as ECS components with automatic marshaling
2. **Event System**: Bidirectional event handling between Lua and engine
3. **Asset System**: Lua scripts as hot-reloadable assets
4. **Runtime Scheduling**: Lua script execution integrated with engine update loop

## Phase Structure

### Week 1: Foundation & Runtime Setup
**Goal**: Establish Lua runtime infrastructure in existing scripting crate

#### Core Runtime Implementation
- [ ] Extend `engine-scripting` crate with Lua support
- [ ] Integrate mlua dependency and configure features (Lua 5.4 + LuaJIT optional)
- [ ] Implement `LuaScriptEngine` for the existing `ScriptRuntime` trait
- [ ] Create Lua context management and sandboxing
- [ ] Set up script loading and execution pipeline

#### Basic API Exposure
- [ ] Create core engine API table structure
- [ ] Implement safe error handling and panic catching
- [ ] Set up Lua module system for script organization
- [ ] Create debug print and logging utilities
- [ ] Implement basic performance profiling hooks

### Week 1-2: ECS Integration
**Goal**: Enable Lua scripts to interact with ECS system

#### Component System
- [ ] Create Lua component registration system
- [ ] Implement automatic Lua table ↔ Rust component marshaling
- [ ] Build component serialization using serde + mlua
- [ ] Add change tracking integration for Lua-modified components
- [ ] Create component type registry

#### World API
- [ ] Expose safe World operations to Lua (`world.create_entity()`, etc.)
- [ ] Implement query system in Lua (`world:query("Transform, Velocity")`)
- [ ] Add entity manipulation functions
- [ ] Build component get/set operations with type safety
- [ ] Create entity lifecycle callbacks

#### Example Lua API:
```lua
-- Create entity with components
local player = world:create_entity({
    Transform = { position = vec3(0, 0, 0) },
    Health = { current = 100, max = 100 },
    PlayerController = {}
})

-- Query entities
for entity, transform, velocity in world:query("Transform, Velocity") do
    transform.position = transform.position + velocity.value * dt
end
```

### Week 2: Script Lifecycle & Events
**Goal**: Provide complete script execution model

#### Script Component System
- [ ] Create `LuaScript` component for attaching scripts to entities
- [ ] Implement script lifecycle (init, update, fixed_update, cleanup)
- [ ] Build script instance management per entity
- [ ] Add script enable/disable functionality
- [ ] Create script execution ordering system

#### Event System Integration
- [ ] Create Lua event dispatcher
- [ ] Implement event subscription from Lua scripts
- [ ] Add typed event parameters with automatic marshaling
- [ ] Build event priority and filtering system
- [ ] Create custom event definition support

#### Example Script:
```lua
local MyScript = {}

function MyScript:init()
    self:subscribe("collision", self.on_collision)
    self.speed = 10.0
end

function MyScript:update(dt)
    local transform = self.entity:get_component("Transform")
    transform.position.x = transform.position.x + self.speed * dt
end

function MyScript:on_collision(event)
    print("Collision with", event.other_entity)
end

return MyScript
```

### Week 2-3: Asset Integration & Hot-Reload
**Goal**: Seamless development workflow with instant script updates

#### Asset System Integration
- [ ] Register Lua scripts as assets in the asset system
- [ ] Implement script dependency tracking
- [ ] Create script metadata (exposed properties, etc.)
- [ ] Build script validation on load
- [ ] Add script bundling for production

#### Hot-Reload System
- [ ] Implement file watcher for Lua script changes
- [ ] Create stateful reload with data preservation
- [ ] Build reload transaction system (rollback on error)
- [ ] Add development console for live Lua execution
- [ ] Implement reload hooks for cleanup/reinit

### Week 3: Engine API Bindings
**Goal**: Expose comprehensive engine functionality to Lua

#### Core Systems
- [ ] Math library (vectors, matrices, quaternions)
- [ ] Input system bindings (keyboard, mouse, gamepad)
- [ ] Asset loading from Lua (textures, sounds, models)
- [ ] Timer and coroutine utilities
- [ ] Random number generation

#### Rendering Integration
- [ ] Basic rendering commands (draw_sprite, draw_text)
- [ ] Camera manipulation from Lua
- [ ] Particle system control
- [ ] Debug rendering (lines, shapes)
- [ ] UI immediate mode functions

#### Physics & Audio
- [ ] Raycast and collision queries
- [ ] Physics properties modification
- [ ] Audio playback control
- [ ] Spatial audio positioning
- [ ] Music and ambience management

### Week 3-4: Editor Integration & Debugging
**Goal**: Professional development experience

#### Editor Integration
- [ ] Lua script creation wizard
- [ ] Script property inspector (exposed variables)
- [ ] Syntax highlighting for Lua files
- [ ] Auto-completion data generation
- [ ] Script error visualization

#### Debugging Support
- [ ] Lua debugger integration
- [ ] Breakpoint support
- [ ] Variable inspection
- [ ] Call stack visualization
- [ ] Performance profiler for Lua scripts

### Week 4: Documentation & Polish
**Goal**: Production-ready scripting system

#### Documentation
- [ ] Comprehensive Lua API reference
- [ ] Getting started tutorial
- [ ] Common patterns cookbook
- [ ] Performance best practices
- [ ] Migration guide from other engines

#### Examples & Templates
- [ ] Game genre templates (platformer, shooter, puzzle)
- [ ] Common script components
- [ ] Utility script library
- [ ] Integration examples
- [ ] Performance benchmarks

## Implementation Details

### Crate Structure Enhancement
```
engine-scripting/
├── src/
│   ├── lua/
│   │   ├── engine.rs      # LuaScriptEngine implementation
│   │   ├── bindings/      # Engine API bindings
│   │   ├── ecs.rs         # ECS integration
│   │   ├── events.rs      # Event system bindings
│   │   └── assets.rs      # Asset integration
│   ├── lib.rs            # Extended with Lua support
│   └── ...existing code...
├── lua/
│   ├── core/            # Core Lua modules
│   ├── std/             # Standard library
│   └── examples/        # Example scripts
└── tests/
    └── lua/             # Lua integration tests
```

### Key Components

#### 1. Lua Script Engine
```rust
pub struct LuaScriptEngine {
    lua: mlua::Lua,
    scripts: HashMap<AssetId, LuaScript>,
    component_registry: LuaComponentRegistry,
    hot_reload: HotReloadManager,
}

impl ScriptRuntime for LuaScriptEngine {
    fn execute_script(&mut self, id: ScriptId) -> Result<(), ScriptError> {
        // Implementation
    }
}
```

#### 2. Component Marshaling
```rust
pub trait LuaComponent: Component {
    fn to_lua<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>>;
    fn from_lua(value: LuaValue) -> LuaResult<Self>;
}
```

#### 3. Safe World Access
```rust
impl LuaUserData for WorldRef {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("create_entity", |_, this, table: LuaTable| {
            // Safe entity creation with component initialization
        });
        
        methods.add_method("query", |_, this, component_names: String| {
            // Safe query iteration
        });
    }
}
```

## Technical Advantages Over C#

### Performance
- **Runtime size**: 245KB (Lua) vs 100MB+ (.NET)
- **Startup time**: <10ms vs 100ms+
- **Memory usage**: Minimal vs managed heap
- **Hot-reload**: Native vs complex AssemblyLoadContext

### Development Speed
- **Implementation time**: 4 weeks vs 8 weeks
- **Learning curve**: Hours vs days
- **Integration complexity**: Simple FFI vs complex hosting

### Maintenance
- **Codebase impact**: Minimal vs significant
- **Dependencies**: Single crate vs .NET SDK
- **Cross-platform**: Trivial vs platform-specific concerns

## Success Metrics

### Functionality
- [ ] Lua scripts can create and manipulate ECS entities
- [ ] Hot-reload works in <100ms
- [ ] Full engine API accessible from Lua
- [ ] Debugging support with breakpoints

### Performance
- [ ] <1% performance overhead for script execution
- [ ] <50ms hot-reload time
- [ ] Memory usage <10MB for typical game scripts
- [ ] 60+ FPS maintained with 100+ active scripts

### Developer Experience
- [ ] Script errors clearly reported
- [ ] Comprehensive API documentation
- [ ] Example scripts for common patterns
- [ ] Smooth hot-reload workflow

## Timeline
- **Total Duration**: 4 weeks
- **Phase Start**: TBD
- **Phase End**: TBD
- **Milestones**:
  - Week 1: Basic Lua runtime functional
  - Week 2: ECS integration complete
  - Week 3: Hot-reload and full API
  - Week 4: Editor integration delivered

## Future Enhancements (Post-Phase 26)
- Visual scripting with Lua backend
- Lua script optimization and bundling
- Advanced debugging features (time-travel, profiling)
- Community script marketplace
- Lua to native compilation for release builds