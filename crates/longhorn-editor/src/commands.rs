//! Remote command processing for the editor.
//!
//! This module handles all remote commands sent to the editor via the remote control interface.
//! Commands are processed synchronously and return a response.

use longhorn_core::{EntityHandle, EntityId, Name, Transform, World};
use longhorn_engine::Engine;

use crate::remote::{
    AssetBrowserData, AssetFileInfo, ClickableInfo, EntityDetails, EntityInfo, PanelInfo,
    RemoteCommand, RemoteResponse, ResponseData, ScriptEditorData, ScriptErrorData, TransformData,
    UiStateData,
};
use crate::ui_state::TriggerAction;
use crate::{DirectoryNode, Editor, ToolbarAction};

/// Process a remote command and return a response.
///
/// This is the main entry point for handling remote commands. It delegates to
/// specific handler functions based on the command type.
pub fn process_remote_command(
    editor: &mut Editor,
    command: RemoteCommand,
    engine: &mut Engine,
) -> RemoteResponse {
    match command {
        // Playback commands
        RemoteCommand::Ping => RemoteResponse::ok(),
        RemoteCommand::Play => {
            editor.handle_toolbar_action(ToolbarAction::Play, engine);
            RemoteResponse::ok()
        }
        RemoteCommand::Pause => {
            editor.handle_toolbar_action(ToolbarAction::Pause, engine);
            RemoteResponse::ok()
        }
        RemoteCommand::Resume => {
            editor.handle_toolbar_action(ToolbarAction::Resume, engine);
            RemoteResponse::ok()
        }
        RemoteCommand::Stop => {
            editor.handle_toolbar_action(ToolbarAction::Stop, engine);
            RemoteResponse::ok()
        }
        RemoteCommand::ToggleConsole => {
            // Console is now always visible in dock, this is a no-op
            RemoteResponse::ok()
        }

        // State queries
        RemoteCommand::GetState => handle_get_state(editor, engine),
        RemoteCommand::GetEntities => handle_get_entities(engine),
        RemoteCommand::GetEntity { id } => handle_get_entity(engine, id),

        // Entity manipulation
        RemoteCommand::SelectEntity { id } => handle_select_entity(editor, engine, id),
        RemoteCommand::CreateEntity { name } => handle_create_entity(engine, &name),
        RemoteCommand::DeleteEntity { id } => handle_delete_entity(editor, engine, id),
        RemoteCommand::SetProperty {
            entity,
            component,
            field,
            value,
        } => set_entity_property(engine.world_mut(), entity, &component, &field, value),

        // Project
        RemoteCommand::LoadProject { path } => handle_load_project(editor, engine, &path),

        // Script Editor
        RemoteCommand::OpenScript { path } => handle_open_script(editor, engine, &path),
        RemoteCommand::SaveScript => handle_save_script(editor),
        RemoteCommand::GetScriptEditorState => handle_get_script_editor_state(editor),

        // UI State
        RemoteCommand::GetUiState => handle_get_ui_state(editor),
        RemoteCommand::ListPanels => handle_list_panels(editor),
        RemoteCommand::GetClickableElements => handle_get_clickable_elements(editor),
        RemoteCommand::FocusPanel { panel } => {
            editor.ui_state_mut().request_focus(panel);
            RemoteResponse::ok()
        }
        RemoteCommand::TriggerElement { id } => {
            editor.ui_state_mut().request_trigger(id);
            RemoteResponse::ok()
        }
        RemoteCommand::ClickElement { id } => {
            editor
                .ui_state_mut()
                .request_trigger_action(id, TriggerAction::Click);
            RemoteResponse::ok()
        }
        RemoteCommand::DoubleClickElement { id } => {
            editor
                .ui_state_mut()
                .request_trigger_action(id, TriggerAction::DoubleClick);
            RemoteResponse::ok()
        }
        RemoteCommand::RightClickElement { id } => {
            editor
                .ui_state_mut()
                .request_trigger_action(id, TriggerAction::RightClick);
            RemoteResponse::ok()
        }

        // Scene Tree
        RemoteCommand::ExpandTreeNode { path } => {
            editor.ui_state_mut().request_tree_expand(path);
            RemoteResponse::ok()
        }
        RemoteCommand::CollapseTreeNode { path } => {
            editor.ui_state_mut().request_tree_collapse(path);
            RemoteResponse::ok()
        }
        RemoteCommand::SelectByPath { path } => handle_select_by_path(editor, engine, &path),

        // Asset Browser
        RemoteCommand::GetAssetBrowserState => handle_get_asset_browser_state(editor),
        RemoteCommand::OpenAssetFile { path } => handle_open_asset_file(editor, engine, &path),
        RemoteCommand::SelectAssetFile { path } => {
            log::info!("Remote: Selecting asset file '{}'", path);
            let file_path = std::path::PathBuf::from(&path);
            editor.project_panel_state_mut().selected_file = Some(file_path);
            RemoteResponse::ok()
        }
        RemoteCommand::DoubleClickAssetFile { path } => {
            handle_double_click_asset_file(editor, engine, &path)
        }
        RemoteCommand::AssetContextOpenInEditor { path } => {
            handle_asset_context_open_in_editor(editor, engine, &path)
        }
    }
}

