// Integration test for QuickJS ops
use longhorn_scripting::LonghornJsRuntime;

#[test]
fn test_op_log_from_js() {
    let mut runtime = LonghornJsRuntime::new();

    // Test console.log which uses __longhorn_log
    let result = runtime.execute_script(
        "test_log",
        r#"
        __longhorn_log("info", "Hello from JavaScript!");
        "success"
        "#,
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

#[test]
fn test_multiple_log_levels() {
    let mut runtime = LonghornJsRuntime::new();

    // Test different log levels
    let result = runtime.execute_script(
        "test_log_levels",
        r#"
        __longhorn_log("info", "info message");
        __longhorn_log("warn", "warn message");
        __longhorn_log("error", "error message");
        __longhorn_log("debug", "debug message");
        "all_levels_logged"
        "#,
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "all_levels_logged");
}

#[test]
fn test_emit_event_from_js() {
    use longhorn_scripting::take_pending_events;

    // Clear any pending events
    take_pending_events();

    let mut runtime = LonghornJsRuntime::new();

    // Test emit_event
    let result = runtime.execute_script(
        "test_emit",
        r#"
        __longhorn_emit_event("player_died", JSON.stringify({ lives: 0 }));
        "emitted"
        "#,
    );

    assert!(result.is_ok());

    // Check that the event was queued
    let events = take_pending_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].0, "player_died");
}

#[test]
fn test_emit_to_entity_from_js() {
    use longhorn_scripting::take_pending_targeted_events;

    // Clear any pending events
    take_pending_targeted_events();

    let mut runtime = LonghornJsRuntime::new();

    // Test emit_to_entity
    let result = runtime.execute_script(
        "test_emit_to",
        r#"
        __longhorn_emit_to_entity(123, "damage", JSON.stringify({ amount: 50 }));
        "sent"
        "#,
    );

    assert!(result.is_ok());

    // Check that the event was queued
    let events = take_pending_targeted_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].0, 123); // entity_id
    assert_eq!(events[0].1, "damage"); // event_name
}

#[test]
fn test_basic_js_evaluation() {
    let mut runtime = LonghornJsRuntime::new();

    // Test basic math
    let result = runtime.execute_script("test_math", "2 + 2");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "4");

    // Test string concatenation
    let result = runtime.execute_script("test_string", "'hello' + ' ' + 'world'");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "hello world");

    // Test boolean
    let result = runtime.execute_script("test_bool", "true && false");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "false");
}

#[test]
fn test_error_handling() {
    let mut runtime = LonghornJsRuntime::new();

    // Test that errors are properly caught
    let result = runtime.execute_script("test_error", "throw new Error('test error')");
    assert!(result.is_err());
    // QuickJS error messages may differ from V8
    let error_str = result.unwrap_err().to_string();
    assert!(
        error_str.contains("test error") || error_str.contains("error"),
        "Expected error message to contain 'test error' or 'error', got: {}",
        error_str
    );
}

#[test]
fn test_class_definition() {
    let mut runtime = LonghornJsRuntime::new();

    // Test that classes work
    let result = runtime.execute_script(
        "test_class",
        r#"
        class Player {
            constructor(name) {
                this.name = name;
                this.health = 100;
            }

            takeDamage(amount) {
                this.health -= amount;
            }
        }

        const p = new Player("Hero");
        p.takeDamage(20);
        p.health
        "#,
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "80");
}
