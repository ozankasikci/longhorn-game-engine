mod compiler;
mod js_runtime;
mod ops;
mod runtime;

pub use compiler::*;
pub use js_runtime::*;
pub use ops::{set_console_callback, ConsoleCallback, JsSelf, JsSprite, JsTransform, JsVec2, OpsState};
pub use runtime::*;

/// Embedded bootstrap JavaScript code
pub const BOOTSTRAP_JS: &str = include_str!("bootstrap.js");
