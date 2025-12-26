use egui::Ui;
use longhorn_core::{World, Name, EntityHandle, Parent, Children, Transform, GlobalTransform};
use longhorn_assets::{AssetManager, FilesystemSource};
use crate::{EditorState, UiStateTracker};
use crate::styling::{Spacing, Typography, Icons, IconSize, Colors, Radius};
use crate::ui::context_menus::show_scene_tree_create_menu;
pub use crate::ui::context_menus::SceneTreeAction;
use std::collections::HashSet;

pub struct SceneTreePanel {
    pub expanded_entities: HashSet<u64>, // Track which entities are expanded (using entity bits)
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
        action: &mut Option<SceneTreeAction>,
    ) {
        let entity = node.entity;
        let entity_bits = entity.to_bits().get();
        let is_selected = state.is_selected(entity);
        let has_children = !node.children.is_empty();
        let is_expanded = self.expanded_entities.contains(&entity_bits);

        let mut reparent_target = None;

        let horizontal_response = ui.horizontal(|ui| {
            // Indent based on depth
            ui.add_space(depth as f32 * Spacing::TREE_INDENT);
            ui.add_space(Spacing::LIST_ITEM_PADDING_H);

            // Expand/collapse button if has children
            if has_children {
                let arrow_icon = if is_expanded { Icons::CARET_DOWN } else { Icons::CARET_RIGHT };
                if ui.add(egui::Button::new(Icons::icon_sized(arrow_icon, IconSize::SM)).frame(false)).clicked() {
                    if is_expanded {
                        self.expanded_entities.remove(&entity_bits);
                    } else {
                        self.expanded_entities.insert(entity_bits);
                    }
                }
            } else {
                // Add spacing to align childless entities
                ui.add_space(IconSize::SM + Spacing::ITEM_GAP);
            }

            ui.add_space(Spacing::ITEM_GAP);

            // Entity icon
            ui.label(Icons::icon_sized(Icons::ENTITY, IconSize::SM));

            ui.add_space(Spacing::ICON_TEXT_GAP);

            // Register as clickable for remote control
            let element_id = format!("entity_{}", entity.id());
            ui_state.register_clickable(&element_id, &node.name, "selectable");

            // Check if should be triggered remotely
            let should_trigger = ui_state.take_pending_trigger()
                .map(|id| id == element_id)
                .unwrap_or(false);

            // Create selectable label with drag sensing enabled
            let mut label_resp = ui.selectable_label(is_selected, &node.name);
            label_resp = label_resp.interact(egui::Sense::click_and_drag());

            // Handle clicks for selection
            if label_resp.clicked() || should_trigger {
                log::info!("SceneTree - selecting entity '{}': ID {} (raw: {:?}, to_bits: {})",
                    node.name, entity.id(), entity, entity_bits);
                state.select(Some(entity));
            }

            // Enable dragging only when actually being dragged
            if label_resp.drag_started() || label_resp.dragged() {
                log::info!("SceneTree DND: Setting drag payload for entity bits {}", entity_bits);
                label_resp.dnd_set_drag_payload(entity_bits);
            }

            label_resp
        });

        let response = horizontal_response.inner;

        // Check if something is being dragged over this entity
        let can_accept_payload = response.dnd_hover_payload::<u64>().is_some();
        if can_accept_payload {
            log::info!("SceneTree DND: Hover detected on entity {} ({})", entity.id(), node.name);
        }

        // Check if something was dropped on this entity
        if let Some(dropped_entity_bits) = response.dnd_release_payload::<u64>() {
            log::info!("SceneTree DND: Drop detected! Dropped {} onto entity {} ({})",
                dropped_entity_bits, entity.id(), node.name);
            // Don't allow dropping on self
            if *dropped_entity_bits != entity_bits {
                log::info!("SceneTree DND: Valid drop (not self), will reparent");
                reparent_target = Some(*dropped_entity_bits);
            } else {
                log::warn!("SceneTree DND: Dropping on self, ignoring");
            }
        }

        // Highlight when hovering during drag
        if can_accept_payload {
            ui.painter().rect_stroke(
                response.rect,
                Radius::SMALL,
                egui::Stroke::new(2.0, Colors::ACCENT),
            );
        }

        // Perform reparenting if drop occurred
        if let Some(dragged_bits) = reparent_target {
            log::info!("SceneTree DND: Attempting to reparent entity bits {}", dragged_bits);
            // Find the dragged entity (safely handle case where entity was deleted during drag)
            if let Some(dragged_entity) = world.inner().iter()
                .find(|e| e.entity().to_bits().get() == dragged_bits)
                .map(|e| e.entity())
            {
                let dragged_handle = EntityHandle::new(dragged_entity);
                let target_handle = EntityHandle::new(entity);

                log::info!("SceneTree DND: Found dragged entity {}, setting parent to {}",
                    dragged_entity.id(), entity.id());

                // Preserve world transform when reparenting
                // Get current world position of dragged entity (copy to avoid borrow issues)
                let dragged_global_opt = world.get::<GlobalTransform>(dragged_handle).ok().map(|r| *r);
                let parent_global_opt = world.get::<GlobalTransform>(target_handle).ok().map(|r| *r);

                if let (Some(dragged_global), Some(parent_global)) = (dragged_global_opt, parent_global_opt) {
                    // Calculate new local transform to maintain world position
                    // local = parent_global.inverse() * entity_global
                    let new_local_transform = parent_global.to_local_transform(&dragged_global);

                    // Update the entity's local Transform
                    let _ = world.set(dragged_handle, new_local_transform);
                    log::info!("SceneTree DND: Adjusted local transform to preserve world position");
                }

                // Use hierarchy system to set parent with cycle detection
                match longhorn_core::ecs::hierarchy::set_parent(world, dragged_handle, target_handle) {
                    Ok(()) => {
                        log::info!("SceneTree DND: SUCCESS - Reparented entity {} to {}",
                            dragged_entity.id(), entity.id());
                    }
                    Err(e) => {
                        log::warn!("SceneTree DND: FAILED to reparent: {:?}", e);
                    }
                }
            } else {
                log::warn!("SceneTree DND: Dropped entity no longer exists (bits: {})", dragged_bits);
            }
        }

        // Context menu for this entity
        horizontal_response.response.context_menu(|ui| {
            if let Some(ctx_action) = show_scene_tree_create_menu(ui) {
                *action = Some(ctx_action);
            }
        });

        ui.add_space(Spacing::ITEM_GAP);

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
                    action,
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
    ) -> Option<SceneTreeAction> {
        let mut action: Option<SceneTreeAction> = None;

        ui.label(Typography::heading("Scene Tree"));
        ui.add_space(Spacing::SECTION_HEADER_BOTTOM);
        ui.separator();
        ui.add_space(Spacing::SECTION_HEADER_BOTTOM);

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
            ui.label(Typography::empty_state("(No entities)"));
            return None;
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
                &mut action,
            );
        }

        // Add drop zone at bottom to make entities root-level
        ui.add_space(Spacing::SECTION_GAP);
        let drop_zone_height = Spacing::ROW_HEIGHT + Spacing::LIST_ITEM_PADDING_V * 2.0;
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), drop_zone_height),
            egui::Sense::hover(),
        );

        // Visual feedback for drop zone
        if response.hovered() && ui.input(|i| i.pointer.any_down()) {
            ui.painter().rect_filled(
                rect,
                Radius::SMALL,
                Colors::ACCENT.gamma_multiply(0.15),
            );
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Drop here to make root entity",
                egui::FontId::default(),
                Colors::ACCENT,
            );
        } else {
            ui.painter().rect_stroke(
                rect,
                Radius::SMALL,
                egui::Stroke::new(1.0, Colors::STROKE_DEFAULT),
            );
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Drop here to make root entity",
                egui::FontId::default(),
                Colors::TEXT_MUTED,
            );
        }

        // Check for drop on root zone
        if let Some(dropped_entity_bits) = response.dnd_release_payload::<u64>() {
            log::info!("SceneTree DND: Drop detected on root zone! Entity bits: {}", dropped_entity_bits);
            // Find the entity and clear its parent
            if let Some(dropped_entity) = world.inner().iter()
                .find(|e| e.entity().to_bits().get() == *dropped_entity_bits)
                .map(|e| e.entity())
            {
                let handle = EntityHandle::new(dropped_entity);
                log::info!("SceneTree DND: Found entity {}, clearing parent", dropped_entity.id());

                // Preserve world transform when making root
                // The local Transform should equal the current GlobalTransform (copy to avoid borrow issues)
                let global_opt = world.get::<GlobalTransform>(handle).ok().map(|r| *r);
                if let Some(global) = global_opt {
                    let new_local = Transform::from_components(
                        global.position,
                        global.rotation,
                        global.scale,
                    );
                    let _ = world.set(handle, new_local);
                    log::info!("SceneTree DND: Set local transform to preserve world position when becoming root");
                }

                match longhorn_core::ecs::hierarchy::clear_parent(world, handle) {
                    Ok(()) => {
                        log::info!("SceneTree DND: SUCCESS - Cleared parent for entity {} (now root)", dropped_entity.id());
                    }
                    Err(e) => {
                        log::warn!("SceneTree DND: FAILED to clear parent: {:?}", e);
                    }
                }
            } else {
                log::warn!("SceneTree DND: Entity not found for bits: {}", dropped_entity_bits);
            }
        }

        // Context menu for the entire panel (right-click anywhere)
        // Allocate remaining space to capture right-clicks
        let remaining = ui.available_size();
        if remaining.y > 0.0 {
            let (rect, response) = ui.allocate_exact_size(remaining, egui::Sense::click());
            if rect.height() > 0.0 {
                response.context_menu(|ui| {
                    if let Some(ctx_action) = show_scene_tree_create_menu(ui) {
                        action = Some(ctx_action);
                    }
                });
            }
        }

        action
    }
}

impl Default for SceneTreePanel {
    fn default() -> Self {
        Self::new()
    }
}