// --- State Query Handlers ---

fn handle_get_state(editor: &Editor, engine: &Engine) -> RemoteResponse {
    let selected = editor.state().selected_entity.map(|e| e.id() as u64);
    RemoteResponse::with_data(ResponseData::State {
        mode: format!("{:?}", editor.state().mode),
        paused: editor.state().paused,
        entity_count: engine.world().len(),
        selected_entity: selected,
    })
}

fn handle_get_entities(engine: &Engine) -> RemoteResponse {
    let entities: Vec<EntityInfo> = engine
        .world()
        .inner()
        .iter()
        .map(|entity_ref| {
            let entity = entity_ref.entity();
            let handle = EntityHandle::new(entity);
            let name = engine
                .world()
                .get::<Name>(handle)
                .ok()
                .map(|n| n.0.clone())
                .unwrap_or_else(|| format!("Entity {}", entity.id()));
            EntityInfo {
                id: entity.id() as u64,
                name,
            }
        })
        .collect();
    RemoteResponse::with_data(ResponseData::Entities(entities))
}

fn handle_get_entity(engine: &Engine, id: u64) -> RemoteResponse {
    let found = engine
        .world()
        .inner()
        .iter()
        .find(|e| e.entity().id() as u64 == id);

    match found {
        Some(entity_ref) => {
            let entity = entity_ref.entity();
            let handle = EntityHandle::new(entity);

            let name = engine
                .world()
                .get::<Name>(handle)
                .ok()
                .map(|n| n.0.clone())
                .unwrap_or_else(|| format!("Entity {}", id));

            let transform = engine
                .world()
                .get::<Transform>(handle)
                .ok()
                .map(|t| TransformData {
                    position_x: t.position.x,
                    position_y: t.position.y,
                    rotation: t.rotation,
                    scale_x: t.scale.x,
                    scale_y: t.scale.y,
                });

            RemoteResponse::with_data(ResponseData::Entity(EntityDetails {
                id,
                name,
                transform,
            }))
        }
        None => RemoteResponse::error(format!("Entity not found: {}", id)),
    }
}

// --- Entity Manipulation Handlers ---

fn handle_select_entity(editor: &mut Editor, engine: &Engine, id: u64) -> RemoteResponse {
    let found = engine
        .world()
        .inner()
        .iter()
        .find(|e| e.entity().id() as u64 == id)
        .map(|e| e.entity());

    match found {
        Some(entity) => {
            editor.state_mut().select(Some(entity));
            RemoteResponse::ok()
        }
        None => RemoteResponse::error(format!("Entity not found: {}", id)),
    }
}

fn handle_create_entity(engine: &mut Engine, name: &str) -> RemoteResponse {
    let entity = engine
        .world_mut()
        .spawn()
        .with(Name::new(name))
        .with(Transform::default())
        .build();
    let id = entity.id().to_bits().get();
    log::info!("Created entity '{}' with id {}", name, id);
    RemoteResponse::with_data(ResponseData::Created { id })
}

fn handle_delete_entity(editor: &mut Editor, engine: &mut Engine, id: u64) -> RemoteResponse {
    match EntityId::from_bits(id) {
        Some(entity_id) => {
            let handle = EntityHandle::new(entity_id);
            if engine.world_mut().despawn(handle).is_ok() {
                // Deselect if this was selected
                if editor.state().selected_entity.map(|e| e.id() as u64) == Some(id) {
                    editor.state_mut().select(None);
                }
                log::info!("Deleted entity {}", id);
                RemoteResponse::ok()
            } else {
                RemoteResponse::error(format!("Entity not found: {}", id))
            }
        }
        None => RemoteResponse::error(format!("Invalid entity id: {}", id)),
    }
}

