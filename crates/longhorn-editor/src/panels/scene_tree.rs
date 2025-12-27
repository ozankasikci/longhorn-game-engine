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

/// Drop position relative to a target entity
#[derive(Debug, Clone, Copy, PartialEq)]
enum DropPosition {
    /// Insert before target (as sibling)
    Before,
    /// Make child of target
    AsChild,
    /// Insert after target (as sibling)
    After,
}

/// Current drag-drop state stored in egui temp data
#[derive(Debug, Clone)]
struct DragDropState {
    target_entity: hecs::Entity,
    position: DropPosition,
    line_y: f32,
    line_indent: f32,
}

pub struct SceneTreePanel {
    pub expanded_entities: HashSet<u64>,
    /// Ordered list of root entity IDs (entities without parents)
    root_order: Vec<u64>,
}

struct EntityNode {
    entity: hecs::Entity,
    name: String,
    children: Vec<EntityNode>,
    parent: Option<hecs::Entity>,
    index_in_parent: usize,
}

impl EntityNode {
    fn from_entity(world: &World, entity: hecs::Entity, parent: Option<hecs::Entity>, index: usize) -> Self {
        let handle = EntityHandle::new(entity);

        let name = world.get::<Name>(handle)
            .ok()
            .map(|n| n.0.clone())
            .unwrap_or_else(|| format!("Entity {}", entity.id()));

        let mut children = Vec::new();

        if let Ok(children_comp) = world.get::<Children>(handle) {
            for (idx, &child_entity) in children_comp.iter().enumerate() {
                children.push(EntityNode::from_entity(world, child_entity, Some(entity), idx));
            }
        }

        EntityNode { entity, name, children, parent, index_in_parent: index }
    }
}

impl SceneTreePanel {
    pub fn new() -> Self {
        Self {
            expanded_entities: HashSet::new(),
            root_order: Vec::new(),
        }
    }

    /// Sync root_order with actual root entities in the world.
    /// Adds new roots, removes deleted ones, preserves order of existing ones.
    fn sync_root_order(&mut self, root_entities: &[hecs::Entity]) {
        let root_bits: HashSet<u64> = root_entities.iter()
            .map(|e| e.to_bits().get())
            .collect();

        // Remove entities that are no longer roots
        self.root_order.retain(|bits| root_bits.contains(bits));

        // Add new roots that aren't in our order yet (at the end)
        for entity in root_entities {
            let bits = entity.to_bits().get();
            if !self.root_order.contains(&bits) {
                self.root_order.push(bits);
            }
        }
    }

