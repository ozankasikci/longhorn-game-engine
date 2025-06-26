# Phase 31: Scripting API Design & Implementation Completion

## Overview
Phase 31 focuses on completing the scripting API, improving developer experience, and implementing missing features. This phase builds on the secure foundation from Phase 29 and optimized performance from Phase 30.

## Previous Phase Context
Following Phase 29's security fixes and Phase 30's performance optimizations, Phase 31 completes the scripting system with a comprehensive, well-designed API and full feature implementation.

## Goals
1. **Complete missing API implementations** - Event system, input, physics integration
2. **Improve API consistency** - Standardize interfaces and naming conventions
3. **Enhance developer experience** - Better error messages, debugging support, documentation
4. **Add advanced features** - Script debugging, profiling, advanced hot reload

## Current Problems

### API Design Issues
- **Inconsistent interfaces**: Multiple ways to load scripts with different patterns
- **Unclear ownership**: Entity/component references have ambiguous lifetimes
- **Incomplete implementation**: Event system, input, and physics integration missing
- **Poor error messages**: Generic errors don't help with debugging

### Missing Features
- **Event system integration**: Only stub implementation exists
- **Input system connection**: No input handling in scripts
- **Physics integration**: No physics API available
- **Asset loading**: Scripts can't load assets
- **Debugging support**: No debugging or profiling tools

### Files to Modify
- `crates/implementation/engine-scripting/src/api.rs`
- `crates/implementation/engine-scripting/src/bindings.rs`
- `crates/implementation/engine-scripting/src/lua/events.rs`
- `crates/implementation/engine-scripting/src/lua/engine.rs`
- All files in `crates/implementation/engine-scripting/src/lua/`

## Implementation Tasks

### Task 1: API Consistency and Standardization (Priority: High)

#### 1.1 Standardize Script Loading Interface
**Location**: `api.rs`, `manager.rs`
```rust
pub trait ScriptLoader {
    fn load_script(&mut self, request: ScriptLoadRequest) -> Result<ScriptHandle, ScriptError>;
    fn unload_script(&mut self, handle: ScriptHandle) -> Result<(), ScriptError>;
    fn reload_script(&mut self, handle: ScriptHandle) -> Result<(), ScriptError>;
}

pub struct ScriptLoadRequest {
    pub source: ScriptSource,
    pub entity_binding: Option<EntityId>,
    pub execution_context: ExecutionContext,
}

pub enum ScriptSource {
    File(PathBuf),
    String { content: String, name: String },
    Bytecode(Vec<u8>),
}
```

#### 1.2 Clear Ownership and Lifetime Management
**Location**: `bindings.rs`
```rust
pub struct EntityHandle {
    entity_id: EntityId,
    world_version: u64,
    access_permissions: AccessPermissions,
}

impl EntityHandle {
    pub fn is_valid(&self, world: &World) -> bool {
        world.version() == self.world_version && 
        world.entity_exists(self.entity_id)
    }
    
    pub fn get_component<T>(&self, world: &World) -> Result<ComponentRef<T>, ScriptError> {
        if !self.is_valid(world) {
            return Err(ScriptError::InvalidEntityHandle);
        }
        
        if !self.access_permissions.can_read::<T>() {
            return Err(ScriptError::AccessDenied);
        }
        
        world.get_component(self.entity_id)
    }
}
```

#### 1.3 Consistent Error Handling and Messages
**Location**: `error.rs`
```rust
#[derive(Debug, thiserror::Error)]
pub enum ScriptError {
    #[error("Script '{script_name}' failed at line {line}: {message}")]
    RuntimeError {
        script_name: String,
        line: u32,
        message: String,
        stack_trace: Vec<String>,
    },
    
    #[error("Component '{component}' not found on entity {entity}")]
    ComponentNotFound {
        entity: EntityId,
        component: String,
    },
    
    #[error("Invalid entity handle: entity may have been destroyed")]
    InvalidEntityHandle,
    
    #[error("Access denied: script lacks permission for '{operation}'")]
    AccessDenied { operation: String },
}
```

### Task 2: Complete Event System Integration (Priority: High)

