// Project panel - displays project assets and file browser

use crate::drag_drop::{handle_drag_source, handle_drop_target, DragDropState, DragItem};
use crate::folder_manager::{FolderManager, FolderOperationError};
use crate::keyboard_shortcuts::{ShortcutAction, ShortcutManager};
use crate::multi_selection::MultiSelection;
use crate::search::SearchFilter;
use crate::undo_redo::{FolderOperation, UndoRedoStack};
use eframe::egui;
use engine_editor_assets::ProjectAsset;
use std::path::{Path, PathBuf};

pub struct ProjectPanel {
    folder_manager: Option<FolderManager>,
    selected_path: Option<PathBuf>,
    show_new_folder_dialog: bool,
    new_folder_name: String,
    parent_path_for_new_folder: PathBuf,
    show_rename_dialog: bool,
    rename_path: PathBuf,
    rename_new_name: String,
    error_message: Option<String>,
    confirm_delete_path: Option<PathBuf>,
    drag_drop_state: DragDropState,
    multi_selection: MultiSelection,
    all_visible_paths: Vec<PathBuf>,
    undo_redo_stack: UndoRedoStack,
    shortcut_manager: ShortcutManager,
    search_filter: Option<SearchFilter>,
    search_query: String,
    show_search: bool,
    cached_assets: Option<Vec<ProjectAsset>>,
    needs_refresh: bool,
}

impl ProjectPanel {
    pub fn new() -> Self {
        Self {
            folder_manager: None,
            selected_path: None,
            show_new_folder_dialog: false,
            new_folder_name: String::new(),
            parent_path_for_new_folder: PathBuf::new(),
            show_rename_dialog: false,
            rename_path: PathBuf::new(),
            rename_new_name: String::new(),
            error_message: None,
            confirm_delete_path: None,
            drag_drop_state: DragDropState::default(),
            multi_selection: MultiSelection::new(),
            all_visible_paths: Vec::new(),
            undo_redo_stack: UndoRedoStack::new(),
            shortcut_manager: ShortcutManager::new(),
            search_filter: None,
            search_query: String::new(),
            show_search: false,
            cached_assets: None,
            needs_refresh: true,
        }
    }

    pub fn set_project_root(&mut self, project_root: impl Into<PathBuf>) {
        self.folder_manager = Some(FolderManager::new(project_root));
        self.needs_refresh = true;
    }

    pub fn show(&mut self, ui: &mut egui::Ui, project_assets: &[ProjectAsset]) {
        // Handle keyboard shortcuts
        let shortcuts = self.shortcut_manager.check_shortcuts(ui.ctx());
        for action in shortcuts {
            self.handle_shortcut_action(action);
        }

        self.show_internal(ui, project_assets);
    }

    fn show_internal(&mut self, ui: &mut egui::Ui, fallback_assets: &[ProjectAsset]) {
        // Load real project structure only when needed
        if self.needs_refresh {
            if let Some(folder_manager) = &self.folder_manager {
                match folder_manager.load_project_structure(&PathBuf::new()) {
                    Ok(assets) => {
                        self.cached_assets = Some(assets);
                        self.error_message = None;
                    }
                    Err(err) => {
                        self.error_message = Some(format!("Failed to load project: {}", err));
                        self.cached_assets = None;
                    }
                }
            }
            self.needs_refresh = false;
        }

        // Use cached assets or fallback (clone to avoid borrow issues)
        let project_assets = if let Some(cached) = &self.cached_assets {
            cached.clone()
        } else {
            fallback_assets.to_vec()
        };

        // Search bar
        if self.show_search {
            ui.horizontal(|ui| {
                ui.label("🔍");
                let response = ui.text_edit_singleline(&mut self.search_query);
                if response.changed() {
                    if self.search_query.is_empty() {
                        self.search_filter = None;
                    } else {
                        self.search_filter = Some(SearchFilter::new(self.search_query.clone()));
                    }
                }
                if ui.button("✕").clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.show_search = false;
                    self.search_filter = None;
                    self.search_query.clear();
                }
            });
            ui.separator();
        }

