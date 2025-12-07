//! Remote command processing for the editor.
//!
//! This module handles all remote commands sent to the editor via the remote control interface.
//! Commands are processed synchronously and return a response.

use longhorn_core::{AssetId, EntityHandle, EntityId, Name, Sprite, Transform, Vec2};
use longhorn_engine::Engine;

use crate::remote::{
    AssetBrowserData, AssetFileInfo, AssetInfo, ClickableInfo, ComponentInfo, EntityDetails,
    EntityDump, EntityInfo, LogEntry, LogTailResult, PanelInfo, RemoteCommand, RemoteResponse,
    RenderStateData, ResponseData, ScreenshotResult, ScriptEditorData, ScriptErrorData, SpriteData,
    TextureLoadResult, TransformData, UiStateData, WaitFramesResult,
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
        } => set_entity_property(engine, entity, &component, &field, value),

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

        // Debug commands
        RemoteCommand::GetEntityComponents { id } => handle_get_entity_components(engine, id),
        RemoteCommand::GetAssets => handle_get_assets(engine),
        RemoteCommand::GetRenderState => handle_get_render_state(editor, engine),
        RemoteCommand::DumpEntity { id } => handle_dump_entity(engine, id),

        // Asset loading commands
        RemoteCommand::LoadTexture { id } => handle_load_texture(engine, id),
        RemoteCommand::LoadAllTextures => handle_load_all_textures(engine),

        // Testing commands
        RemoteCommand::TakeScreenshot { path } => handle_take_screenshot(editor, &path),
        RemoteCommand::GetLogTail { lines } => handle_get_log_tail(lines),
        RemoteCommand::WaitFrames { count } => handle_wait_frames(editor, count),
    }
}

// --- State Query Handlers ---

fn handle_get_state(editor: &Editor, engine: &Engine) -> RemoteResponse {
    let selected = editor.state().selected_entity.map(|e| e.to_bits().get());
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
                id: entity.to_bits().get(),
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
        .find(|e| e.entity().to_bits().get() == id);

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
        .find(|e| e.entity().to_bits().get() == id)
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
    engine: &mut Engine,
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
            let world = engine.world_mut();
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
            let world = engine.world_mut();
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
        "Sprite" => {
            let world = engine.world_mut();
            let mut sprite = match world.get::<Sprite>(handle) {
                Ok(s) => (*s).clone(),
                Err(_) => {
                    // Create a new sprite if one doesn't exist
                    Sprite::new(AssetId(0), Vec2::new(64.0, 64.0))
                }
            };

            match field {
                "texture" => {
                    let asset_id = if let Some(v) = value.as_u64() {
                        AssetId(v)
                    } else if let Some(v) = value.as_i64() {
                        AssetId(v as u64)
                    } else {
                        return RemoteResponse::error("texture must be a number (AssetId)");
                    };
                    sprite.texture = asset_id;

                    // Load the texture into the AssetManager cache so it's available for rendering
                    // We need to update the sprite first, then load the texture
                    if engine.world_mut().set(handle, sprite).is_err() {
                        return RemoteResponse::error("Failed to set Sprite");
                    }

                    // Now load the texture
                    if let Err(e) = engine.assets_mut().load_texture_by_id(asset_id) {
                        log::warn!("Failed to load texture {}: {}", asset_id.0, e);
                        // Don't fail the command - sprite was set, texture just couldn't be loaded
                    } else {
                        log::info!("Loaded texture {} into cache", asset_id.0);
                    }

                    return RemoteResponse::ok();
                }
                "size.x" | "size_x" => {
                    if let Some(v) = value.as_f64() {
                        sprite.size.x = v as f32;
                    }
                }
                "size.y" | "size_y" => {
                    if let Some(v) = value.as_f64() {
                        sprite.size.y = v as f32;
                    }
                }
                "flip_x" => {
                    if let Some(v) = value.as_bool() {
                        sprite.flip_x = v;
                    }
                }
                "flip_y" => {
                    if let Some(v) = value.as_bool() {
                        sprite.flip_y = v;
                    }
                }
                "color.r" => {
                    if let Some(v) = value.as_f64() {
                        sprite.color[0] = v as f32;
                    }
                }
                "color.g" => {
                    if let Some(v) = value.as_f64() {
                        sprite.color[1] = v as f32;
                    }
                }
                "color.b" => {
                    if let Some(v) = value.as_f64() {
                        sprite.color[2] = v as f32;
                    }
                }
                "color.a" => {
                    if let Some(v) = value.as_f64() {
                        sprite.color[3] = v as f32;
                    }
                }
                _ => return RemoteResponse::error(format!("Unknown Sprite field: {}", field)),
            }

            if engine.world_mut().set(handle, sprite).is_err() {
                return RemoteResponse::error("Failed to set Sprite");
            }
            RemoteResponse::ok()
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

// --- Debug Handlers ---

fn handle_get_entity_components(engine: &Engine, id: u64) -> RemoteResponse {
    let entity_id = match EntityId::from_bits(id) {
        Some(id) => id,
        None => return RemoteResponse::error(format!("Invalid entity id: {}", id)),
    };
    let handle = EntityHandle::new(entity_id);

    let mut components = Vec::new();

    // Check for Name
    if let Ok(name) = engine.world().get::<Name>(handle) {
        components.push(ComponentInfo {
            name: "Name".to_string(),
            data: serde_json::json!({ "value": name.0 }),
        });
    }

    // Check for Transform
    if let Ok(transform) = engine.world().get::<Transform>(handle) {
        components.push(ComponentInfo {
            name: "Transform".to_string(),
            data: serde_json::json!({
                "position": { "x": transform.position.x, "y": transform.position.y },
                "rotation": transform.rotation,
                "scale": { "x": transform.scale.x, "y": transform.scale.y }
            }),
        });
    }

    // Check for Sprite
    if let Ok(sprite) = engine.world().get::<Sprite>(handle) {
        components.push(ComponentInfo {
            name: "Sprite".to_string(),
            data: serde_json::json!({
                "texture_id": sprite.texture.0,
                "size": { "x": sprite.size.x, "y": sprite.size.y },
                "color": sprite.color,
                "flip_x": sprite.flip_x,
                "flip_y": sprite.flip_y
            }),
        });
    }

    // Check for Script component
    if let Ok(script) = engine.world().get::<longhorn_core::Script>(handle) {
        components.push(ComponentInfo {
            name: "Script".to_string(),
            data: serde_json::json!({ "path": script.path }),
        });
    }

    RemoteResponse::with_data(ResponseData::Components(components))
}

fn handle_get_assets(engine: &Engine) -> RemoteResponse {
    let mut assets = Vec::new();

    // Get asset registry info from the engine
    if let Some(game_path) = engine.game_path() {
        let assets_json_path = game_path.join("assets.json");
        if assets_json_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&assets_json_path) {
                if let Ok(map) = serde_json::from_str::<std::collections::HashMap<String, u64>>(&content) {
                    for (path, id) in map {
                        // Check if texture is loaded in the AssetManager cache
                        let loaded = engine.assets().is_texture_loaded(AssetId(id));
                        assets.push(AssetInfo {
                            id,
                            path,
                            loaded,
                        });
                    }
                }
            }
        }
    }

    RemoteResponse::with_data(ResponseData::Assets(assets))
}

