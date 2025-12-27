//! Remote command processing for the editor.
//!
//! This module handles all remote commands sent to the editor via the remote control interface.
//! Commands are processed synchronously and return a response.
//!
//! The module is organized into focused submodules:
//! - [`entity`]: Entity manipulation (create, delete, select, properties, hierarchy)
//! - [`asset`]: Asset browsing and loading
//! - [`ui`]: UI state, panels, and script editing
//! - [`debug`]: Debug and render state queries
//! - [`testing`]: Test harness, gizmo, and scene tree simulation

mod asset;
mod debug;
mod entity;
mod testing;
mod ui;

use longhorn_engine::Engine;
use longhorn_remote::{RemoteCommand, RemoteResponse};

use crate::ui_state::TriggerAction;
use crate::{Editor, ToolbarAction};

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
        RemoteCommand::GetState => entity::handle_get_state(editor, engine),
        RemoteCommand::GetEntities => entity::handle_get_entities(engine),
        RemoteCommand::GetEntity { id } => entity::handle_get_entity(engine, id),

        // Entity manipulation
        RemoteCommand::SelectEntity { id } => entity::handle_select_entity(editor, engine, id),
        RemoteCommand::CreateEntity { name } => entity::handle_create_entity(engine, &name),
        RemoteCommand::DeleteEntity { id } => entity::handle_delete_entity(editor, engine, id),
        RemoteCommand::SetProperty {
            entity,
            component,
            field,
            value,
        } => entity::set_entity_property(engine, entity, &component, &field, value),
        RemoteCommand::SetEntityParent { child_id, parent_id } => {
            entity::handle_set_entity_parent(engine, child_id, parent_id)
        }
        RemoteCommand::ClearEntityParent { child_id } => {
            entity::handle_clear_entity_parent(engine, child_id)
        }

        // Project
        RemoteCommand::LoadProject { path } => ui::handle_load_project(editor, engine, &path),

        // Script Editor
        RemoteCommand::OpenScript { path } => ui::handle_open_script(editor, engine, &path),
        RemoteCommand::SaveScript => ui::handle_save_script(editor),
        RemoteCommand::GetScriptEditorState => ui::handle_get_script_editor_state(editor),

        // UI State
        RemoteCommand::GetUiState => ui::handle_get_ui_state(editor),
        RemoteCommand::ListPanels => ui::handle_list_panels(editor),
        RemoteCommand::GetClickableElements => ui::handle_get_clickable_elements(editor),
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
        RemoteCommand::SelectByPath { path } => ui::handle_select_by_path(editor, engine, &path),

        // Asset Browser
        RemoteCommand::GetAssetBrowserState => asset::handle_get_asset_browser_state(editor),
        RemoteCommand::OpenAssetFile { path } => {
            asset::handle_open_asset_file(editor, engine, &path)
        }
        RemoteCommand::SelectAssetFile { path } => {
            log::info!("Remote: Selecting asset file '{}'", path);
            let file_path = std::path::PathBuf::from(&path);
            editor.project_panel_state_mut().selected_file = Some(file_path);
            RemoteResponse::ok()
        }
        RemoteCommand::DoubleClickAssetFile { path } => {
            asset::handle_double_click_asset_file(editor, engine, &path)
        }
        RemoteCommand::AssetContextOpenInEditor { path } => {
            asset::handle_asset_context_open_in_editor(editor, engine, &path)
        }

        // Debug commands
        RemoteCommand::GetEntityComponents { id } => entity::handle_get_entity_components(engine, id),
        RemoteCommand::GetAssets => asset::handle_get_assets(engine),
        RemoteCommand::GetRenderState => debug::handle_get_render_state(editor, engine),
        RemoteCommand::DumpEntity { id } => entity::handle_dump_entity(engine, id),

        // Asset loading commands
        RemoteCommand::LoadTexture { id } => asset::handle_load_texture(engine, id),
        RemoteCommand::LoadAllTextures => asset::handle_load_all_textures(engine),

        // Testing commands
        RemoteCommand::TakeScreenshot { path } => testing::handle_take_screenshot(editor, &path),
        RemoteCommand::GetLogTail { lines } => testing::handle_get_log_tail(lines),
        RemoteCommand::WaitFrames { count } => testing::handle_wait_frames(editor, count),

        // Gizmo commands
        RemoteCommand::GetGizmoState => testing::handle_get_gizmo_state(editor),
        RemoteCommand::SimulateGizmoDrag {
            entity_id,
            handle,
            delta_x,
            delta_y,
        } => testing::handle_simulate_gizmo_drag(editor, engine, entity_id, &handle, delta_x, delta_y),

        // Scene Tree Drag-Drop commands
        RemoteCommand::SimulateSceneTreeDrag {
            dragged_entity_id,
            target_entity_id,
        } => testing::handle_simulate_scene_tree_drag(editor, engine, dragged_entity_id, target_entity_id),
        RemoteCommand::SimulateSceneTreeDragToRoot { entity_id } => {
            testing::handle_simulate_scene_tree_drag_to_root(editor, engine, entity_id)
        }
    }
}
