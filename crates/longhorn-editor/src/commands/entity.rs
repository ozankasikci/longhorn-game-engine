//! Entity manipulation command handlers.
//!
//! Handles entity creation, deletion, selection, property setting, and hierarchy management.

use longhorn_core::{AssetId, EntityHandle, EntityId, Name, Sprite, Transform, Vec2};
use longhorn_engine::Engine;
use longhorn_remote::{
    ComponentInfo, EntityDetails, EntityDump, EntityInfo, RemoteResponse, ResponseData,
    SpriteData, TransformData,
};

use crate::Editor;

// --- State Query Handlers ---

pub fn handle_get_state(editor: &Editor, engine: &Engine) -> RemoteResponse {
    let selected = editor.state().selected_entity.map(|e| e.to_bits().get());
    RemoteResponse::with_data(ResponseData::State {
        mode: format!("{:?}", editor.state().mode),
        paused: editor.state().paused,
        entity_count: engine.world().len(),
        selected_entity: selected,
    })
}

pub fn handle_get_entities(engine: &Engine) -> RemoteResponse {
    let entities: Vec<EntityInfo> = engine
        .world()
        .inner()
        .iter()
        .map(|entity_ref| {
            let entity = entity_ref.entity();
            let handle = EntityHandle::new(entity);
            let name = engine
                .world()
                .get::<Name>(handle)
                .ok()
                .map(|n| n.0.clone())
                .unwrap_or_else(|| format!("Entity {}", entity.id()));
            EntityInfo {
                id: entity.to_bits().get(),
                name,
            }
        })
        .collect();
    RemoteResponse::with_data(ResponseData::Entities(entities))
}

pub fn handle_get_entity(engine: &Engine, id: u64) -> RemoteResponse {
    let found = engine
        .world()
        .inner()
        .iter()
        .find(|e| e.entity().to_bits().get() == id);

    match found {
        Some(entity_ref) => {
            let entity = entity_ref.entity();
            let handle = EntityHandle::new(entity);

            let name = engine
                .world()
                .get::<Name>(handle)
                .ok()
                .map(|n| n.0.clone())
                .unwrap_or_else(|| format!("Entity {}", id));

            let transform = engine
                .world()
                .get::<Transform>(handle)
                .ok()
                .map(|t| TransformData {
                    position_x: t.position.x,
                    position_y: t.position.y,
                    rotation: t.rotation,
                    scale_x: t.scale.x,
                    scale_y: t.scale.y,
                });

            RemoteResponse::with_data(ResponseData::Entity(EntityDetails {
                id,
                name,
                transform,
            }))
        }
        None => RemoteResponse::error(format!("Entity not found: {}", id)),
    }
}

// --- Entity Manipulation Handlers ---

pub fn handle_select_entity(editor: &mut Editor, engine: &Engine, id: u64) -> RemoteResponse {
    let found = engine
        .world()
        .inner()
        .iter()
        .find(|e| e.entity().to_bits().get() == id)
        .map(|e| e.entity());

    match found {
        Some(entity) => {
            editor.state_mut().select(Some(entity));
            RemoteResponse::ok()
        }
        None => RemoteResponse::error(format!("Entity not found: {}", id)),
    }
}

pub fn handle_create_entity(engine: &mut Engine, name: &str) -> RemoteResponse {
    let entity = engine
        .world_mut()
        .spawn()
        .with(Name::new(name))
        .with(Transform::default())
        .build();
    let id = entity.id().to_bits().get();
    log::info!("Created entity '{}' with id {}", name, id);
    RemoteResponse::with_data(ResponseData::Created { id })
}

pub fn handle_delete_entity(editor: &mut Editor, engine: &mut Engine, id: u64) -> RemoteResponse {
    match EntityId::from_bits(id) {
        Some(entity_id) => {
            let handle = EntityHandle::new(entity_id);
            if engine.world_mut().despawn(handle).is_ok() {
                // Deselect if this was selected
                if editor.state().selected_entity.map(|e| e.id() as u64) == Some(id) {
                    editor.state_mut().select(None);
                }
                log::info!("Deleted entity {}", id);
                RemoteResponse::ok()
            } else {
                RemoteResponse::error(format!("Entity not found: {}", id))
            }
        }
        None => RemoteResponse::error(format!("Invalid entity id: {}", id)),
    }
}

