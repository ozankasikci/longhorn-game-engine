use crate::ecs::{Enabled, EntityHandle, Name, Script, Sprite, World};
use crate::math::Transform;
use crate::types::{AssetId, LonghornError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Trait for providing asset registry functionality to Scene
pub trait AssetRegistry {
    /// Get the path for a given asset ID
    fn get_path(&self, id: AssetId) -> Option<&str>;

    /// Get the ID for a given path
    fn get_id(&self, path: &str) -> Option<AssetId>;
}

/// Trait for loading textures during scene spawning
pub trait AssetLoader {
    /// Load a texture by path and return its asset ID
    fn load_texture(&mut self, path: &str) -> std::io::Result<AssetId>;

    /// Load a texture by ID and return its asset ID (for fallback when path loading fails)
    fn load_texture_by_id(&mut self, id: AssetId) -> std::io::Result<AssetId>;
}

/// Serialized entity data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedEntity {
    pub id: u64,
    pub components: SerializedComponents,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<SerializedEntity>,
}

/// Container for all component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedComponents {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Name")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Transform")]
    pub transform: Option<SerializedTransform>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Sprite")]
    pub sprite: Option<SerializedSprite>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Script")]
    pub script: Option<Script>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Enabled")]
    pub enabled: Option<bool>,
}

/// Serialized transform component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedTransform {
    pub position: [f32; 2],
    pub rotation: f32,
    pub scale: [f32; 2],
}

impl From<Transform> for SerializedTransform {
    fn from(t: Transform) -> Self {
        Self {
            position: [t.position.x, t.position.y],
            rotation: t.rotation,
            scale: [t.scale.x, t.scale.y],
        }
    }
}

impl From<SerializedTransform> for Transform {
    fn from(st: SerializedTransform) -> Self {
        use glam::Vec2;
        Transform {
            position: Vec2::new(st.position[0], st.position[1]),
            rotation: st.rotation,
            scale: Vec2::new(st.scale[0], st.scale[1]),
        }
    }
}

/// Serialized sprite component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedSprite {
    pub texture_path: String,
    pub texture_id: u64,
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub flip_x: bool,
    pub flip_y: bool,
}

/// Scene data structure for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub entities: Vec<SerializedEntity>,
}

impl Scene {
    /// Create a new empty scene with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entities: Vec::new(),
        }
    }

}

/// Recursively serialize an entity and its children
fn serialize_entity<R: AssetRegistry>(
        world: &World,
        registry: &R,
        entity_id: hecs::Entity,
    ) -> SerializedEntity {
        let entity_u64 = entity_id.to_bits().get();
        let entity_handle = EntityHandle::new(entity_id);

        let mut components = SerializedComponents {
            name: None,
            transform: None,
            sprite: None,
            script: None,
            enabled: None,
        };

        // Try to get Name component
        if let Ok(name) = world.inner().get::<&Name>(entity_id) {
            components.name = Some(name.as_str().to_string());
        }

        // Try to get Transform component
        if let Ok(transform) = world.inner().get::<&Transform>(entity_id) {
            components.transform = Some((*transform).into());
        }

        // Try to get Sprite component
        if let Ok(sprite) = world.inner().get::<&Sprite>(entity_id) {
            let texture_path = registry
                .get_path(sprite.texture)
                .unwrap_or("unknown")
                .to_string();

            components.sprite = Some(SerializedSprite {
                texture_path,
                texture_id: sprite.texture.0,
                size: [sprite.size.x, sprite.size.y],
                color: sprite.color,
                flip_x: sprite.flip_x,
                flip_y: sprite.flip_y,
            });
        }

        // Try to get Script component
        if let Ok(script) = world.inner().get::<&Script>(entity_id) {
            components.script = Some((*script).clone());
        }

        // Try to get Enabled component
        if let Ok(enabled) = world.inner().get::<&Enabled>(entity_id) {
            components.enabled = Some(enabled.is_enabled());
        }

        // Recursively serialize children
        let mut children = Vec::new();
        if let Ok(children_comp) = world.get::<crate::ecs::Children>(entity_handle) {
            for &child_id in children_comp.iter() {
                children.push(serialize_entity(world, registry, child_id));
            }
        }

        SerializedEntity {
            id: entity_u64,
            components,
            children,
        }
    }

