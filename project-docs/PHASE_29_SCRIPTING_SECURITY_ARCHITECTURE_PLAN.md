# Phase 29: Scripting Security & Architecture Fixes

## Overview
Phase 29 focuses on critical security vulnerabilities and architectural problems in the scripting system that make it unsuitable for production use. This phase addresses the most severe issues that pose security risks and architectural debt.

## Previous Phase Context
Building on Phase 28's Lua scripting integration, we've identified critical security and architectural issues that need immediate attention before further development.

## Goals
1. **Eliminate security vulnerabilities** - Remove unsafe Lua standard library access
2. **Implement proper sandboxing** - Add resource limits and script isolation
3. **Fix architectural problems** - Remove global state abuse and improve separation of concerns
4. **Establish proper error handling** - Replace silent failures with comprehensive error management

## Current Problems

### Critical Security Issues
- **Unsafe Lua stdlib access**: `StdLib::OS` and `StdLib::IO` expose dangerous functions
- **No sandboxing**: Scripts run with full Lua environment access
- **No resource limits**: Vulnerable to infinite loops and memory exhaustion
- **Global state pollution**: Scripts can interfere with each other

### Architecture Issues
- **Global state abuse**: `SHARED_COMPONENT_STATE` and `CONSOLE_MESSAGES` break ECS patterns
- **Poor separation**: Overlapping responsibilities between engine/system/manager
- **Multiple sources of truth**: Scripts stored in multiple places causing inconsistency

### Files to Modify
- `crates/implementation/engine-scripting/src/runtime.rs`
- `crates/implementation/engine-scripting/src/manager.rs` 
- `crates/implementation/engine-scripting/src/shared_state.rs`
- `crates/implementation/engine-scripting/src/lua_script_system.rs`
- `crates/implementation/engine-scripting/src/lib.rs`

## Implementation Tasks

### Task 1: Implement Lua Sandboxing (Priority: Critical)

#### 1.1 Safe Standard Library Configuration
**Location**: `runtime.rs`
```rust
// Replace unsafe configuration with safe subset
let lua = Lua::new_with(
    StdLib::TABLE | StdLib::STRING | StdLib::MATH | 
    StdLib::COROUTINE | StdLib::UTF8,
    LuaOptions::default()
)?;
```

#### 1.2 Resource Limits System
**New file**: `resource_limits.rs`
```rust
pub struct ScriptResourceLimits {
    pub max_memory_mb: usize,
    pub max_execution_time_ms: u64,
    pub max_recursion_depth: u32,
}
```

#### 1.3 Sandboxed Execution Context
**Location**: `runtime.rs`
- Add timeout interruption hooks
- Implement memory usage monitoring
- Add execution time tracking

### Task 2: Eliminate Global State (Priority: High)

#### 2.1 Remove SHARED_COMPONENT_STATE
**Location**: `shared_state.rs`
- Replace global HashMap with ECS-integrated component manager
- Direct component access through ECS world

#### 2.2 Replace CONSOLE_MESSAGES Global
**Location**: `shared_state.rs`
- Convert to ECS resource system
- Implement bounded message queue
- Add proper cleanup

### Task 3: Architecture Refactoring (Priority: High)

#### 3.1 Separate Concerns
**Files**: `runtime.rs`, `manager.rs`, `lua_script_system.rs`
- `ScriptEngine`: Execution only
- `ScriptManager`: Lifecycle management only
- `ScriptSystem`: ECS integration only

#### 3.2 Single Source of Truth
**Location**: `manager.rs`
- Scripts stored only in ScriptManager
- Entity relationships only in ScriptSystem
- Component data only in ECS World

### Task 4: Comprehensive Error Handling (Priority: Medium)

#### 4.1 Error Type System
**New file**: `error.rs`
```rust
#[derive(Debug, thiserror::Error)]
pub enum ScriptError {
    #[error("Security violation: {violation}")]
    SecurityViolation { violation: String },
    #[error("Resource limit exceeded: {limit_type}")]
    ResourceLimitExceeded { limit_type: String },
    // ... other error types
}
```

#### 4.2 Error Recovery
**Location**: `lua_script_system.rs`
- Replace `.ok()` calls with proper error handling
- Add error recovery mechanisms
- Implement script disabling for security violations

### Task 5: API Security Hardening (Priority: Medium)

#### 5.1 Safe API Design
**Location**: `api.rs`, `bindings.rs`
- Remove unsafe function exposure
- Add permission-based access control
- Validate all API calls

## Testing Requirements

### Security Testing
1. **Penetration tests** for unsafe function access
2. **Resource exhaustion tests** for limits
3. **Script isolation tests** for sandboxing
4. **API security tests** for permission validation

### Architecture Testing
1. **Unit tests** for each refactored component
2. **Integration tests** with real ECS systems
3. **Regression tests** for existing functionality
4. **Performance tests** to ensure no degradation

## Success Criteria

### Security
- [ ] No access to `os.*` or `io.*` functions
- [ ] Resource limits properly enforced (memory, time, recursion)
- [ ] Scripts cannot access other scripts' data
- [ ] Security audit passes all tests

### Architecture
- [ ] No global state variables remain
- [ ] Clean separation between engine/manager/system
- [ ] Single source of truth for all data
- [ ] Proper error handling throughout

### Compatibility
- [ ] Existing safe scripts continue to work
- [ ] Performance regression < 5%
- [ ] All tests pass

## Timeline
- **Week 1**: Implement sandboxing and resource limits
- **Week 2**: Remove global state and refactor architecture
- **Week 3**: Add comprehensive error handling and testing

## Risks and Mitigation

### Risk: Breaking Existing Scripts
- **Mitigation**: Comprehensive testing with existing scripts
- **Fallback**: Feature flags for gradual rollout

### Risk: Performance Impact
- **Mitigation**: Benchmark at each step
- **Fallback**: Optimize hot paths, configurable limits

### Risk: Complex Refactoring
- **Mitigation**: Incremental changes with testing
- **Fallback**: Keep old system as backup during transition

## Dependencies
- None (this is foundational work)

## Next Phase
Phase 30 will focus on memory management and performance improvements once security and architecture are solid.

## Notes
This phase is critical for system security and must be completed before any production deployment or further feature development.