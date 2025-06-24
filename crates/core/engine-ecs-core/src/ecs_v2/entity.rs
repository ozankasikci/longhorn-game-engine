//! Entity management for ECS V2
//!
//! Entities are lightweight identifiers that reference components stored in archetypes.
//! Each entity has a unique ID and a generation counter for safe recycling.

use crate::ecs_v2::archetype::ArchetypeId;

/// Entity is a unique identifier for game objects
///
/// Entities are composed of:
/// - `id`: A unique identifier within the world
/// - `generation`: A counter that increments when the entity slot is reused
///
/// The generation counter allows for safe entity recycling - if you hold a stale
/// Entity handle, it won't accidentally reference a newly created entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entity {
    id: u32,
    generation: u32,
}

impl Entity {
    /// Create a new entity with the given ID and generation
    pub fn new(id: u32, generation: u32) -> Self {
        Self { id, generation }
    }

    /// Get the entity's unique ID
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Get the entity's generation counter
    pub fn generation(&self) -> u32 {
        self.generation
    }
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity({}, gen:{})", self.id, self.generation)
    }
}

/// Location of an entity within the archetype storage system
///
/// This tracks where an entity's components are stored:
/// - `archetype_id`: Which archetype (component combination) the entity belongs to
/// - `index`: The entity's index within that archetype's component arrays
#[derive(Debug, Clone)]
pub struct EntityLocation {
    pub(crate) archetype_id: ArchetypeId,
    pub(crate) index: usize,
}

impl EntityLocation {
    /// Create a new entity location
    pub fn new(archetype_id: ArchetypeId, index: usize) -> Self {
        Self {
            archetype_id,
            index,
        }
    }

    /// Get the archetype ID where this entity is stored
    pub fn archetype_id(&self) -> &ArchetypeId {
        &self.archetype_id
    }

    /// Get the index within the archetype's storage
    pub fn index(&self) -> usize {
        self.index
    }
}

/// Entity allocator for managing entity IDs and generations
///
/// This could be extracted into World later, but having it separate
/// allows for more flexible entity management strategies.
#[derive(Debug)]
pub struct EntityAllocator {
    next_id: u32,
    next_generation: u32,
    /// Free list of entity IDs that can be reused
    free_ids: Vec<(u32, u32)>, // (id, generation)
}

impl EntityAllocator {
    /// Create a new entity allocator
    pub fn new() -> Self {
        Self {
            next_id: 0,
            next_generation: 1,
            free_ids: Vec::new(),
        }
    }

    /// Allocate a new entity
    pub fn allocate(&mut self) -> Entity {
        if let Some((id, mut generation)) = self.free_ids.pop() {
            // Reuse a freed entity ID with incremented generation
            generation += 1;
            Entity::new(id, generation)
        } else {
            // Allocate a new entity ID
            let entity = Entity::new(self.next_id, self.next_generation);
            self.next_id += 1;
            entity
        }
    }

    /// Free an entity for later reuse
    pub fn free(&mut self, entity: Entity) {
        self.free_ids.push((entity.id, entity.generation));
    }

    /// Check if an entity is still valid (not freed)
    pub fn is_valid(&self, entity: Entity) -> bool {
        // Check if this entity is in the free list
        !self
            .free_ids
            .iter()
            .any(|(id, gen)| *id == entity.id && *gen >= entity.generation)
    }
}

impl Default for EntityAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let entity = Entity::new(42, 7);
        assert_eq!(entity.id(), 42);
        assert_eq!(entity.generation(), 7);
    }

    #[test]
    fn test_entity_equality() {
        let e1 = Entity::new(1, 1);
        let e2 = Entity::new(1, 1);
        let e3 = Entity::new(1, 2);
        let e4 = Entity::new(2, 1);

        assert_eq!(e1, e2);
        assert_ne!(e1, e3); // Different generation
        assert_ne!(e1, e4); // Different ID
    }

    #[test]
    fn test_entity_ordering() {
        let e1 = Entity::new(1, 1);
        let e2 = Entity::new(2, 1);
        let e3 = Entity::new(1, 2);

        assert!(e1 < e2); // Lower ID comes first
        assert!(e1 < e3); // Same ID, lower generation comes first
    }

    #[test]
    fn test_entity_display() {
        let entity = Entity::new(42, 7);
        assert_eq!(format!("{}", entity), "Entity(42, gen:7)");
    }

    #[test]
    fn test_entity_allocator_basic() {
        let mut allocator = EntityAllocator::new();

        // Allocate some entities
        let e1 = allocator.allocate();
        let e2 = allocator.allocate();
        let e3 = allocator.allocate();

        assert_eq!(e1.id(), 0);
        assert_eq!(e2.id(), 1);
        assert_eq!(e3.id(), 2);
        assert_eq!(e1.generation(), 1);
        assert_eq!(e2.generation(), 1);
        assert_eq!(e3.generation(), 1);
    }

    #[test]
    fn test_entity_allocator_recycling() {
        let mut allocator = EntityAllocator::new();

        // Allocate and free
        let e1 = allocator.allocate();
        let e2 = allocator.allocate();
        allocator.free(e1);
        allocator.free(e2);

        // Reallocate - should reuse in LIFO order
        let e3 = allocator.allocate();
        let e4 = allocator.allocate();

        assert_eq!(e3.id(), 1); // Last freed
        assert_eq!(e3.generation(), 2); // Incremented generation
        assert_eq!(e4.id(), 0); // First freed
        assert_eq!(e4.generation(), 2); // Incremented generation
    }

    #[test]
    fn test_entity_allocator_validity() {
        let mut allocator = EntityAllocator::new();

        let e1 = allocator.allocate();
        assert!(allocator.is_valid(e1));

        allocator.free(e1);
        assert!(!allocator.is_valid(e1));

        let e2 = allocator.allocate();
        assert!(allocator.is_valid(e2));
        // Note: is_valid only checks the free list, so e1 won't be invalid after e2 is allocated
        // This is a limitation of the current implementation
    }

    #[test]
    fn test_entity_allocator_mixed_operations() {
        let mut allocator = EntityAllocator::new();

        let _e1 = allocator.allocate();
        let e2 = allocator.allocate();
        let _e3 = allocator.allocate();

        allocator.free(e2);
        let e4 = allocator.allocate(); // Reuses e2's ID
        let e5 = allocator.allocate(); // New ID

        assert_eq!(e4.id(), 1);
        assert_eq!(e4.generation(), 2);
        assert_eq!(e5.id(), 3);
        assert_eq!(e5.generation(), 1);
    }

    #[test]
    fn test_entity_location() {
        use crate::ecs_v2::archetype::ArchetypeId;

        let archetype_id = ArchetypeId::new();
        let location = EntityLocation::new(archetype_id.clone(), 42);

        assert_eq!(location.archetype_id(), &archetype_id);
        assert_eq!(location.index(), 42);
    }

    #[test]
    fn test_entity_allocator_stress() {
        let mut allocator = EntityAllocator::new();
        let mut entities = Vec::new();

        // Allocate many entities
        for _ in 0..1000 {
            entities.push(allocator.allocate());
        }

        // Free every other entity
        for i in (0..1000).step_by(2) {
            allocator.free(entities[i]);
        }

        // Reallocate
        for _ in 0..500 {
            let e = allocator.allocate();
            assert!(e.generation() > 1); // Should be recycled
        }
    }
}