fn set_entity_property(
    world: &mut World,
    entity_id: u64,
    component: &str,
    field: &str,
    value: serde_json::Value,
) -> RemoteResponse {
    let entity_id = match EntityId::from_bits(entity_id) {
        Some(id) => id,
        None => return RemoteResponse::error(format!("Invalid entity id: {}", entity_id)),
    };
    let handle = EntityHandle::new(entity_id);

    match component {
        "Transform" => {
            let mut transform = match world.get::<Transform>(handle) {
                Ok(t) => (*t).clone(),
                Err(_) => return RemoteResponse::error("Entity has no Transform"),
            };

            match field {
                "position.x" => {
                    if let Some(v) = value.as_f64() {
                        transform.position.x = v as f32;
                    }
                }
                "position.y" => {
                    if let Some(v) = value.as_f64() {
                        transform.position.y = v as f32;
                    }
                }
                "rotation" => {
                    if let Some(v) = value.as_f64() {
                        transform.rotation = v as f32;
                    }
                }
                "scale.x" => {
                    if let Some(v) = value.as_f64() {
                        transform.scale.x = v as f32;
                    }
                }
                "scale.y" => {
                    if let Some(v) = value.as_f64() {
                        transform.scale.y = v as f32;
                    }
                }
                _ => return RemoteResponse::error(format!("Unknown field: {}", field)),
            }

            if world.set(handle, transform).is_err() {
                return RemoteResponse::error("Failed to set Transform");
            }
            RemoteResponse::ok()
        }
        "Name" => {
            if field == "name" || field == "0" {
                if let Some(s) = value.as_str() {
                    if world.set(handle, Name::new(s)).is_err() {
                        return RemoteResponse::error("Failed to set Name");
                    }
                    return RemoteResponse::ok();
                }
            }
            RemoteResponse::error(format!("Invalid Name field: {}", field))
        }
        _ => RemoteResponse::error(format!("Unknown component: {}", component)),
    }
}

// --- Project Handlers ---

fn handle_load_project(editor: &mut Editor, engine: &mut Engine, path: &str) -> RemoteResponse {
    match engine.load_game(path) {
        Ok(()) => {
            log::info!("Loaded project: {}", path);
            editor.refresh_project_tree(engine);
            editor.setup_event_subscriptions(engine);
            RemoteResponse::ok()
        }
        Err(e) => RemoteResponse::error(format!("Failed to load project: {}", e)),
    }
}

// --- Script Editor Handlers ---

fn handle_open_script(editor: &mut Editor, engine: &Engine, path: &str) -> RemoteResponse {
    log::info!("Remote: Opening script '{}'", path);
    if let Some(project_path) = engine.game_path() {
        let script_path = std::path::PathBuf::from(path);
        match editor.script_editor_state_mut().open(script_path, project_path) {
            Ok(()) => {
                log::info!("Script opened successfully: {}", path);
                editor.recheck_script_errors();
                editor.ensure_script_editor_visible();
                RemoteResponse::ok()
            }
            Err(e) => {
                log::error!("Failed to open script '{}': {}", path, e);
                RemoteResponse::error(format!("Failed to open script: {}", e))
            }
        }
    } else {
        log::error!("Cannot open script: No project loaded");
        RemoteResponse::error("No project loaded")
    }
}

fn handle_save_script(editor: &mut Editor) -> RemoteResponse {
    log::info!("Remote: Saving script");
    if editor.script_editor_state().is_open() {
        match editor.script_editor_state_mut().save() {
            Ok(()) => {
                log::info!("Script saved successfully");
                editor.recheck_script_errors();
                RemoteResponse::ok()
            }
            Err(e) => {
                log::error!("Failed to save script: {}", e);
                RemoteResponse::error(format!("Failed to save script: {}", e))
            }
        }
    } else {
        RemoteResponse::error("No script is open")
    }
}

fn handle_get_script_editor_state(editor: &Editor) -> RemoteResponse {
    let state = editor.script_editor_state();
    let data = ScriptEditorData {
        is_open: state.is_open(),
        file_path: state.open_file.as_ref().map(|p| p.display().to_string()),
        is_dirty: state.is_dirty(),
        error_count: state.errors.len(),
        errors: state
            .errors
            .iter()
            .map(|e| ScriptErrorData {
                line: e.line,
                message: e.message.clone(),
            })
            .collect(),
    };
    RemoteResponse::with_data(ResponseData::ScriptEditor(data))
}

