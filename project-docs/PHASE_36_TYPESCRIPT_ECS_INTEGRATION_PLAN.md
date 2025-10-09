# Phase 36: TypeScript ECS Integration Plan

## Overview
This phase implements missing ECS bindings for TypeScript scripts to access and modify entity components, particularly Transform components for position manipulation.

## Project Context
Previous phases have established:
- Phase 32-35: TypeScript compilation and execution infrastructure
- Strong V8 integration with TypeScript compilation pipeline
- API surface definitions in `engine.d.ts`
- Script lifecycle management (init, update, destroy)

## Current State Analysis

### ✅ Working Infrastructure
- **V8 + TypeScript Pipeline**: SWC compilation + V8 execution
- **Script System**: TypeScript script compilation and hot reload
- **API Definitions**: Complete type definitions in `assets/scripts/engine.d.ts`
- **Console Integration**: `console.log()` working properly
- **Basic Bindings**: Input, time, and debug APIs partially implemented

### ❌ Critical Missing Components
- **Entity Access**: `Engine.world.getCurrentEntity()` returns undefined
- **Component Access**: `entity.getComponent<Transform>()` not implemented
- **ECS Integration**: No connection between scripts and actual ECS World
- **Transform Bindings**: Cannot modify entity positions from scripts

## Implementation Gaps Detail

### 1. Entity Access Missing
**File**: `crates/implementation/engine-scripting/src/typescript_script_system.rs:882-887`
```rust
// Current mock implementation
let create_entity_fn = v8::Function::new(scope, |scope, _args, mut rv| {
    let entity_id = v8::Number::new(scope, 1.0); // ❌ Mock data
    rv.set(entity_id.into());
    log::debug!("World.createEntity called, returned mock entity ID: 1");
});
```

### 2. Component System Not Connected  
**File**: `assets/scripts/typescript_entity_controller.ts:6-25`
```typescript
// Intended usage - currently non-functional
export class EntityController {
    init(): void {
        const entity = Engine.world.getCurrentEntity(); // ❌ Returns undefined
        this.transform = entity.getComponent<Transform>(); // ❌ Not implemented
    }
    
    update(deltaTime: number): void {
        this.transform.position.z -= deltaTime * 5.0; // ❌ Cannot modify position
    }
}
```

## Phase 36 Implementation Plan

### Milestone 1: Entity Context System
**Timeline**: 3-4 days
**Goal**: Provide current entity context to scripts

#### Tasks:
1. **Script Context Enhancement**
   - Add entity context to TypeScript script execution
   - Pass current entity ID to script instances
   - Store entity context in V8 isolate global state

2. **getCurrentEntity() Implementation**
   - Implement `Engine.world.getCurrentEntity()` binding
   - Return actual Entity object wrapping current entity ID
   - Connect to real ECS entity system

**Files to Modify**:
- `crates/implementation/engine-scripting/src/typescript_script_system.rs`
- `crates/implementation/engine-scripting-typescript/src/bindings/ecs.rs`

### Milestone 2: Component Access System  
**Timeline**: 4-5 days
**Goal**: Enable scripts to get/set components

#### Tasks:
1. **Entity Interface Implementation**
   - Implement `entity.getComponent<T>()` method
   - Implement `entity.addComponent<T>()` method
   - Implement `entity.removeComponent<T>()` method
   - Connect to actual ECS component storage

2. **Transform Component Bindings**
   - Bind Transform component to TypeScript
   - Enable position/rotation/scale access
   - Support real-time component modification

3. **Component Serialization**
   - Convert Rust components to JavaScript objects
   - Convert JavaScript objects back to Rust components
   - Handle Vector3 and other engine types

**Files to Modify**:
- `crates/implementation/engine-scripting-typescript/src/bindings/ecs.rs`
- `crates/implementation/engine-scripting/src/typescript_script_system.rs`

### Milestone 3: ECS World Integration
**Timeline**: 3-4 days  
**Goal**: Connect scripts to actual ECS World

