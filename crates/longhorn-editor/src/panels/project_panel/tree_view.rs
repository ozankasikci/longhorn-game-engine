use egui::{Ui, Vec2};
use crate::project_panel_state::{ProjectPanelState, DirectoryNode};
use crate::styling::{Colors, Icons, IconSize, Radius};
use crate::ui::context_menus::show_folder_context_menu;
use super::ProjectPanelAction;

/// Consistent tree item height
const TREE_ITEM_HEIGHT: f32 = 20.0;
/// Indentation per level
const TREE_INDENT: f32 = 16.0;

/// Render the folder tree view
pub fn show_tree_view(
    ui: &mut Ui,
    state: &mut ProjectPanelState,
    root: &DirectoryNode,
) -> Option<ProjectPanelAction> {
    // Set consistent spacing for tree
    ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);

    // Detect if external files are being dragged over the window
    let files_hovering = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());

    show_tree_node(ui, state, root, 0, files_hovering)
}

fn show_tree_node(
    ui: &mut Ui,
    state: &mut ProjectPanelState,
    node: &DirectoryNode,
    depth: usize,
    files_hovering: bool,
) -> Option<ProjectPanelAction> {
    let mut action = None;

    let is_expanded = state.expanded_folders.contains(&node.path);
    let is_selected = state.selected_folder == node.path;
    let has_children = !node.children.is_empty();

    let indent = depth as f32 * TREE_INDENT;

    let row_response = ui.horizontal(|ui| {
        ui.set_min_height(TREE_ITEM_HEIGHT);
        ui.add_space(4.0 + indent);

        // Expand/collapse icon
        if has_children {
            let arrow = if is_expanded { Icons::CARET_DOWN } else { Icons::CARET_RIGHT };
            if ui.add(egui::Button::new(Icons::icon_sized(arrow, IconSize::SM)).frame(false)).clicked() {
                if is_expanded {
                    state.expanded_folders.remove(&node.path);
                } else {
                    state.expanded_folders.insert(node.path.clone());
                }
            }
        } else {
            ui.add_space(IconSize::SM + 4.0);
        }

        // Folder icon
        let folder_icon = if is_expanded { Icons::FOLDER_OPEN } else { Icons::FOLDER };
        ui.label(Icons::icon_sized(folder_icon, IconSize::SM));
        ui.add_space(4.0);

        // Folder name
        let label_response = ui.selectable_label(is_selected, &node.name);
        if label_response.clicked() {
            state.selected_folder = node.path.clone();
            if has_children {
                if is_expanded {
                    state.expanded_folders.remove(&node.path);
                } else {
                    state.expanded_folders.insert(node.path.clone());
                }
            }
        }
        label_response
    });

    let response = row_response.inner;

    // Highlight folder when external files hover over it
    if files_hovering && row_response.response.hovered() {
        ui.painter().rect_stroke(
            row_response.response.rect,
            Radius::SMALL,
            egui::Stroke::new(2.0, Colors::ACCENT),
        );
        state.drop_target = Some(node.path.clone());
    }

    // Context menu
    response.context_menu(|ui| {
        if let Some(ctx_action) = show_folder_context_menu(ui, &node.path) {
            action = Some(ProjectPanelAction::Context(ctx_action));
        }
    });

    // Render children if expanded
    if is_expanded {
        for child in &node.children {
            if let Some(child_action) = show_tree_node(ui, state, child, depth + 1, files_hovering) {
                action = Some(child_action);
            }
        }
    }

    action
}