// --- UI State Handlers ---

fn handle_get_ui_state(editor: &Editor) -> RemoteResponse {
    let snapshot = editor.ui_state().snapshot();
    let data = UiStateData {
        focused_panel: snapshot.focused_panel,
        panels: snapshot
            .panels
            .into_iter()
            .map(|p| PanelInfo {
                id: p.id,
                title: p.title,
                is_focused: p.is_focused,
            })
            .collect(),
        clickable_elements: snapshot
            .clickable_elements
            .into_iter()
            .map(|c| ClickableInfo {
                id: c.id,
                label: c.label,
                element_type: c.element_type,
            })
            .collect(),
    };
    RemoteResponse::with_data(ResponseData::UiState(data))
}

fn handle_list_panels(editor: &Editor) -> RemoteResponse {
    let panels: Vec<PanelInfo> = editor
        .ui_state()
        .panels()
        .iter()
        .map(|p| PanelInfo {
            id: p.id.clone(),
            title: p.title.clone(),
            is_focused: p.is_focused,
        })
        .collect();
    RemoteResponse::with_data(ResponseData::Panels(panels))
}

fn handle_get_clickable_elements(editor: &Editor) -> RemoteResponse {
    let elements: Vec<ClickableInfo> = editor
        .ui_state()
        .clickable_elements()
        .iter()
        .map(|c| ClickableInfo {
            id: c.id.clone(),
            label: c.label.clone(),
            element_type: c.element_type.clone(),
        })
        .collect();
    RemoteResponse::with_data(ResponseData::Clickables(elements))
}

fn handle_select_by_path(editor: &mut Editor, engine: &Engine, path: &str) -> RemoteResponse {
    // Find entity by name path (for now, simple name match)
    // Path format: "EntityName" or "Parent/Child/Entity"
    let entity_name = path.split('/').last().unwrap_or(path);
    let found = engine
        .world()
        .inner()
        .iter()
        .find(|entity_ref| {
            let handle = EntityHandle::new(entity_ref.entity());
            engine
                .world()
                .get::<Name>(handle)
                .ok()
                .map(|n| n.0 == entity_name)
                .unwrap_or(false)
        })
        .map(|e| e.entity());

    match found {
        Some(entity) => {
            editor.state_mut().select(Some(entity));
            RemoteResponse::ok()
        }
        None => RemoteResponse::error(format!("Entity not found by path: {}", path)),
    }
}

// --- Asset Browser Handlers ---

fn handle_get_asset_browser_state(editor: &Editor) -> RemoteResponse {
    let state = editor.project_panel_state();
    let selected_folder = state.selected_folder.display().to_string();
    let selected_file = state.selected_file.as_ref().map(|p| p.display().to_string());

    // Collect files from current folder
    let mut files = Vec::new();
    if let Some(tree) = editor.project_tree() {
        fn find_folder<'a>(
            node: &'a DirectoryNode,
            path: &std::path::Path,
        ) -> Option<&'a DirectoryNode> {
            if node.path == path {
                return Some(node);
            }
            for child in &node.children {
                if let Some(found) = find_folder(child, path) {
                    return Some(found);
                }
            }
            None
        }

        let folder = find_folder(tree, &state.selected_folder).unwrap_or(tree);

        for file in &folder.files {
            files.push(AssetFileInfo {
                path: file.path.display().to_string(),
                name: file.name.clone(),
                file_type: format!("{:?}", file.file_type),
                is_text_editable: file.file_type.is_text_editable(),
            });
        }
    }

    let data = AssetBrowserData {
        selected_folder,
        selected_file,
        files,
    };
    RemoteResponse::with_data(ResponseData::AssetBrowser(data))
}

