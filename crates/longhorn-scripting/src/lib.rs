mod compiler;
mod js_runtime;
mod ops;
mod runtime;

pub use compiler::*;
pub use js_runtime::*;
pub use ops::*;
pub use runtime::*;

/// Embedded bootstrap JavaScript code
pub const BOOTSTRAP_JS: &str = include_str!("bootstrap.js");
