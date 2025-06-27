//! Scripting system for the mobile game engine
//!
//! This crate provides scripting capabilities for game logic,
//! allowing runtime behavior modification and extensibility.

pub mod api;
pub mod bindings;
pub mod manager;
pub mod runtime;
pub mod types;
pub mod lua;
pub mod components;
pub mod file_manager;
pub mod component_manager;
pub mod lua_script_system;
#[deprecated(since = "1.0.0", note = "Use ecs_component_storage instead")]
pub mod shared_state;
pub mod security_tests;
pub mod resource_limits;
pub mod ecs_console;
pub mod ecs_component_storage;
pub mod secure_lua_engine;
pub mod resource_limits_enforcement_tests;
pub mod architecture_separation_tests;
pub mod script_engine;
pub mod error_handling_tests;
pub mod error;
pub mod api_security_tests;

#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
pub mod error_tests;
pub mod unified_loader;

pub use api::ScriptApi;
pub use unified_loader::UnifiedScriptLoader;
pub use manager::{ScriptManager, ScriptRef};
pub use script_engine::ScriptEngine;
pub use secure_lua_engine::SecureLuaScriptEngine;
pub use runtime::ScriptRuntime;
pub use types::{ScriptId, ScriptMetadata, ScriptType};
pub use file_manager::{ScriptFileManager, ScriptFileInfo, ScriptValidation};
pub use component_manager::{LuaScriptComponentManager, EntityScriptInfo, ScriptComponentStatus};
pub use lua_script_system::LuaScriptSystem;
pub use components::Velocity;
pub use lua::engine::{get_and_clear_console_messages, ConsoleMessage as LuaConsoleMessage};

// Re-export the new comprehensive error types
pub use error::{ScriptError, SecuritySeverity};

/// Scripting system result type
pub type ScriptResult<T> = Result<T, ScriptError>;

