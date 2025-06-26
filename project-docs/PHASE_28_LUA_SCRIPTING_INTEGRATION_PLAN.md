# Phase 28: Lua Scripting Integration Plan

## Overview

Phase 28 focuses on integrating Lua scripting into the unified game loop architecture established in Phase 27. This will enable runtime behavior modification, rapid prototyping, and extensible game logic without engine recompilation.

## Research Summary

Based on research into game engine scripting patterns across Unity, Unreal, Godot, and Rust-based systems, the following key principles have been identified:

### 1. **Lifecycle Method Pattern**
- **Init**: Called once when script/entity is created
- **Update**: Called every frame during game loop 
- **Destroy**: Called when script/entity is removed
- **Optional Methods**: FixedUpdate, LateUpdate, OnEnable, OnDisable

### 2. **ECS Integration Patterns**
- **Script Components**: Lua scripts as ECS components attached to entities
- **System Registration**: Scripts can define and register their own systems
- **Data Binding**: Direct access to entity components from Lua
- **Event-Driven**: Scripts respond to game events and entity changes

### 3. **Hot Reload Architecture**
- **File Watching**: Automatic detection of script file changes
- **State Preservation**: Maintain script state across reloads
- **Error Handling**: Graceful fallback when script compilation fails
- **Surgical Updates**: Only reload changed scripts, not entire system

### 4. **Performance Considerations**
- **Single Lua State**: One lua_State for all scripts with shared bindings
- **Execution Order**: Deterministic script execution based on priority
- **Minimal Overhead**: Efficient C++/Rust to Lua interop
- **Async Support**: Non-blocking script execution where possible

## Architecture Design

### Core Components

1. **LuaScriptSystem** 
   - Integrates with HybridGameLoop
   - Manages script lifecycle (init, update, destroy)
   - Handles execution order and priorities
   - Provides hot reload functionality

2. **Script Component**
   - ECS component linking entities to Lua scripts
   - Stores script path, execution order, enabled state
   - Instance-specific data and configuration

3. **Lua Bindings**
   - Engine API exposed to Lua scripts
   - ECS access (components, entities, queries)
   - Math utilities, input handling, logging
   - Asset loading and management

4. **Hot Reload Manager Integration**
   - Extends existing hot reload system for .lua files
   - Preserves script state during reloads
   - Error reporting and fallback mechanisms

### Integration Points

1. **HybridGameLoop Integration**
   ```rust
   // In system scheduler
   scheduler.add_system(Box::new(LuaScriptSystem::new()));
   ```

2. **Hot Reload Integration**
   ```rust
   // Script-specific hot reload handler
   hot_reload_manager.register_handler(AssetType::Script, script_reload_handler);
   ```

3. **ECS Component Registration**
   ```rust
   // Register script component
   world.register_component::<LuaScript>();
   ```

## Implementation Plan

### Phase 28.1: Foundation (Week 1)
- [ ] Create LuaScriptSystem integrated with HybridGameLoop
- [ ] Implement basic script lifecycle (init, update, destroy)
- [ ] Add script component with execution order support
- [ ] Basic Lua bindings for logging and math

### Phase 28.2: ECS Integration (Week 2)  
- [ ] Lua bindings for ECS entities and components
- [ ] Transform component access (position, rotation, scale)
- [ ] Entity queries and component manipulation
- [ ] Script-to-script communication patterns

### Phase 28.3: Hot Reload (Week 3)
- [ ] Integrate script hot reload with existing system
- [ ] State preservation across reloads
- [ ] Error handling and recovery
- [ ] Performance optimization

### Phase 28.4: Advanced Features (Week 4)
- [ ] Event system integration
- [ ] Asset loading from scripts
- [ ] Coroutine support for async operations
- [ ] Performance profiling and optimization

## Technical Specifications

