use std::path::Path;
use egui::Ui;
use crate::ContextAction;

/// Actions that can be triggered from the scene tree context menu
#[derive(Debug, Clone, PartialEq)]
pub enum SceneTreeAction {
    /// Create a new entity (as child of selected, or root if nothing selected)
    CreateEntity(Option<hecs::Entity>),
    /// Delete the specified entity
    DeleteEntity(hecs::Entity),
    /// Start rename mode for the specified entity
    RenameEntity(hecs::Entity),
    /// Duplicate the specified entity
    DuplicateEntity(hecs::Entity),
    /// Entity was renamed (for marking scene dirty)
    EntityRenamed(hecs::Entity),
}

/// Renders a "Create" submenu for creating scenes, scripts, and folders.
///
/// This is a reusable utility for showing the Create menu in various panels
/// (project panel grid view, tree view, scene hierarchy, etc.)
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `target_folder` - The folder where new files/folders will be created
///
/// # Returns
/// * `Option<ContextAction>` - The action to perform if user clicked an option
pub fn show_create_submenu(ui: &mut Ui, target_folder: &Path) -> Option<ContextAction> {
    let mut action = None;

    ui.menu_button("Create", |ui| {
        if ui.button("Scene").clicked() {
            action = Some(ContextAction::CreateScene(target_folder.to_path_buf()));
            ui.close_menu();
        }
        if ui.button("Script").clicked() {
            action = Some(ContextAction::CreateScript(target_folder.to_path_buf()));
            ui.close_menu();
        }
        if ui.button("Folder").clicked() {
            action = Some(ContextAction::CreateFolder(target_folder.to_path_buf()));
            ui.close_menu();
        }
    });

    action
}

/// Renders an "Import Asset" button for importing assets into a folder.
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `target_folder` - The folder where assets will be imported
///
/// # Returns
/// * `Option<ContextAction>` - The ImportAsset action if clicked
pub fn show_import_asset_button(ui: &mut Ui, target_folder: &Path) -> Option<ContextAction> {
    if ui.button("Import Asset...").clicked() {
        ui.close_menu();
        return Some(ContextAction::ImportAsset(target_folder.to_path_buf()));
    }
    None
}

/// Renders the standard context menu for folders (Create submenu + Import Asset + Rename/Delete).
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `target_folder` - The folder to operate on
///
/// # Returns
/// * `Option<ContextAction>` - The action to perform if user clicked an option
pub fn show_folder_context_menu(ui: &mut Ui, target_folder: &Path) -> Option<ContextAction> {
    let mut action = show_create_submenu(ui, target_folder);

    ui.separator();

    if action.is_none() {
        action = show_import_asset_button(ui, target_folder);
    }

    ui.separator();

    if action.is_none() {
        if ui.button("Rename").clicked() {
            action = Some(ContextAction::Rename(target_folder.to_path_buf()));
            ui.close_menu();
        }
    }

    if action.is_none() {
        if ui.button("Delete").clicked() {
            action = Some(ContextAction::Delete(target_folder.to_path_buf()));
            ui.close_menu();
        }
    }

    action
}

/// Renders a "Create" submenu for the scene tree (entity hierarchy).
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `parent` - Optional parent entity for the new entity
///
/// # Returns
/// * `Option<SceneTreeAction>` - The action to perform if user clicked an option
pub fn show_scene_tree_create_menu(ui: &mut Ui, parent: Option<hecs::Entity>) -> Option<SceneTreeAction> {
    let mut action = None;

    ui.menu_button("Create", |ui| {
        if ui.button("Entity").clicked() {
            action = Some(SceneTreeAction::CreateEntity(parent));
            ui.close_menu();
        }
    });

    action
}

/// Renders the full context menu for an entity in the scene tree.
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `entity` - The entity to show the context menu for
///
/// # Returns
/// * `Option<SceneTreeAction>` - The action to perform if user clicked an option
pub fn show_entity_context_menu(ui: &mut Ui, entity: hecs::Entity) -> Option<SceneTreeAction> {
    let mut action = show_scene_tree_create_menu(ui, Some(entity));

    ui.separator();

    if action.is_none() {
        if ui.button("Duplicate").clicked() {
            action = Some(SceneTreeAction::DuplicateEntity(entity));
            ui.close_menu();
        }
    }

    if action.is_none() {
        if ui.button("Rename").clicked() {
            action = Some(SceneTreeAction::RenameEntity(entity));
            ui.close_menu();
        }
    }

    ui.separator();

    if action.is_none() {
        if ui.button("Delete").clicked() {
            action = Some(SceneTreeAction::DeleteEntity(entity));
            ui.close_menu();
        }
    }

    action
}

/// Renders the context menu for empty space in the scene tree (no entity selected).
///
/// # Arguments
/// * `ui` - The egui UI context
///
/// # Returns
/// * `Option<SceneTreeAction>` - The action to perform if user clicked an option
pub fn show_scene_tree_empty_context_menu(ui: &mut Ui) -> Option<SceneTreeAction> {
    show_scene_tree_create_menu(ui, None)
}
