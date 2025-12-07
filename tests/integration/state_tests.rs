//! State-related integration tests.
//!
//! Tests for editor state transitions (play/pause/stop), mode changes, etc.

use longhorn_test_client::EditorClient;

/// Test that we can connect to the editor and ping it.
#[test]
#[ignore]
fn test_connect_and_ping() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");
    client.ping().expect("Ping failed");
}

/// Test getting the editor state.
#[test]
#[ignore]
fn test_get_state() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    let state = client.get_state().expect("Failed to get state");

    // Initial state should be Scene mode
    assert_eq!(state.mode, "Scene");
    assert!(!state.paused);
}

/// Test play/pause/stop cycle.
#[test]
#[ignore]
fn test_play_pause_stop_cycle() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    // Initial state: Scene mode
    let state = client.get_state().expect("Failed to get state");
    assert_eq!(state.mode, "Scene");

    // Enter play mode
    client.play().expect("Play failed");
    let state = client.get_state().expect("Failed to get state");
    assert_eq!(state.mode, "Play");
    assert!(!state.paused);

    // Pause
    client.pause().expect("Pause failed");
    let state = client.get_state().expect("Failed to get state");
    assert_eq!(state.mode, "Play");
    assert!(state.paused);

    // Resume
    client.resume().expect("Resume failed");
    let state = client.get_state().expect("Failed to get state");
    assert_eq!(state.mode, "Play");
    assert!(!state.paused);

    // Stop returns to scene mode
    client.stop().expect("Stop failed");
    let state = client.get_state().expect("Failed to get state");
    assert_eq!(state.mode, "Scene");
}

/// Test getting log tail.
#[test]
#[ignore]
fn test_get_log_tail() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    let logs = client.get_log_tail(10).expect("Failed to get log tail");

    // Should get some log entries (may be empty if log file doesn't exist)
    println!("Got {} log entries", logs.entries.len());
    for entry in &logs.entries {
        println!("[{}] {}: {}", entry.timestamp, entry.level, entry.message);
    }
}

/// Test getting UI state.
#[test]
#[ignore]
fn test_get_ui_state() {
    let mut client = EditorClient::connect_default().expect("Failed to connect to editor");

    let ui_state = client.get_ui_state().expect("Failed to get UI state");

    // Should have some panels
    assert!(!ui_state.panels.is_empty(), "Expected some panels to be registered");

    println!("Focused panel: {:?}", ui_state.focused_panel);
    println!("Panels: {:?}", ui_state.panels.len());
    for panel in &ui_state.panels {
        println!("  - {} ({})", panel.title, panel.id);
    }

    println!("Clickable elements: {:?}", ui_state.clickable_elements.len());
}
