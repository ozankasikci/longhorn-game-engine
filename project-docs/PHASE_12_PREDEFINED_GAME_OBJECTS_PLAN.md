# Phase 12: Predefined Game Objects with Mesh and Material System

## Overview
This phase focuses on implementing Unity-style predefined game objects (starting with Cube) using a proper component-based mesh and material rendering system. We'll establish the foundation for how mesh data, rendering properties, and materials work together in the Longhorn Game Engine.

## Research Summary

### Unity's Architecture
Unity separates mesh handling into two components:
- **MeshFilter**: Holds reference to mesh data (geometry, vertices, indices)
- **MeshRenderer**: Handles rendering properties and material assignments

This separation follows the Single Responsibility Principle and allows:
- Shared mesh data across multiple objects
- Individual rendering properties per object
- Clean separation between geometry and appearance

### Best Practices from Industry
1. **Component Separation**: Keep mesh geometry separate from rendering properties
2. **Material System**: Materials should be independent resources that can be shared or instanced
3. **Resource Management**: Use proxy/handle system for actual GPU resources
4. **Flexibility**: Support both shared and per-instance material properties

## Proposed Architecture

### Component Structure

```rust
// Core trait for mesh data providers
trait MeshProvider {
    fn get_mesh_data(&self) -> MeshData;
}

// Component that holds mesh reference (like Unity's MeshFilter)
struct MeshFilter {
    mesh: MeshHandle,  // Reference to mesh asset
}

// Component that handles rendering (like Unity's MeshRenderer)
struct MeshRenderer {
    materials: Vec<MaterialHandle>,  // Materials for each submesh
    cast_shadows: bool,
    receive_shadows: bool,
    layer_mask: u32,
}

// Actual mesh data (stored in resource system)
struct MeshData {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    submeshes: Vec<SubMesh>,
    bounds: AABB,
}

// Material definition
struct Material {
    shader: ShaderHandle,
    properties: MaterialProperties,
    render_state: RenderState,
}
```

### Predefined Game Objects

```rust
// Factory for creating predefined game objects
impl GameObjectFactory {
    pub fn create_cube(world: &mut World, position: Vec3, size: f32) -> Entity {
        let entity = world.spawn();
        
        // Add required components
        world.add_component(entity, Transform::new(position));
        world.add_component(entity, MeshFilter::new(MeshLibrary::CUBE));
        world.add_component(entity, MeshRenderer::default());
        world.add_component(entity, Name::new("Cube"));
        
        entity
    }
}
```

## Implementation Plan

### Phase 12.1: Core Mesh Components (2-3 hours)
1. Create `MeshFilter` component in `engine-components-3d`
2. Create `MeshRenderer` component with material support
3. Define `MeshData` structure in `engine-geometry-core`
4. Implement mesh handle system in `engine-resource-core`

### Phase 12.2: Material System Foundation (2-3 hours)
1. Define `Material` structure in `engine-materials-core`
2. Implement material property system (uniforms, textures)
3. Create material handle/reference system
4. Add default material library

### Phase 12.3: Mesh Library and Primitives (2-3 hours)
1. Create `MeshLibrary` with predefined meshes
2. Implement procedural mesh generation for:
   - Cube
   - Sphere
   - Plane
   - Cylinder
   - Capsule
3. Store generated meshes in resource system

### Phase 12.4: Game Object Factory (1-2 hours)
1. Create `GameObjectFactory` in `engine-scene`
2. Implement factory methods for each primitive
3. Add component presets for common configurations
4. Support for custom initial properties

### Phase 12.5: Render System Integration (2-3 hours)
1. Update renderer to query `MeshFilter` + `MeshRenderer` pairs
2. Implement material binding in render pipeline
3. Support multiple materials per mesh (submeshes)
4. Handle shared vs instanced materials

### Phase 12.6: Editor Integration (1-2 hours)
1. Add "Create" menu with primitive options
2. Update inspector to show MeshFilter/MeshRenderer
3. Material assignment UI in inspector
4. Mesh statistics display

## Technical Decisions

### 1. Component Separation
**Decision**: Follow Unity's pattern of separate MeshFilter and MeshRenderer
**Rationale**: 
- Clean separation of concerns
- Allows mesh sharing between objects
- Easier to extend with additional renderer types

### 2. Material Ownership
**Decision**: Materials are separate resources, referenced by handle
**Rationale**:
- Enables material sharing
- Supports runtime material switching
- Allows for material instancing when needed

### 3. Mesh Storage
**Decision**: Meshes stored in central resource system, components hold handles
**Rationale**:
- Prevents duplication of mesh data
- Enables efficient GPU resource management
- Supports dynamic loading/unloading

### 4. Predefined Primitives
**Decision**: Generate primitives procedurally at startup
**Rationale**:
- No need to ship mesh files for basic shapes
- Can generate at different resolutions
- Consistent vertex format

## Migration Path

Current system uses a single `Mesh` component with embedded `MeshType`. Migration steps:

1. Keep existing `Mesh` component temporarily
2. Add new `MeshFilter`/`MeshRenderer` components
3. Update systems to check for both old and new components
4. Migrate existing code to use new components
5. Deprecate and remove old `Mesh` component

## Success Criteria

1. Can create cube game object with single function call
2. Cube renders with proper material in scene view
3. Inspector shows MeshFilter and MeshRenderer components
4. Can change material at runtime through inspector
5. Multiple cubes can share same mesh data
6. Performance is equal or better than current system

## Future Extensions

- Mesh LOD (Level of Detail) support
- Procedural mesh modification components
- Mesh combining/batching system
- Custom mesh importers
- Skinned mesh support (SkinnedMeshRenderer)
- Mesh instancing for performance