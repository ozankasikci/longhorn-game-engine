// crates/longhorn-scripting/src/ops.rs
use deno_core::op2;
use serde::{Deserialize, Serialize};

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
pub struct JsSprite {
    pub texture: u64,
    pub size: JsVec2,
    pub color: [f64; 4],
    pub flip_x: bool,
    pub flip_y: bool,
}

// Note: In a full implementation, these ops would access the actual World.
// For now, we define the interface. The ScriptRuntime will inject the World
// via op state before calling scripts.

#[op2(fast)]
pub fn op_log(#[string] level: String, #[string] message: String) {
    match level.as_str() {
        "error" => log::error!("[Script] {}", message),
        "warn" => log::warn!("[Script] {}", message),
        "info" => log::info!("[Script] {}", message),
        "debug" => log::debug!("[Script] {}", message),
        _ => log::info!("[Script] {}", message),
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
