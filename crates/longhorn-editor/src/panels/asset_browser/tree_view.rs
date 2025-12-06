use egui::Ui;
use crate::asset_browser_state::{AssetBrowserState, DirectoryNode};
use crate::styling::Spacing;
use super::AssetBrowserAction;

/// Render the folder tree view - follows scene_tree.rs pattern
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
    ui.horizontal(|ui| {
        ui.add_space(indent);
        if ui.selectable_label(is_selected, &display_text).clicked() {
            state.selected_folder = node.path.clone();
            if has_children {
                if is_expanded {
                    state.expanded_folders.remove(&node.path);
                } else {
                    state.expanded_folders.insert(node.path.clone());
                }
            }
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