fn handle_get_render_state(_editor: &Editor, engine: &Engine) -> RemoteResponse {
    // Get texture IDs from renderer
    let (loaded_texture_count, texture_ids) = match engine.renderer() {
        Some(renderer) => {
            let ids = renderer.loaded_texture_ids();
            (ids.len(), ids.into_iter().map(|id| id.0).collect())
        }
        None => (0, Vec::new()),
    };

    // Count sprites in the world
    let sprite_count = engine
        .world()
        .inner()
        .query::<&Sprite>()
        .iter()
        .count();

    let data = RenderStateData {
        loaded_texture_count,
        texture_ids,
        sprite_count,
    };

    RemoteResponse::with_data(ResponseData::RenderState(data))
}

fn handle_dump_entity(engine: &Engine, id: u64) -> RemoteResponse {
    let entity_id = match EntityId::from_bits(id) {
        Some(id) => id,
        None => return RemoteResponse::error(format!("Invalid entity id: {}", id)),
    };
    let handle = EntityHandle::new(entity_id);

    // Get Name
    let name = engine.world().get::<Name>(handle).ok().map(|n| n.0.clone());

    // Get Transform
    let transform = engine.world().get::<Transform>(handle).ok().map(|t| TransformData {
        position_x: t.position.x,
        position_y: t.position.y,
        rotation: t.rotation,
        scale_x: t.scale.x,
        scale_y: t.scale.y,
    });

    // Get Sprite
    let sprite = engine.world().get::<Sprite>(handle).ok().map(|s| SpriteData {
        texture_id: s.texture.0,
        size_x: s.size.x,
        size_y: s.size.y,
        color: s.color,
        flip_x: s.flip_x,
        flip_y: s.flip_y,
    });

    // Check for Script
    let has_script = engine.world().get::<longhorn_core::Script>(handle).is_ok();

    // Get component names
    let mut component_names = Vec::new();
    if name.is_some() {
        component_names.push("Name".to_string());
    }
    if transform.is_some() {
        component_names.push("Transform".to_string());
    }
    if sprite.is_some() {
        component_names.push("Sprite".to_string());
    }
    if has_script {
        component_names.push("Script".to_string());
    }

    let dump = EntityDump {
        id,
        name,
        transform,
        sprite,
        has_script,
        component_names,
    };

    RemoteResponse::with_data(ResponseData::EntityDump(dump))
}

// --- Asset Loading Handlers ---

