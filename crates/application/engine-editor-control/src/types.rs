//! Type definitions for editor control commands and responses

use serde::{Deserialize, Serialize};

/// Commands that can be sent to control the editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditorCommand {
    /// Script Management
    AddScript { entity_id: u32, script_path: String },
    RemoveScript { entity_id: u32, script_path: String },
    ReplaceScript { entity_id: u32, old_path: String, new_path: String },
    GetEntityScripts { entity_id: u32 },
    
    /// Scene Inspection
    GetSceneObjects,
    GetEntityInfo { entity_id: u32 },
    GetLogs { lines: Option<usize> },
    
    /// Game State Control
    StartGame,
    StopGame,
    PauseGame,
    ResumeGame,
    GetGameState,
    
    /// Script Testing
    TriggerHotReload { script_path: String },
    ForceScriptReinitialization,
    GetScriptErrors,
    GetCompilationEvents,
    
    /// TypeScript Debugging
    GetTypeScriptSystemStatus,
    GetScriptInstances,
    GetInitializedEntities,
    GetDeadScripts,
    GetScriptExecutionLogs,
    TestScriptExecution { entity_id: u32 },
    ValidateScriptFiles,
    GetV8RuntimeStats,
    TriggerScriptRecompilation { script_path: String },
    SimulateFileChange { script_path: String },
    
    /// General
    Ping,
    Shutdown,
}

/// Responses from the editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditorResponse {
    /// Success responses
    Success,
    Pong,
    
    /// Data responses
    SceneObjects(Vec<SceneObject>),
    EntityInfo(EntityInfo),
    EntityScripts(Vec<String>),
    Logs(Vec<String>),
    GameState(GameStateInfo),
    ScriptErrors(Vec<ScriptError>),
    CompilationEvents(Vec<CompilationEvent>),
    
    /// TypeScript Debugging responses
    TypeScriptSystemStatus(TypeScriptSystemStatus),
    ScriptInstances(Vec<ScriptInstanceInfo>),
    InitializedEntities(Vec<u32>),
    DeadScripts(Vec<u32>),
    ScriptExecutionLogs(Vec<String>),
    V8RuntimeStats(V8RuntimeStats),
    FileValidationResults(Vec<FileValidationResult>),
    
    /// Error responses
    Error { message: String },
    EntityNotFound { entity_id: u32 },
    ScriptNotFound { script_path: String },
}

/// Information about a scene object/entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneObject {
    pub entity_id: u32,
    pub name: String,
    pub transform: Option<TransformInfo>,
    pub scripts: Vec<String>,
    pub components: Vec<String>,
}

/// Entity detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityInfo {
    pub entity_id: u32,
    pub name: Option<String>,
    pub transform: Option<TransformInfo>,
    pub scripts: ScriptInfo,
    pub components: Vec<ComponentInfo>,
}

/// Transform component information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformInfo {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

/// Script information for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptInfo {
    pub typescript_scripts: Vec<String>,
}

/// Component information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub component_type: String,
    pub data: serde_json::Value,
}

/// Game state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateInfo {
    pub is_playing: bool,
    pub is_paused: bool,
    pub frame_count: u64,
    pub delta_time: f64,
}

/// Script compilation/runtime error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptError {
    pub script_path: String,
    pub error_type: String,
    pub message: String,
    pub line: Option<u32>,
}

/// Script compilation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationEvent {
    pub script_path: String,
    pub event_type: String, // "started", "completed", "failed"
    pub timestamp: u64,
    pub success: Option<bool>,
}

/// Request/Response wrapper for network communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorMessage {
    pub id: String,
    pub command: EditorCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorReply {
    pub id: String,
    pub response: EditorResponse,
}

/// Actions that can be sent to the main editor thread
#[derive(Debug, Clone)]
pub enum EditorAction {
    StartPlay,
    StopPlay,
    PausePlay,
    ResumePlay,
    SyncScriptRemoval { entity_id: u32, script_path: String },
    ForceScriptReinitialization,
}

/// TypeScript system status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeScriptSystemStatus {
    pub runtime_available: bool,
    pub total_entities: usize,
    pub initialized_entities: usize,
    pub script_instances: usize,
    pub dead_scripts: usize,
    pub last_update_time: Option<String>,
    pub compilation_events_pending: usize,
}

/// Information about a script instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptInstanceInfo {
    pub script_id: u32,
    pub entity_id: u32,
    pub script_path: String,
    pub initialized: bool,
    pub compilation_successful: bool,
    pub last_error: Option<String>,
}

/// V8 runtime statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V8RuntimeStats {
    pub heap_used_bytes: usize,
    pub heap_total_bytes: usize,
    pub external_memory_bytes: usize,
    pub script_instances_count: usize,
    pub global_context_available: bool,
}

/// File validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileValidationResult {
    pub file_path: String,
    pub exists: bool,
    pub readable: bool,
    pub size_bytes: Option<u64>,
    pub last_modified: Option<String>,
    pub content_hash: Option<String>,
    pub syntax_valid: Option<bool>,
    pub error: Option<String>,
}