#### Tasks:
1. **World Access Integration**
   - Connect V8 bindings to actual ECS World instance
   - Implement safe world access from scripts
   - Handle world locking and thread safety

2. **Entity Management**
   - Implement `Engine.world.createEntity()`
   - Implement `Engine.world.destroyEntity()`
   - Implement `Engine.world.getEntity(id)`

3. **Component Queries**
   - Basic component iteration support
   - Entity filtering by component type

**Files to Modify**:
- `crates/implementation/engine-scripting/src/typescript_script_system.rs`
- `crates/application/engine-editor-framework/src/unified_coordinator.rs`

### Milestone 4: Real-Time Position Modification
**Timeline**: 2-3 days
**Goal**: Working entity position modification from scripts

#### Tasks:
1. **Transform Component Integration**
   - Real Transform component access
   - Position modification that affects rendering
   - Rotation and scale modification

2. **Example Script Implementation**
   - Update `typescript_entity_controller.ts` to work properly
   - Create movement examples
   - Create rotation examples

3. **Testing and Validation**
   - Verify position changes reflect in editor
   - Test with multiple entities
   - Performance validation

**Files to Modify**:
- `assets/scripts/typescript_entity_controller.ts`
- `assets/scripts/engine.d.ts` (if needed)

## Implementation Architecture

### TypeScript API Surface
```typescript
// Target working API
declare const Engine: {
    world: {
        getCurrentEntity(): Entity;     // ← TO IMPLEMENT
        getEntity(id: number): Entity | null;
        createEntity(): Entity;
        destroyEntity(entity: Entity): void;
    }
};

interface Entity {
    id(): number;
    getComponent<T>(): T | null;        // ← TO IMPLEMENT  
    addComponent<T>(component: T): void;
    removeComponent<T>(): boolean;
}

interface Transform {
    position: Vector3;                  // ← TO IMPLEMENT
    rotation: Vector3;
    scale: Vector3;
}
```

### V8 Binding Architecture
```rust
// Target implementation structure
pub struct TypeScriptEntityContext {
    entity_id: Entity,
    world: Arc<Mutex<World>>,
}

impl TypeScriptEntityContext {
    pub fn bind_to_v8(&self, scope: &mut v8::HandleScope) -> Result<(), ScriptError> {
        // Bind Entity interface methods
        // Bind Transform component access
        // Connect to ECS World
    }
}
```

## Testing Strategy

### Unit Tests
- Component serialization/deserialization
- Entity context management
- V8 binding correctness

### Integration Tests  
- Script-to-ECS communication
- Real-time component modification
- Multi-entity scenarios

### Example Scripts
- Basic position movement
- Rotation animations
- Input-driven entity control

## Success Criteria

### ✅ Phase Complete When:
1. **Entity Access**: `Engine.world.getCurrentEntity()` returns actual entity
2. **Component Access**: `entity.getComponent<Transform>()` returns real transform
3. **Position Modification**: Scripts can modify entity positions and see changes in editor
4. **Real ECS Integration**: Scripts operate on actual ECS World, not mock data
5. **Working Examples**: `typescript_entity_controller.ts` successfully moves entities

### Performance Requirements
- Component access under 1ms per call
- No noticeable impact on frame rate
- Memory usage remains stable during script execution

## Risk Assessment

### High Risk Areas
- **Thread Safety**: V8 + ECS World concurrent access
- **Memory Management**: Component lifetime in V8 heap
- **Performance**: V8 ↔ Rust boundary overhead

### Mitigation Strategies  
- Use Arc<Mutex<World>> for safe world access
- Implement component copying instead of direct references
- Profile and optimize critical paths

## Dependencies
- Existing V8 + TypeScript infrastructure (Phase 32-35)
- ECS World architecture
- Component serialization system

## Next Phase Preview
**Phase 37**: Advanced Scripting Features
- Physics integration (`Engine.physics.applyForce()`)
- Input handling (`Engine.input.isKeyDown()`)
- Event system (`Engine.events.addEventListener()`)
- Performance optimization and caching