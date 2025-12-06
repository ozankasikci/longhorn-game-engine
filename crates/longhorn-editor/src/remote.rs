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

    // Utility
    Ping,
}

/// Information about an entity (minimal)
#[derive(Debug, Clone, Serialize)]
pub struct EntityInfo {
    pub id: u64,
    pub name: String,
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
    Created { id: u64 },
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