        ui.horizontal(|ui| {
            ui.label("Asset Browser");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button("🔄")
                    .on_hover_text(format!(
                        "Refresh assets ({})",
                        self.shortcut_manager
                            .format_shortcut(ShortcutAction::Refresh)
                    ))
                    .clicked()
                {
                    // Clear any error messages when refreshing
                    self.error_message = None;
                    self.needs_refresh = true;
                }

                if ui
                    .button("🔍")
                    .on_hover_text(format!(
                        "Search ({})",
                        self.shortcut_manager
                            .format_shortcut(ShortcutAction::Search)
                    ))
                    .clicked()
                {
                    self.show_search = !self.show_search;
                    if !self.show_search {
                        self.search_filter = None;
                        self.search_query.clear();
                    }
                }
                if ui
                    .button("📁")
                    .on_hover_text(format!(
                        "Create new folder ({})",
                        self.shortcut_manager
                            .format_shortcut(ShortcutAction::NewFolder)
                    ))
                    .clicked()
                {
                    self.show_new_folder_dialog = true;
                    self.parent_path_for_new_folder = PathBuf::new();
                    self.new_folder_name.clear();
                    self.error_message = None;
                }
                if ui.button("➕").on_hover_text("Create new asset").clicked() {
                    // Create asset menu
                }

                // Undo/Redo buttons
                ui.separator();

                if self.undo_redo_stack.can_undo() {
                    if ui
                        .button("↶")
                        .on_hover_text(format!(
                            "Undo ({})",
                            self.shortcut_manager.format_shortcut(ShortcutAction::Undo)
                        ))
                        .clicked()
                    {
                        self.perform_undo();
                    }
                } else {
                    ui.add_enabled(false, egui::Button::new("↶"));
                }

                if self.undo_redo_stack.can_redo() {
                    if ui
                        .button("↷")
                        .on_hover_text(format!(
                            "Redo ({})",
                            self.shortcut_manager.format_shortcut(ShortcutAction::Redo)
                        ))
                        .clicked()
                    {
                        self.perform_redo();
                    }
                } else {
                    ui.add_enabled(false, egui::Button::new("↷"));
                }
            });
        });

        ui.separator();

        // Show error message if any
        if let Some(error) = &self.error_message {
            ui.colored_label(egui::Color32::RED, error);
            ui.separator();
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Collect all visible paths for multi-selection
            self.all_visible_paths.clear();
            self.collect_visible_paths(&project_assets, &mut PathBuf::new());

            let mut path = PathBuf::new();

            // Apply search filter if active
            if let Some(filter) = &self.search_filter {
                let filtered_paths = filter.filter_assets(&project_assets, &PathBuf::new());
                for asset in &project_assets {
                    self.show_filtered_asset(ui, asset, &mut path, &filtered_paths);
                }
            } else {
                for asset in &project_assets {
                    self.show_project_asset_with_path(ui, asset, &mut path);
                }
            }
        });

        // Show dialogs
        self.show_new_folder_dialog(ui.ctx());
        self.show_rename_dialog(ui.ctx());
        self.show_delete_confirmation(ui.ctx());

        // Render drag preview
        self.drag_drop_state.render_drag_preview(ui);
    }

    fn show_project_asset(&mut self, ui: &mut egui::Ui, asset: &ProjectAsset) {
        let mut path = PathBuf::new();
        self.show_project_asset_with_path(ui, asset, &mut path);
    }

    fn show_project_asset_with_path(
        &mut self,
        ui: &mut egui::Ui,
        asset: &ProjectAsset,
        current_path: &mut PathBuf,
    ) {
        let asset_path = current_path.join(&asset.name);

        match &asset.children {
            Some(children) => {
                // Folder with children
                let is_selected = self.multi_selection.is_selected(&asset_path);
                let id = ui.make_persistent_id(&asset_path);
                let response = egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id,
                    false,
                )
                .show_header(ui, |ui| {
                    let response = ui.selectable_label(is_selected, &asset.name);

                    // Handle selection for folders
                    if response.clicked() {
                        let modifiers = ui.input(|i| i.modifiers);

                        if modifiers.ctrl && modifiers.shift {
                            if let Some(anchor) = self.multi_selection.anchor().cloned() {
                                self.multi_selection.add_range(
                                    &anchor,
                                    &asset_path,
                                    &self.all_visible_paths,
                                );
                            }
                        } else if modifiers.shift {
                            if let Some(last) = self.multi_selection.last_selected().cloned() {
                                self.multi_selection.select_range(
                                    &last,
                                    &asset_path,
                                    &self.all_visible_paths,
                                );
                            } else {
                                self.multi_selection.select_single(asset_path.clone());
                            }
                        } else if modifiers.ctrl || modifiers.command {
                            self.multi_selection.toggle_selection(asset_path.clone());
                        } else {
                            self.multi_selection.select_single(asset_path.clone());
                        }

                        self.selected_path = Some(asset_path.clone());
                    }

                    response
                })
                .body(|ui| {
                    current_path.push(&asset.name);
                    for child in children {
                        self.show_project_asset_with_path(ui, child, current_path);
                    }
                    current_path.pop();
                });

                // Handle drag source
                let drag_item = DragItem::Folder(asset_path.clone());
                handle_drag_source(ui, &mut self.drag_drop_state, drag_item, &response.0);

                // Handle drop target
                if let Some((dropped_item, _)) = handle_drop_target(
                    ui,
                    &mut self.drag_drop_state,
                    asset_path.clone(),
                    &response.0,
                ) {
                    self.handle_drop(dropped_item, asset_path.clone());
                }

                // Context menu for folders
                response.0.context_menu(|ui| {
                    if ui.button("New Folder").clicked() {
                        self.show_new_folder_dialog = true;
                        self.parent_path_for_new_folder = asset_path.clone();
                        self.new_folder_name.clear();
                        self.error_message = None;
                        ui.close_menu();
                    }

                    if ui.button("Rename").clicked() {
                        self.show_rename_dialog = true;
                        self.rename_path = asset_path.clone();
                        self.rename_new_name = asset.name.clone();
                        self.error_message = None;
                        ui.close_menu();
                    }

                    if ui.button("Delete").clicked() {
                        self.confirm_delete_path = Some(asset_path.clone());
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.button("Open in File Explorer").clicked() {
                        self.open_in_file_explorer(&asset_path);
                        ui.close_menu();
                    }
                });
            }
            None => {
                // File asset
                let is_selected = self.multi_selection.is_selected(&asset_path);
                let response = ui.selectable_label(is_selected, &asset.name);

                // Handle selection based on modifiers
                if response.clicked() {
                    let modifiers = ui.input(|i| i.modifiers);

                    if modifiers.ctrl && modifiers.shift {
                        // Ctrl+Shift+Click: Add range to selection
                        if let Some(anchor) = self.multi_selection.anchor().cloned() {
                            self.multi_selection.add_range(
                                &anchor,
                                &asset_path,
                                &self.all_visible_paths,
                            );
                        }
                    } else if modifiers.shift {
                        // Shift+Click: Select range
                        if let Some(last) = self.multi_selection.last_selected().cloned() {
                            self.multi_selection.select_range(
                                &last,
                                &asset_path,
                                &self.all_visible_paths,
                            );
                        } else {
                            self.multi_selection.select_single(asset_path.clone());
                        }
                    } else if modifiers.ctrl || modifiers.command {
                        // Ctrl/Cmd+Click: Toggle selection
                        self.multi_selection.toggle_selection(asset_path.clone());
                    } else {
                        // Normal click: Select single
                        self.multi_selection.select_single(asset_path.clone());
                    }

                    self.selected_path = Some(asset_path.clone());
                }

                // Handle drag source for files
                let drag_item = DragItem::File(asset_path.clone());
                handle_drag_source(ui, &mut self.drag_drop_state, drag_item, &response);

                // Context menu for files
                response.context_menu(|ui| {
                    if ui.button("Delete").clicked() {
                        // File deletion not implemented yet
                        ui.close_menu();
                    }
                });
            }
        }
    }

    fn show_new_folder_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_new_folder_dialog {
            return;
        }

        egui::Window::new("New Folder")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Folder name:");
                    ui.text_edit_singleline(&mut self.new_folder_name);
                });

                if let Some(error) = &self.error_message {
                    ui.colored_label(egui::Color32::RED, error);
                }

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() && !self.new_folder_name.is_empty() {
                        if let Some(folder_manager) = &self.folder_manager {
                            match folder_manager.create_folder(
                                &self.parent_path_for_new_folder,
                                &self.new_folder_name,
                            ) {
                                Ok(_) => {
                                    // Record operation for undo
                                    self.undo_redo_stack.push(FolderOperation::Create {
                                        parent: self.parent_path_for_new_folder.clone(),
                                        name: self.new_folder_name.clone(),
                                    });

                                    self.show_new_folder_dialog = false;
                                    self.error_message = None;
                                    self.needs_refresh = true;
                                }
                                Err(err) => {
                                    self.error_message = Some(err.to_string());
                                }
                            }
                        } else {
                            self.error_message = Some("Folder manager not initialized".to_string());
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        self.show_new_folder_dialog = false;
                        self.error_message = None;
                    }
                });
            });
    }

    fn show_rename_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_rename_dialog {
            return;
        }

        egui::Window::new("Rename Folder")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("New name:");
                    ui.text_edit_singleline(&mut self.rename_new_name);
                });

                if let Some(error) = &self.error_message {
                    ui.colored_label(egui::Color32::RED, error);
                }

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Rename").clicked() && !self.rename_new_name.is_empty() {
                        if let Some(folder_manager) = &self.folder_manager {
                            match folder_manager
                                .rename_folder(&self.rename_path, &self.rename_new_name)
                            {
                                Ok(_) => {
                                    // Record operation for undo
                                    self.undo_redo_stack.push(FolderOperation::Rename {
                                        old_path: self.rename_path.clone(),
                                        new_name: self.rename_new_name.clone(),
                                    });

                                    self.show_rename_dialog = false;
                                    self.error_message = None;
                                    self.needs_refresh = true;
                                }
                                Err(err) => {
                                    self.error_message = Some(err.to_string());
                                }
                            }
                        } else {
                            self.error_message = Some("Folder manager not initialized".to_string());
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        self.show_rename_dialog = false;
                        self.error_message = None;
                    }
                });
            });
    }

    fn show_delete_confirmation(&mut self, ctx: &egui::Context) {
        if let Some(path) = self.confirm_delete_path.clone() {
            let mut delete_clicked = false;
            let mut cancel_clicked = false;
            let mut delete_result = Ok(());
            let mut has_folder_manager = self.folder_manager.is_some();

            egui::Window::new("Confirm Delete")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "Are you sure you want to delete '{}'?",
                        path.display()
                    ));
                    ui.label("This action cannot be undone.");

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            delete_clicked = true;
                        }

                        if ui.button("Cancel").clicked() {
                            cancel_clicked = true;
                        }
                    });
                });

            if delete_clicked {
                if let Some(folder_manager) = &self.folder_manager {
                    // Read folder contents before deletion for undo
                    let contents =
                        if let Ok(data) = std::fs::read(&folder_manager.project_root.join(&path)) {
                            data
                        } else {
                            Vec::new() // For directories, we'll store empty data
                        };

                    delete_result = folder_manager.delete_folder(&path);

                    if delete_result.is_ok() {
                        // Record operation for undo
                        self.undo_redo_stack.push(FolderOperation::Delete {
                            path: path.clone(),
                            contents,
                        });
                    }
                } else {
                    has_folder_manager = false;
                }

                match delete_result {
                    Ok(_) => {
                        self.confirm_delete_path = None;
                        self.error_message = None;
                        self.needs_refresh = true;
                    }
                    Err(err) => {
                        self.error_message = Some(err.to_string());
                        self.confirm_delete_path = None;
                    }
                }

                if !has_folder_manager {
                    self.error_message = Some("Folder manager not initialized".to_string());
                    self.confirm_delete_path = None;
                }
            }

            if cancel_clicked {
                self.confirm_delete_path = None;
            }
        }
    }

    fn open_in_file_explorer(&self, path: &Path) {
        if let Some(folder_manager) = &self.folder_manager {
            let full_path = folder_manager.project_root.join(path);

            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("explorer")
                    .arg(&full_path)
                    .spawn();
            }

            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open").arg(&full_path).spawn();
            }

            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("xdg-open")
                    .arg(&full_path)
                    .spawn();
            }
        }
    }

    fn handle_drop(&mut self, dropped_item: DragItem, target_path: PathBuf) {
        if let Some(folder_manager) = &self.folder_manager {
            match dropped_item {
                DragItem::Folder(source_path) => {
                    match folder_manager.move_folder(&source_path, &target_path) {
                        Ok(_) => {
                            // Record operation for undo
                            self.undo_redo_stack.push(FolderOperation::Move {
                                source: source_path,
                                target_parent: target_path,
                            });

                            self.error_message = None;
                            self.needs_refresh = true;
                        }
                        Err(err) => {
                            self.error_message = Some(format!("Failed to move folder: {}", err));
                        }
                    }
                }
                DragItem::File(source_path) => {
                    match folder_manager.move_file(&source_path, &target_path) {
                        Ok(_) => {
                            // Record operation for undo (using Move for files too)
                            self.undo_redo_stack.push(FolderOperation::Move {
                                source: source_path,
                                target_parent: target_path,
                            });

                            self.error_message = None;
                            self.needs_refresh = true;
                        }
                        Err(err) => {
                            self.error_message = Some(format!("Failed to move file: {}", err));
                        }
                    }
                }
            }
        }
    }

    fn collect_visible_paths(&mut self, assets: &[ProjectAsset], current_path: &mut PathBuf) {
        for asset in assets {
            let asset_path = current_path.join(&asset.name);
            self.all_visible_paths.push(asset_path.clone());

            if let Some(children) = &asset.children {
                current_path.push(&asset.name);
                self.collect_visible_paths(children, current_path);
                current_path.pop();
            }
        }
    }

    fn perform_undo(&mut self) {
        if let Some(operation) = self.undo_redo_stack.undo() {
            if let Some(folder_manager) = &self.folder_manager {
                match operation {
                    FolderOperation::Create { parent, name } => {
                        // Undo create by deleting the folder
                        let path = parent.join(&name);
                        if let Err(err) = folder_manager.delete_folder(&path) {
                            self.error_message = Some(format!("Failed to undo create: {}", err));
                        }
                    }
                    FolderOperation::Delete { path, .. } => {
                        // Undo delete by recreating the folder
                        // Note: This only recreates the folder, not its contents
                        if let Some(parent) = path.parent() {
                            if let Some(folder_name) = path.file_name() {
                                let parent_path = parent
                                    .strip_prefix(&folder_manager.project_root)
                                    .unwrap_or(parent);
                                if let Err(err) = folder_manager
                                    .create_folder(parent_path, &folder_name.to_string_lossy())
                                {
                                    self.error_message =
                                        Some(format!("Failed to undo delete: {}", err));
                                }
                            }
                        }
                    }
                    FolderOperation::Rename { old_path, new_name } => {
                        // Undo rename by renaming back
                        if let Some(parent) = old_path.parent() {
                            if let Some(old_name) = old_path.file_name() {
                                let current_path = parent.join(&new_name);
                                if let Err(err) = folder_manager
                                    .rename_folder(&current_path, &old_name.to_string_lossy())
                                {
                                    self.error_message =
                                        Some(format!("Failed to undo rename: {}", err));
                                }
                            }
                        }
                    }
                    FolderOperation::Move {
                        source,
                        target_parent,
                    } => {
                        // Undo move by moving back
                        if let Some(source_parent) = source.parent() {
                            if let Some(item_name) = source.file_name() {
                                let current_path = target_parent.join(item_name);
                                if let Err(err) =
                                    folder_manager.move_folder(&current_path, source_parent)
                                {
                                    // Try as file if folder move fails
                                    if let Err(err) =
                                        folder_manager.move_file(&current_path, source_parent)
                                    {
                                        self.error_message =
                                            Some(format!("Failed to undo move: {}", err));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            self.needs_refresh = true;
        }
    }

    fn handle_shortcut_action(&mut self, action: ShortcutAction) {
        match action {
            ShortcutAction::Undo => self.perform_undo(),
            ShortcutAction::Redo => self.perform_redo(),
            ShortcutAction::Delete => {
                if let Some(selected) = self.selected_path.clone() {
                    self.confirm_delete_path = Some(selected);
                }
            }
            ShortcutAction::Rename => {
                if let Some(selected) = self.selected_path.clone() {
                    self.show_rename_dialog = true;
                    self.rename_path = selected.clone();
                    if let Some(name) = selected.file_name() {
                        self.rename_new_name = name.to_string_lossy().to_string();
                    }
                    self.error_message = None;
                }
            }
            ShortcutAction::Search => {
                self.show_search = !self.show_search;
                if !self.show_search {
                    self.search_filter = None;
                    self.search_query.clear();
                }
            }
            ShortcutAction::NewFolder => {
                self.show_new_folder_dialog = true;
                self.parent_path_for_new_folder = self.selected_path.clone().unwrap_or_default();
                self.new_folder_name.clear();
                self.error_message = None;
            }
            ShortcutAction::Refresh => {
                self.error_message = None;
                self.needs_refresh = true;
            }
            ShortcutAction::SelectAll => {
                // Select all visible items
                for path in self.all_visible_paths.clone() {
                    self.multi_selection.toggle_selection(path);
                }
            }
            _ => {} // Copy, Paste, Cut not implemented yet
        }
    }

    fn show_filtered_asset(
        &mut self,
        ui: &mut egui::Ui,
        asset: &ProjectAsset,
        current_path: &mut PathBuf,
        filtered_paths: &[PathBuf],
    ) {
        let asset_path = current_path.join(&asset.name);
        let is_match = filtered_paths.contains(&asset_path);

        match &asset.children {
            Some(children) => {
                // Check if any children match
                let has_matching_children = children.iter().any(|child| {
                    let child_path = asset_path.join(&child.name);
                    filtered_paths
                        .iter()
                        .any(|p| p.starts_with(&child_path) || p == &child_path)
                });

                if is_match || has_matching_children {
                    // Show folder if it matches or contains matches
                    let is_selected = self.multi_selection.is_selected(&asset_path);
                    let id = ui.make_persistent_id(&asset_path);
                    let response =
                        egui::collapsing_header::CollapsingState::load_with_default_open(
                            ui.ctx(),
                            id,
                            true,
                        )
                        .show_header(ui, |ui| {
                            let response = ui.selectable_label(is_selected, &asset.name);
                            self.handle_asset_selection(&response, &asset_path, ui);
                            response
                        })
                        .body(|ui| {
                            current_path.push(&asset.name);
                            for child in children {
                                self.show_filtered_asset(ui, child, current_path, filtered_paths);
                            }
                            current_path.pop();
                        });

                    self.handle_folder_interactions(ui, &response.0, &asset_path, &asset.name);
                }
            }
            None => {
                // Show file if it matches
                if is_match {
                    let is_selected = self.multi_selection.is_selected(&asset_path);
                    let response = ui.selectable_label(is_selected, &asset.name);
                    self.handle_asset_selection(&response, &asset_path, ui);
                    self.handle_file_interactions(ui, &response, &asset_path);
                }
            }
        }
    }

    fn handle_asset_selection(
        &mut self,
        response: &egui::Response,
        asset_path: &PathBuf,
        ui: &mut egui::Ui,
    ) {
        if response.clicked() {
            let modifiers = ui.input(|i| i.modifiers);

            if modifiers.ctrl && modifiers.shift {
                if let Some(anchor) = self.multi_selection.anchor().cloned() {
                    self.multi_selection
                        .add_range(&anchor, asset_path, &self.all_visible_paths);
                }
            } else if modifiers.shift {
                if let Some(last) = self.multi_selection.last_selected().cloned() {
                    self.multi_selection
                        .select_range(&last, asset_path, &self.all_visible_paths);
                } else {
                    self.multi_selection.select_single(asset_path.clone());
                }
            } else if modifiers.ctrl || modifiers.command {
                self.multi_selection.toggle_selection(asset_path.clone());
            } else {
                self.multi_selection.select_single(asset_path.clone());
            }

            self.selected_path = Some(asset_path.clone());
        }
    }

    fn handle_folder_interactions(
        &mut self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        asset_path: &PathBuf,
        asset_name: &str,
    ) {
        // Handle drag source
        let drag_item = DragItem::Folder(asset_path.clone());
        handle_drag_source(ui, &mut self.drag_drop_state, drag_item, response);

        // Handle drop target
        if let Some((dropped_item, _)) =
            handle_drop_target(ui, &mut self.drag_drop_state, asset_path.clone(), response)
        {
            self.handle_drop(dropped_item, asset_path.clone());
        }

        // Context menu for folders
        response.context_menu(|ui| {
            if ui
                .button(format!(
                    "New Folder ({})",
                    self.shortcut_manager
                        .format_shortcut(ShortcutAction::NewFolder)
                ))
                .clicked()
            {
                self.show_new_folder_dialog = true;
                self.parent_path_for_new_folder = asset_path.clone();
                self.new_folder_name.clear();
                self.error_message = None;
                ui.close_menu();
            }

            if ui
                .button(format!(
                    "Rename ({})",
                    self.shortcut_manager
                        .format_shortcut(ShortcutAction::Rename)
                ))
                .clicked()
            {
                self.show_rename_dialog = true;
                self.rename_path = asset_path.clone();
                self.rename_new_name = asset_name.to_string();
                self.error_message = None;
                ui.close_menu();
            }

            if ui
                .button(format!(
                    "Delete ({})",
                    self.shortcut_manager
                        .format_shortcut(ShortcutAction::Delete)
                ))
                .clicked()
            {
                self.confirm_delete_path = Some(asset_path.clone());
                ui.close_menu();
            }

            ui.separator();

            if ui.button("Open in File Explorer").clicked() {
                self.open_in_file_explorer(asset_path);
                ui.close_menu();
            }
        });
    }

    fn handle_file_interactions(
        &mut self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        asset_path: &PathBuf,
    ) {
        // Handle drag source for files
        let drag_item = DragItem::File(asset_path.clone());
        handle_drag_source(ui, &mut self.drag_drop_state, drag_item, response);

        // Context menu for files
        response.context_menu(|ui| {
            if ui
                .button(format!(
                    "Delete ({})",
                    self.shortcut_manager
                        .format_shortcut(ShortcutAction::Delete)
                ))
                .clicked()
            {
                // File deletion not implemented yet
                ui.close_menu();
            }
        });
    }

    fn perform_redo(&mut self) {
        if let Some(operation) = self.undo_redo_stack.redo() {
            if let Some(folder_manager) = &self.folder_manager {
                match operation {
                    FolderOperation::Create { parent, name } => {
                        // Redo create
                        if let Err(err) = folder_manager.create_folder(&parent, &name) {
                            self.error_message = Some(format!("Failed to redo create: {}", err));
                        }
                    }
                    FolderOperation::Delete { path, .. } => {
                        // Redo delete
                        if let Err(err) = folder_manager.delete_folder(&path) {
                            self.error_message = Some(format!("Failed to redo delete: {}", err));
                        }
                    }
                    FolderOperation::Rename { old_path, new_name } => {
                        // Redo rename
                        if let Err(err) = folder_manager.rename_folder(&old_path, &new_name) {
                            self.error_message = Some(format!("Failed to redo rename: {}", err));
                        }
                    }
                    FolderOperation::Move {
                        source,
                        target_parent,
                    } => {
                        // Redo move
                        if let Err(err) = folder_manager.move_folder(&source, &target_parent) {
                            // Try as file if folder move fails
                            if let Err(err) = folder_manager.move_file(&source, &target_parent) {
                                self.error_message = Some(format!("Failed to redo move: {}", err));
                            }
                        }
                    }
                }
            }
            self.needs_refresh = true;
        }
    }
}
