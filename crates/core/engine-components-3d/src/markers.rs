//! Component markers for single-world architecture
//!
//! These components are used to distinguish between editor-only entities
//! and runtime entities that should be processed by game systems.

use engine_component_traits::Component;
use engine_ecs_core::{World, Entity};
use serde::{Deserialize, Serialize};

/// Marker component for entities that exist only in the editor
/// and should not be processed by game systems.
/// 
/// Examples:
/// - Editor cameras
/// - Gizmos and visual aids
/// - UI elements that are part of the editor interface
/// - Debug visualization entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorOnly;

impl Component for EditorOnly {}

/// Marker component for entities that should be processed by game systems
/// during play mode (scripts, physics, etc.).
/// 
/// Examples:
/// - Game objects with scripts
/// - Physics bodies
/// - Entities that should be included in the game logic
/// - Player characters, NPCs, interactive objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeEntity;

impl Component for RuntimeEntity {}

/// Marker component for entities that should be visible in both editor and runtime.
/// This is the default for most game objects.
/// 
/// Examples:
/// - Static meshes that are part of the scene
/// - Lights that should render in both modes
/// - Environment objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedEntity;

impl Component for SharedEntity {}

/// Marker component for the main game camera that should be used
/// for rendering during play mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MainGameCamera;

impl Component for MainGameCamera {}

/// Marker component for editor cameras used for scene navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorCamera;

impl Component for EditorCamera {}

/// Helper functions for working with entity markers
impl EditorOnly {
    /// Check if an entity should be processed by game systems
    pub fn is_game_entity<T: Component>(
        world: &engine_ecs_core::World,
        entity: engine_ecs_core::Entity,
    ) -> bool {
        // Entity is a game entity if it's NOT marked as EditorOnly
        !world.has_component::<EditorOnly>(entity)
    }
}

impl RuntimeEntity {
    /// Check if an entity should be processed by script systems
    pub fn is_script_entity(
        world: &engine_ecs_core::World,
        entity: engine_ecs_core::Entity,
    ) -> bool {
        // Entity should be processed by scripts if it's marked as RuntimeEntity
        world.has_component::<RuntimeEntity>(entity)
    }
}

/// System set identifiers for organizing when systems run
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemSet {
    /// Systems that only run in editor mode (editing, gizmos, etc.)
    EditorSystems,
    /// Systems that only run during play mode (scripts, physics, etc.)
    RuntimeSystems,
    /// Systems that always run (rendering, input, etc.)
    SharedSystems,
}

/// Play mode state for controlling system execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayMode {
    /// Editor mode - only editor and shared systems run
    Editing,
    /// Play mode - runtime and shared systems run
    Playing,
    /// Paused - only shared systems run (for debugging)
    Paused,
}