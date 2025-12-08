pub mod types;
pub mod server;

// Re-export commonly used types
pub use types::{
    RemoteCommand, RemoteResponse, ResponseData, PendingCommand,
    EntityInfo, EntityDetails, TransformData,
    ScriptEditorData, ScriptErrorData,
    PanelInfo, ClickableInfo, UiStateData,
    AssetBrowserData, AssetFileInfo,
    SpriteData, ComponentInfo, EntityDump,
    AssetInfo, RenderStateData, TextureLoadResult,
    ScreenshotResult, LogEntry, LogTailResult, WaitFramesResult,
    GizmoStateData, GizmoDragResult,
};

pub use server::RemoteServer;
