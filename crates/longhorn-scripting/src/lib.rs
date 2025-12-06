mod compiler;
mod js_runtime;
mod ops;
mod runtime;

pub use compiler::*;
pub use js_runtime::*;
pub use ops::{set_console_callback, take_pending_events, take_pending_targeted_events, ConsoleCallback, JsSelf, JsSprite, JsTransform, JsVec2, OpsState};
pub use runtime::*;

/// Re-export the extension module for integration tests
pub use ops::longhorn_ops;

/// Embedded bootstrap JavaScript code
pub const BOOTSTRAP_JS: &str = include_str!("bootstrap.js");
