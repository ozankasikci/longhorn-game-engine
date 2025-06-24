//! Shared types for editor panels

use eframe::egui;

// Re-export from scene view
pub use engine_editor_scene_view::types::{PlayState, SceneTool};

/// Simple trait for gizmo system interaction
pub trait GizmoSystem {
    fn get_active_tool(&self) -> SceneTool;
    fn set_active_tool(&mut self, tool: SceneTool);
    fn enable_move_gizmo(&mut self);
    fn disable_move_gizmo(&mut self);
}

/// Console message types
#[derive(Clone, Debug, PartialEq)]
pub enum ConsoleMessageType {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub enum ConsoleMessage {
    Message {
        message: String,
        message_type: ConsoleMessageType,
        timestamp: std::time::Instant,
    },
    UserAction(String),
}

impl ConsoleMessage {
    pub fn info(message: &str) -> Self {
        Self::Message {
            message: message.to_string(),
            message_type: ConsoleMessageType::Info,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn warning(message: &str) -> Self {
        Self::Message {
            message: message.to_string(),
            message_type: ConsoleMessageType::Warning,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn error(message: &str) -> Self {
        Self::Message {
            message: message.to_string(),
            message_type: ConsoleMessageType::Error,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn get_all_logs_as_string(messages: &[ConsoleMessage]) -> String {
        messages
            .iter()
            .filter_map(|msg| match msg {
                ConsoleMessage::Message { message, .. } => Some(message.clone()),
                ConsoleMessage::UserAction(_) => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// Re-export ProjectAsset from the asset crate
pub use engine_editor_assets::ProjectAsset;

/// Hierarchy object representation
#[derive(Clone)]
pub struct HierarchyObject {
    pub name: String,
    pub object_type: ObjectType,
    pub children: Option<Vec<HierarchyObject>>,
}

impl HierarchyObject {
    pub fn new(name: &str, object_type: ObjectType) -> Self {
        Self {
            name: name.to_string(),
            object_type,
            children: None,
        }
    }

    pub fn parent(name: &str, children: Vec<HierarchyObject>) -> Self {
        Self {
            name: name.to_string(),
            object_type: ObjectType::GameObject,
            children: Some(children),
        }
    }
}

#[derive(Clone)]
pub enum ObjectType {
    GameObject,
    Camera,
    Light,
}
