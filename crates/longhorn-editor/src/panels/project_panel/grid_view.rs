use egui::{Ui, RichText, Vec2};
use crate::project_panel_state::{ProjectPanelState, DirectoryNode, FileType};
use crate::styling::{Colors, Typography, Icons, IconSize, Radius};
use crate::ui_state::{UiStateTracker, TriggerAction};
use crate::ui::context_menus::{show_create_submenu, show_folder_context_menu};
use super::{ProjectPanelAction, ContextAction};

/// Consistent list item height
const LIST_ITEM_MIN_HEIGHT: f32 = 22.0;

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

    // Detect if external files are being dragged over the window
    let files_hovering = ui.ctx().input(|i| !i.raw.hovered_files.is_empty());

    // Apply consistent spacing for this panel
    ui.spacing_mut().item_spacing = Vec2::new(4.0, 1.0);

    // Breadcrumb navigation bar
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.add_space(8.0);

        // Home icon button
        if ui.add(egui::Button::new(Icons::icon(Icons::HOME)).frame(false))
            .on_hover_text("Project root")
            .clicked()
        {
            state.selected_folder = root.path.clone();
        }

        // Breadcrumb path
        if folder.path != root.path {
            let relative_path = folder.path.strip_prefix(&root.path).unwrap_or(&folder.path);
            let components: Vec<_> = relative_path.components().collect();

            for (i, component) in components.iter().enumerate() {
                ui.label(RichText::new("/").color(Colors::TEXT_MUTED).size(12.0));

                let component_name = component.as_os_str().to_string_lossy();
                let is_last = i == components.len() - 1;

                let btn_text = if is_last {
                    RichText::new(component_name.as_ref()).strong()
                } else {
                    RichText::new(component_name.as_ref()).color(Colors::TEXT_MUTED)
                };

                if ui.add(egui::Button::new(btn_text).frame(false)).clicked() {
                    let mut path = root.path.clone();
                    for c in &components[0..=i] {
                        path.push(c);
                    }
                    state.selected_folder = path.clone();
                    state.current_folder = path;
                }
            }
        }
    });
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(6.0);

    // Empty folder state
    let is_empty = folder.children.is_empty() && folder.files.is_empty();
    if is_empty {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(Typography::empty_state("Empty folder"));
        });
    }

    // Folders section
    if !folder.children.is_empty() {
        // Section header
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(Typography::section_header("FOLDERS"));
        });
        ui.add_space(4.0);

        // Folder list
        for child in &folder.children {
            let is_selected = state.selected_folder == child.path;

            let response = show_list_item(ui, |ui| {
                let icon = if is_selected { Icons::FOLDER_OPEN } else { Icons::FOLDER };
                ui.label(Icons::icon_sized(icon, IconSize::MD));
                ui.add_space(6.0);

                let text = if is_selected {
                    RichText::new(&child.name).strong()
                } else {
                    RichText::new(&child.name)
                };
                ui.selectable_label(is_selected, text)
            });

            // Highlight folder when external files hover over it
            if files_hovering && response.hovered() {
                ui.painter().rect_stroke(
                    response.rect,
                    Radius::SMALL,
                    egui::Stroke::new(2.0, Colors::ACCENT),
                );
                state.drop_target = Some(child.path.clone());
            }

            if response.double_clicked() {
                state.selected_folder = child.path.clone();
                state.expanded_folders.insert(child.path.clone());
            }

            response.context_menu(|ui| {
                if let Some(ctx_action) = show_folder_context_menu(ui, &child.path) {
                    action = Some(ProjectPanelAction::Context(ctx_action));
                }
            });
        }

        ui.add_space(8.0);
    }

    // Files section
    if !folder.files.is_empty() {
        // Section header
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(Typography::section_header("FILES"));
        });
        ui.add_space(4.0);

        // File list
        for file in &folder.files {
            let is_selected = state.selected_file.as_ref() == Some(&file.path);

            let icon_char = file.file_type.icon_char();
            let [r, g, b] = file.file_type.icon_color();
            let icon_color = egui::Color32::from_rgb(r, g, b);

            let element_id = format!("asset_file_{}", file.name);
            ui_state.register_clickable(&element_id, &file.name, "file");
            let pending_action = ui_state.has_pending_trigger_for(&element_id).cloned();

            let mut response = show_list_item(ui, |ui| {
                ui.label(RichText::new(icon_char).size(IconSize::MD).color(icon_color));
                ui.add_space(6.0);

                let text = if is_selected {
                    RichText::new(&file.name).strong()
                } else {
                    RichText::new(&file.name)
                };

                let label_response = if file.file_type == FileType::Image {
                    ui.dnd_drag_source(
                        egui::Id::new(format!("drag_source_{}", file.path.display())),
                        file.path.clone(),
                        |ui| ui.selectable_label(is_selected, text),
                    ).response
                } else {
                    ui.selectable_label(is_selected, text)
                };

                // File size on the right
                if let Some(_size) = file.size {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        ui.label(Typography::muted(file.format_size()));
                    });
                }

                label_response
            });

            // Tooltip
            if let Some(_size) = file.size {
                let ext = file.extension.as_ref().map(|e| e.as_str()).unwrap_or("unknown");
                response = response.on_hover_text(format!(
                    "{}\nSize: {}\nType: {}",
                    file.name, file.format_size(), ext
                ));
            }

            // Click handling
            let is_clicked = response.clicked() || pending_action == Some(TriggerAction::Click);
            if is_clicked {
                let is_manual_double_click = state.check_double_click(&file.path);

                if is_manual_double_click {
                    log::info!("=== DOUBLE-CLICK DETECTED === File: {:?}", file.path);
                    let new_action = if file.file_type == FileType::Scene {
                        ProjectPanelAction::OpenScene(file.path.clone())
                    } else if file.file_type.is_text_editable() {
                        ProjectPanelAction::OpenScript(file.path.clone())
                    } else if file.file_type == FileType::Image {
                        ProjectPanelAction::OpenImage(file.path.clone())
                    } else {
                        ProjectPanelAction::OpenExternal(file.path.clone())
                    };
                    action = Some(new_action);
                } else {
                    state.selected_file = Some(file.path.clone());
                }
            }

            // Remote double-click
            if pending_action == Some(TriggerAction::DoubleClick) {
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

            // Remote right-click
            if pending_action == Some(TriggerAction::RightClick) && file.file_type.is_text_editable() {
                action = Some(ProjectPanelAction::OpenScript(file.path.clone()));
            }

            // Context menu
            response.context_menu(|ui| {
                if let Some(ctx_action) = show_create_submenu(ui, &folder.path) {
                    action = Some(ProjectPanelAction::Context(ctx_action));
                }
                ui.separator();
                if file.file_type.is_text_editable() {
                    if ui.button("Open in Editor").clicked() {
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

            if pending_action.is_some() {
                ui_state.take_pending_trigger_action();
            }
        }
    }

    // Empty space - drop zone and context menu
    let remaining = ui.available_size();
    if remaining.y > 0.0 {
        let (rect, response) = ui.allocate_exact_size(remaining, egui::Sense::click());
        if rect.height() > 0.0 {
            // Show drop zone indicator when files are hovering
            if files_hovering && response.hovered() {
                ui.painter().rect_filled(
                    rect,
                    Radius::SMALL,
                    Colors::ACCENT.gamma_multiply(0.15),
                );
                ui.painter().rect_stroke(
                    rect,
                    Radius::SMALL,
                    egui::Stroke::new(2.0, Colors::ACCENT),
                );
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Drop files here to import",
                    egui::FontId::default(),
                    Colors::ACCENT,
                );
                // Use current folder as drop target
                state.drop_target = Some(folder.path.clone());
            }

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

/// Renders a consistent list item with proper padding and height
fn show_list_item<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> egui::Response {
    let response = ui.horizontal(|ui| {
        ui.set_min_height(LIST_ITEM_MIN_HEIGHT);
        ui.add_space(8.0);
        add_contents(ui)
    });
    response.response
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
