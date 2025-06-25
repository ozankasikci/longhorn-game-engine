# Phase 24: Core Systems Decoupling Plan

## Overview
Remove unnecessary dependencies between core systems, particularly the tight coupling between renderer-core and ECS-core, to improve modularity and testability.

## Goals
1. Remove ECS dependency from renderer-core
2. Use trait objects or generic parameters for loose coupling
3. Enable independent testing of core systems
4. Improve system composability and reusability

## Current State Analysis

### Dependency Issues

1. **Renderer → ECS Coupling**:
   ```rust
   // Current: engine-renderer-core/Cargo.toml
   [dependencies]
   engine-ecs-core = { path = "../engine-ecs-core" }
   ```
   - Renderer knows about entities and components
   - Tight coupling makes testing difficult
   - Can't use renderer without ECS

2. **Cross-Cutting Concerns**:
   - Transform components used by multiple systems
   - Resource handles shared across crates
   - Event system dependencies scattered

3. **Circular Dependency Risks**:
   - Materials need renderer types
   - Renderer needs material types
   - Both need component types

## Implementation Plan

### Step 1: Identify Coupling Points (Week 1)

1. **Analyze renderer-core usage of ECS**:
   - Scan for Entity/Component references
   - Identify shared types
   - Document usage patterns

2. **Map cross-system dependencies**:
   - Create dependency graph
   - Identify circular risks
   - Find abstraction boundaries

3. **Categorize coupling types**:
   - Data sharing (transforms, handles)
   - Behavioral coupling (systems)
   - Type dependencies (components)

### Step 2: Create Abstraction Interfaces (Week 1-2)

1. **Define renderable trait**:
   ```rust
   // In engine-renderer-core
   pub trait Renderable {
       fn transform(&self) -> &Transform;
       fn mesh_handle(&self) -> Option<MeshHandle>;
       fn material_handle(&self) -> Option<MaterialHandle>;
   }
   ```

2. **Create transform abstraction**:
   ```rust
   // Move to engine-math-core
   pub trait TransformProvider {
       fn world_matrix(&self) -> Mat4;
       fn position(&self) -> Vec3;
       fn rotation(&self) -> Quat;
       fn scale(&self) -> Vec3;
   }
   ```

3. **Define query abstraction**:
   ```rust
   // In engine-renderer-core
   pub trait RenderableQuery {
       type Item: Renderable;
       type Iter: Iterator<Item = Self::Item>;
       
       fn iter(&self) -> Self::Iter;
   }
   ```

### Step 3: Refactor Renderer Core (Week 2-3)

1. **Remove direct ECS imports**:
   - Replace Entity with opaque ID type
   - Use traits instead of concrete components
   - Make renderer ECS-agnostic

2. **Update render extraction**:
   ```rust
   // Before
   pub fn extract_renderables(world: &World) -> Vec<RenderData> {
       world.query::<(&Transform, &MeshComponent)>()
           .iter()
           .map(|(t, m)| RenderData::new(t, m))
           .collect()
   }
   
   // After
   pub fn extract_renderables<Q: RenderableQuery>(
       query: &Q
   ) -> Vec<RenderData> {
       query.iter()
           .map(|r| RenderData::from_renderable(r))
           .collect()
   }
   ```

3. **Create adapter layer**:
   ```rust
   // In engine-runtime or integration layer
   pub struct EcsRenderableQuery<'a> {
       world: &'a World,
   }
   
   impl<'a> RenderableQuery for EcsRenderableQuery<'a> {
       // Implement trait to bridge ECS and renderer
   }
   ```

### Step 4: Decouple Other Systems (Week 3-4)

1. **Physics-ECS decoupling**:
   - Define PhysicsBody trait
   - Remove direct component access
   - Use transform providers

2. **Audio-ECS decoupling**:
   - Create AudioSource trait
   - Abstract spatial queries
   - Remove entity references

3. **Camera-ECS decoupling**:
   - Define ViewProvider trait
   - Abstract camera queries
   - Use generic parameters

