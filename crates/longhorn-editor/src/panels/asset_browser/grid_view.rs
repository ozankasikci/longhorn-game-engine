use egui::Ui;
use crate::asset_browser_state::{AssetBrowserState, DirectoryNode, FileType};
use super::AssetBrowserAction;

/// Render the grid view of the selected folder's contents
pub fn show_grid_view(
    ui: &mut Ui,
    state: &mut AssetBrowserState,
    root: &DirectoryNode,
) -> Option<AssetBrowserAction> {
    // Find the selected folder in the tree
    let folder = find_folder(root, &state.selected_folder).unwrap_or(root);

    let mut action = None;

    // Show breadcrumb path
    ui.horizontal(|ui| {
        ui.label(folder.path.display().to_string());
    });
    ui.separator();

    // Grid layout for files and subfolders
    let available_width = ui.available_width();
    let item_size = 80.0;
    let columns = ((available_width / item_size) as usize).max(1);

    egui::Grid::new("asset_grid_items")
        .num_columns(columns)
        .spacing([8.0, 8.0])
        .show(ui, |ui| {
            let mut col = 0;

            // Show subfolders first
            for child in &folder.children {
                if show_grid_item(ui, state, &child.path, &child.name, true) {
                    state.selected_folder = child.path.clone();
                    state.expanded_folders.insert(child.path.clone());
                }
                col += 1;
                if col >= columns {
                    ui.end_row();
                    col = 0;
                }
            }

            // Then show files
            for file in &folder.files {
                let is_selected = state.selected_file.as_ref() == Some(&file.path);
                if show_file_grid_item(ui, state, file, is_selected) {
                    // Double-click handling
                    action = Some(match file.file_type {
                        FileType::Script => AssetBrowserAction::OpenScript(file.path.clone()),
                        FileType::Image => AssetBrowserAction::OpenImage(file.path.clone()),
                        _ => AssetBrowserAction::OpenExternal(file.path.clone()),
                    });
                }
                col += 1;
                if col >= columns {
                    ui.end_row();
                    col = 0;
                }
            }
        });

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

fn show_grid_item(
    ui: &mut Ui,
    _state: &mut AssetBrowserState,
    _path: &std::path::Path,
    name: &str,
    is_folder: bool,
) -> bool {
    let icon = if is_folder { "[D]" } else { "[F]" };

    ui.vertical(|ui| {
        ui.set_width(72.0);
        ui.set_height(72.0);

        let response = ui.button(format!("{}\n{}", icon, truncate_name(name, 10)));
        response.double_clicked()
    }).inner
}

fn show_file_grid_item(
    ui: &mut Ui,
    state: &mut AssetBrowserState,
    file: &crate::asset_browser_state::FileEntry,
    is_selected: bool,
) -> bool {
    let icon = match file.file_type {
        FileType::Script => "[S]",
        FileType::Image => "[I]",
        FileType::Audio => "[A]",
        FileType::Scene => "[C]",
        FileType::Unknown => "[?]",
    };

    ui.vertical(|ui| {
        ui.set_width(72.0);
        ui.set_height(72.0);

        let text = format!("{}\n{}", icon, truncate_name(&file.name, 10));
        let response = ui.selectable_label(is_selected, text);

        if response.clicked() {
            state.selected_file = Some(file.path.clone());
        }

        response.double_clicked()
    }).inner
}

fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}...", &name[..max_len.saturating_sub(3)])
    }
}
