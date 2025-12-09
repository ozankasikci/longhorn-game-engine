use serde::{Deserialize, Serialize};

/// Commands that can be sent to the editor via remote control
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum RemoteCommand {
    // Playback
    Play,
    Pause,
    Resume,
    Stop,

    // State queries
    GetState,
    GetEntities,
    GetEntity { id: u64 },

    // Entity manipulation
    SelectEntity { id: u64 },
    CreateEntity { name: String },
    DeleteEntity { id: u64 },
    SetProperty {
        entity: u64,
        component: String,
        field: String,
        value: serde_json::Value,
    },
    /// Set an entity's parent (creates hierarchy)
    SetEntityParent {
        child_id: u64,
        parent_id: u64,
    },
    /// Clear an entity's parent (make it root level)
    ClearEntityParent {
        child_id: u64,
    },

    // UI
    ToggleConsole,

    // Project
    LoadProject { path: String },

    // Script Editor
    OpenScript { path: String },
    SaveScript,
    GetScriptEditorState,

    // UI State (for remote control and automated testing)
    GetUiState,
    ListPanels,
    GetClickableElements,
    FocusPanel { panel: String },
    /// Trigger a UI element by ID (simple click)
    TriggerElement { id: String },
    /// Click a UI element by ID
    ClickElement { id: String },
    /// Double-click a UI element by ID
    DoubleClickElement { id: String },
    /// Right-click a UI element by ID (opens context menu)
    RightClickElement { id: String },

    // Scene Tree Control
    ExpandTreeNode { path: String },
    CollapseTreeNode { path: String },
    SelectByPath { path: String },

    // Asset Browser
    GetAssetBrowserState,
    OpenAssetFile { path: String },
    /// Select a file in the asset browser (single click)
    SelectAssetFile { path: String },
    /// Double-click on a file in the asset browser (opens in appropriate editor)
    DoubleClickAssetFile { path: String },
    /// Simulate clicking "Open in Editor" from context menu
    AssetContextOpenInEditor { path: String },

    // Utility
    Ping,

    // Debug commands
    /// Get all components on an entity
    GetEntityComponents { id: u64 },
    /// Get loaded assets info
    GetAssets,
    /// Get renderer state (textures loaded, sprite batch info)
    GetRenderState,
    /// Dump full entity state including all component data
    DumpEntity { id: u64 },

    // Asset loading commands
    /// Load a texture by asset ID
    LoadTexture { id: u64 },
    /// Load all registered textures
    LoadAllTextures,

    // Testing commands
    /// Take a screenshot and save to the specified path
    TakeScreenshot { path: String },
    /// Get the last N log entries
    GetLogTail { lines: usize },
    /// Wait for a number of frames to pass before responding
    WaitFrames { count: u32 },

    // Gizmo commands
    /// Get the current gizmo state
    GetGizmoState,
    /// Simulate dragging a gizmo handle (for automated testing)
    SimulateGizmoDrag {
        entity_id: u64,
        handle: String, // "move_x", "move_y", "move_xy"
        delta_x: f32,
        delta_y: f32,
    },

    // Scene Tree Drag-Drop commands (for testing)
    /// Simulate dragging one entity onto another in the scene tree
    SimulateSceneTreeDrag {
        dragged_entity_id: u64,
        target_entity_id: u64,
    },
    /// Simulate dragging an entity to the root drop zone
    SimulateSceneTreeDragToRoot {
        entity_id: u64,
    },
}

/// Information about an entity (minimal)
#[derive(Debug, Clone, Serialize)]
pub struct EntityInfo {
    pub id: u64,
    pub name: String,
}

/// Detailed entity information with components
#[derive(Debug, Clone, Serialize)]
pub struct EntityDetails {
    pub id: u64,
    pub name: String,
    pub transform: Option<TransformData>,
}

/// Transform component data
#[derive(Debug, Clone, Serialize)]
pub struct TransformData {
    pub position_x: f32,
    pub position_y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
}

/// Script editor state data
#[derive(Debug, Clone, Serialize)]
pub struct ScriptEditorData {
    pub is_open: bool,
    pub file_path: Option<String>,
    pub is_dirty: bool,
    pub error_count: usize,
    pub errors: Vec<ScriptErrorData>,
}

/// Script error data for remote
#[derive(Debug, Clone, Serialize)]
pub struct ScriptErrorData {
    pub line: usize,
    pub message: String,
}

/// UI panel state for remote queries
#[derive(Debug, Clone, Serialize)]
pub struct PanelInfo {
    pub id: String,
    pub title: String,
    pub is_focused: bool,
}

/// Clickable element info for remote queries
#[derive(Debug, Clone, Serialize)]
pub struct ClickableInfo {
    pub id: String,
    pub label: String,
    pub element_type: String,
}

