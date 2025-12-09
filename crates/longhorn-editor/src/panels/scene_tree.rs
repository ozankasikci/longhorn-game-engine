use egui::Ui;
use longhorn_core::{World, Name, EntityHandle, Parent, Children};
use longhorn_assets::{AssetManager, FilesystemSource};
use crate::{EditorState, UiStateTracker};
use std::collections::HashSet;

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

    /// Recursively render an entity node in the tree
    fn show_entity_node(
        &mut self,
        ui: &mut Ui,
        node: &EntityNode,
        world: &mut World,
        state: &mut EditorState,
        ui_state: &mut UiStateTracker,
        game_path: Option<&std::path::Path>,
        asset_manager: &mut AssetManager<FilesystemSource>,
        depth: usize,
    ) {
        let entity = node.entity;
        let entity_bits = entity.to_bits().get();
        let is_selected = state.is_selected(entity);
        let has_children = !node.children.is_empty();
        let is_expanded = self.expanded_entities.contains(&entity_bits);

        let mut reparent_target = None;

        let response = ui.horizontal(|ui| {
            // Indent based on depth
            ui.add_space(depth as f32 * 16.0);

            // Expand/collapse button if has children
            if has_children {
                let arrow = if is_expanded { "▼" } else { "▶" };
                if ui.small_button(arrow).clicked() {
                    if is_expanded {
                        self.expanded_entities.remove(&entity_bits);
                    } else {
                        self.expanded_entities.insert(entity_bits);
                    }
                }
            } else {
                // Add spacing to align childless entities
                ui.add_space(20.0);
            }

            // Register as clickable for remote control
            let element_id = format!("entity_{}", entity.id());
            ui_state.register_clickable(&element_id, &node.name, "selectable");

            // Check if should be triggered remotely
            let should_trigger = ui_state.take_pending_trigger()
                .map(|id| id == element_id)
                .unwrap_or(false);

            // Entity name (selectable) - wrapped as drag source
            let label_response = ui.selectable_label(is_selected, &node.name);

            if label_response.clicked() || should_trigger {
                log::info!("SceneTree - selecting entity '{}': ID {} (raw: {:?}, to_bits: {})",
                    node.name, entity.id(), entity, entity_bits);
                state.select(Some(entity));
            }

            // Make the label a drag source
            label_response.dnd_set_drag_payload(entity_bits);

            label_response
        }).inner;

        // Check if something was dropped on this entity
        if let Some(dropped_entity_bits) = response.dnd_release_payload::<u64>() {
            // Don't allow dropping on self
            if *dropped_entity_bits != entity_bits {
                reparent_target = Some(*dropped_entity_bits);
            }
        }

        // Highlight when hovering during drag
        if response.hovered() && ui.input(|i| i.pointer.any_down()) {
            ui.painter().rect_stroke(
                response.rect,
                2.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 180, 255)),
            );
        }

        // Perform reparenting if drop occurred
        if let Some(dragged_bits) = reparent_target {
            // Find the dragged entity (safely handle case where entity was deleted during drag)
            if let Some(dragged_entity) = world.inner().iter()
                .find(|e| e.entity().to_bits().get() == dragged_bits)
                .map(|e| e.entity())
            {
                let dragged_handle = EntityHandle::new(dragged_entity);
                let target_handle = EntityHandle::new(entity);

                // Use hierarchy system to set parent with cycle detection
                match longhorn_core::ecs::hierarchy::set_parent(world, dragged_handle, target_handle) {
                    Ok(()) => {
                        log::info!("Reparented entity {} to {}", dragged_entity.id(), entity.id());
                    }
                    Err(e) => {
                        log::warn!("Failed to reparent: {:?}", e);
                    }
                }
            } else {
                log::warn!("Dropped entity no longer exists (bits: {})", dragged_bits);
            }
        }

        // Recursively show children if expanded
        if is_expanded {
            for child in &node.children {
                self.show_entity_node(
                    ui,
                    child,
                    world,
                    state,
                    ui_state,
                    game_path,
                    asset_manager,
                    depth + 1,
                );
            }
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

        // Collect all entities
        let all_entities: Vec<_> = world.inner().iter().map(|e| e.entity()).collect();

        // Handle pending select by path
        if let Some(path) = ui_state.take_pending_tree_select() {
            let entity_name = path.split('/').last().unwrap_or(&path);
            if let Some(entity) = all_entities.iter().find(|e| {
                let handle = EntityHandle::new(**e);
                world.get::<Name>(handle)
                    .ok()
                    .map(|n| n.0 == entity_name)
                    .unwrap_or(false)
            }) {
                state.select(Some(*entity));
            }
        }

        if all_entities.is_empty() {
            ui.label("(No entities)");
            return;
        }

        // Find root entities (entities without Parent component)
        let root_entities: Vec<_> = all_entities.iter()
            .filter(|&&entity| {
                let handle = EntityHandle::new(entity);
                world.get::<Parent>(handle).is_err()
            })
            .copied()
            .collect();

        // Build tree for each root entity
        for root_entity in root_entities {
            let tree = EntityNode::from_entity(world, root_entity);
            self.show_entity_node(
                ui,
                &tree,
                world,
                state,
                ui_state,
                game_path,
                asset_manager,
                0, // depth = 0 for root
            );
        }

        // Add drop zone at bottom to make entities root-level
        ui.add_space(10.0);
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 30.0),
            egui::Sense::hover(),
        );

        // Visual feedback for drop zone
        if response.hovered() && ui.input(|i| i.pointer.any_down()) {
            ui.painter().rect_filled(
                rect,
                2.0,
                egui::Color32::from_rgba_premultiplied(100, 180, 255, 30),
            );
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Drop here to make root entity",
                egui::FontId::default(),
                egui::Color32::from_rgb(100, 180, 255),
            );
        } else {
            ui.painter().rect_stroke(
                rect,
                2.0,
                egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.bg_stroke.color),
            );
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Drop here to make root entity",
                egui::FontId::default(),
                ui.style().visuals.text_color(),
            );
        }

        // Check for drop on root zone
        if let Some(dropped_entity_bits) = response.dnd_release_payload::<u64>() {
            // Find the entity and clear its parent
            if let Some(dropped_entity) = world.inner().iter()
                .find(|e| e.entity().to_bits().get() == *dropped_entity_bits)
                .map(|e| e.entity())
            {
                let handle = EntityHandle::new(dropped_entity);
                match longhorn_core::ecs::hierarchy::clear_parent(world, handle) {
                    Ok(()) => {
                        log::info!("Cleared parent for entity {} (now root)", dropped_entity.id());
                    }
                    Err(e) => {
                        log::warn!("Failed to clear parent: {:?}", e);
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
