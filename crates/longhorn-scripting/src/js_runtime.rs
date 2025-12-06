// crates/longhorn-scripting/src/js_runtime.rs
//! QuickJS JavaScript runtime wrapper using rquickjs

use rquickjs::{Context, Function, Runtime, Value};

use crate::ops::{get_console_callback, push_pending_event, push_pending_targeted_event};

/// Wrapper around rquickjs Runtime and Context
pub struct LonghornJsRuntime {
    runtime: Runtime,
    context: Context,
}

impl LonghornJsRuntime {
    /// Create a new JavaScript runtime with Longhorn ops
    pub fn new() -> Self {
        let runtime = Runtime::new().expect("Failed to create QuickJS runtime");
        let context = Context::full(&runtime).expect("Failed to create QuickJS context");

        let mut instance = Self { runtime, context };
        instance.register_ops();
        instance
    }

    /// Register Longhorn ops as global functions
    fn register_ops(&mut self) {
        self.context.with(|ctx| {
            let globals = ctx.globals();

            // Register __longhorn_log(level, message)
            let log_fn = Function::new(ctx.clone(), |level: String, message: String| {
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
            })
            .expect("Failed to create log function");
            globals
                .set("__longhorn_log", log_fn)
                .expect("Failed to register __longhorn_log");

            // Register __longhorn_emit_event(event_name, data_json)
            let emit_fn = Function::new(ctx.clone(), |event_name: String, data_json: String| {
                let data: serde_json::Value =
                    serde_json::from_str(&data_json).unwrap_or(serde_json::Value::Null);
                push_pending_event(event_name, data);
            })
            .expect("Failed to create emit_event function");
            globals
                .set("__longhorn_emit_event", emit_fn)
                .expect("Failed to register __longhorn_emit_event");

            // Register __longhorn_emit_to_entity(entity_id, event_name, data_json)
            let emit_to_fn =
                Function::new(ctx.clone(), |entity_id: u64, event_name: String, data_json: String| {
                    let data: serde_json::Value =
                        serde_json::from_str(&data_json).unwrap_or(serde_json::Value::Null);
                    push_pending_targeted_event(entity_id, event_name, data);
                })
                .expect("Failed to create emit_to_entity function");
            globals
                .set("__longhorn_emit_to_entity", emit_to_fn)
                .expect("Failed to register __longhorn_emit_to_entity");
        });
    }

    /// Execute JavaScript code and return the result as a string
    pub fn execute_script(&mut self, name: &str, code: &str) -> Result<String, JsRuntimeError> {
        self.context.with(|ctx| {
            match ctx.eval::<Value, _>(code) {
                Ok(value) => {
                    // Convert result to string
                    let result_str = value_to_string(&value);
                    Ok(result_str)
                }
                Err(e) => Err(JsRuntimeError::Execution(format!(
                    "{}: {}",
                    name,
                    e.to_string()
                ))),
            }
        })
    }

    /// Get memory usage stats
    pub fn memory_usage(&self) -> MemoryUsage {
        // QuickJS doesn't expose detailed memory stats easily,
        // but we can return basic info
        MemoryUsage {
            used_bytes: 0, // Would need runtime.compute_memory_info()
            limit_bytes: 0,
        }
    }
}

impl Default for LonghornJsRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a QuickJS Value to a Rust String
fn value_to_string(value: &Value) -> String {
    if value.is_undefined() {
        "undefined".to_string()
    } else if value.is_null() {
        "null".to_string()
    } else if let Some(b) = value.as_bool() {
        b.to_string()
    } else if let Some(n) = value.as_int() {
        n.to_string()
    } else if let Some(n) = value.as_float() {
        n.to_string()
    } else if let Some(s) = value.as_string() {
        s.to_string().unwrap_or_default()
    } else {
        // For objects/arrays, try to get JSON representation
        "[object]".to_string()
    }
}

#[derive(Debug, Clone)]
pub struct MemoryUsage {
    pub used_bytes: usize,
    pub limit_bytes: usize,
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
        let result = runtime
            .execute_script("test", "'hello' + ' world'")
            .unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_execute_error() {
        let mut runtime = LonghornJsRuntime::new();
        let result = runtime.execute_script("test", "throw new Error('test error')");
        assert!(result.is_err());
    }

    #[test]
    fn test_console_log() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        crate::ops::set_console_callback(Some(Arc::new(move |level, msg| {
            called_clone.store(true, Ordering::SeqCst);
            assert_eq!(level, "info");
            assert_eq!(msg, "test message");
        })));

        let mut runtime = LonghornJsRuntime::new();
        runtime
            .execute_script("test", r#"__longhorn_log("info", "test message")"#)
            .unwrap();

        assert!(called.load(Ordering::SeqCst));
        crate::ops::set_console_callback(None);
    }
}