fn handle_load_texture(engine: &mut Engine, id: u64) -> RemoteResponse {
    let asset_id = AssetId(id);

    // Get the path from the asset manager's registry
    let path = match engine.assets().get_asset_path(asset_id) {
        Some(p) => p.to_string(),
        None => return RemoteResponse::error(format!("Asset ID {} not found in registry", id)),
    };

    // Load the texture
    match engine.assets_mut().load_texture_by_id(asset_id) {
        Ok(_) => {
            log::info!("Loaded texture {} from path '{}'", id, path);
            RemoteResponse::with_data(ResponseData::TextureLoaded(TextureLoadResult {
                id,
                path,
                success: true,
                error: None,
            }))
        }
        Err(e) => {
            log::error!("Failed to load texture {}: {}", id, e);
            RemoteResponse::with_data(ResponseData::TextureLoaded(TextureLoadResult {
                id,
                path,
                success: false,
                error: Some(e.to_string()),
            }))
        }
    }
}

fn handle_load_all_textures(engine: &mut Engine) -> RemoteResponse {
    let mut results = Vec::new();

    // Get all registered assets from the registry
    if let Some(game_path) = engine.game_path() {
        let assets_json_path = game_path.join("assets.json");
        if assets_json_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&assets_json_path) {
                if let Ok(map) = serde_json::from_str::<std::collections::HashMap<String, u64>>(&content) {
                    for (path, id) in map {
                        // Only load image assets
                        if path.ends_with(".png") || path.ends_with(".jpg") || path.ends_with(".jpeg") {
                            let asset_id = AssetId(id);
                            match engine.assets_mut().load_texture_by_id(asset_id) {
                                Ok(_) => {
                                    log::info!("Loaded texture {} from path '{}'", id, path);
                                    results.push(TextureLoadResult {
                                        id,
                                        path,
                                        success: true,
                                        error: None,
                                    });
                                }
                                Err(e) => {
                                    log::error!("Failed to load texture {} ({}): {}", id, path, e);
                                    results.push(TextureLoadResult {
                                        id,
                                        path,
                                        success: false,
                                        error: Some(e.to_string()),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    RemoteResponse::with_data(ResponseData::TexturesLoaded(results))
}

// --- Testing Command Handlers ---

fn handle_take_screenshot(editor: &mut Editor, path: &str) -> RemoteResponse {
    // Request screenshot capture - will be handled by the main loop on next frame
    editor.request_screenshot(path.to_string());

    // Return immediately - actual screenshot happens on next frame
    RemoteResponse::with_data(ResponseData::Screenshot(ScreenshotResult {
        path: path.to_string(),
        width: 0, // Will be filled in by actual capture
        height: 0,
    }))
}

fn handle_get_log_tail(lines: usize) -> RemoteResponse {
    // Read from the log file
    let log_path = std::path::Path::new("logs/editor.log");

    let entries = if log_path.exists() {
        match std::fs::read_to_string(log_path) {
            Ok(content) => {
                let all_lines: Vec<&str> = content.lines().collect();
                let start = all_lines.len().saturating_sub(lines);
                all_lines[start..]
                    .iter()
                    .filter_map(|line| parse_log_line(line))
                    .collect()
            }
            Err(e) => {
                log::warn!("Failed to read log file: {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    RemoteResponse::with_data(ResponseData::LogTail(LogTailResult { entries }))
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
    // Parse log lines in format: [TIMESTAMP LEVEL target] message
    // Example: [2024-01-15T10:30:00Z INFO longhorn_editor] Loading project...

    // Simple parsing - just return the line as-is for now
    // A more sophisticated parser could extract timestamp/level/message
    if line.starts_with('[') {
        if let Some(bracket_end) = line.find(']') {
            let header = &line[1..bracket_end];
            let message = line[bracket_end + 1..].trim();

            // Try to extract level from header
            let level = if header.contains("ERROR") {
                "ERROR"
            } else if header.contains("WARN") {
                "WARN"
            } else if header.contains("INFO") {
                "INFO"
            } else if header.contains("DEBUG") {
                "DEBUG"
            } else if header.contains("TRACE") {
                "TRACE"
            } else {
                "INFO"
            };

            // Extract timestamp (first part before space)
            let timestamp = header.split_whitespace().next().unwrap_or("").to_string();

            return Some(LogEntry {
                timestamp,
                level: level.to_string(),
                message: message.to_string(),
            });
        }
    }

    // Fallback: treat entire line as message
    Some(LogEntry {
        timestamp: String::new(),
        level: "INFO".to_string(),
        message: line.to_string(),
    })
}

fn handle_wait_frames(editor: &Editor, count: u32) -> RemoteResponse {
    // Request frame wait - will be handled by the main loop
    editor.request_wait_frames(count);

    // Return immediately - main loop will delay the response
    RemoteResponse::with_data(ResponseData::FramesWaited(WaitFramesResult {
        frames_waited: count,
    }))
}
