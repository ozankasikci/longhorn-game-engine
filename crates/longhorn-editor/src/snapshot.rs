use longhorn_core::{World, Name, Transform, Sprite, Enabled, EntityHandle, Script};
use serde::{Deserialize, Serialize};

/// Snapshot of an entity's components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub name: Option<Name>,
    pub transform: Option<Transform>,
    pub sprite: Option<Sprite>,
    pub enabled: Option<Enabled>,
    pub script: Option<Script>,
}

/// Snapshot of the entire scene for restore
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SceneSnapshot {
    pub entities: Vec<EntitySnapshot>,
}

impl SceneSnapshot {
    /// Capture current world state
    pub fn capture(world: &World) -> Self {
        // Collect entity IDs first, then sort by ID to ensure deterministic order
        let mut entity_ids: Vec<_> = world.query::<()>().iter()
            .map(|(entity, _)| entity)
            .collect();
        entity_ids.sort_by_key(|e| e.id());

        let mut entities = Vec::new();
        for entity in entity_ids {
            let handle = EntityHandle::new(entity);
            let snapshot = EntitySnapshot {
                name: world.get::<Name>(handle).ok().map(|r| (*r).clone()),
                transform: world.get::<Transform>(handle).ok().map(|r| (*r).clone()),
                sprite: world.get::<Sprite>(handle).ok().map(|r| (*r).clone()),
                enabled: world.get::<Enabled>(handle).ok().map(|r| (*r).clone()),
                script: world.get::<Script>(handle).ok().map(|r| (*r).clone()),
            };
            entities.push(snapshot);
        }

        SceneSnapshot { entities }
    }

    /// Restore world to this snapshot
    pub fn restore(self, world: &mut World) {
        // Clear all existing entities
        world.clear();

        // Recreate entities from snapshot
        for entity_data in self.entities {
            let mut builder = world.spawn();

            if let Some(name) = entity_data.name {
                builder = builder.with(name);
            }
            if let Some(transform) = entity_data.transform {
                builder = builder.with(transform);
            }
            if let Some(sprite) = entity_data.sprite {
                builder = builder.with(sprite);
            }
            if let Some(enabled) = entity_data.enabled {
                builder = builder.with(enabled);
            }
            if let Some(script) = entity_data.script {
                builder = builder.with(script);
            }

            builder.build();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn test_capture_and_restore() {
        let mut world = World::new();

        // Create test entity
        world.spawn()
            .with(Name::new("TestEntity"))
            .with(Transform::from_position(Vec2::new(100.0, 200.0)))
            .with(Enabled::default())
            .build();

        assert_eq!(world.len(), 1);

        // Capture snapshot
        let snapshot = SceneSnapshot::capture(&world);
        assert_eq!(snapshot.entities.len(), 1);

        // Modify world
        world.spawn()
            .with(Name::new("NewEntity"))
            .build();
        assert_eq!(world.len(), 2);

        // Restore snapshot
        snapshot.restore(&mut world);
        assert_eq!(world.len(), 1);

        // Verify restored entity
        let entity = world.find("TestEntity");
        assert!(entity.is_some());
    }

    #[test]
    fn test_capture_empty_world() {
        let world = World::new();
        let snapshot = SceneSnapshot::capture(&world);
        assert_eq!(snapshot.entities.len(), 0);
    }
}
