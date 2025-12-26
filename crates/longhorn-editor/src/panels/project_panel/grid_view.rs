use egui::{Ui, RichText};
use crate::project_panel_state::{ProjectPanelState, DirectoryNode, FileType};
use crate::styling::{Colors, Spacing, Typography, Icons, IconSize};
use crate::ui_state::{UiStateTracker, TriggerAction};
use crate::ui::context_menus::{show_create_submenu, show_folder_context_menu};
use super::{ProjectPanelAction, ContextAction};

/// Render the grid view of the selected folder's contents
pub fn show_grid_view(
    ui: &mut Ui,
    state: &mut ProjectPanelState,
    root: &DirectoryNode,
    ui_state: &mut UiStateTracker,
) -> Option<ProjectPanelAction> {
    let folder = find_folder(root, &state.selected_folder).unwrap_or(root);
    log::debug!("Grid view: folder={}, files={}, children={}",
        folder.name, folder.files.len(), folder.children.len());
    let mut action = None;

    // Breadcrumb navigation
    ui.horizontal(|ui| {
        ui.add_space(Spacing::LIST_ITEM_PADDING_H);

        // Home icon
        if ui.add(egui::Button::new(Icons::icon(Icons::HOME)).frame(false))
            .on_hover_text("Project root")
            .clicked()
        {
            state.selected_folder = root.path.clone();
        }

        // Only show breadcrumb if we're in a subfolder
        if folder.path != root.path {
            ui.add_space(Spacing::ITEM_GAP);

            // Get path relative to project root
            let relative_path = folder.path.strip_prefix(&root.path).unwrap_or(&folder.path);
            let components: Vec<_> = relative_path.components().collect();

            for (i, component) in components.iter().enumerate() {
                // Separator
                ui.label(Icons::icon_colored(Icons::CHEVRON_RIGHT, Colors::TEXT_MUTED));
                ui.add_space(Spacing::ITEM_GAP);

                let component_name = component.as_os_str().to_string_lossy();
                let is_last = i == components.len() - 1;

                // Style differently for current folder
                let btn_text = if is_last {
                    RichText::new(component_name.as_ref()).strong()
                } else {
                    RichText::new(component_name.as_ref()).color(Colors::TEXT_MUTED)
                };

                let btn = egui::Button::new(btn_text).frame(false);
                if ui.add(btn).clicked() {
                    // Build absolute path up to this component
                    let mut path = root.path.clone();
                    for c in &components[0..=i] {
                        path.push(c);
                    }
                    state.selected_folder = path.clone();
                    state.current_folder = path;
                }
                ui.add_space(Spacing::ITEM_GAP);
            }
        }
    });
    ui.add_space(Spacing::SECTION_HEADER_BOTTOM);
    ui.separator();
    ui.add_space(Spacing::SECTION_HEADER_BOTTOM);

    // Check if folder is empty - but don't return early, allow context menu
    let is_empty = folder.children.is_empty() && folder.files.is_empty();
    if is_empty {
        ui.label(Typography::empty_state("Empty folder"));
        ui.add_space(Spacing::SECTION_GAP);
    }

    // Folders section
    if !folder.children.is_empty() {
        ui.add_space(Spacing::SECTION_HEADER_TOP);
        ui.label(Typography::section_header("FOLDERS"));
        ui.add_space(Spacing::SECTION_HEADER_BOTTOM);

        for child in &folder.children {
            let is_selected = state.selected_folder == child.path;

            // Folder item
            let response = ui.horizontal(|ui| {
                ui.add_space(Spacing::LIST_ITEM_PADDING_H);

                // Folder icon
                let icon = if is_selected { Icons::FOLDER_OPEN } else { Icons::FOLDER };
                ui.label(Icons::icon_sized(icon, IconSize::MD));

                ui.add_space(Spacing::ICON_TEXT_GAP);

                // Folder name
                let text = if is_selected {
                    RichText::new(&child.name).strong()
                } else {
                    RichText::new(&child.name)
                };

                ui.selectable_label(is_selected, text)
            }).response;

            if response.double_clicked() {
                state.selected_folder = child.path.clone();
                state.expanded_folders.insert(child.path.clone());
            }

            // Context menu for folders
            response.context_menu(|ui| {
                if let Some(ctx_action) = show_folder_context_menu(ui, &child.path) {
                    action = Some(ProjectPanelAction::Context(ctx_action));
                }
            });

            ui.add_space(Spacing::ITEM_GAP);
        }

        ui.add_space(Spacing::SECTION_GAP);
    }

    // Files section
    if !folder.files.is_empty() {
        ui.add_space(Spacing::SECTION_HEADER_TOP);
        ui.label(Typography::section_header("FILES"));
        ui.add_space(Spacing::SECTION_HEADER_BOTTOM);
    }

    for file in &folder.files {
        let is_selected = state.selected_file.as_ref() == Some(&file.path);

        // Get colored icon from FileType
        let icon_char = file.file_type.icon_char();
        let [r, g, b] = file.file_type.icon_color();
        let icon_color = egui::Color32::from_rgb(r, g, b);

        // Create unique ID for this file (use filename for simplicity)
        let element_id = format!("asset_file_{}", file.name);

        // Register as clickable element
        ui_state.register_clickable(&element_id, &file.name, "file");

        // Check for pending remote trigger
        let pending_action = ui_state.has_pending_trigger_for(&element_id).cloned();

        // File item layout
        let mut response = ui.horizontal(|ui| {
            ui.add_space(Spacing::LIST_ITEM_PADDING_H);

            // Icon
            ui.label(RichText::new(icon_char).size(IconSize::MD).color(icon_color));

            ui.add_space(Spacing::ICON_TEXT_GAP);

            // File name - this is the clickable part
            let text = if is_selected {
                RichText::new(&file.name).strong()
            } else {
                RichText::new(&file.name)
            };

            let label_response = if file.file_type == FileType::Image {
                // Make draggable for images
                ui.dnd_drag_source(
                    egui::Id::new(format!("drag_source_{}", file.path.display())),
                    file.path.clone(),
                    |ui| {
                        ui.selectable_label(is_selected, text)
                    },
                ).response
            } else {
                ui.selectable_label(is_selected, text)
            };

            // Show file size hint on the right
            if let Some(_size) = file.size {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(Typography::muted(file.format_size()));
                });
            }

            label_response
        }).inner;

        // Enhanced tooltip with file info
        if let Some(_size) = file.size {
            let ext = file.extension.as_ref().map(|e| e.as_str()).unwrap_or("unknown");
            response = response.on_hover_text(format!(
                "{}\nSize: {}\nType: {}",
                file.name,
                file.format_size(),
                ext
            ));
        }

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
                let new_action = if file.file_type == FileType::Scene {
                    log::info!("  -> Creating OpenScene action");
                    ProjectPanelAction::OpenScene(file.path.clone())
                } else if file.file_type.is_text_editable() {
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
            action = Some(if file.file_type == FileType::Scene {
                ProjectPanelAction::OpenScene(file.path.clone())
            } else if file.file_type.is_text_editable() {
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
            // Create submenu - creates in the current folder
            if let Some(ctx_action) = show_create_submenu(ui, &folder.path) {
                action = Some(ProjectPanelAction::Context(ctx_action));
            }
            ui.separator();
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

        ui.add_space(Spacing::ITEM_GAP);
    }

    // Allocate remaining space to capture right-clicks on empty area
    let remaining = ui.available_size();
    if remaining.y > 0.0 {
        let (rect, response) = ui.allocate_exact_size(remaining, egui::Sense::click());

        if rect.height() > 0.0 {
            response.context_menu(|ui| {
                if let Some(ctx_action) = show_folder_context_menu(ui, &folder.path) {
                    action = Some(ProjectPanelAction::Context(ctx_action));
                }
            });
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
