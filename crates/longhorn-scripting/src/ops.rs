// crates/longhorn-scripting/src/ops.rs
//! Longhorn engine ops - functions exposed to JavaScript scripts
//!
//! These ops are registered as global functions in the QuickJS runtime
//! and called from JavaScript via the bootstrap.js wrappers.

use longhorn_core::Vec2;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Callback for console output (type-erased to avoid editor dependency)
pub type ConsoleCallback = Arc<dyn Fn(&str, &str) + Send + Sync>;

/// Thread-local console callback
thread_local! {
    static CONSOLE_CALLBACK: std::cell::RefCell<Option<ConsoleCallback>> = std::cell::RefCell::new(None);
}

/// Thread-local storage for pending events emitted by scripts
thread_local! {
    static PENDING_EVENTS: std::cell::RefCell<Vec<(String, serde_json::Value)>> =
        std::cell::RefCell::new(Vec::new());
}

/// Thread-local storage for pending entity-targeted events
thread_local! {
    static PENDING_TARGETED_EVENTS: std::cell::RefCell<Vec<(u64, String, serde_json::Value)>> =
        std::cell::RefCell::new(Vec::new());
}

/// Set the console callback for the current thread
pub fn set_console_callback(callback: Option<ConsoleCallback>) {
    CONSOLE_CALLBACK.with(|cb| {
        *cb.borrow_mut() = callback;
    });
}

/// Get the console callback for the current thread
pub fn get_console_callback() -> Option<ConsoleCallback> {
    CONSOLE_CALLBACK.with(|cb| cb.borrow().clone())
}

/// Push a pending event (called from js_runtime ops)
pub fn push_pending_event(event_name: String, data: serde_json::Value) {
    PENDING_EVENTS.with(|events| {
        events.borrow_mut().push((event_name, data));
    });
}

/// Push a pending targeted event (called from js_runtime ops)
pub fn push_pending_targeted_event(entity_id: u64, event_name: String, data: serde_json::Value) {
    PENDING_TARGETED_EVENTS.with(|events| {
        events.borrow_mut().push((entity_id, event_name, data));
    });
}

/// Collect all pending events emitted by scripts and clear the queue
pub fn take_pending_events() -> Vec<(String, serde_json::Value)> {
    PENDING_EVENTS.with(|events| std::mem::take(&mut *events.borrow_mut()))
}

/// Collect all pending targeted events emitted by scripts and clear the queue
pub fn take_pending_targeted_events() -> Vec<(u64, String, serde_json::Value)> {
    PENDING_TARGETED_EVENTS.with(|events| std::mem::take(&mut *events.borrow_mut()))
}

/// Shared state accessible from ops
pub struct OpsState {
    /// Current entity ID being processed
    pub current_entity_id: Option<u64>,
}

impl OpsState {
    pub fn new() -> Self {
        Self {
            current_entity_id: None,
        }
    }
}

impl Default for OpsState {
    fn default() -> Self {
        Self::new()
    }
}

/// Transform data for JS interop
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsTransform {
    pub position: JsVec2,
    pub rotation: f64,
    pub scale: JsVec2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsVec2 {
    pub x: f64,
    pub y: f64,
}

/// Sprite data for JS interop
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsSprite {
    pub texture: u64,
    pub size: JsVec2,
    pub color: [f64; 4],
    pub flip_x: bool,
    pub flip_y: bool,
}

impl From<&longhorn_core::Transform> for JsTransform {
    fn from(t: &longhorn_core::Transform) -> Self {
        Self {
            position: JsVec2 {
                x: t.position.x as f64,
                y: t.position.y as f64,
            },
            rotation: t.rotation as f64,
            scale: JsVec2 {
                x: t.scale.x as f64,
                y: t.scale.y as f64,
            },
        }
    }
}

impl From<JsTransform> for longhorn_core::Transform {
    fn from(t: JsTransform) -> Self {
        Self {
            position: Vec2::new(t.position.x as f32, t.position.y as f32),
            rotation: t.rotation as f32,
            scale: Vec2::new(t.scale.x as f32, t.scale.y as f32),
        }
    }
}

impl From<&longhorn_core::Sprite> for JsSprite {
    fn from(s: &longhorn_core::Sprite) -> Self {
        Self {
            texture: s.texture.0,
            size: JsVec2 {
                x: s.size.x as f64,
                y: s.size.y as f64,
            },
            color: [
                s.color[0] as f64,
                s.color[1] as f64,
                s.color[2] as f64,
                s.color[3] as f64,
            ],
            flip_x: s.flip_x,
            flip_y: s.flip_y,
        }
    }
}

impl From<JsSprite> for longhorn_core::Sprite {
    fn from(s: JsSprite) -> Self {
        Self {
            texture: longhorn_core::AssetId::new(s.texture),
            size: Vec2::new(s.size.x as f32, s.size.y as f32),
            color: [
                s.color[0] as f32,
                s.color[1] as f32,
                s.color[2] as f32,
                s.color[3] as f32,
            ],
            flip_x: s.flip_x,
            flip_y: s.flip_y,
        }
    }
}

/// The 'self' object passed to script lifecycle methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsSelf {
    pub id: u64,
    pub transform: Option<JsTransform>,
    pub sprite: Option<JsSprite>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ops_state_default() {
        let state = OpsState::default();
        assert!(state.current_entity_id.is_none());
    }

    #[test]
    fn test_js_transform_serialization() {
        let transform = JsTransform {
            position: JsVec2 { x: 10.0, y: 20.0 },
            rotation: 0.5,
            scale: JsVec2 { x: 1.0, y: 1.0 },
        };

        let json = serde_json::to_string(&transform).unwrap();
        assert!(json.contains("position"));
        assert!(json.contains("10"));
    }

    #[test]
    fn test_pending_events() {
        // Clear any existing events
        take_pending_events();

        // Push some events
        push_pending_event("test_event".to_string(), serde_json::json!({"foo": "bar"}));
        push_pending_event("another_event".to_string(), serde_json::json!(42));

        // Take events
        let events = take_pending_events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].0, "test_event");
        assert_eq!(events[1].0, "another_event");

        // Queue should be empty now
        let events = take_pending_events();
        assert!(events.is_empty());
    }

    #[test]
    fn test_pending_targeted_events() {
        // Clear any existing events
        take_pending_targeted_events();

        // Push a targeted event
        push_pending_targeted_event(123, "hit".to_string(), serde_json::json!({"damage": 10}));

        // Take events
        let events = take_pending_targeted_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, 123); // entity_id
        assert_eq!(events[0].1, "hit"); // event_name
    }
}
