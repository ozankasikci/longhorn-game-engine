// crates/longhorn-scripting/src/js_runtime.rs
use deno_core::{FastString, JsRuntime, RuntimeOptions};

/// Wrapper around deno_core::JsRuntime
pub struct LonghornJsRuntime {
    runtime: JsRuntime,
}

impl LonghornJsRuntime {
    /// Create a new JavaScript runtime
    pub fn new() -> Self {
        let runtime = JsRuntime::new(RuntimeOptions {
            ..Default::default()
        });

        Self { runtime }
    }

    /// Execute JavaScript code and return the result as a string
    pub fn execute_script(&mut self, name: &'static str, code: &str) -> Result<String, JsRuntimeError> {
        let result = self.runtime.execute_script(
            name,
            FastString::from(code.to_string())
        );

        match result {
            Ok(global) => {
                let scope = &mut self.runtime.handle_scope();
                let local = deno_core::v8::Local::new(scope, global);
                let result_str = local.to_rust_string_lossy(scope);
                Ok(result_str)
            }
            Err(e) => Err(JsRuntimeError::Execution(e.to_string())),
        }
    }

    /// Get mutable reference to inner runtime (for advanced ops)
    pub fn inner_mut(&mut self) -> &mut JsRuntime {
        &mut self.runtime
    }
}

impl Default for LonghornJsRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JsRuntimeError {
    #[error("JavaScript execution error: {0}")]
    Execution(String),

    #[error("Script compilation error: {0}")]
    Compilation(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let _runtime = LonghornJsRuntime::new();
        // Just verify it doesn't panic
    }

    #[test]
    fn test_execute_simple_script() {
        let mut runtime = LonghornJsRuntime::new();
        let result = runtime.execute_script("test", "1 + 2").unwrap();
        assert_eq!(result, "3");
    }

    #[test]
    fn test_execute_string_script() {
        let mut runtime = LonghornJsRuntime::new();
        let result = runtime.execute_script("test", "'hello' + ' world'").unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_execute_error() {
        let mut runtime = LonghornJsRuntime::new();
        let result = runtime.execute_script("test", "throw new Error('test error')");
        assert!(result.is_err());
    }
}