/// Full UI state snapshot
#[derive(Debug, Clone, Serialize)]
pub struct UiStateData {
    pub focused_panel: Option<String>,
    pub panels: Vec<PanelInfo>,
    pub clickable_elements: Vec<ClickableInfo>,
}

/// Asset browser state for remote queries
#[derive(Debug, Clone, Serialize)]
pub struct AssetBrowserData {
    pub selected_folder: String,
    pub selected_file: Option<String>,
    pub files: Vec<AssetFileInfo>,
}

/// Asset file info
#[derive(Debug, Clone, Serialize)]
pub struct AssetFileInfo {
    pub path: String,
    pub name: String,
    pub file_type: String,
    pub is_text_editable: bool,
}

/// Sprite component data for debugging
#[derive(Debug, Clone, Serialize)]
pub struct SpriteData {
    pub texture_id: u64,
    pub size_x: f32,
    pub size_y: f32,
    pub color: [f32; 4],
    pub flip_x: bool,
    pub flip_y: bool,
}

/// Component info for debugging
#[derive(Debug, Clone, Serialize)]
pub struct ComponentInfo {
    pub name: String,
    pub data: serde_json::Value,
}

/// Full entity dump with all components
#[derive(Debug, Clone, Serialize)]
pub struct EntityDump {
    pub id: u64,
    pub name: Option<String>,
    pub transform: Option<TransformData>,
    pub sprite: Option<SpriteData>,
    pub has_script: bool,
    pub component_names: Vec<String>,
}

/// Asset info for debugging
#[derive(Debug, Clone, Serialize)]
pub struct AssetInfo {
    pub id: u64,
    pub path: String,
    pub loaded: bool,
}

/// Render state info for debugging
#[derive(Debug, Clone, Serialize)]
pub struct RenderStateData {
    pub loaded_texture_count: usize,
    pub texture_ids: Vec<u64>,
    pub sprite_count: usize,
}

/// Texture load result
#[derive(Debug, Clone, Serialize)]
pub struct TextureLoadResult {
    pub id: u64,
    pub path: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Screenshot result
#[derive(Debug, Clone, Serialize)]
pub struct ScreenshotResult {
    pub path: String,
    pub width: u32,
    pub height: u32,
}

/// Log entry for get_log_tail
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

/// Log tail result
#[derive(Debug, Clone, Serialize)]
pub struct LogTailResult {
    pub entries: Vec<LogEntry>,
}

/// Wait frames result
#[derive(Debug, Clone, Serialize)]
pub struct WaitFramesResult {
    pub frames_waited: u32,
}

/// Gizmo state data for remote queries
#[derive(Debug, Clone, Serialize)]
pub struct GizmoStateData {
    pub mode: String, // "none", "move", "rotate", "scale"
    pub active_handle: Option<String>,
    pub hover_handle: Option<String>,
    pub is_dragging: bool,
}

/// Gizmo drag simulation result
#[derive(Debug, Clone, Serialize)]
pub struct GizmoDragResult {
    pub entity_id: u64,
    pub old_position: [f32; 2],
    pub new_position: [f32; 2],
}

/// Response data variants
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ResponseData {
    State {
        mode: String,
        paused: bool,
        entity_count: usize,
        selected_entity: Option<u64>,
    },
    Entities(Vec<EntityInfo>),
    Entity(EntityDetails),
    Created { id: u64 },
    ScriptEditor(ScriptEditorData),
    UiState(UiStateData),
    Panels(Vec<PanelInfo>),
    Clickables(Vec<ClickableInfo>),
    AssetBrowser(AssetBrowserData),
    Components(Vec<ComponentInfo>),
    Assets(Vec<AssetInfo>),
    RenderState(RenderStateData),
    EntityDump(EntityDump),
    TextureLoaded(TextureLoadResult),
    TexturesLoaded(Vec<TextureLoadResult>),
    Screenshot(ScreenshotResult),
    LogTail(LogTailResult),
    FramesWaited(WaitFramesResult),
    GizmoState(GizmoStateData),
    GizmoDrag(GizmoDragResult),
}

/// Response sent back to the client
#[derive(Debug, Clone, Serialize)]
pub struct RemoteResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ResponseData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl RemoteResponse {
    pub fn ok() -> Self {
        Self { ok: true, data: None, error: None }
    }

    pub fn with_data(data: ResponseData) -> Self {
        Self { ok: true, data: Some(data), error: None }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self { ok: false, data: None, error: Some(message.into()) }
    }
}

/// A command with its response channel
pub struct PendingCommand {
    pub command: RemoteCommand,
    pub response_tx: std::sync::mpsc::Sender<RemoteResponse>,
}
