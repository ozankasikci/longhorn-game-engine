use egui::Ui;
use crate::project_panel_state::{ProjectPanelState, DirectoryNode};
use crate::styling::Spacing;
use crate::ui::context_menus::show_folder_context_menu;
use super::ProjectPanelAction;

/// Render the folder tree view - follows scene_tree.rs pattern
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
    let indent = depth as f32 * Spacing::INDENT;

    // Build display text with arrow
    let arrow = if has_children {
        if is_expanded { "v " } else { "> " }
    } else {
        "  "
    };
    let display_text = format!("{}{}", arrow, node.name);

    // Render as selectable label (same pattern as scene_tree.rs)
    let response = ui.horizontal(|ui| {
        ui.add_space(indent);
        let label_response = ui.selectable_label(is_selected, &display_text);
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