pub fn handle_set_entity_parent(
    engine: &mut Engine,
    child_id: u64,
    parent_id: u64,
) -> RemoteResponse {
    use longhorn_core::ecs::hierarchy::set_parent;

    let child_entity = match EntityId::from_bits(child_id) {
        Some(id) => EntityHandle::new(id),
        None => return RemoteResponse::error(format!("Invalid child entity id: {}", child_id)),
    };

    let parent_entity = match EntityId::from_bits(parent_id) {
        Some(id) => EntityHandle::new(id),
        None => return RemoteResponse::error(format!("Invalid parent entity id: {}", parent_id)),
    };

    match set_parent(engine.world_mut(), child_entity, parent_entity) {
        Ok(()) => {
            log::info!("Set parent: entity {} -> parent {}", child_id, parent_id);
            RemoteResponse::ok()
        }
        Err(e) => RemoteResponse::error(format!("Failed to set parent: {:?}", e)),
    }
}

pub fn handle_clear_entity_parent(engine: &mut Engine, child_id: u64) -> RemoteResponse {
    use longhorn_core::ecs::hierarchy::clear_parent;

    let child_entity = match EntityId::from_bits(child_id) {
        Some(id) => EntityHandle::new(id),
        None => return RemoteResponse::error(format!("Invalid child entity id: {}", child_id)),
    };

    match clear_parent(engine.world_mut(), child_entity) {
        Ok(()) => {
            log::info!("Cleared parent for entity {}", child_id);
            RemoteResponse::ok()
        }
        Err(e) => RemoteResponse::error(format!("Failed to clear parent: {:?}", e)),
    }
}

pub fn set_entity_property(
    engine: &mut Engine,
    entity_id: u64,
    component: &str,
    field: &str,
    value: serde_json::Value,
) -> RemoteResponse {
    let entity_id = match EntityId::from_bits(entity_id) {
        Some(id) => id,
        None => return RemoteResponse::error(format!("Invalid entity id: {}", entity_id)),
    };
    let handle = EntityHandle::new(entity_id);

    match component {
        "Transform" => set_transform_property(engine, handle, field, value),
        "Name" => set_name_property(engine, handle, field, value),
        "Sprite" => set_sprite_property(engine, handle, field, value),
        _ => RemoteResponse::error(format!("Unknown component: {}", component)),
    }
}

fn set_transform_property(
    engine: &mut Engine,
    handle: EntityHandle,
    field: &str,
    value: serde_json::Value,
) -> RemoteResponse {
    let world = engine.world_mut();
    let mut transform = match world.get::<Transform>(handle) {
        Ok(t) => (*t).clone(),
        Err(_) => return RemoteResponse::error("Entity has no Transform"),
    };

    match field {
        "position.x" => {
            if let Some(v) = value.as_f64() {
                transform.position.x = v as f32;
            }
        }
        "position.y" => {
            if let Some(v) = value.as_f64() {
                transform.position.y = v as f32;
            }
        }
        "rotation" => {
            if let Some(v) = value.as_f64() {
                transform.rotation = v as f32;
            }
        }
        "scale.x" => {
            if let Some(v) = value.as_f64() {
                transform.scale.x = v as f32;
            }
        }
        "scale.y" => {
            if let Some(v) = value.as_f64() {
                transform.scale.y = v as f32;
            }
        }
        _ => return RemoteResponse::error(format!("Unknown field: {}", field)),
    }

    if world.set(handle, transform).is_err() {
        return RemoteResponse::error("Failed to set Transform");
    }
    RemoteResponse::ok()
}

fn set_name_property(
    engine: &mut Engine,
    handle: EntityHandle,
    field: &str,
    value: serde_json::Value,
) -> RemoteResponse {
    let world = engine.world_mut();
    if field == "name" || field == "0" {
        if let Some(s) = value.as_str() {
            if world.set(handle, Name::new(s)).is_err() {
                return RemoteResponse::error("Failed to set Name");
            }
            return RemoteResponse::ok();
        }
    }
    RemoteResponse::error(format!("Invalid Name field: {}", field))
}

