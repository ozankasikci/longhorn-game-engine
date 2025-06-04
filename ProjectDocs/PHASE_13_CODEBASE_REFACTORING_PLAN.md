# Phase 13: Codebase Refactoring and Organization

## Overview
Major refactoring phase to improve code organization, fix architecture violations, and split oversized files for better maintainability.

## Phase Duration
4 weeks (broken into 4 sub-phases)

## Current Issues
1. **Oversized Files**:
   - `ecs_v2.rs` - 1,619 lines
   - `scene_view_impl.rs` - 1,018 lines
   - 6 other files > 500 lines

2. **Architecture Violations**:
   - Core crates contain implementation details
   - Missing implementation crates
   - Inconsistent naming conventions

3. **Poor Documentation**:
   - Only 1 README across 25 crates
   - Limited test organization

## Sub-Phases

### Phase 13.1: ECS V2 Modularization (Week 1)
**Goal**: Split the monolithic ECS v2 implementation into logical modules

**Tasks**:
1. Create new module structure under `ecs_v2/`
2. Extract Entity system (150 lines)
3. Extract Component system (300 lines)
4. Extract Archetype system (250 lines)
5. Extract World implementation (400 lines)
6. Extract Query system (300 lines)
7. Extract Bundle system (120 lines)
8. Create compatibility re-exports
9. Update all imports and run tests

**Deliverables**:
- Modular ECS structure with no file > 500 lines
- All tests passing
- Documentation for each module

### Phase 13.2: Scene View Refactoring (Week 2)
**Goal**: Break down the scene view implementation into focused modules

**Tasks**:
1. Create `rendering/` submodule structure
2. Extract grid rendering (200 lines)
3. Extract mesh rendering (300 lines)
4. Extract entity rendering (150 lines)
5. Extract projection utilities (150 lines)
6. Refactor main scene_view_impl.rs to orchestrator (200 lines)
7. Update scene view imports
8. Test all rendering functionality

**Deliverables**:
- Modular scene rendering system
- Improved code organization
- Enhanced maintainability

### Phase 13.3: Architecture Compliance (Week 3)
**Goal**: Fix violations of the 4-tier architecture

**Tasks**:
1. Move camera optimization from core to impl (522 lines)
2. Move camera culling implementation details
3. Create `engine-resource-impl` crate
4. Move resource cache from core to impl
5. Create `engine-math-impl` crate
6. Move complex math operations
7. Update crate dependencies
8. Fix import paths

**Deliverables**:
- Proper separation of core traits and implementations
- New implementation crates
- Clean architecture boundaries

### Phase 13.4: Documentation and Standardization (Week 4)
**Goal**: Improve documentation and standardize the codebase

**Tasks**:
1. Add README.md to all 25 crates
2. Standardize crate naming (add `-impl` suffix)
3. Create `engine-test-utils` crate
4. Consolidate common test code
5. Add module-level documentation
6. Create architecture diagram
7. Update main README
8. Run final integration tests

**Deliverables**:
- Complete documentation
- Standardized naming
- Improved test organization
- Architecture documentation

## Success Criteria
- No single file exceeds 500 lines (except generated code)
- All crates follow the 4-tier architecture
- Every crate has a README.md
- Consistent naming scheme across all crates
- All tests pass
- Improved compilation times

## Risk Mitigation
1. **Breaking Changes**: Use re-exports for backward compatibility
2. **Import Chaos**: Update imports incrementally with automated tools
3. **Test Failures**: Run tests after each extraction
4. **Performance**: Profile before and after changes

## Dependencies
- Phase 12 (current phase - to be defined) must be complete
- No active feature development during refactoring

## Next Phase
After Phase 13, the codebase will be ready for:
- Phase 14: Performance Optimization
- Phase 15: Advanced Rendering Features
- Phase 16: Mobile Platform Support