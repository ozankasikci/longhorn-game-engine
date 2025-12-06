use egui::Ui;
use crate::asset_browser_state::{AssetBrowserState, DirectoryNode};
use super::AssetBrowserAction;

/// Render the folder tree view
pub fn show_tree_view(
    ui: &mut Ui,
    state: &mut AssetBrowserState,
    root: &DirectoryNode,
) -> Option<AssetBrowserAction> {
    show_tree_node(ui, state, root, 0)
}

fn show_tree_node(
    ui: &mut Ui,
    state: &mut AssetBrowserState,
    node: &DirectoryNode,
    depth: usize,
) -> Option<AssetBrowserAction> {
    let mut action = None;

    let is_expanded = state.expanded_folders.contains(&node.path);
    let is_selected = state.selected_folder == node.path;

    // Indent based on depth
    ui.horizontal(|ui| {
        ui.add_space(depth as f32 * 16.0);

        // Expand/collapse button for folders with children
        let icon = if node.children.is_empty() {
            "  "
        } else if is_expanded {
            "v "
        } else {
            "> "
        };

        let folder_icon = "[D]";
        let label = format!("{}{} {}", icon, folder_icon, node.name);

        let response = ui.selectable_label(is_selected, label);

        if response.clicked() {
            state.selected_folder = node.path.clone();
            if !node.children.is_empty() {
                if is_expanded {
                    state.expanded_folders.remove(&node.path);
                } else {
                    state.expanded_folders.insert(node.path.clone());
                }
            }
        }
    });

    // Show children if expanded
    if is_expanded {
        for child in &node.children {
            if let Some(child_action) = show_tree_node(ui, state, child, depth + 1) {
                action = Some(child_action);
            }
        }
    }

    action
}
