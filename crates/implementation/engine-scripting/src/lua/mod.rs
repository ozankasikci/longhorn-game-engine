//! Lua scripting engine implementation

pub mod engine;
pub mod bindings;
pub mod ecs;
pub mod events;
pub mod events_simple;
pub mod input;
pub mod physics;
pub mod debugging;
pub mod profiler;

#[cfg(test)]
mod physics_simple_test;
pub mod assets;

pub use engine::LuaScriptEngine;