#### 2.1 Event System Implementation
**Location**: `lua/events.rs`
```rust
pub struct LuaEventSystem {
    event_listeners: HashMap<EventType, Vec<LuaFunction>>,
    event_queue: VecDeque<ScriptEvent>,
    lua_registry: RegistryKey,
}

impl LuaEventSystem {
    pub fn register_event_listener(&mut self, event_type: EventType, callback: LuaFunction) {
        self.event_listeners
            .entry(event_type)
            .or_default()
            .push(callback);
    }
    
    pub fn emit_event(&mut self, event: ScriptEvent) {
        self.event_queue.push_back(event);
    }
    
    pub fn process_events(&mut self, lua: &Lua) -> Result<(), ScriptError> {
        while let Some(event) = self.event_queue.pop_front() {
            self.dispatch_event(lua, event)?;
        }
        Ok(())
    }
}
```

#### 2.2 Lua Event API
**Location**: `bindings.rs`
```rust
fn register_event_api(lua: &Lua) -> Result<(), ScriptError> {
    let globals = lua.globals();
    
    // Event listening
    globals.set("on_event", lua.create_function(|_, (event_type, callback): (String, LuaFunction)| {
        EVENT_SYSTEM.register_listener(event_type, callback)
    })?)?;
    
    // Event emission
    globals.set("emit_event", lua.create_function(|_, (event_type, data): (String, LuaValue)| {
        EVENT_SYSTEM.emit(event_type, data)
    })?)?;
    
    Ok(())
}
```

### Task 3: Input System Integration (Priority: High)

#### 3.1 Input System Bridge
**New file**: `lua/input.rs`
```rust
pub struct LuaInputManager {
    input_state: InputState,
    key_bindings: HashMap<String, Vec<LuaFunction>>,
}

impl LuaInputManager {
    pub fn update(&mut self, input_events: &[InputEvent]) {
        for event in input_events {
            match event {
                InputEvent::KeyPressed(key) => {
                    if let Some(callbacks) = self.key_bindings.get(&key.to_string()) {
                        for callback in callbacks {
                            // Execute callback
                        }
                    }
                }
                // Handle other input events
            }
        }
    }
}
```

#### 3.2 Lua Input API
**Location**: `bindings.rs`
```rust
fn register_input_api(lua: &Lua) -> Result<(), ScriptError> {
    let globals = lua.globals();
    
    // Input queries
    globals.set("is_key_pressed", lua.create_function(|_, key: String| {
        INPUT_MANAGER.is_key_pressed(&key)
    })?)?;
    
    globals.set("get_mouse_position", lua.create_function(|_, ()| {
        let pos = INPUT_MANAGER.mouse_position();
        Ok((pos.x, pos.y))
    })?)?;
    
    // Input bindings
    globals.set("bind_key", lua.create_function(|_, (key, callback): (String, LuaFunction)| {
        INPUT_MANAGER.bind_key(key, callback)
    })?)?;
    
    Ok(())
}
```

### Task 4: Physics Integration (Priority: Medium)

#### 4.1 Physics System Bridge
**New file**: `lua/physics.rs`
```rust
pub struct LuaPhysicsManager {
    physics_world: PhysicsWorld,
    rigid_body_handles: HashMap<EntityId, RigidBodyHandle>,
}

impl LuaPhysicsManager {
    pub fn add_rigid_body(&mut self, entity: EntityId, body_desc: RigidBodyDesc) -> Result<(), ScriptError> {
        let handle = self.physics_world.add_rigid_body(body_desc);
        self.rigid_body_handles.insert(entity, handle);
        Ok(())
    }
    
    pub fn apply_force(&mut self, entity: EntityId, force: Vector3<f32>) -> Result<(), ScriptError> {
        if let Some(handle) = self.rigid_body_handles.get(&entity) {
            self.physics_world.apply_force(*handle, force);
            Ok(())
        } else {
            Err(ScriptError::PhysicsBodyNotFound { entity })
        }
    }
}
```

#### 4.2 Lua Physics API
**Location**: `bindings.rs`
```rust
fn register_physics_api(lua: &Lua) -> Result<(), ScriptError> {
    let globals = lua.globals();
    
    globals.set("add_rigid_body", lua.create_function(|_, (entity_id, body_type): (u64, String)| {
        PHYSICS_MANAGER.add_rigid_body(entity_id, body_type)
    })?)?;
    
    globals.set("apply_force", lua.create_function(|_, (entity_id, x, y, z): (u64, f32, f32, f32)| {
        PHYSICS_MANAGER.apply_force(entity_id, Vector3::new(x, y, z))
    })?)?;
    
    Ok(())
}
```

