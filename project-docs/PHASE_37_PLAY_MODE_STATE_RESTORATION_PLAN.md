# PHASE 37: Play Mode State Restoration System

## Overview
Implement Unity-style comprehensive state restoration for the Longhorn Game Engine's play mode functionality. When users exit play mode, all entity states should automatically revert to their pre-play configuration.

## Problem Statement
Currently, when entering play mode and making changes to entities (position, rotation, scale, etc.), these changes persist after stopping play mode. This differs from industry-standard behavior where play mode changes are temporary and automatically reverted.

## Research Findings
Based on analysis of major game engines:

### Unity Engine
- Uses comprehensive serialization/deserialization snapshots
- Captures complete world state before play mode
- Restores from snapshot when exiting play mode
- Handles nested prefabs and complex hierarchies

### Unreal Engine
- Implements "Keep Simulation Changes" toggle
- Allows selective preservation of play mode changes
- Uses transaction system for state management

### Godot Engine
- Automatic scene tree state restoration
- Node-based snapshot system
- Memory-efficient differential snapshots

## Proposed Architecture

### Core Components

#### 1. WorldSnapshot System
```rust
pub struct WorldSnapshot {
    entities: HashMap<EntityId, EntityMemento>,
    timestamp: Instant,
    snapshot_id: Uuid,
}

pub struct EntityMemento {
    entity_id: EntityId,
    components: HashMap<ComponentTypeId, Vec<u8>>, // Serialized component data
    children: Vec<EntityId>,
    parent: Option<EntityId>,
}
```

#### 2. PlayStateManager Integration
- Extend existing `PlayStateManager` with snapshot functionality
- Trigger snapshot capture on `start()` calls
- Trigger restoration on `stop()` calls
- Handle pause/resume without state changes

#### 3. Component Serialization
- Leverage existing `serde` infrastructure
- Implement `WorldSnapshot` trait for all restorable components
- Support for Transform, Name, TypeScriptScript, and custom components

### Implementation Strategy

#### Phase 1: Core Infrastructure (Week 1)
- [ ] Create `WorldSnapshot` and `EntityMemento` structures
- [ ] Implement basic serialization for Transform components
- [ ] Add snapshot capture/restore to `PlayStateManager`
- [ ] Create unit tests for snapshot functionality

#### Phase 2: Component Support (Week 2)
- [ ] Extend serialization to Name and TypeScriptScript components
- [ ] Implement differential snapshots for performance
- [ ] Add memory management and cleanup
- [ ] Handle entity hierarchy preservation

#### Phase 3: Integration (Week 3)
- [ ] Integrate with existing action processing system
- [ ] Add snapshot triggers to StartPlay/StopPlay actions
- [ ] Update UnifiedEditorCoordinator integration
- [ ] Implement error handling and recovery

#### Phase 4: Performance & Polish (Week 4)
- [ ] Optimize memory usage for large scenes
- [ ] Add progress indicators for large snapshots
- [ ] Implement incremental backup system
- [ ] Add user settings for snapshot behavior

## Technical Considerations

### Memory Management
- Snapshot size proportional to scene complexity
- Implement compression for large component data
- Use weak references where possible
- Automatic cleanup of old snapshots

### Performance Impact
- Snapshot capture: O(n) where n = number of entities
- Memory usage: ~2x scene size during play mode
- Restoration time: <100ms for typical scenes
- Background compression to reduce memory footprint

### Error Handling
- Graceful degradation if snapshot fails
- Fallback to current behavior if restoration fails
- User notification for snapshot-related errors
- Automatic recovery from corrupted snapshots

## File Structure
```
crates/application/engine-editor-framework/src/
├── play_state.rs (existing - extend)
├── world_snapshot.rs (new)
├── entity_memento.rs (new)
└── snapshot_manager.rs (new)

crates/core/engine-ecs-core/src/
├── serialization/ (new)
│   ├── mod.rs
│   ├── component_serializer.rs
│   └── world_serializer.rs
```

## Success Criteria
1. ✅ All entity changes made during play mode are automatically reverted on stop
2. ✅ Performance impact <10% for scenes with <1000 entities
3. ✅ Memory usage increase <50% during play mode
4. ✅ Restoration completes in <100ms for typical scenes
5. ✅ System works with all existing component types
6. ✅ Comprehensive test coverage for edge cases

## Dependencies
- Existing PlayStateManager system
- Engine ECS core serialization infrastructure
- UnifiedEditorCoordinator action processing
- Component type registration system

## Timeline
- **Total Duration**: 4 weeks
- **Milestone 1**: Core snapshot infrastructure (Week 1)
- **Milestone 2**: Component serialization complete (Week 2) 
- **Milestone 3**: Full integration working (Week 3)
- **Milestone 4**: Performance optimized and polished (Week 4)

## Risks & Mitigation
- **Risk**: Large memory usage for complex scenes
  - **Mitigation**: Implement compression and differential snapshots
- **Risk**: Slow restoration for large scenes
  - **Mitigation**: Background processing and progress indicators
- **Risk**: Component serialization failures
  - **Mitigation**: Graceful fallback and error recovery system