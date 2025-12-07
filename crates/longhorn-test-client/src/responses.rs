//! Response types from the editor.
//!
//! These mirror the types in longhorn-editor's remote.rs but are defined here
//! to avoid creating a dependency on the editor crate.

use serde::Deserialize;

/// Editor state information.
#[derive(Debug, Clone, Deserialize)]
pub struct EditorState {
    pub mode: String,
    pub paused: bool,
    pub entity_count: usize,
    pub selected_entity: Option<u64>,
}

/// Basic entity information.
#[derive(Debug, Clone, Deserialize)]
pub struct EntityInfo {
    pub id: u64,
    pub name: String,
}

/// Detailed entity information with components.
#[derive(Debug, Clone, Deserialize)]
pub struct EntityDetails {
    pub id: u64,
    pub name: String,
    pub transform: Option<TransformData>,
}

/// Transform component data.
#[derive(Debug, Clone, Deserialize)]
pub struct TransformData {
    pub position_x: f32,
    pub position_y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
}

/// Script editor state data.
#[derive(Debug, Clone, Deserialize)]
pub struct ScriptEditorData {
    pub is_open: bool,
    pub file_path: Option<String>,
    pub is_dirty: bool,
    pub error_count: usize,
    pub errors: Vec<ScriptErrorData>,
}

/// Script error data.
#[derive(Debug, Clone, Deserialize)]
pub struct ScriptErrorData {
    pub line: usize,
    pub message: String,
}

/// UI panel state.
#[derive(Debug, Clone, Deserialize)]
pub struct PanelInfo {
    pub id: String,
    pub title: String,
    pub is_focused: bool,
}

/// Clickable element info.
#[derive(Debug, Clone, Deserialize)]
pub struct ClickableInfo {
    pub id: String,
    pub label: String,
    pub element_type: String,
}

/// Full UI state snapshot.
#[derive(Debug, Clone, Deserialize)]
pub struct UiStateData {
    pub focused_panel: Option<String>,
    pub panels: Vec<PanelInfo>,
    pub clickable_elements: Vec<ClickableInfo>,
}

/// Asset browser state.
#[derive(Debug, Clone, Deserialize)]
pub struct AssetBrowserData {
    pub selected_folder: String,
    pub selected_file: Option<String>,
    pub files: Vec<AssetFileInfo>,
}

/// Asset file info.
#[derive(Debug, Clone, Deserialize)]
pub struct AssetFileInfo {
    pub path: String,
    pub name: String,
    pub file_type: String,
    pub is_text_editable: bool,
}

/// Sprite component data.
#[derive(Debug, Clone, Deserialize)]
pub struct SpriteData {
    pub texture_id: u64,
    pub size_x: f32,
    pub size_y: f32,
    pub color: [f32; 4],
    pub flip_x: bool,
    pub flip_y: bool,
}

/// Component info.
#[derive(Debug, Clone, Deserialize)]
pub struct ComponentInfo {
    pub name: String,
    pub data: serde_json::Value,
}

/// Full entity dump with all components.
#[derive(Debug, Clone, Deserialize)]
pub struct EntityDump {
    pub id: u64,
    pub name: Option<String>,
    pub transform: Option<TransformData>,
    pub sprite: Option<SpriteData>,
    pub has_script: bool,
    pub component_names: Vec<String>,
}

/// Asset info.
#[derive(Debug, Clone, Deserialize)]
pub struct AssetInfo {
    pub id: u64,
    pub path: String,
    pub loaded: bool,
}

/// Render state info.
#[derive(Debug, Clone, Deserialize)]
pub struct RenderStateData {
    pub loaded_texture_count: usize,
    pub texture_ids: Vec<u64>,
    pub sprite_count: usize,
}

/// Texture load result.
#[derive(Debug, Clone, Deserialize)]
pub struct TextureLoadResult {
    pub id: u64,
    pub path: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Screenshot result.
#[derive(Debug, Clone, Deserialize)]
pub struct ScreenshotResult {
    pub path: String,
    pub width: u32,
    pub height: u32,
}

/// Log entry.
#[derive(Debug, Clone, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

/// Log tail result.
#[derive(Debug, Clone, Deserialize)]
pub struct LogTailResult {
    pub entries: Vec<LogEntry>,
}

/// Wait frames result.
#[derive(Debug, Clone, Deserialize)]
pub struct WaitFramesResult {
    pub frames_waited: u32,
}

/// Created entity response.
#[derive(Debug, Clone, Deserialize)]
pub struct CreatedEntity {
    pub id: u64,
}

/// Generic remote response from the editor.
#[derive(Debug, Clone, Deserialize)]
pub struct RemoteResponse {
    pub ok: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}
