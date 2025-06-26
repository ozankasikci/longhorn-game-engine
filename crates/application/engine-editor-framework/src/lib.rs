//! Core framework and state management for the Longhorn Game Engine editor
//!
//! This crate provides:
//! - Editor state management
//! - Play state coordination
//! - World setup utilities
//! - Bridge systems for engine integration
//! - Common types and data structures

pub mod bridge;
pub mod editor_coordinator;
pub mod editor_state;
pub mod play_state;
pub mod types;
pub mod world_setup;
pub mod unified_coordinator;

// Re-export main types
pub use editor_coordinator::EditorCoordinator;
pub use editor_state::{ConsoleMessage, ConsoleMessageType, EditorState, GameObject};
pub use play_state::{PlayState, PlayStateManager};
pub use types::{HierarchyObject, ObjectType, SceneObject};
pub use unified_coordinator::UnifiedEditorCoordinator;

// Re-export bridge types
pub use bridge::EcsSceneBridge;

// Re-export editor integration traits when editor feature is enabled
#[cfg(feature = "editor")]
pub use unified_coordinator::editor_integration::UnifiedCoordinatorExt;