    /// Insert a root entity at a specific index
    fn insert_root_at(&mut self, entity_bits: u64, index: usize) {
        // Find current position before removing
        let old_index = self.root_order.iter().position(|&b| b == entity_bits);

        // Remove if already present
        self.root_order.retain(|&b| b != entity_bits);

        // Adjust index if the entity was before the target position
        // (because removing it shifted everything down)
        let adjusted_index = match old_index {
            Some(old) if old < index => index.saturating_sub(1),
            _ => index,
        };

        // Insert at adjusted position
        let final_index = adjusted_index.min(self.root_order.len());
        self.root_order.insert(final_index, entity_bits);
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
        tree_left: f32,
        tree_width: f32,
    ) {
        let entity = node.entity;
        let entity_bits = entity.to_bits().get();
        let is_selected = state.is_selected(entity);
        let has_children = !node.children.is_empty();
        let is_expanded = self.expanded_entities.contains(&entity_bits);
        let is_renaming = state.renaming_entity == Some(entity);

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
                    if world.set(handle, Name::new(&rename_text)).is_ok() {
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

        let row_response = horizontal_response.response.interact(egui::Sense::drag());
        let inner_response = horizontal_response.inner;
        let row_rect = row_response.rect;

        // Drag-drop detection with position zones
        let drag_state_id = egui::Id::new("scene_tree_drag_state");

        if let Some(payload) = row_response.dnd_hover_payload::<u64>() {
            let dragged_bits = *payload;
            if dragged_bits != entity_bits {
                // Get cursor position relative to this row
                if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
                    let relative_y = (pointer_pos.y - row_rect.top()) / row_rect.height();

                    let position = if relative_y < 0.25 {
                        DropPosition::Before
                    } else if relative_y > 0.75 {
                        DropPosition::After
                    } else {
                        DropPosition::AsChild
                    };

                    let (line_y, line_indent) = match position {
                        DropPosition::Before => (row_rect.top(), indent),
                        DropPosition::After => (row_rect.bottom(), indent),
                        DropPosition::AsChild => (0.0, 0.0), // Not used for AsChild
                    };

                    let drag_state = DragDropState {
                        target_entity: entity,
                        position,
                        line_y,
                        line_indent,
                    };

                    ui.ctx().data_mut(|d| d.insert_temp(drag_state_id, drag_state));
                }
            }
        }

        // Draw visual feedback based on stored drag state
        let current_drag_state: Option<DragDropState> = ui.ctx().data(|d| d.get_temp(drag_state_id));

        if let Some(ref drag_state) = current_drag_state {
            if drag_state.target_entity == entity {
                match drag_state.position {
                    DropPosition::AsChild => {
                        // Draw border around item
                        ui.painter().rect_stroke(
                            row_rect,
                            Radius::SMALL,
                            egui::Stroke::new(2.0, Colors::ACCENT),
                        );
                    }
                    DropPosition::Before | DropPosition::After => {
                        // Draw horizontal insertion line
                        let line_rect = egui::Rect::from_min_size(
                            egui::pos2(tree_left + drag_state.line_indent, drag_state.line_y - 1.0),
                            egui::vec2(tree_width - drag_state.line_indent, 2.0),
                        );
                        ui.painter().rect_filled(line_rect, 0.0, Colors::ACCENT);
                    }
                }
            }
        }

        // Handle drop
        if let Some(dropped_entity_bits) = row_response.dnd_release_payload::<u64>() {
            if *dropped_entity_bits != entity_bits {
                if let Some(dragged_entity) = world.inner().iter()
                    .find(|e| e.entity().to_bits().get() == *dropped_entity_bits)
                    .map(|e| e.entity())
                {
                    // Get current drag state to know drop position
                    let drop_state: Option<DragDropState> = ui.ctx().data(|d| d.get_temp(drag_state_id));

                    let dragged_handle = EntityHandle::new(dragged_entity);
                    let target_handle = EntityHandle::new(entity);

                    // Preserve global transform
                    let dragged_global_opt = world.get::<GlobalTransform>(dragged_handle).ok().map(|r| *r);

                    if let Some(drop_state) = drop_state {
                        match drop_state.position {
                            DropPosition::AsChild => {
                                // Make child of target
                                if let Some(dragged_global) = dragged_global_opt {
                                    let parent_global_opt = world.get::<GlobalTransform>(target_handle).ok().map(|g| *g);
                                    if let Some(parent_global) = parent_global_opt {
                                        let new_local = parent_global.to_local_transform(&dragged_global);
                                        let _ = world.set(dragged_handle, new_local);
                                    }
                                }
                                let _ = longhorn_core::ecs::hierarchy::set_parent(world, dragged_handle, target_handle);
                            }
                            DropPosition::Before => {
                                // Insert as sibling before target
                                self.insert_as_sibling(world, dragged_handle, target_handle, node.parent, node.index_in_parent, dragged_global_opt);
                            }
                            DropPosition::After => {
                                // Insert as sibling after target
                                self.insert_as_sibling(world, dragged_handle, target_handle, node.parent, node.index_in_parent + 1, dragged_global_opt);
                            }
                        }
                        *action = Some(SceneTreeAction::EntityRenamed(dragged_entity)); // Trigger dirty state
                    }

                    // Clear drag state
                    ui.ctx().data_mut(|d| d.remove::<DragDropState>(drag_state_id));
                }
            }
        }

        // Context menu - attach to the inner label/text response for proper right-click detection
        inner_response.context_menu(|ui| {
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
                    tree_left, tree_width,
                );
            }
        }
    }

    fn insert_as_sibling(
        &mut self,
        world: &mut World,
        dragged: EntityHandle,
        _target: EntityHandle,
        target_parent: Option<hecs::Entity>,
        insert_index: usize,
        dragged_global_opt: Option<GlobalTransform>,
    ) {
        if let Some(parent_entity) = target_parent {
            // Target has a parent - insert into that parent at index
            let parent_handle = EntityHandle::new(parent_entity);

            // Preserve global transform
            if let Some(dragged_global) = dragged_global_opt {
                let parent_global_opt = world.get::<GlobalTransform>(parent_handle).ok().map(|g| *g);
                if let Some(parent_global) = parent_global_opt {
                    let new_local = parent_global.to_local_transform(&dragged_global);
                    let _ = world.set(dragged, new_local);
                }
            }

            let _ = longhorn_core::ecs::hierarchy::set_parent_at_index(world, dragged, parent_handle, insert_index);
        } else {
            // Target is a root entity - reorder within roots

            // Preserve global transform when becoming root
            if let Some(dragged_global) = dragged_global_opt {
                let new_local = Transform::from_components(
                    dragged_global.position, dragged_global.rotation, dragged_global.scale,
                );
                let _ = world.set(dragged, new_local);
            }

            // Clear parent if it has one
            let _ = longhorn_core::ecs::hierarchy::clear_parent(world, dragged);

            // Update root ordering
            let dragged_bits = dragged.id().to_bits().get();
            self.insert_root_at(dragged_bits, insert_index);
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

        // Get tree area bounds for drawing insertion lines
        let tree_left = ui.cursor().left();
        let tree_width = ui.available_width();

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

        // Find root entities (entities without Parent component)
        let root_entities: Vec<_> = all_entities.iter()
            .filter(|&&entity| {
                let handle = EntityHandle::new(entity);
                world.get::<Parent>(handle).is_err()
            })
            .copied()
            .collect();

        // Sync our root ordering with actual root entities
        self.sync_root_order(&root_entities);

        // Build entity lookup for ordered iteration
        let entity_map: std::collections::HashMap<u64, hecs::Entity> = root_entities.iter()
            .map(|e| (e.to_bits().get(), *e))
            .collect();

        // Build and render tree in our stored order
        // Clone root_order to avoid borrow conflict with show_entity_node
        let root_order_copy = self.root_order.clone();
        for (idx, &entity_bits) in root_order_copy.iter().enumerate() {
            if let Some(&root_entity) = entity_map.get(&entity_bits) {
                let tree = EntityNode::from_entity(world, root_entity, None, idx);
                self.show_entity_node(
                    ui, &tree, world, state, ui_state,
                    game_path, asset_manager, 0, &mut action,
                    tree_left, tree_width,
                );
            }
        }

        // Empty area at bottom - drop here to make root
        ui.add_space(8.0);
        let remaining = ui.available_size();
        if remaining.y > 0.0 {
            let (rect, response) = ui.allocate_exact_size(remaining, egui::Sense::click());

            // Handle drop in empty area - make root entity
            let drag_state_id = egui::Id::new("scene_tree_drag_state");

            if response.dnd_hover_payload::<u64>().is_some() {
                // Show visual feedback - line at top of empty area
                let line_rect = egui::Rect::from_min_size(
                    egui::pos2(tree_left, rect.top() - 1.0),
                    egui::vec2(tree_width, 2.0),
                );
                ui.painter().rect_filled(line_rect, 0.0, Colors::ACCENT);

                // Clear any entity-specific drag state so we know this is "drop to root"
                ui.ctx().data_mut(|d| d.remove::<DragDropState>(drag_state_id));
            }

            if let Some(dropped_entity_bits) = response.dnd_release_payload::<u64>() {
                if let Some(dropped_entity) = world.inner().iter()
                    .find(|e| e.entity().to_bits().get() == *dropped_entity_bits)
                    .map(|e| e.entity())
                {
                    let handle = EntityHandle::new(dropped_entity);

                    // Preserve global transform when making root
                    let global_opt = world.get::<GlobalTransform>(handle).ok().map(|r| *r);
                    if let Some(global) = global_opt {
                        let new_local = Transform::from_components(
                            global.position, global.rotation, global.scale,
                        );
                        let _ = world.set(handle, new_local);
                    }

                    let _ = longhorn_core::ecs::hierarchy::clear_parent(world, handle);
                    action = Some(SceneTreeAction::EntityRenamed(dropped_entity)); // Trigger dirty state
                }
            }

            // Context menu for empty space
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

// Make root_order accessible for serialization if needed
impl SceneTreePanel {
    pub fn root_order(&self) -> &[u64] {
        &self.root_order
    }

    pub fn set_root_order(&mut self, order: Vec<u64>) {
        self.root_order = order;
    }
}