### Script Lifecycle API
```lua
-- Basic script structure
local MyScript = {}

function MyScript:init()
    -- Called once when entity/script is created
    self.start_time = engine.time.total_time
    print("Script initialized!")
end

function MyScript:update(delta_time)
    -- Called every frame
    local transform = self.entity:get_component("Transform")
    transform.position[1] = transform.position[1] + delta_time
end

function MyScript:destroy()
    -- Called when entity/script is removed
    print("Script destroyed!")
end

return MyScript
```

### ECS Access Pattern
```lua
-- Component manipulation
local transform = entity:get_component("Transform")
transform.position = {10.0, 5.0, 0.0}
transform.rotation = {0.0, 0.0, 0.0, 1.0}

-- Entity queries
local enemies = engine.ecs:query("Transform", "Health")
for entity, transform, health in enemies do
    if health.current <= 0 then
        entity:destroy()
    end
end
```

### Hot Reload Support
```lua
-- Persistent data across reloads
persistent_data = persistent_data or {
    player_score = 0,
    level_state = "playing"
}

-- Reload counter for debugging
reload_count = (reload_count or 0) + 1
print("Script reloaded " .. reload_count .. " times")
```

## Success Criteria

### Functional Requirements
1. ✅ Scripts can be attached to entities via LuaScript component
2. ✅ Lifecycle methods (init, update, destroy) work correctly
3. ✅ Scripts can access and modify entity components
4. ✅ Hot reload preserves script state
5. ✅ Execution order is deterministic and configurable
6. ✅ Error handling prevents engine crashes

### Performance Requirements
1. ✅ Script execution overhead < 5% of frame time
2. ✅ Hot reload completes within 100ms
3. ✅ Memory usage scales linearly with script count
4. ✅ No frame drops during script compilation

### Developer Experience Requirements
1. ✅ Clear error messages with line numbers
2. ✅ IntelliSense/autocomplete for engine API
3. ✅ Debugging support with variable inspection
4. ✅ Hot reload works reliably without state loss

## Testing Strategy

### Unit Tests
- Script lifecycle method execution
- Component access and modification
- Error handling and recovery
- Hot reload state preservation

### Integration Tests
- ECS query functionality from Lua
- Multi-script execution order
- Performance under load
- Memory leak detection

### Performance Tests
- Script execution benchmarks
- Hot reload timing
- Memory usage profiling
- Frame time impact analysis

## Risks and Mitigation

### Technical Risks
1. **Lua-Rust Interop Overhead**
   - Mitigation: Use mlua's efficient UserData bindings
   - Benchmark critical paths early

2. **Hot Reload State Corruption**
   - Mitigation: Robust state capture/restore mechanisms
   - Fallback to script restart on errors

3. **Script Error Crashes**
   - Mitigation: Comprehensive error handling in all Lua calls
   - Sandbox unsafe operations

### Development Risks
1. **API Design Complexity**
   - Mitigation: Start with minimal API, iterate based on usage
   - Follow established patterns from other engines

2. **Integration Complexity**
   - Mitigation: Incremental integration with thorough testing
   - Maintain backward compatibility

## Future Enhancements (Post-Phase 28)

1. **Visual Scripting**: Node-based visual scripting interface
2. **Script Debugging**: Integrated debugger with breakpoints
3. **Asset Scripting**: Scripts for asset import/processing
4. **Multiplayer Scripting**: Network-aware script execution
5. **Script Optimization**: JIT compilation and caching

## Dependencies

- **Phase 27**: Unified architecture and hot reload system
- **mlua**: Rust-Lua binding library
- **engine-ecs-core**: ECS system for component access
- **engine-runtime-core**: Game loop integration

## Resources

- [mlua Documentation](https://docs.rs/mlua/)
- [Lua 5.4 Reference Manual](https://www.lua.org/manual/5.4/)
- [Game Programming Patterns - Command](http://gameprogrammingpatterns.com/command.html)
- [Unity MonoBehaviour Lifecycle](https://docs.unity3d.com/Manual/ExecutionOrder.html)

---

**Phase 28 Status**: Planning Complete
**Next Phase**: Implementation begins with LuaScriptSystem foundation
**Estimated Duration**: 4 weeks
**Priority**: High - Core engine feature