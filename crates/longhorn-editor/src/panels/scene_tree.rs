use egui::{Ui, Vec2};
use longhorn_core::{World, Name, EntityHandle, Parent, Children, Transform, GlobalTransform};
use longhorn_assets::{AssetManager, FilesystemSource};
use crate::{EditorState, UiStateTracker};
use crate::styling::{Typography, Icons, IconSize, Colors, Radius};
use crate::ui::context_menus::{show_entity_context_menu, show_scene_tree_empty_context_menu};
pub use crate::ui::context_menus::SceneTreeAction;
use std::collections::HashSet;

/// Consistent tree item height
const TREE_ITEM_HEIGHT: f32 = 20.0;
/// Indentation per level
const TREE_INDENT: f32 = 16.0;

pub struct SceneTreePanel {
    pub expanded_entities: HashSet<u64>,
}

struct EntityNode {
    entity: hecs::Entity,
    name: String,
    children: Vec<EntityNode>,
}

impl EntityNode {
    fn from_entity(world: &World, entity: hecs::Entity) -> Self {
        let handle = EntityHandle::new(entity);

        let name = world.get::<Name>(handle)
            .ok()
            .map(|n| n.0.clone())
            .unwrap_or_else(|| format!("Entity {}", entity.id()));

        let mut children = Vec::new();

        if let Ok(children_comp) = world.get::<Children>(handle) {
            for &child_entity in children_comp.iter() {
                children.push(EntityNode::from_entity(world, child_entity));
            }
        }

        EntityNode { entity, name, children }
    }
}

impl SceneTreePanel {
    pub fn new() -> Self {
        Self {
            expanded_entities: HashSet::new(),
        }
    }

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
        let is_renaming = state.renaming_entity == Some(entity);

        let mut reparent_target = None;
        let indent = depth as f32 * TREE_INDENT;

        // Get rename buffer for this entity
        let rename_id = egui::Id::new(("entity_rename", entity_bits));
        let mut rename_text = ui.ctx().data_mut(|d| {
            d.get_temp::<String>(rename_id).unwrap_or_else(|| node.name.clone())
        });

        let horizontal_response = ui.horizontal(|ui| {
            ui.set_min_height(TREE_ITEM_HEIGHT);
            ui.add_space(4.0 + indent);

            // Expand/collapse button
            if has_children {
                let arrow = if is_expanded { Icons::CARET_DOWN } else { Icons::CARET_RIGHT };
                if ui.add(egui::Button::new(Icons::icon_sized(arrow, IconSize::SM)).frame(false)).clicked() {
                    if is_expanded {
                        self.expanded_entities.remove(&entity_bits);
                    } else {
                        self.expanded_entities.insert(entity_bits);
                    }
                }
            } else {
                ui.add_space(IconSize::SM + 4.0);
            }

            // Entity icon
            ui.label(Icons::icon_sized(Icons::ENTITY, IconSize::SM));
            ui.add_space(4.0);

            // Register for remote control
            let element_id = format!("entity_{}", entity.id());
            ui_state.register_clickable(&element_id, &node.name, "selectable");

            let should_trigger = ui_state.take_pending_trigger()
                .map(|id| id == element_id)
                .unwrap_or(false);

            if is_renaming {
                // Show text edit for renaming
                let text_edit = egui::TextEdit::singleline(&mut rename_text)
                    .desired_width(100.0);
                let response = ui.add(text_edit);

                // Check for key presses BEFORE focus handling
                let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                let escape_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));
                let focus_lost = response.lost_focus();

                // Handle Escape to cancel (don't commit)
                if escape_pressed {
                    state.renaming_entity = None;
                    ui.ctx().data_mut(|d| d.remove::<String>(rename_id));
                    response.surrender_focus();
                }
                // Handle Enter to confirm OR focus lost (click outside)
                else if enter_pressed || focus_lost {
                    // Commit the rename
                    let handle = EntityHandle::new(entity);
                    log::info!("=== RENAME COMMIT: enter={}, lost_focus={}, text='{}' ===",
                        enter_pressed, focus_lost, rename_text);
                    if let Err(e) = world.set(handle, Name::new(&rename_text)) {
                        log::error!("Failed to rename entity: {:?}", e);
                    } else {
                        log::info!("Renamed entity to: {} - setting EntityRenamed action", rename_text);
                        // Signal that entity was renamed (for dirty tracking)
                        *action = Some(SceneTreeAction::EntityRenamed(entity));
                    }
                    state.renaming_entity = None;
                    ui.ctx().data_mut(|d| d.remove::<String>(rename_id));
                    response.surrender_focus();
                }
                // Continue renaming - store text and ensure focus
                else {
                    ui.ctx().data_mut(|d| d.insert_temp(rename_id, rename_text.clone()));
                    // Auto-focus on first frame
                    if !response.has_focus() {
                        response.request_focus();
                    }
                }

                response
            } else {
                // Selectable label with drag support
                let mut label_resp = ui.selectable_label(is_selected, &node.name);
                label_resp = label_resp.interact(egui::Sense::click_and_drag());

                if label_resp.clicked() || should_trigger {
                    log::info!("SceneTree - selecting entity '{}': ID {}", node.name, entity.id());
                    state.select(Some(entity));
                }

                // Enter key on selected entity starts rename (like Finder on Mac)
                if is_selected && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    state.renaming_entity = Some(entity);
                    // Initialize rename buffer with current name
                    ui.ctx().data_mut(|d| d.insert_temp(rename_id, node.name.clone()));
                }