### Task 5: Advanced Developer Tools (Priority: Medium)

#### 5.1 Script Debugging Support
**New file**: `debugging.rs`
```rust
pub struct ScriptDebugger {
    breakpoints: HashSet<(ScriptId, u32)>,
    call_stack: Vec<CallFrame>,
    variable_inspector: VariableInspector,
}

impl ScriptDebugger {
    pub fn set_breakpoint(&mut self, script: ScriptId, line: u32) {
        self.breakpoints.insert((script, line));
    }
    
    pub fn step_into(&mut self) -> DebugAction {
        DebugAction::StepInto
    }
    
    pub fn inspect_variables(&self, scope: ScopeId) -> HashMap<String, LuaValue> {
        self.variable_inspector.get_variables(scope)
    }
}
```

#### 5.2 Performance Profiling
**New file**: `profiler.rs`
```rust
pub struct ScriptProfiler {
    function_timings: HashMap<String, FunctionProfile>,
    memory_snapshots: Vec<MemorySnapshot>,
    current_session: ProfilingSession,
}

pub struct FunctionProfile {
    total_time: Duration,
    call_count: u64,
    average_time: Duration,
    peak_time: Duration,
}
```

#### 5.3 Enhanced Hot Reload
**Location**: `file_manager.rs`
```rust
pub struct AdvancedHotReload {
    state_preservation: StatePreservation,
    dependency_graph: DependencyGraph,
    reload_strategies: HashMap<ScriptId, ReloadStrategy>,
}

pub enum ReloadStrategy {
    FullReload,
    StatePreserving,
    IncrementalUpdate,
}
```

## Testing Requirements

### API Testing
1. **API consistency tests** - Verify all interfaces follow standards
2. **Error handling tests** - Comprehensive error path coverage
3. **Documentation tests** - Ensure all examples work
4. **Integration tests** - Test with real game scenarios

### Feature Testing
1. **Event system tests** - Event dispatch and handling
2. **Input system tests** - Input processing and bindings
3. **Physics tests** - Physics integration functionality
4. **Debugging tests** - Debugger functionality verification

### Developer Experience Testing
1. **Error message quality** - Verify helpful error messages
2. **Performance testing** - Ensure no regression from features
3. **Documentation coverage** - Complete API documentation
4. **Example script testing** - Verify tutorial scripts work

## Success Criteria

### API Quality
- [ ] Consistent interface patterns across all APIs
- [ ] Clear ownership and lifetime semantics
- [ ] Comprehensive error handling with helpful messages
- [ ] Complete API documentation with examples

### Feature Completeness
- [ ] Event system fully implemented and tested
- [ ] Input system integrated and functional
- [ ] Physics integration working with examples
- [ ] Asset loading available to scripts

### Developer Experience
- [ ] Script debugging tools functional
- [ ] Performance profiling available
- [ ] Hot reload preserves state correctly
- [ ] Comprehensive tutorial and examples

## Timeline
- **Week 1**: API standardization and event system
- **Week 2**: Input and physics integration
- **Week 3**: Advanced developer tools and documentation
- **Week 4**: Testing, polish, and final integration

## Risks and Mitigation

### Risk: API Complexity
- **Mitigation**: Keep APIs simple and consistent
- **Fallback**: Provide high-level convenience functions

### Risk: Performance Impact of Features
- **Mitigation**: Benchmark all new features
- **Fallback**: Feature flags for optional functionality

### Risk: Integration Complexity
- **Mitigation**: Incremental integration with extensive testing
- **Fallback**: Modular design allows disabling problematic features

## Dependencies
- **Phase 29**: Security and architecture fixes (completed)
- **Phase 30**: Memory and performance optimization (completed)
- **External systems**: Event, input, and physics systems from other crates

## Next Steps
After Phase 31, the scripting system will be production-ready with comprehensive features and excellent developer experience.

## Notes
- This phase focuses on developer experience and feature completeness
- All new features must maintain security guarantees from Phase 29
- Performance impact should be minimal thanks to Phase 30 optimizations
- Comprehensive testing and documentation are essential for adoption