fn handle_open_asset_file(editor: &mut Editor, engine: &Engine, path: &str) -> RemoteResponse {
    log::info!("Remote: Opening asset file '{}'", path);
    if let Some(game_path) = engine.game_path() {
        let file_path = std::path::PathBuf::from(path);

        // Determine file type
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());
        let file_type = crate::project_panel_state::FileType::from_extension(extension.as_deref());

        log::info!(
            "File type for '{}': {:?}, is_text_editable: {}",
            path,
            file_type,
            file_type.is_text_editable()
        );

        if file_type.is_text_editable() {
            // Get relative path from project root
            if let Ok(relative) = file_path.strip_prefix(game_path) {
                let script_path = relative.to_path_buf();
                log::info!("Opening script from remote: {:?}", script_path);
                match editor.script_editor_state_mut().open(script_path, game_path) {
                    Ok(()) => {
                        editor.recheck_script_errors();
                        editor.ensure_script_editor_visible();
                        RemoteResponse::ok()
                    }
                    Err(e) => {
                        log::error!("Failed to open script: {}", e);
                        RemoteResponse::error(format!("Failed to open script: {}", e))
                    }
                }
            } else {
                log::error!(
                    "Path {:?} is not under project {:?}",
                    file_path,
                    game_path
                );
                RemoteResponse::error("File path is not under project root")
            }
        } else {
            log::info!(
                "File type {:?} is not text-editable, opening externally",
                file_type
            );
            if let Err(e) = open::that(&file_path) {
                RemoteResponse::error(format!("Failed to open external: {}", e))
            } else {
                RemoteResponse::ok()
            }
        }
    } else {
        RemoteResponse::error("No project loaded")
    }
}

fn handle_double_click_asset_file(
    editor: &mut Editor,
    engine: &Engine,
    path: &str,
) -> RemoteResponse {
    log::info!("Remote: Double-clicking asset file '{}'", path);
    if let Some(game_path) = engine.game_path() {
        let file_path = std::path::PathBuf::from(path);

        // Determine file type
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());
        let file_type = crate::project_panel_state::FileType::from_extension(extension.as_deref());

        log::info!(
            "Double-click: File type for '{}': {:?}, is_text_editable: {}",
            path,
            file_type,
            file_type.is_text_editable()
        );

        if file_type.is_text_editable() {
            // Open in script editor
            if let Ok(relative) = file_path.strip_prefix(game_path) {
                let script_path = relative.to_path_buf();
                log::info!("Double-click: Opening script {:?}", script_path);
                match editor.script_editor_state_mut().open(script_path, game_path) {
                    Ok(()) => {
                        editor.recheck_script_errors();
                        editor.ensure_script_editor_visible();
                        RemoteResponse::ok()
                    }
                    Err(e) => {
                        log::error!("Double-click: Failed to open script: {}", e);
                        RemoteResponse::error(format!("Failed to open script: {}", e))
                    }
                }
            } else {
                log::error!(
                    "Double-click: Path {:?} is not under project {:?}",
                    file_path,
                    game_path
                );
                RemoteResponse::error("File path is not under project root")
            }
        } else if file_type == crate::project_panel_state::FileType::Image {
            log::info!("Double-click: TODO: Open image preview for {:?}", path);
            RemoteResponse::ok()
        } else {
            // Open externally
            log::info!("Double-click: Opening externally {:?}", path);
            if let Err(e) = open::that(&file_path) {
                RemoteResponse::error(format!("Failed to open external: {}", e))
            } else {
                RemoteResponse::ok()
            }
        }
    } else {
        RemoteResponse::error("No project loaded")
    }
}

fn handle_asset_context_open_in_editor(
    editor: &mut Editor,
    engine: &Engine,
    path: &str,
) -> RemoteResponse {
    log::info!("Remote: Context menu 'Open in Editor' for '{}'", path);
    if let Some(game_path) = engine.game_path() {
        let file_path = std::path::PathBuf::from(path);

        // Determine file type
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());
        let file_type = crate::project_panel_state::FileType::from_extension(extension.as_deref());

        log::info!(
            "Context Open in Editor: File type for '{}': {:?}, is_text_editable: {}",
            path,
            file_type,
            file_type.is_text_editable()
        );

        if file_type.is_text_editable() {
            // Open in script editor
            if let Ok(relative) = file_path.strip_prefix(game_path) {
                let script_path = relative.to_path_buf();
                log::info!("Context Open in Editor: Opening script {:?}", script_path);
                match editor.script_editor_state_mut().open(script_path, game_path) {
                    Ok(()) => {
                        editor.recheck_script_errors();
                        editor.ensure_script_editor_visible();
                        RemoteResponse::ok()
                    }
                    Err(e) => {
                        log::error!("Context Open in Editor: Failed to open script: {}", e);
                        RemoteResponse::error(format!("Failed to open script: {}", e))
                    }
                }
            } else {
                log::error!(
                    "Context Open in Editor: Path {:?} is not under project {:?}",
                    file_path,
                    game_path
                );
                RemoteResponse::error("File path is not under project root")
            }
        } else {
            RemoteResponse::error(format!("File type {:?} is not text-editable", file_type))
        }
    } else {
        RemoteResponse::error("No project loaded")
    }
}