                if label_resp.drag_started() || label_resp.dragged() {
                    label_resp.dnd_set_drag_payload(entity_bits);
                }

                label_resp
            }
        });

        let response = horizontal_response.inner;

        // Drag-drop handling
        let can_accept_payload = response.dnd_hover_payload::<u64>().is_some();

        if let Some(dropped_entity_bits) = response.dnd_release_payload::<u64>() {
            if *dropped_entity_bits != entity_bits {
                reparent_target = Some(*dropped_entity_bits);
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

        // Perform reparenting
        if let Some(dragged_bits) = reparent_target {
            if let Some(dragged_entity) = world.inner().iter()
                .find(|e| e.entity().to_bits().get() == dragged_bits)
                .map(|e| e.entity())
            {
                let dragged_handle = EntityHandle::new(dragged_entity);
                let target_handle = EntityHandle::new(entity);

                let dragged_global_opt = world.get::<GlobalTransform>(dragged_handle).ok().map(|r| *r);
                let parent_global_opt = world.get::<GlobalTransform>(target_handle).ok().map(|r| *r);

                if let (Some(dragged_global), Some(parent_global)) = (dragged_global_opt, parent_global_opt) {
                    let new_local_transform = parent_global.to_local_transform(&dragged_global);
                    let _ = world.set(dragged_handle, new_local_transform);
                }

                let _ = longhorn_core::ecs::hierarchy::set_parent(world, dragged_handle, target_handle);
            }
        }

        // Context menu - attach to the inner label/text response for proper right-click detection
        response.context_menu(|ui| {
            if let Some(ctx_action) = show_entity_context_menu(ui, entity) {
                *action = Some(ctx_action);
            }
        });

        // Render children if expanded
        if is_expanded {
            for child in &node.children {
                self.show_entity_node(
                    ui, child, world, state, ui_state,
                    game_path, asset_manager, depth + 1, action,
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

        // Set consistent spacing
        ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);

        // Header
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(Typography::heading("Scene Tree"));
        });
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // Collect entities
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
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(Typography::empty_state("(No entities)"));
            });
            return None;
        }

        // Find root entities
        let root_entities: Vec<_> = all_entities.iter()
            .filter(|&&entity| {
                let handle = EntityHandle::new(entity);
                world.get::<Parent>(handle).is_err()
            })
            .copied()
            .collect();

        // Build and render tree
        for root_entity in root_entities {
            let tree = EntityNode::from_entity(world, root_entity);
            self.show_entity_node(
                ui, &tree, world, state, ui_state,
                game_path, asset_manager, 0, &mut action,
            );
        }

        // Drop zone for making entities root-level (also serves as context menu area)
        ui.add_space(8.0);
        let drop_zone_height = 28.0;
        let (rect, drop_response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), drop_zone_height),
            egui::Sense::click(),  // Changed to click to support context menu
        );

        if drop_response.hovered() && ui.input(|i| i.pointer.any_down()) {
            ui.painter().rect_filled(rect, Radius::SMALL, Colors::ACCENT.gamma_multiply(0.15));
            ui.painter().text(
                rect.center(), egui::Align2::CENTER_CENTER,
                "Drop here to make root entity",
                egui::FontId::default(), Colors::ACCENT,
            );
        } else {
            ui.painter().rect_stroke(rect, Radius::SMALL, egui::Stroke::new(1.0, Colors::STROKE_DEFAULT));
            ui.painter().text(
                rect.center(), egui::Align2::CENTER_CENTER,
                "Drop here to make root entity",
                egui::FontId::default(), Colors::TEXT_MUTED,
            );
        }

        // Handle drop on root zone
        if let Some(dropped_entity_bits) = drop_response.dnd_release_payload::<u64>() {
            if let Some(dropped_entity) = world.inner().iter()
                .find(|e| e.entity().to_bits().get() == *dropped_entity_bits)
                .map(|e| e.entity())
            {
                let handle = EntityHandle::new(dropped_entity);

                let global_opt = world.get::<GlobalTransform>(handle).ok().map(|r| *r);
                if let Some(global) = global_opt {
                    let new_local = Transform::from_components(
                        global.position, global.rotation, global.scale,
                    );
                    let _ = world.set(handle, new_local);
                }

                let _ = longhorn_core::ecs::hierarchy::clear_parent(world, handle);
            }
        }

        // Context menu on drop zone
        drop_response.context_menu(|ui| {
            if let Some(ctx_action) = show_scene_tree_empty_context_menu(ui) {
                action = Some(ctx_action);
            }
        });

        // Context menu for remaining empty space
        let remaining = ui.available_size();
        if remaining.y > 0.0 {
            let (rect, response) = ui.allocate_exact_size(remaining, egui::Sense::click());
            if rect.height() > 0.0 {
                response.context_menu(|ui| {
                    if let Some(ctx_action) = show_scene_tree_empty_context_menu(ui) {
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
