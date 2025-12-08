use egui::Ui;
use longhorn_core::{World, Name, EntityHandle, Sprite, Parent, Children};
use longhorn_assets::{AssetManager, FilesystemSource};
use crate::{EditorState, UiStateTracker};
use std::path::PathBuf;
use std::collections::HashSet;
use glam::Vec2;

pub struct SceneTreePanel {
    expanded_entities: HashSet<u64>, // Track which entities are expanded (using entity bits)
}

/// Represents an entity node in the hierarchy tree
struct EntityNode {
    entity: hecs::Entity,
    name: String,
    children: Vec<EntityNode>,
}

impl EntityNode {
    /// Recursively build tree from root entity
    fn from_entity(world: &World, entity: hecs::Entity) -> Self {
        let handle = EntityHandle::new(entity);

        let name = world.get::<Name>(handle)
            .ok()
            .map(|n| n.0.clone())
            .unwrap_or_else(|| format!("Entity {}", entity.id()));

        let mut children = Vec::new();

        // Get children if this entity has Children component
        if let Ok(children_comp) = world.get::<Children>(handle) {
            for &child_entity in children_comp.iter() {
                // Recursively build child nodes
                children.push(EntityNode::from_entity(world, child_entity));
            }
        }

        EntityNode {
            entity,
            name,
            children,
        }
    }
}

impl SceneTreePanel {
    pub fn new() -> Self {
        Self {
            expanded_entities: HashSet::new(),
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        world: &mut World,
        state: &mut EditorState,
        ui_state: &mut UiStateTracker,
        game_path: Option<&std::path::Path>,
        asset_manager: &mut AssetManager<FilesystemSource>,
    ) {
        ui.heading("Scene Tree");
        ui.separator();

        let mut entities: Vec<_> = world.inner().iter().map(|entity_ref| entity_ref.entity()).collect();

        // Handle pending select by path
        if let Some(path) = ui_state.take_pending_tree_select() {
            let entity_name = path.split('/').last().unwrap_or(&path);
            if let Some(entity) = entities.iter().find(|e| {
                let handle = EntityHandle::new(**e);
                world.get::<Name>(handle)
                    .ok()
                    .map(|n| n.0 == entity_name)
                    .unwrap_or(false)
            }) {
                state.select(Some(*entity));
            }
        }

        if entities.is_empty() {
            ui.label("(No entities)");
        } else {
            entities.sort_by_key(|e| e.id());

            for entity in entities {
                let handle = EntityHandle::new(entity);
                let name = world.get::<Name>(handle)
                    .ok()
                    .map(|n| n.0.clone())
                    .unwrap_or_else(|| format!("Entity {}", entity.id()));

                let is_selected = state.is_selected(entity);

                // Register as clickable element for remote control
                let element_id = format!("entity_{}", entity.id());
                ui_state.register_clickable(&element_id, &name, "selectable");

                // Check if this element should be triggered
                let should_trigger = ui_state.take_pending_trigger()
                    .map(|id| id == element_id)
                    .unwrap_or(false);

                // Wrap the selectable label in a drop zone for drag-drop support
                let (drop_result, dropped_payload) = ui.dnd_drop_zone::<PathBuf, ()>(
                    egui::Frame::none(),
                    |ui| {
                        // Create the selectable label inside the drop zone
                        let response = ui.selectable_label(is_selected, &name);

                        // Handle click selection
                        if response.clicked() || should_trigger {
                            log::info!("SceneTree - selecting entity '{}': ID {} (raw: {:?}, to_bits: {})", name, entity.id(), entity, entity.to_bits().get());
                            state.select(Some(entity));
                        }
                    },
                );

                // Add visual feedback when dragging over
                if drop_result.response.hovered() && ui.input(|i| i.pointer.any_down()) {
                    // Highlight the entity when hovering with a drag
                    ui.painter().rect_stroke(
                        drop_result.response.rect,
                        2.0,
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 180, 255)),
                    );
                }

                if let Some(dropped_path) = dropped_payload {
                    // Check if it's an image file
                    if let Some(ext) = dropped_path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        if matches!(ext_str.as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp") {
                            log::info!("Image file dropped on entity: {:?}", dropped_path);

                            // Get relative path from game root
                            if let Some(game_path) = game_path {
                                if let Ok(relative_path) = dropped_path.strip_prefix(game_path) {
                                    let path_str = relative_path.to_string_lossy().to_string();
                                    log::info!("Relative path: {}", path_str);

                                    // Try to get existing asset ID from registry, or import the asset
                                    let asset_id = if let Some(id) = asset_manager.registry().get_id(&path_str) {
                                        id
                                    } else {
                                        // If not in registry, auto-import it
                                        log::info!("Image {} not in asset registry, importing...", path_str);
                                        match asset_manager.import_asset(&*dropped_path, &path_str) {
                                            Ok(id) => {
                                                log::info!("Successfully imported {} with ID {}", path_str, id.0);
                                                id
                                            }
                                            Err(e) => {
                                                log::error!("Failed to import asset {}: {}", path_str, e);
                                                // Skip this drop if import fails
                                                continue;
                                            }
                                        }
                                    };

                                    // Check if entity already has a Sprite component
                                    let has_sprite = world.get::<Sprite>(handle).is_ok();

                                    if has_sprite {
                                        // Replace the texture
                                        if let Ok(mut sprite) = world.get_mut::<Sprite>(handle) {
                                            log::info!("Replacing sprite texture on entity {} with asset {}", entity.id(), asset_id.0);
                                            sprite.texture = asset_id;
                                        }
                                    } else {
                                        // Add new Sprite component with default size
                                        log::info!("Adding sprite component to entity {} with asset {}", entity.id(), asset_id.0);
                                        // TODO: Get actual image dimensions from loaded texture data
                                        let sprite = Sprite::new(asset_id, Vec2::new(32.0, 32.0));
                                        if let Err(e) = world.set(handle, sprite) {
                                            log::error!("Failed to add sprite component: {:?}", e);
                                        }
                                    }
                                } else {
                                    log::warn!("Dropped file is not under game path: {:?}", dropped_path);
                                }
                            } else {
                                log::warn!("No game path set, cannot process dropped file");
                            }
                        } else {
                            log::info!("Non-image file dropped, ignoring: {:?}", dropped_path);
                        }
                    }
                }
            }
        }
    }
}

impl Default for SceneTreePanel {
    fn default() -> Self {
        Self::new()
    }
}
