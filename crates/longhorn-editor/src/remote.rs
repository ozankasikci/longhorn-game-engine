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

    // UI
    ToggleConsole,

    // Project
    LoadProject { path: String },

    // Script Editor
    OpenScript { path: String },
    SaveScript,
    GetScriptEditorState,

    // Utility
    Ping,
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
