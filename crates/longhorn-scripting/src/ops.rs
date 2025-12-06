// crates/longhorn-scripting/src/ops.rs
use deno_core::op2;
use longhorn_core::Vec2;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Callback for console output (type-erased to avoid editor dependency)
pub type ConsoleCallback = Arc<dyn Fn(&str, &str) + Send + Sync>;

/// Thread-local console callback
thread_local! {
    static CONSOLE_CALLBACK: std::cell::RefCell<Option<ConsoleCallback>> = std::cell::RefCell::new(None);
}

/// Set the console callback for the current thread
pub fn set_console_callback(callback: Option<ConsoleCallback>) {
    CONSOLE_CALLBACK.with(|cb| {
        *cb.borrow_mut() = callback;
    });
}

/// Get the console callback for the current thread
fn get_console_callback() -> Option<ConsoleCallback> {
    CONSOLE_CALLBACK.with(|cb| cb.borrow().clone())
}

/// Shared state accessible from ops
pub struct OpsState {
    // These will be set before script execution
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
            position: JsVec2 { x: t.position.x as f64, y: t.position.y as f64 },
            rotation: t.rotation as f64,
            scale: JsVec2 { x: t.scale.x as f64, y: t.scale.y as f64 },
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
            size: JsVec2 { x: s.size.x as f64, y: s.size.y as f64 },
            color: [s.color[0] as f64, s.color[1] as f64, s.color[2] as f64, s.color[3] as f64],
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
            color: [s.color[0] as f32, s.color[1] as f32, s.color[2] as f32, s.color[3] as f32],
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

// Note: In a full implementation, these ops would access the actual World.
// For now, we define the interface. The ScriptRuntime will inject the World
// via op state before calling scripts.

#[op2(fast)]
pub fn op_log(#[string] level: String, #[string] message: String) {
    // Send to callback if set (for editor console)
    if let Some(callback) = get_console_callback() {
        callback(&level, &message);
    }

    // Also log via log crate for file output
    match level.as_str() {
        "error" => log::error!(target: "script", "{}", message),
        "warn" => log::warn!(target: "script", "{}", message),
        "info" => log::info!(target: "script", "{}", message),
        "debug" => log::debug!(target: "script", "{}", message),
        _ => log::info!(target: "script", "{}", message),
    }
}

#[op2(fast)]
#[bigint]
pub fn op_get_current_entity() -> u64 {
    // This will be replaced with actual state lookup
    // For now, return 0 to indicate no current entity
    0
}

// Extension definition for all longhorn ops
deno_core::extension!(
    longhorn_ops,
    ops = [op_log, op_get_current_entity],
);

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

    // Note: op_log and op_get_current_entity are transformed by the #[op2] macro
    // and are meant to be called from JavaScript, not directly from Rust tests.
    // Integration tests will verify these ops work correctly.
}