### Step 5: Create Integration Layer (Week 4-5)

1. **System orchestration**:
   ```rust
   // In engine-runtime
   pub struct SystemIntegration {
       ecs: Box<dyn EcsSystem>,
       renderer: Box<dyn RenderSystem>,
       physics: Box<dyn PhysicsSystem>,
   }
   
   impl SystemIntegration {
       pub fn update(&mut self) {
           // Extract data from ECS
           let renderables = self.create_renderable_query();
           
           // Update renderer with extracted data
           self.renderer.render(&renderables);
       }
   }
   ```

2. **Data synchronization**:
   - Create data transfer objects
   - Define update protocols
   - Handle system ordering

3. **Event routing**:
   - Central event dispatcher
   - System-specific event types
   - Loose coupling via traits

### Step 6: Testing Infrastructure (Week 5-6)

1. **Mock implementations**:
   ```rust
   pub struct MockRenderableQuery {
       items: Vec<MockRenderable>,
   }
   
   #[test]
   fn test_renderer_without_ecs() {
       let query = MockRenderableQuery::new();
       let renderer = Renderer::new();
       renderer.render(&query);
   }
   ```

2. **Integration tests**:
   - Test each system independently
   - Verify integration layer
   - Check performance impact

3. **Benchmarks**:
   - Measure abstraction overhead
   - Compare with direct coupling
   - Optimize hot paths

## Migration Strategy

### Phase 1: Add Abstractions
1. Create traits alongside existing code
2. Implement adapters for current usage
3. No breaking changes

### Phase 2: Update Systems
1. Migrate one system at a time
2. Update tests incrementally
3. Maintain compatibility layer

### Phase 3: Remove Old Code
1. Delete direct dependencies
2. Clean up adapters
3. Finalize abstractions

## Architecture Patterns

### Dependency Injection
```rust
pub struct RendererBuilder {
    device: Option<Box<dyn GraphicsDevice>>,
    query_provider: Option<Box<dyn QueryProvider>>,
}

impl RendererBuilder {
    pub fn with_query_provider<Q: QueryProvider>(mut self, provider: Q) -> Self {
        self.query_provider = Some(Box::new(provider));
        self
    }
    
    pub fn build(self) -> Result<Renderer> {
        // Construct renderer with injected dependencies
    }
}
```

### Service Locator
```rust
pub struct ServiceRegistry {
    services: HashMap<TypeId, Box<dyn Any>>,
}

impl ServiceRegistry {
    pub fn register<T: 'static>(&mut self, service: T) {
        self.services.insert(TypeId::of::<T>(), Box::new(service));
    }
    
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.services.get(&TypeId::of::<T>())
            .and_then(|s| s.downcast_ref())
    }
}
```

## Success Criteria

1. **Decoupling Achieved**:
   - No direct ECS imports in renderer-core
   - All systems testable in isolation
   - Clear abstraction boundaries

2. **Performance Maintained**:
   - No significant overhead from abstractions
   - Inlining preserves performance
   - Zero-cost abstractions verified

3. **Improved Testability**:
   - Unit tests for each core system
   - Mock implementations available
   - Integration tests simplified

4. **Better Composability**:
   - Systems can be mixed and matched
   - Alternative implementations possible
   - Plugin architecture enabled

## Risks and Mitigations

1. **Over-Abstraction**:
   - Risk: Too many trait layers
   - Mitigation: Keep abstractions minimal and focused

2. **Performance Regression**:
   - Risk: Virtual dispatch overhead
   - Mitigation: Use generics for hot paths, profile regularly

3. **Complexity Increase**:
   - Risk: Harder to understand flow
   - Mitigation: Clear documentation, sequence diagrams

## Long-Term Benefits

1. **Alternative ECS Support**:
   - Could swap to different ECS
   - Support multiple ECS systems
   - ECS-free renderer mode

2. **Modular Architecture**:
   - Pick and choose systems
   - Easier to add new systems
   - Better for embedded use

3. **Testing and Quality**:
   - Comprehensive unit tests
   - Better code coverage
   - Faster test execution