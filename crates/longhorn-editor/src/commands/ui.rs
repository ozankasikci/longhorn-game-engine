//! UI state and script editor command handlers.
//!
//! Handles UI state queries, panel management, and script editing operations.

use longhorn_core::{EntityHandle, Name};
use longhorn_engine::Engine;
use longhorn_remote::{
    ClickableInfo, PanelInfo, RemoteResponse, ResponseData, ScriptEditorData, ScriptErrorData,
    UiStateData,
};

use crate::Editor;

// --- UI State Handlers ---

pub fn handle_get_ui_state(editor: &Editor) -> RemoteResponse {
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

pub fn handle_list_panels(editor: &Editor) -> RemoteResponse {
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

pub fn handle_get_clickable_elements(editor: &Editor) -> RemoteResponse {
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

pub fn handle_select_by_path(editor: &mut Editor, engine: &Engine, path: &str) -> RemoteResponse {
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

// --- Project Handlers ---

pub fn handle_load_project(
    editor: &mut Editor,
    engine: &mut Engine,
    path: &str,
) -> RemoteResponse {
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

pub fn handle_open_script(editor: &mut Editor, engine: &Engine, path: &str) -> RemoteResponse {
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

pub fn handle_save_script(editor: &mut Editor) -> RemoteResponse {
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

pub fn handle_get_script_editor_state(editor: &Editor) -> RemoteResponse {
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
