// Integration test for Deno ops
use deno_core::{JsRuntime, RuntimeOptions};
use longhorn_scripting::longhorn_ops;

#[test]
fn test_op_log_from_js() {
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![longhorn_ops::init_ops()],
        ..Default::default()
    });

    // Test console.log which uses op_log
    let result = runtime.execute_script(
        "test_log",
        r#"
        Deno.core.ops.op_log("info", "Hello from JavaScript!");
        "success"
        "#,
    );

    assert!(result.is_ok());
}

#[test]
fn test_op_get_current_entity_from_js() {
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![longhorn_ops::init_ops()],
        ..Default::default()
    });

    // Test op_get_current_entity
    let result = runtime.execute_script(
        "test_entity",
        r#"
        const entityId = Deno.core.ops.op_get_current_entity();
        entityId === 0n ? "success" : "failure"
        "#,
    );

    assert!(result.is_ok());
    let scope = &mut runtime.handle_scope();
    let local = deno_core::v8::Local::new(scope, result.unwrap());
    let result_str = local.to_rust_string_lossy(scope);
    assert_eq!(result_str, "success");
}

#[test]
fn test_multiple_log_levels() {
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![longhorn_ops::init_ops()],
        ..Default::default()
    });

    // Test different log levels
    let result = runtime.execute_script(
        "test_log_levels",
        r#"
        Deno.core.ops.op_log("info", "info message");
        Deno.core.ops.op_log("warn", "warn message");
        Deno.core.ops.op_log("error", "error message");
        Deno.core.ops.op_log("debug", "debug message");
        "all_levels_logged"
        "#,
    );

    assert!(result.is_ok());
}
