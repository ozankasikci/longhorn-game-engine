use egui::Ui;
use crate::project_panel_state::{ProjectPanelState, DirectoryNode};
use crate::styling::{Spacing, Icons, IconSize};
use crate::ui::context_menus::show_folder_context_menu;
use super::ProjectPanelAction;

/// Render the folder tree view
pub fn show_tree_view(
    ui: &mut Ui,
    state: &mut ProjectPanelState,
    root: &DirectoryNode,
) -> Option<ProjectPanelAction> {
    show_tree_node(ui, state, root, 0)
}

fn show_tree_node(
    ui: &mut Ui,
    state: &mut ProjectPanelState,
    node: &DirectoryNode,
    depth: usize,
) -> Option<ProjectPanelAction> {
    let mut action = None;

    let is_expanded = state.expanded_folders.contains(&node.path);
    let is_selected = state.selected_folder == node.path;
    let has_children = !node.children.is_empty();

    // Indent based on depth
    let indent = depth as f32 * Spacing::TREE_INDENT;

    // Render as selectable label with icon
    let response = ui.horizontal(|ui| {
        ui.add_space(indent);
        ui.add_space(Spacing::LIST_ITEM_PADDING_H);

        // Expand/collapse icon or spacer
        if has_children {
            let arrow_icon = if is_expanded { Icons::CARET_DOWN } else { Icons::CARET_RIGHT };
            ui.label(Icons::icon_sized(arrow_icon, IconSize::SM));
        } else {
            ui.add_space(IconSize::SM);
        }

        ui.add_space(Spacing::ITEM_GAP);

        // Folder icon
        let folder_icon = if is_expanded { Icons::FOLDER_OPEN } else { Icons::FOLDER };
        ui.label(Icons::icon_sized(folder_icon, IconSize::SM));

        ui.add_space(Spacing::ICON_TEXT_GAP);

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
    }).inner;

    // Context menu for tree folders
    response.context_menu(|ui| {
        if let Some(ctx_action) = show_folder_context_menu(ui, &node.path) {
            action = Some(ProjectPanelAction::Context(ctx_action));
        }
    });

    ui.add_space(Spacing::ITEM_GAP);

    // Render children if expanded
    if is_expanded {
        for child in &node.children {
            if let Some(child_action) = show_tree_node(ui, state, child, depth + 1) {
                action = Some(child_action);
            }
        }
    }

    action
}