impl Scene {
    /// Extract scene data from an ECS World
    ///
    /// # Arguments
    /// * `world` - The ECS world to extract entities from
    /// * `registry` - Asset registry to look up texture paths from IDs
    pub fn from_world<R: AssetRegistry>(world: &World, registry: &R) -> Self {
        let mut entities = Vec::new();

        // Find all root entities (entities without Parent component)
        for (entity_id, _) in world.query::<()>().iter() {
            let entity_handle = EntityHandle::new(entity_id);

            // Only serialize entities that don't have a Parent component
            if world.get::<crate::ecs::Parent>(entity_handle).is_err() {
                entities.push(serialize_entity(world, registry, entity_id));
            }
        }

        Self {
            name: "Scene".to_string(),
            entities,
        }
    }

    /// Save the scene to a JSON file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Serialize to JSON
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| LonghornError::Serialization(e.to_string()))?;

        // Write to file
        fs::write(path, json)?;

        Ok(())
    }

    /// Load a scene from a JSON file
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        // Read file contents
        let contents = fs::read_to_string(path)?;

        // Deserialize from JSON
        let scene: Scene = serde_json::from_str(&contents)
            .map_err(|e| LonghornError::Serialization(e.to_string()))?;

        Ok(scene)
    }
}

/// Recursively spawn an entity and its children
fn spawn_entity<L: AssetLoader>(
    world: &mut World,
    asset_loader: &mut L,
    serialized: &SerializedEntity,
    parent: Option<EntityHandle>,
) -> Result<EntityHandle> {
    let mut builder = world.spawn();

    // Add Name component if present
    if let Some(ref name) = serialized.components.name {
        builder = builder.with(Name::new(name.clone()));
    }

    // Add Transform component if present
    if let Some(ref transform) = serialized.components.transform {
        builder = builder.with(Transform::from(transform.clone()));
    }

    // Add Sprite component if present
    if let Some(ref sprite_data) = serialized.components.sprite {
        // Try to load the texture
        let texture_id = match asset_loader.load_texture(&sprite_data.texture_path) {
            Ok(id) => Some(id),
            Err(_) => {
                // Fallback to loading by ID
                asset_loader
                    .load_texture_by_id(AssetId(sprite_data.texture_id))
                    .ok()
            }
        };

        // Only add Sprite if we successfully loaded a texture
        if let Some(texture_id) = texture_id {
            builder = builder.with(Sprite {
                texture: texture_id,
                size: glam::Vec2::new(sprite_data.size[0], sprite_data.size[1]),
                color: sprite_data.color,
                flip_x: sprite_data.flip_x,
                flip_y: sprite_data.flip_y,
            });
        }
    }

    // Add Script component if present
    if let Some(ref script) = serialized.components.script {
        builder = builder.with(script.clone());
    }

    // Add Enabled component if present
    if let Some(enabled) = serialized.components.enabled {
        builder = builder.with(Enabled::new(enabled));
    }

    // Build the entity
    let entity_handle = builder.build();

    // Set parent if provided
    if let Some(parent_handle) = parent {
        crate::ecs::hierarchy::add_child(world, parent_handle, entity_handle)
            .map_err(|e| LonghornError::InvalidOperation(format!("Failed to set parent: {:?}", e)))?;
    }

    // Recursively spawn children
    for child in &serialized.children {
        spawn_entity(world, asset_loader, child, Some(entity_handle))?;
    }

    Ok(entity_handle)
}

impl Scene {
    /// Spawn all entities from this scene into a World
    ///
    /// # Arguments
    /// * `world` - The target ECS world to spawn entities into
    /// * `asset_loader` - Asset loader to load textures referenced in sprites
    ///
    /// # Returns
    /// A mapping from serialized entity IDs to new entity handles
    pub fn spawn_into<L: AssetLoader>(
        &self,
        world: &mut World,
        asset_loader: &mut L,
    ) -> Result<HashMap<u64, EntityHandle>> {
        let mut entity_map = HashMap::new();

        for serialized_entity in &self.entities {
            let entity_handle = spawn_entity(world, asset_loader, serialized_entity, None)?;
            entity_map.insert(serialized_entity.id, entity_handle);
        }

        Ok(entity_map)
    }

