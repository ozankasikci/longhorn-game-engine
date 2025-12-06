use egui::{Ui, RichText};
use crate::project_panel_state::{ProjectPanelState, DirectoryNode, FileType};
use crate::styling::{Colors, Spacing};
use crate::ui_state::{UiStateTracker, TriggerAction};
use super::{ProjectPanelAction, ContextAction};

/// Render the grid view of the selected folder's contents
pub fn show_grid_view(
    ui: &mut Ui,
    state: &mut ProjectPanelState,
    root: &DirectoryNode,
    ui_state: &mut UiStateTracker,
) -> Option<ProjectPanelAction> {
    let folder = find_folder(root, &state.selected_folder).unwrap_or(root);
    // Debug: log only once every 60 frames to avoid log spam
    static mut FRAME_COUNT: u32 = 0;
    unsafe {
        FRAME_COUNT += 1;
        if FRAME_COUNT % 60 == 0 {
            log::debug!("Grid view: folder={}, files={}, children={}",
                folder.name, folder.files.len(), folder.children.len());
        }
    }
    let mut action = None;

    // Breadcrumb navigation - simple path display
    ui.horizontal(|ui| {
        let path_str = folder.name.clone();
        ui.label(RichText::new(path_str).strong());
    });
    ui.add_space(Spacing::MARGIN_SMALL);
    ui.separator();
    ui.add_space(Spacing::MARGIN_SMALL);

    // Add context menu for the folder area
    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
        // Allocate a small invisible rect for the background context menu
        let response = ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::click());
        response.context_menu(|ui| {
            if ui.button("Import Asset...").clicked() {
                action = Some(ProjectPanelAction::Context(ContextAction::ImportAsset(folder.path.clone())));
                ui.close_menu();
            }
        });
    });

    // Check if folder is empty
    if folder.children.is_empty() && folder.files.is_empty() {
        ui.label(RichText::new("Empty folder").color(Colors::TEXT_MUTED));
        return action;
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

        // Context menu for folders
        response.context_menu(|ui| {
            if ui.button("Import Asset...").clicked() {
                action = Some(ProjectPanelAction::Context(ContextAction::ImportAsset(child.path.clone())));
                ui.close_menu();
            }
        });
    }

    // Then files
    for file in &folder.files {
        let is_selected = state.selected_file.as_ref() == Some(&file.path);

        let icon = match file.file_type {
            FileType::Script => "[JS]",
            FileType::Text => "[TXT]",
            FileType::Image => "[IMG]",
            FileType::Audio => "[SND]",
            FileType::Scene => "[SCN]",
            FileType::Unknown => "[???]",
        };
        let label = format!("{} {}", icon, file.name);

        // Create unique ID for this file (use filename for simplicity)
        let element_id = format!("asset_file_{}", file.name);

        // Register as clickable element
        ui_state.register_clickable(&element_id, &file.name, "file");

        // Check for pending remote trigger
        let pending_action = ui_state.has_pending_trigger_for(&element_id).cloned();

        // Enable drag source for image files
        let response = if file.file_type == FileType::Image {
            // Make this item draggable by wrapping it in a drag source
            let response = ui.dnd_drag_source(
                egui::Id::new(format!("drag_source_{}", file.path.display())),
                file.path.clone(),
                |ui| {
                    ui.selectable_label(is_selected, &label)
                },
            ).response;

            response
        } else {
            ui.selectable_label(is_selected, &label)
        };

        // Log ALL interaction states for debugging
        if response.clicked() {
            log::info!("UI CLICK: response.clicked()=true for {:?}", file.name);
        }
        if response.double_clicked() {
            log::info!("UI DOUBLE-CLICK: response.double_clicked()=true for {:?}", file.name);
        }
        if response.hovered() && ui.input(|i| i.pointer.any_click()) {
            log::debug!("UI HOVER+CLICK: hovered and pointer clicked for {:?}", file.name);
        }

        // Handle click (from UI or remote)
        // Use manual double-click detection since egui's double_clicked() is unreliable
        let is_clicked = response.clicked() || pending_action == Some(TriggerAction::Click);
        if is_clicked {
            // Check if this is actually a double-click using our manual timer
            let is_manual_double_click = state.check_double_click(&file.path);

            if is_manual_double_click {
                // This is a double-click!
                log::info!("=== DOUBLE-CLICK DETECTED ===");
                log::info!("  File: {:?}", file.path);
                log::info!("  file_type: {:?}", file.file_type);
                log::info!("  is_text_editable: {}", file.file_type.is_text_editable());
                let new_action = if file.file_type.is_text_editable() {
                    log::info!("  -> Creating OpenScript action");
                    ProjectPanelAction::OpenScript(file.path.clone())
                } else if file.file_type == FileType::Image {
                    log::info!("  -> Creating OpenImage action");
                    ProjectPanelAction::OpenImage(file.path.clone())
                } else {
                    log::info!("  -> Creating OpenExternal action");
                    ProjectPanelAction::OpenExternal(file.path.clone())
                };
                log::info!("  Action created: {:?}", new_action);
                action = Some(new_action);
            } else {
                // Single click - just select
                state.selected_file = Some(file.path.clone());
                if pending_action.is_some() {
                    log::info!("REMOTE: Single-click on {:?}", file.path);
                }
            }
        }

        // Handle explicit double-click from remote command
        let is_remote_double_click = pending_action == Some(TriggerAction::DoubleClick);
        if is_remote_double_click {
            log::info!("REMOTE: Double-click on {:?}, file_type={:?}, is_text_editable={}",
                file.path, file.file_type, file.file_type.is_text_editable());
            action = Some(if file.file_type.is_text_editable() {
                ProjectPanelAction::OpenScript(file.path.clone())
            } else if file.file_type == FileType::Image {
                ProjectPanelAction::OpenImage(file.path.clone())
            } else {
                ProjectPanelAction::OpenExternal(file.path.clone())
            });
        }

        // Handle right-click (from UI or remote) - opens context menu action directly
        let is_right_clicked = pending_action == Some(TriggerAction::RightClick);
        if is_right_clicked && file.file_type.is_text_editable() {
            // Directly trigger "Open in Editor" for right-click on text files
            log::info!("REMOTE: Right-click (Open in Editor) on {:?}", file.path);
            action = Some(ProjectPanelAction::OpenScript(file.path.clone()));
        }

        // Context menu (right-click) - for manual interaction only
        response.context_menu(|ui| {
            // Show "Open in Editor" for text-editable files
            if file.file_type.is_text_editable() {
                if ui.button("Open in Editor").clicked() {
                    log::info!("UI: Open in Editor clicked for {:?}", file.path);
                    action = Some(ProjectPanelAction::OpenScript(file.path.clone()));
                    ui.close_menu();
                }
                ui.separator();
            }
            if ui.button("Rename").clicked() {
                action = Some(ProjectPanelAction::Context(ContextAction::Rename(file.path.clone())));
                ui.close_menu();
            }
            if ui.button("Delete").clicked() {
                action = Some(ProjectPanelAction::Context(ContextAction::Delete(file.path.clone())));
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Open Externally").clicked() {
                action = Some(ProjectPanelAction::OpenExternal(file.path.clone()));
                ui.close_menu();
            }
        });

        // Consume the trigger if it was for this element
        if pending_action.is_some() {
            ui_state.take_pending_trigger_action();
        }
    }

    if action.is_some() {
        log::info!("=== show_grid_view returning action: {:?} ===", action);
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