fn set_sprite_property(
    engine: &mut Engine,
    handle: EntityHandle,
    field: &str,
    value: serde_json::Value,
) -> RemoteResponse {
    let mut sprite = match engine.world().get::<Sprite>(handle) {
        Ok(s) => (*s).clone(),
        Err(_) => Sprite::new(AssetId(0), Vec2::new(64.0, 64.0)),
    };

    match field {
        "texture" => {
            let asset_id = if let Some(v) = value.as_u64() {
                AssetId(v)
            } else if let Some(v) = value.as_i64() {
                AssetId(v as u64)
            } else {
                return RemoteResponse::error("texture must be a number (AssetId)");
            };
            sprite.texture = asset_id;

            if engine.world_mut().set(handle, sprite).is_err() {
                return RemoteResponse::error("Failed to set Sprite");
            }

            if let Err(e) = engine.assets_mut().load_texture_by_id(asset_id) {
                log::warn!("Failed to load texture {}: {}", asset_id.0, e);
            } else {
                log::info!("Loaded texture {} into cache", asset_id.0);
            }

            return RemoteResponse::ok();
        }
        "size.x" | "size_x" => {
            if let Some(v) = value.as_f64() {
                sprite.size.x = v as f32;
            }
        }
        "size.y" | "size_y" => {
            if let Some(v) = value.as_f64() {
                sprite.size.y = v as f32;
            }
        }
        "flip_x" => {
            if let Some(v) = value.as_bool() {
                sprite.flip_x = v;
            }
        }
        "flip_y" => {
            if let Some(v) = value.as_bool() {
                sprite.flip_y = v;
            }
        }
        "color.r" => {
            if let Some(v) = value.as_f64() {
                sprite.color[0] = v as f32;
            }
        }
        "color.g" => {
            if let Some(v) = value.as_f64() {
                sprite.color[1] = v as f32;
            }
        }
        "color.b" => {
            if let Some(v) = value.as_f64() {
                sprite.color[2] = v as f32;
            }
        }
        "color.a" => {
            if let Some(v) = value.as_f64() {
                sprite.color[3] = v as f32;
            }
        }
        _ => return RemoteResponse::error(format!("Unknown Sprite field: {}", field)),
    }

    if engine.world_mut().set(handle, sprite).is_err() {
        return RemoteResponse::error("Failed to set Sprite");
    }
    RemoteResponse::ok()
}

// --- Debug/Inspection Handlers ---

pub fn handle_get_entity_components(engine: &Engine, id: u64) -> RemoteResponse {
    let entity_id = match EntityId::from_bits(id) {
        Some(id) => id,
        None => return RemoteResponse::error(format!("Invalid entity id: {}", id)),
    };
    let handle = EntityHandle::new(entity_id);

    let mut components = Vec::new();

    if let Ok(name) = engine.world().get::<Name>(handle) {
        components.push(ComponentInfo {
            name: "Name".to_string(),
            data: serde_json::json!({ "value": name.0 }),
        });
    }

    if let Ok(transform) = engine.world().get::<Transform>(handle) {
        components.push(ComponentInfo {
            name: "Transform".to_string(),
            data: serde_json::json!({
                "position": { "x": transform.position.x, "y": transform.position.y },
                "rotation": transform.rotation,
                "scale": { "x": transform.scale.x, "y": transform.scale.y }
            }),
        });
    }

    if let Ok(sprite) = engine.world().get::<Sprite>(handle) {
        components.push(ComponentInfo {
            name: "Sprite".to_string(),
            data: serde_json::json!({
                "texture_id": sprite.texture.0,
                "size": { "x": sprite.size.x, "y": sprite.size.y },
                "color": sprite.color,
                "flip_x": sprite.flip_x,
                "flip_y": sprite.flip_y
            }),
        });
    }

    if let Ok(script) = engine.world().get::<longhorn_core::Script>(handle) {
        components.push(ComponentInfo {
            name: "Script".to_string(),
            data: serde_json::json!({ "path": script.path }),
        });
    }

    RemoteResponse::with_data(ResponseData::Components(components))
}

pub fn handle_dump_entity(engine: &Engine, id: u64) -> RemoteResponse {
    let entity_id = match EntityId::from_bits(id) {
        Some(id) => id,
        None => return RemoteResponse::error(format!("Invalid entity id: {}", id)),
    };
    let handle = EntityHandle::new(entity_id);

    let name = engine.world().get::<Name>(handle).ok().map(|n| n.0.clone());

    let transform = engine
        .world()
        .get::<Transform>(handle)
        .ok()
        .map(|t| TransformData {
            position_x: t.position.x,
            position_y: t.position.y,
            rotation: t.rotation,
            scale_x: t.scale.x,
            scale_y: t.scale.y,
        });

    let sprite = engine
        .world()
        .get::<Sprite>(handle)
        .ok()
        .map(|s| SpriteData {
            texture_id: s.texture.0,
            size_x: s.size.x,
            size_y: s.size.y,
            color: s.color,
            flip_x: s.flip_x,
            flip_y: s.flip_y,
        });

    let has_script = engine.world().get::<longhorn_core::Script>(handle).is_ok();

    let mut component_names = Vec::new();
    if name.is_some() {
        component_names.push("Name".to_string());
    }
    if transform.is_some() {
        component_names.push("Transform".to_string());
    }
    if sprite.is_some() {
        component_names.push("Sprite".to_string());
    }
    if has_script {
        component_names.push("Script".to_string());
    }

    let dump = EntityDump {
        id,
        name,
        transform,
        sprite,
        has_script,
        component_names,
    };

    RemoteResponse::with_data(ResponseData::EntityDump(dump))
}
