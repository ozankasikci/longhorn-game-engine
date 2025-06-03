# ECS Migration Prototype Code

## Quick Validation of the Clone-Based Approach

Here's a prototype to test if the clone-based migration will work:

```rust
// Step 1: Update Component trait (in engine-ecs-core/src/ecs_v2.rs)
pub trait Component: 'static + Send + Sync {
    fn type_id() -> TypeId where Self: Sized {
        TypeId::of::<Self>()
    }
    
    /// Clone this component as a type-erased box
    fn clone_boxed(&self) -> Box<dyn Component>;
    
    /// Clone this component into a pre-allocated box
    fn clone_into_box(&self, target: &mut Box<dyn Component>);
}

// Step 2: Add derive macro helper (create new file: engine-ecs-core/src/component_derive.rs)
/// Helper macro to implement Component trait with cloning
#[macro_export]
macro_rules! impl_component {
    ($type:ty) => {
        impl Component for $type {
            fn clone_boxed(&self) -> Box<dyn Component> {
                Box::new(self.clone())
            }
            
            fn clone_into_box(&self, target: &mut Box<dyn Component>) {
                if let Some(typed_target) = target.downcast_mut::<$type>() {
                    *typed_target = self.clone();
                }
            }
        }
    };
}

// Step 3: Update existing components
impl_component!(Transform);
impl_component!(Mesh);
impl_component!(Material);
impl_component!(Visibility);
impl_component!(Name);
impl_component!(Camera);
impl_component!(SpriteRenderer);

// Step 4: Add component info to track migrations
pub struct ComponentInfo {
    type_id: TypeId,
    clone_fn: fn(&ErasedComponentArray, usize) -> Option<Box<dyn Component>>,
}

// Step 5: Enhanced ErasedComponentArray with cloning
impl ErasedComponentArray {
    pub fn clone_component_boxed(&self, index: usize) -> Option<Box<dyn Component>> {
        // This would need to be implemented with the actual component type
        // For now, return None - would be filled in during registration
        None
    }
}

// Step 6: Migration implementation sketch
fn migrate_entity_to_new_archetype<T: Component>(
    &mut self,
    entity: Entity,
    old_location: EntityLocation, 
    target_archetype_id: ArchetypeId,
    new_component: T,
    new_component_ticks: ComponentTicks
) -> Result<(), &'static str> {
    // Get the old archetype
    let old_archetype = self.archetypes.get(&old_location.archetype_id)
        .ok_or("Old archetype not found")?;
    
    // Collect components to migrate
    let mut components_to_migrate: Vec<(TypeId, Box<dyn Component>, ComponentTicks)> = Vec::new();
    
    // For each component type in old archetype
    for (type_id, component_array) in &old_archetype.components {
        if let Some(component) = component_array.clone_component_boxed(old_location.index) {
            let ticks = component_array.get_ticks(old_location.index)
                .cloned()
                .unwrap_or_else(|| ComponentTicks::new(self.change_tick()));
            components_to_migrate.push((*type_id, component, ticks));
        }
    }
    
    // Remove entity from old archetype
    old_archetype.remove_entity(old_location.index);
    
    // Ensure target archetype exists
    if !self.archetypes.contains_key(&target_archetype_id) {
        self.archetypes.insert(
            target_archetype_id.clone(),
            Archetype::new(target_archetype_id.clone())
        );
    }
    
    // Add entity to new archetype
    let new_archetype = self.archetypes.get_mut(&target_archetype_id).unwrap();
    let new_index = new_archetype.add_entity(entity);
    
    // Add all migrated components
    for (type_id, component, ticks) in components_to_migrate {
        // This would need type-erased component addition
        // new_archetype.add_component_boxed(type_id, component, ticks);
    }
    
    // Add the new component
    new_archetype.add_component(new_component, new_component_ticks);
    
    // Update entity location
    self.entity_locations.insert(entity, EntityLocation {
        archetype_id: target_archetype_id,
        index: new_index,
    });
    
    Ok(())
}
```

## Alternative: Component Bundle Approach (Simpler)

If the clone-based approach proves too complex, here's a simpler bundle-based solution:

```rust
// Define common component bundles
pub struct GameObject3DBundle {
    pub transform: Transform,
    pub mesh: Mesh,
    pub material: Material,
    pub visibility: Visibility,
    pub name: Name,
}

pub struct CameraBundle {
    pub transform: Transform,
    pub camera: Camera,
    pub name: Name,
}

pub struct SpriteBundle {
    pub transform: Transform,
    pub sprite: SpriteRenderer,
    pub visibility: Visibility,
    pub name: Name,
}

// Add bundle spawn method to World
impl World {
    pub fn spawn_bundle<B: Bundle>(&mut self, bundle: B) -> Entity {
        let entity = self.spawn();
        bundle.insert(entity, self);
        entity
    }
}

// Bundle trait
pub trait Bundle {
    fn insert(self, entity: Entity, world: &mut World);
}

impl Bundle for GameObject3DBundle {
    fn insert(self, entity: Entity, world: &mut World) {
        // Create archetype with all components at once
        let archetype_id = ArchetypeId::new()
            .with_component::<Transform>()
            .with_component::<Mesh>()
            .with_component::<Material>()
            .with_component::<Visibility>()
            .with_component::<Name>();
            
        // Add entity and all components to archetype
        // This avoids migration entirely
    }
}
```

## Testing the Approach

```rust
#[test]
fn test_component_cloning() {
    let transform = Transform {
        position: [1.0, 2.0, 3.0],
        rotation: [0.0, 45.0, 0.0],
        scale: [2.0, 2.0, 2.0],
    };
    
    let cloned: Box<dyn Component> = transform.clone_boxed();
    // Verify the clone worked (would need downcast to check values)
}

#[test] 
fn test_entity_migration() {
    let mut world = World::new();
    
    // Create entity with Transform
    let entity = world.spawn_with(Transform::default());
    assert_eq!(world.archetype_count(), 1);
    
    // Add Mesh component (triggers migration)
    world.add_component(entity, Mesh::default()).unwrap();
    assert_eq!(world.archetype_count(), 2);
    
    // Verify entity still has both components
    assert!(world.get_component::<Transform>(entity).is_some());
    assert!(world.get_component::<Mesh>(entity).is_some());
}
```

## Decision Point

Before implementing the full solution, we should decide:

1. **Clone-based migration**: More flexible but requires modifying all components
2. **Bundle approach**: Simpler but less flexible (can't add components dynamically)
3. **Hybrid**: Use bundles for common cases, implement migration for edge cases

The bundle approach would be fastest to implement and would cover 90% of use cases.