    /// Recursively collect all entities (including children) into a HashMap
    fn collect_all_entities<'a>(
        serialized: &'a SerializedEntity,
        map: &mut std::collections::HashMap<u64, &'a SerializedEntity>,
    ) {
        map.insert(serialized.id, serialized);
        for child in &serialized.children {
            Self::collect_all_entities(child, map);
        }
    }

    /// Restore entities in-place from this scene, preserving entity IDs
    ///
    /// This method updates existing entities to match the snapshot without
    /// destroying and recreating them, which preserves entity IDs and prevents
    /// component scrambling.
    ///
    /// # Arguments
    /// * `world` - The target ECS world to restore entities into
    /// * `asset_loader` - Asset loader to load textures referenced in sprites
    pub fn restore_into<L: AssetLoader>(
        &self,
        world: &mut World,
        asset_loader: &mut L,
    ) -> Result<()> {
        use std::collections::HashMap;
        use glam::Vec2;

        // Build a map of serialized entity ID -> SerializedEntity
        // This recursively collects ALL entities including children
        let mut snapshot_entities: HashMap<u64, &SerializedEntity> = HashMap::new();
        for entity in &self.entities {
            Self::collect_all_entities(entity, &mut snapshot_entities);
        }

        // Track which snapshot entities we've processed
        let mut processed_ids = std::collections::HashSet::new();

        // Collect all entity IDs first to avoid borrow checker issues
        let entity_ids: Vec<_> = world.query::<()>().iter().map(|(id, _)| id).collect();

        // Update existing entities in-place
        for entity_id in entity_ids {
            let entity_bits = entity_id.to_bits().get();

            if let Some(serialized) = snapshot_entities.get(&entity_bits) {
                // This entity exists in the snapshot - update its components
                processed_ids.insert(entity_bits);

                // Remove all existing components by getting mutable access
                // Note: We can't remove components in hecs, so we'll just overwrite them

                // Update/add Name
                if let Some(ref name) = serialized.components.name {
                    let _ = world.inner_mut().insert_one(entity_id, Name::new(name.clone()));
                } else if world.has::<Name>(EntityHandle::new(entity_id)) {
                    let _ = world.inner_mut().remove_one::<Name>(entity_id);
                }

                // Update/add Transform
                if let Some(ref transform) = serialized.components.transform {
                    let _ = world.inner_mut().insert_one(entity_id, Transform::from(transform.clone()));
                } else if world.has::<Transform>(EntityHandle::new(entity_id)) {
                    let _ = world.inner_mut().remove_one::<Transform>(entity_id);
                }

                // Update/add Sprite
                if let Some(ref sprite_data) = serialized.components.sprite {
                    // Try to load the texture
                    let texture_id = match asset_loader.load_texture(&sprite_data.texture_path) {
                        Ok(id) => id,
                        Err(_) => {
                            // Fallback: try loading by ID
                            match asset_loader.load_texture_by_id(AssetId::new(sprite_data.texture_id)) {
                                Ok(id) => id,
                                Err(e) => {
                                    eprintln!(
                                        "Warning: Failed to load texture '{}': {}. Keeping existing Sprite unchanged.",
                                        sprite_data.texture_path, e
                                    );
                                    // Don't remove the Sprite - keep it unchanged
                                    // Just skip updating it and continue to other components
                                    if let Some(ref script) = serialized.components.script {
                                        let _ = world.inner_mut().insert_one(entity_id, script.clone());
                                    } else if world.has::<Script>(EntityHandle::new(entity_id)) {
                                        let _ = world.inner_mut().remove_one::<Script>(entity_id);
                                    }

                                    if let Some(ref enabled) = serialized.components.enabled {
                                        let _ = world.inner_mut().insert_one(entity_id, Enabled::new(*enabled));
                                    } else if world.has::<Enabled>(EntityHandle::new(entity_id)) {
                                        let _ = world.inner_mut().remove_one::<Enabled>(entity_id);
                                    }
                                    continue;
                                }
                            }
                        }
                    };

                    let sprite = Sprite {
                        texture: texture_id,
                        size: Vec2::new(sprite_data.size[0], sprite_data.size[1]),
                        color: sprite_data.color,
                        flip_x: sprite_data.flip_x,
                        flip_y: sprite_data.flip_y,
                    };
                    let _ = world.inner_mut().insert_one(entity_id, sprite);
                } else if world.has::<Sprite>(EntityHandle::new(entity_id)) {
                    let _ = world.inner_mut().remove_one::<Sprite>(entity_id);
                }

                // Update/add Script
                if let Some(ref script) = serialized.components.script {
                    let _ = world.inner_mut().insert_one(entity_id, script.clone());
                } else if world.has::<Script>(EntityHandle::new(entity_id)) {
                    let _ = world.inner_mut().remove_one::<Script>(entity_id);
                }

                // Update/add Enabled
                if let Some(ref enabled) = serialized.components.enabled {
                    let _ = world.inner_mut().insert_one(entity_id, Enabled::new(*enabled));
                } else if world.has::<Enabled>(EntityHandle::new(entity_id)) {
                    let _ = world.inner_mut().remove_one::<Enabled>(entity_id);
                }
            } else {
                // This entity doesn't exist in the snapshot - despawn it
                let _ = world.inner_mut().despawn(entity_id);
            }
        }

        // Spawn any entities that exist in the snapshot but not in the world
        // (This shouldn't normally happen for play mode snapshots, but handle it for completeness)
        for serialized in &self.entities {
            if !processed_ids.contains(&serialized.id) {
                // This entity only exists in the snapshot - spawn it
                // Note: We can't control the entity ID in hecs, so this entity will get a new ID
                eprintln!(
                    "Warning: Entity {} from snapshot not found in world. Spawning with new ID.",
                    serialized.id
                );

                let mut builder = world.spawn();

                if let Some(ref name) = serialized.components.name {
                    builder = builder.with(Name::new(name.clone()));
                }

                if let Some(ref transform) = serialized.components.transform {
                    builder = builder.with(Transform::from(transform.clone()));
                }

                if let Some(ref sprite_data) = serialized.components.sprite {
                    if let Ok(texture_id) = asset_loader.load_texture(&sprite_data.texture_path)
                        .or_else(|_| asset_loader.load_texture_by_id(AssetId::new(sprite_data.texture_id)))
                    {
                        let sprite = Sprite {
                            texture: texture_id,
                            size: Vec2::new(sprite_data.size[0], sprite_data.size[1]),
                            color: sprite_data.color,
                            flip_x: sprite_data.flip_x,
                            flip_y: sprite_data.flip_y,
                        };
                        builder = builder.with(sprite);
                    }
                }

                if let Some(ref script) = serialized.components.script {
                    builder = builder.with(script.clone());
                }

                if let Some(ref enabled) = serialized.components.enabled {
                    builder = builder.with(Enabled::new(*enabled));
                }

                builder.build();
            }
        }

        Ok(())
    }

    /// Add an entity to the scene
    pub fn add_entity(&mut self, entity: SerializedEntity) {
        self.entities.push(entity);
    }

    /// Get the number of entities in the scene
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::Children;
    use glam::Vec2;
    use std::env;

    // Mock registry for testing
    struct MockRegistry {
        path_to_id: HashMap<String, u64>,
        id_to_path: HashMap<u64, String>,
    }

    impl MockRegistry {
        fn new() -> Self {
            let mut registry = Self {
                path_to_id: HashMap::new(),
                id_to_path: HashMap::new(),
            };
            registry.register("sprites/player.png", 1);
            registry.register("sprites/enemy.png", 2);
            registry
        }

        fn register(&mut self, path: &str, id: u64) {
            self.path_to_id.insert(path.to_string(), id);
            self.id_to_path.insert(id, path.to_string());
        }
    }

    impl AssetRegistry for MockRegistry {
        fn get_path(&self, id: AssetId) -> Option<&str> {
            self.id_to_path.get(&id.0).map(|s| s.as_str())
        }

        fn get_id(&self, path: &str) -> Option<AssetId> {
            self.path_to_id.get(path).map(|&id| AssetId::new(id))
        }
    }

    // Mock asset loader for testing
    struct MockAssetLoader {
        registry: MockRegistry,
    }

    impl MockAssetLoader {
        fn new() -> Self {
            Self {
                registry: MockRegistry::new(),
            }
        }
    }

    impl AssetLoader for MockAssetLoader {
        fn load_texture(&mut self, path: &str) -> std::io::Result<AssetId> {
            self.registry
                .get_id(path)
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("Texture not found: {}", path),
                    )
                })
        }

        fn load_texture_by_id(&mut self, id: AssetId) -> std::io::Result<AssetId> {
            // Check if the ID exists in the registry
            if self.registry.get_path(id).is_some() {
                Ok(id)
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Texture with ID {} not found", id.0),
                ))
            }
        }
    }

    fn temp_path(name: &str) -> std::path::PathBuf {
        env::temp_dir().join(format!(
            "longhorn_scene_test_{}_{}",
            name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn test_create_empty_scene() {
        let scene = Scene::new("Test Scene");
        assert_eq!(scene.name, "Test Scene");
        assert_eq!(scene.entity_count(), 0);
    }

    #[test]
    fn test_add_entity() {
        let mut scene = Scene::new("Test Scene");

        let entity = SerializedEntity {
            id: 1,
            components: SerializedComponents {
                name: Some("Player".to_string()),
                transform: None,
                sprite: None,
                script: None,
                enabled: Some(true),
            },
            children: Vec::new(),
        };

        scene.add_entity(entity);
        assert_eq!(scene.entity_count(), 1);
    }

    #[test]
    fn test_save_and_load_scene() {
        let path = temp_path("save_load");

        // Create a scene
        let mut scene = Scene::new("Main Scene");

        let entity = SerializedEntity {
            id: 1,
            components: SerializedComponents {
                name: Some("Player".to_string()),
                transform: Some(SerializedTransform {
                    position: [100.0, 200.0],
                    rotation: 0.0,
                    scale: [1.0, 1.0],
                }),
                sprite: Some(SerializedSprite {
                    texture_path: "sprites/player.png".to_string(),
                    texture_id: 1,
                    size: [32.0, 32.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    flip_x: false,
                    flip_y: false,
                }),
                script: None,
                enabled: Some(true),
            },
            children: Vec::new(),
        };

        scene.add_entity(entity);

        // Save the scene
        scene.save(&path).unwrap();
        assert!(path.exists());

        // Load the scene
        let loaded = Scene::load(&path).unwrap();

        // Verify data
        assert_eq!(loaded.name, "Main Scene");
        assert_eq!(loaded.entity_count(), 1);

        let entity = &loaded.entities[0];
        assert_eq!(entity.id, 1);
        assert_eq!(
            entity.components.name,
            Some("Player".to_string())
        );
        assert!(entity.components.transform.is_some());
        assert!(entity.components.sprite.is_some());
        assert_eq!(entity.components.enabled, Some(true));

        // Verify transform
        let transform = entity.components.transform.as_ref().unwrap();
        assert_eq!(transform.position, [100.0, 200.0]);
        assert_eq!(transform.rotation, 0.0);
        assert_eq!(transform.scale, [1.0, 1.0]);

        // Verify sprite
        let sprite = entity.components.sprite.as_ref().unwrap();
        assert_eq!(sprite.texture_path, "sprites/player.png");
        assert_eq!(sprite.texture_id, 1);
        assert_eq!(sprite.size, [32.0, 32.0]);
        assert_eq!(sprite.color, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(sprite.flip_x, false);
        assert_eq!(sprite.flip_y, false);

        // Clean up
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_from_world() {
        let mut world = World::new();
        let registry = MockRegistry::new();

        // Spawn some entities with components
        world
            .spawn()
            .with(Name::new("Player"))
            .with(Transform::from_position(Vec2::new(100.0, 200.0)))
            .with(Sprite::new(AssetId::new(1), Vec2::new(32.0, 32.0)))
            .with(Enabled::default())
            .build();

        world
            .spawn()
            .with(Name::new("Enemy"))
            .with(Transform::new())
            .build();

        // Extract scene from world
        let scene = Scene::from_world(&world, &registry);

        assert_eq!(scene.entity_count(), 2);

        // Find the Player entity
        let player = scene
            .entities
            .iter()
            .find(|e| e.components.name.as_ref().map(|n| n.as_str()) == Some("Player"))
            .unwrap();

        assert!(player.components.transform.is_some());
        assert!(player.components.sprite.is_some());
        assert_eq!(player.components.enabled, Some(true));

        // Verify sprite has correct texture path
        let sprite = player.components.sprite.as_ref().unwrap();
        assert_eq!(sprite.texture_path, "sprites/player.png");
        assert_eq!(sprite.texture_id, 1);
    }

    #[test]
    fn test_spawn_into_world() {
        let mut world = World::new();
        let mut asset_loader = MockAssetLoader::new();

        // Create a scene
        let mut scene = Scene::new("Test Scene");

        let entity = SerializedEntity {
            id: 42,
            components: SerializedComponents {
                name: Some("TestEntity".to_string()),
                transform: Some(SerializedTransform {
                    position: [50.0, 75.0],
                    rotation: 1.57,
                    scale: [2.0, 2.0],
                }),
                sprite: Some(SerializedSprite {
                    texture_path: "sprites/player.png".to_string(),
                    texture_id: 1,
                    size: [64.0, 64.0],
                    color: [1.0, 0.5, 0.5, 1.0],
                    flip_x: true,
                    flip_y: false,
                }),
                script: None,
                enabled: Some(false),
            },
            children: Vec::new(),
        };

        scene.add_entity(entity);

        // Spawn into world
        let entity_map = scene.spawn_into(&mut world, &mut asset_loader).unwrap();

        // Verify entity was spawned
        assert_eq!(entity_map.len(), 1);
        assert_eq!(world.len(), 1);

        let handle = entity_map[&42];

        // Verify components
        assert!(world.has::<Name>(handle));
        assert!(world.has::<Transform>(handle));
        assert!(world.has::<Sprite>(handle));
        assert!(world.has::<Enabled>(handle));

        // Verify component values
        let name = world.get::<Name>(handle).unwrap();
        assert_eq!(name.as_str(), "TestEntity");

        let transform = world.get::<Transform>(handle).unwrap();
        assert_eq!(transform.position, Vec2::new(50.0, 75.0));
        assert!((transform.rotation - 1.57).abs() < 0.001);
        assert_eq!(transform.scale, Vec2::new(2.0, 2.0));

        let sprite = world.get::<Sprite>(handle).unwrap();
        assert_eq!(sprite.texture.0, 1);
        assert_eq!(sprite.size, Vec2::new(64.0, 64.0));
        assert_eq!(sprite.color, [1.0, 0.5, 0.5, 1.0]);
        assert_eq!(sprite.flip_x, true);
        assert_eq!(sprite.flip_y, false);

        let enabled = world.get::<Enabled>(handle).unwrap();
        assert_eq!(enabled.is_enabled(), false);
    }

    #[test]
    fn test_roundtrip_world_to_scene_to_world() {
        let path = temp_path("roundtrip");

        // Create original world with entities
        let mut original_world = World::new();
        let registry = MockRegistry::new();

        original_world
            .spawn()
            .with(Name::new("Player"))
            .with(Transform::from_position(Vec2::new(100.0, 200.0)))
            .with(Sprite::new(AssetId::new(1), Vec2::new(32.0, 32.0)))
            .with(Enabled::new(true))
            .build();

        original_world
            .spawn()
            .with(Name::new("Enemy"))
            .with(Transform::from_position(Vec2::new(300.0, 400.0)))
            .with(Sprite::new(AssetId::new(2), Vec2::new(48.0, 48.0)))
            .with(Enabled::new(false))
            .build();

        // Extract scene from world
        let scene = Scene::from_world(&original_world, &registry);

        // Save scene
        scene.save(&path).unwrap();

        // Load scene
        let loaded_scene = Scene::load(&path).unwrap();

        // Spawn into new world
        let mut new_world = World::new();
        let mut asset_loader = MockAssetLoader::new();
        loaded_scene.spawn_into(&mut new_world, &mut asset_loader).unwrap();

        // Verify the new world has the same number of entities
        assert_eq!(new_world.len(), original_world.len());

        // Verify Player entity
        let player = new_world.find("Player").unwrap();
        let player_transform = new_world.get::<Transform>(player).unwrap();
        assert_eq!(player_transform.position, Vec2::new(100.0, 200.0));

        let player_sprite = new_world.get::<Sprite>(player).unwrap();
        assert_eq!(player_sprite.texture.0, 1);
        assert_eq!(player_sprite.size, Vec2::new(32.0, 32.0));

        let player_enabled = new_world.get::<Enabled>(player).unwrap();
        assert_eq!(player_enabled.is_enabled(), true);

        // Verify Enemy entity
        let enemy = new_world.find("Enemy").unwrap();
        let enemy_transform = new_world.get::<Transform>(enemy).unwrap();
        assert_eq!(enemy_transform.position, Vec2::new(300.0, 400.0));

        let enemy_sprite = new_world.get::<Sprite>(enemy).unwrap();
        assert_eq!(enemy_sprite.texture.0, 2);
        assert_eq!(enemy_sprite.size, Vec2::new(48.0, 48.0));

        let enemy_enabled = new_world.get::<Enabled>(enemy).unwrap();
        assert_eq!(enemy_enabled.is_enabled(), false);

        // Clean up
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_spawn_with_missing_texture() {
        let mut world = World::new();
        let mut asset_loader = MockAssetLoader::new();

        // Create a scene with a sprite that references a non-existent texture
        let mut scene = Scene::new("Test Scene");

        let entity = SerializedEntity {
            id: 1,
            components: SerializedComponents {
                name: Some("TestEntity".to_string()),
                transform: Some(SerializedTransform {
                    position: [0.0, 0.0],
                    rotation: 0.0,
                    scale: [1.0, 1.0],
                }),
                sprite: Some(SerializedSprite {
                    texture_path: "nonexistent.png".to_string(),
                    texture_id: 999,
                    size: [32.0, 32.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    flip_x: false,
                    flip_y: false,
                }),
                script: None,
                enabled: Some(true),
            },
            children: Vec::new(),
        };

        scene.add_entity(entity);

        // Spawn should succeed but skip the Sprite component
        let entity_map = scene.spawn_into(&mut world, &mut asset_loader).unwrap();

        assert_eq!(entity_map.len(), 1);
        let handle = entity_map[&1];

        // Verify entity has Name, Transform, and Enabled but NOT Sprite
        assert!(world.has::<Name>(handle));
        assert!(world.has::<Transform>(handle));
        assert!(world.has::<Enabled>(handle));
        assert!(!world.has::<Sprite>(handle));
    }

    #[test]
    fn test_spawn_with_renamed_texture_fallback_to_id() {
        let mut world = World::new();
        let mut asset_loader = MockAssetLoader::new();

        // Create a scene with a sprite that has an old path but valid ID
        // Simulating a file that was renamed: old path doesn't exist, but ID is still valid
        let mut scene = Scene::new("Test Scene");

        let entity = SerializedEntity {
            id: 1,
            components: SerializedComponents {
                name: Some("TestEntity".to_string()),
                transform: Some(SerializedTransform {
                    position: [0.0, 0.0],
                    rotation: 0.0,
                    scale: [1.0, 1.0],
                }),
                sprite: Some(SerializedSprite {
                    texture_path: "sprites/old_player_name.png".to_string(), // Old path (doesn't exist)
                    texture_id: 1, // But ID 1 exists as "sprites/player.png"
                    size: [32.0, 32.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    flip_x: false,
                    flip_y: false,
                }),
                script: None,
                enabled: Some(true),
            },
            children: Vec::new(),
        };

        scene.add_entity(entity);

        // Spawn should succeed by falling back to ID-based loading
        let entity_map = scene.spawn_into(&mut world, &mut asset_loader).unwrap();

        assert_eq!(entity_map.len(), 1);
        let handle = entity_map[&1];

        // Verify entity has all components including Sprite (loaded by ID)
        assert!(world.has::<Name>(handle));
        assert!(world.has::<Transform>(handle));
        assert!(world.has::<Enabled>(handle));
        assert!(world.has::<Sprite>(handle));

        // Verify sprite was loaded with correct ID
        let sprite = world.get::<Sprite>(handle).unwrap();
        assert_eq!(sprite.texture.0, 1);
        assert_eq!(sprite.size, Vec2::new(32.0, 32.0));
    }

    #[test]
    fn test_transform_conversion() {
        let transform = Transform::from_components(
            Vec2::new(10.0, 20.0),
            1.5,
            Vec2::new(2.0, 3.0),
        );

        let serialized: SerializedTransform = transform.into();
        assert_eq!(serialized.position, [10.0, 20.0]);
        assert_eq!(serialized.rotation, 1.5);
        assert_eq!(serialized.scale, [2.0, 3.0]);

        let deserialized: Transform = serialized.into();
        assert_eq!(deserialized.position, Vec2::new(10.0, 20.0));
        assert_eq!(deserialized.rotation, 1.5);
        assert_eq!(deserialized.scale, Vec2::new(2.0, 3.0));
    }

    #[test]
    fn test_script_component_serialization() {
        use crate::ecs::Script;
        use std::collections::HashMap;

        let mut world = World::new();
        let registry = MockRegistry::new();

        // Create entity with Script component
        let mut properties = HashMap::new();
        properties.insert("speed".to_string(), crate::ecs::ScriptValue::Number(10.0));

        let script = Script::with_properties("PlayerController.ts", properties);

        world
            .spawn()
            .with(Name::new("Player"))
            .with(Transform::from_position(Vec2::new(0.0, 0.0)))
            .with(script)
            .build();

        // Extract scene from world
        let scene = Scene::from_world(&world, &registry);

        // Verify Script was serialized
        let player = scene
            .entities
            .iter()
            .find(|e| e.components.name.as_ref().map(|n| n.as_str()) == Some("Player"))
            .unwrap();

        assert!(player.components.script.is_some(), "Script component should be serialized");
        let script_data = player.components.script.as_ref().unwrap();
        assert_eq!(script_data.path, "PlayerController.ts");
        assert_eq!(script_data.enabled, true);
        assert_eq!(script_data.properties.get("speed"), Some(&crate::ecs::ScriptValue::Number(10.0)));

        // Test roundtrip: spawn into new world
        let mut new_world = World::new();
        let mut asset_loader = MockAssetLoader::new();
        scene.spawn_into(&mut new_world, &mut asset_loader).unwrap();

        // Verify Script was deserialized
        let player_handle = new_world.find("Player").unwrap();
        assert!(new_world.has::<Script>(player_handle), "Script component should be restored");

        let restored_script = new_world.get::<Script>(player_handle).unwrap();
        assert_eq!(restored_script.path, "PlayerController.ts");
        assert_eq!(restored_script.enabled, true);
        assert_eq!(restored_script.get_property("speed"), Some(&crate::ecs::ScriptValue::Number(10.0)));
    }

    #[test]
    fn test_hierarchical_scene_serialization() {
        use crate::ecs::hierarchy::set_parent;

        let mut world = World::new();
        let registry = MockRegistry::new();

        // Create parent entity
        let parent = world
            .spawn()
            .with(Name::new("Parent"))
            .with(Transform::from_position(Vec2::new(100.0, 100.0)))
            .with(Children::new())
            .build();

        // Create child entity
        let child = world
            .spawn()
            .with(Name::new("Child"))
            .with(Transform::from_position(Vec2::new(50.0, 0.0)))
            .build();

        // Set up hierarchy
        set_parent(&mut world, child, parent).unwrap();

        // Create scene from world
        let scene = Scene::from_world(&world, &registry);

        // Verify structure
        assert_eq!(scene.entities.len(), 1); // Only root entity
        assert_eq!(scene.entities[0].children.len(), 1); // One child

        // Verify parent
        let parent_entity = &scene.entities[0];
        assert_eq!(parent_entity.components.name, Some("Parent".to_string()));

        // Verify child
        let child_entity = &parent_entity.children[0];
        assert_eq!(child_entity.components.name, Some("Child".to_string()));
    }

    #[test]
    fn test_hierarchical_scene_roundtrip() {
        use crate::ecs::hierarchy::set_parent;

        let mut world1 = World::new();
        let registry = MockRegistry::new();

        // Create grandparent -> parent -> child hierarchy
        let grandparent = world1
            .spawn()
            .with(Name::new("Grandparent"))
            .with(Transform::from_position(Vec2::new(0.0, 0.0)))
            .with(Children::new())
            .build();

        let parent = world1
            .spawn()
            .with(Name::new("Parent"))
            .with(Transform::from_position(Vec2::new(100.0, 0.0)))
            .with(Children::new())
            .build();

        let child = world1
            .spawn()
            .with(Name::new("Child"))
            .with(Transform::from_position(Vec2::new(50.0, 0.0)))
            .build();

        set_parent(&mut world1, parent, grandparent).unwrap();
        set_parent(&mut world1, child, parent).unwrap();

        // Serialize
        let scene = Scene::from_world(&world1, &registry);

        // Deserialize into new world
        let mut world2 = World::new();
        let mut loader = MockAssetLoader::new();
        scene.spawn_into(&mut world2, &mut loader).unwrap();

        // Verify hierarchy was restored
        let entities: Vec<_> = world2.query::<&Name>().iter().map(|(e, n)| (e, n.as_str().to_string())).collect();
        assert_eq!(entities.len(), 3);

        // Find entities by name
        let find_by_name = |name: &str| -> EntityHandle {
            entities.iter()
                .find(|(_, n)| n == name)
                .map(|(e, _)| EntityHandle::new(*e))
                .unwrap()
        };

        let gp = find_by_name("Grandparent");
        let p = find_by_name("Parent");
        let c = find_by_name("Child");

        // Verify parent relationships
        assert!(world2.get::<crate::ecs::Parent>(gp).is_err()); // No parent
        assert_eq!(world2.get::<crate::ecs::Parent>(p).unwrap().get(), gp.id());
        assert_eq!(world2.get::<crate::ecs::Parent>(c).unwrap().get(), p.id());

        // Verify children relationships
        let gp_children = world2.get::<crate::ecs::Children>(gp).unwrap();
        assert_eq!(gp_children.len(), 1);
        assert!(gp_children.iter().any(|&e| e == p.id()));

        let p_children = world2.get::<crate::ecs::Children>(p).unwrap();
        assert_eq!(p_children.len(), 1);
        assert!(p_children.iter().any(|&e| e == c.id()));
    }
}
