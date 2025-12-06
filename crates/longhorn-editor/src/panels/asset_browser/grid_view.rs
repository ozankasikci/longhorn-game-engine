use egui::{Ui, RichText, Color32};
use crate::asset_browser_state::{AssetBrowserState, DirectoryNode, FileType};
use crate::styling::Spacing;
use super::{AssetBrowserAction, ContextAction};

/// Render the grid view of the selected folder's contents
pub fn show_grid_view(
    ui: &mut Ui,
    state: &mut AssetBrowserState,
    root: &DirectoryNode,
) -> Option<AssetBrowserAction> {
    let folder = find_folder(root, &state.selected_folder).unwrap_or(root);
    let mut action = None;

    // Breadcrumb navigation - simple path display
    ui.horizontal(|ui| {
        let path_str = folder.name.clone();
        ui.label(RichText::new(path_str).strong());
    });
    ui.add_space(Spacing::MARGIN_SMALL);
    ui.separator();
    ui.add_space(Spacing::MARGIN_SMALL);

    // Check if folder is empty
    if folder.children.is_empty() && folder.files.is_empty() {
        ui.label(RichText::new("Empty folder").color(Color32::from_gray(100)));
        return None;
    }

    // Simple list view - subfolders first
    for child in &folder.children {
        let is_selected = state.selected_folder == child.path;
        let label = format!("[DIR] {}", child.name);
        let response = ui.selectable_label(is_selected, &label);

        if response.double_clicked() {
            state.selected_folder = child.path.clone();
            state.expanded_folders.insert(child.path.clone());
        }
    }

    // Then files
    for file in &folder.files {
        let is_selected = state.selected_file.as_ref() == Some(&file.path);

        let icon = match file.file_type {
            FileType::Script => "[JS]",
            FileType::Image => "[IMG]",
            FileType::Audio => "[SND]",
            FileType::Scene => "[SCN]",
            FileType::Unknown => "[???]",
        };
        let label = format!("{} {}", icon, file.name);

        let response = ui.selectable_label(is_selected, &label);

        if response.clicked() {
            state.selected_file = Some(file.path.clone());
        }

        // Context menu
        response.context_menu(|ui| {
            if ui.button("Rename").clicked() {
                action = Some(AssetBrowserAction::Context(ContextAction::Rename(file.path.clone())));
                ui.close_menu();
            }
            if ui.button("Delete").clicked() {
                action = Some(AssetBrowserAction::Context(ContextAction::Delete(file.path.clone())));
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Open Externally").clicked() {
                action = Some(AssetBrowserAction::OpenExternal(file.path.clone()));
                ui.close_menu();
            }
        });

        // Double-click to open
        if response.double_clicked() {
            action = Some(match file.file_type {
                FileType::Script => AssetBrowserAction::OpenScript(file.path.clone()),
                FileType::Image => AssetBrowserAction::OpenImage(file.path.clone()),
                _ => AssetBrowserAction::OpenExternal(file.path.clone()),
            });
        }
    }

    action
}

fn find_folder<'a>(root: &'a DirectoryNode, path: &std::path::Path) -> Option<&'a DirectoryNode> {
    if root.path == path {
        return Some(root);
    }
    for child in &root.children {
        if let Some(found) = find_folder(child, path) {
            return Some(found);
        }
    }
    None
}
