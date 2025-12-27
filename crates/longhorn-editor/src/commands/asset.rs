//! Asset browser and loading command handlers.
//!
//! Handles asset browsing, file operations, and texture loading.

use longhorn_core::AssetId;
use longhorn_engine::Engine;
use longhorn_remote::{
    AssetBrowserData, AssetFileInfo, AssetInfo, RemoteResponse, ResponseData, TextureLoadResult,
};

use crate::{DirectoryNode, Editor};

// --- Asset Browser Handlers ---

pub fn handle_get_asset_browser_state(editor: &Editor) -> RemoteResponse {
    let state = editor.project_panel_state();
    let selected_folder = state.selected_folder.display().to_string();
    let selected_file = state.selected_file.as_ref().map(|p| p.display().to_string());

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

pub fn handle_open_asset_file(
    editor: &mut Editor,
    engine: &Engine,
    path: &str,
) -> RemoteResponse {
    log::info!("Remote: Opening asset file '{}'", path);
    if let Some(game_path) = engine.game_path() {
        let file_path = std::path::PathBuf::from(path);

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

pub fn handle_double_click_asset_file(
    editor: &mut Editor,
    engine: &Engine,
    path: &str,
) -> RemoteResponse {
    log::info!("Remote: Double-clicking asset file '{}'", path);
    if let Some(game_path) = engine.game_path() {
        let file_path = std::path::PathBuf::from(path);

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

pub fn handle_asset_context_open_in_editor(
    editor: &mut Editor,
    engine: &Engine,
    path: &str,
) -> RemoteResponse {
    log::info!("Remote: Context menu 'Open in Editor' for '{}'", path);
    if let Some(game_path) = engine.game_path() {
        let file_path = std::path::PathBuf::from(path);

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

// --- Asset Query Handlers ---

pub fn handle_get_assets(engine: &Engine) -> RemoteResponse {
    let mut assets = Vec::new();

    if let Some(game_path) = engine.game_path() {
        let assets_json_path = game_path.join("assets.json");
        if assets_json_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&assets_json_path) {
                if let Ok(map) =
                    serde_json::from_str::<std::collections::HashMap<String, u64>>(&content)
                {
                    for (path, id) in map {
                        let loaded = engine.assets().is_texture_loaded(AssetId(id));
                        assets.push(AssetInfo { id, path, loaded });
                    }
                }
            }
        }
    }

    RemoteResponse::with_data(ResponseData::Assets(assets))
}

// --- Asset Loading Handlers ---

pub fn handle_load_texture(engine: &mut Engine, id: u64) -> RemoteResponse {
    let asset_id = AssetId(id);

    let path = match engine.assets().get_asset_path(asset_id) {
        Some(p) => p.to_string(),
        None => return RemoteResponse::error(format!("Asset ID {} not found in registry", id)),
    };

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

pub fn handle_load_all_textures(engine: &mut Engine) -> RemoteResponse {
    let mut results = Vec::new();

    if let Some(game_path) = engine.game_path() {
        let assets_json_path = game_path.join("assets.json");
        if assets_json_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&assets_json_path) {
                if let Ok(map) =
                    serde_json::from_str::<std::collections::HashMap<String, u64>>(&content)
                {
                    for (path, id) in map {
                        if path.ends_with(".png")
                            || path.ends_with(".jpg")
                            || path.ends_with(".jpeg")
                        {
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
