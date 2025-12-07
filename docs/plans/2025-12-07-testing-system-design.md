# Testing System Design

## Overview

A comprehensive testing system for the Longhorn Editor that supports:
- Automated CI tests (unit, integration, E2E)
- Interactive AI-driven exploration and debugging
- Full GUI with screenshot capture
- State queries, log access, and frame synchronization

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Test Infrastructure                       │
├─────────────────────────────────────────────────────────────┤
│  1. longhorn-test-client (Rust crate)                       │
│     - EditorClient: Unix socket connection + JSON protocol  │
│     - Typed command builders for all 50+ remote commands    │
│     - Screenshot capture & comparison utilities             │
│     - Log file reader/parser                                │
│     - Assertion helpers (state matches, entity exists, etc) │
│                                                             │
│  2. Test Harness (cargo test integration)                   │
│     - Spawns editor process with test project               │
│     - Waits for socket to be ready                          │
│     - Provides EditorClient to each test                    │
│     - Captures screenshots to test_output/ directory        │
│     - Cleans up editor process after tests                  │
│                                                             │
│  3. New Remote Commands (editor-side)                       │
│     - take_screenshot: Captures window to PNG file          │
│     - get_log_tail: Returns recent log entries              │
│     - wait_frames: Advances N frames before responding      │
└─────────────────────────────────────────────────────────────┘
```

## EditorClient API

```rust
// Connection & lifecycle
let client = EditorClient::connect("/tmp/longhorn-editor.sock")?;
client.ping()?;

// State queries
let state = client.get_state()?;
let entities = client.get_entities()?;
let entity = client.get_entity(id)?;
let ui = client.get_ui_state()?;

// Playback control
client.play()?;
client.pause()?;
client.resume()?;
client.stop()?;

// Entity operations
let id = client.create_entity("Player")?;
client.select_entity(id)?;
client.delete_entity(id)?;
client.set_property(id, "Transform", "position.x", 100.0)?;

// UI automation
client.focus_panel("inspector")?;
client.click_element("add_entity")?;
client.expand_tree_node("Player")?;

// Observation
client.take_screenshot("test_output/step1.png")?;
let logs = client.get_log_tail(50)?;
client.wait_frames(5)?;
```

## Test Harness

```rust
pub struct TestHarness {
    editor_process: Child,
    client: EditorClient,
    test_output_dir: PathBuf,
}

impl TestHarness {
    pub fn start() -> Result<Self, TestError>;
    pub fn start_with_project(path: &Path) -> Result<Self, TestError>;
    pub fn client(&self) -> &EditorClient;
    pub fn screenshot(&self, step: &str) -> Result<PathBuf, TestError>;
    pub fn logs(&self, lines: usize) -> Result<Vec<LogEntry>, TestError>;
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        // Graceful shutdown, capture final state on failure
    }
}
```

## New Remote Commands

### TakeScreenshot
```json
{"action": "take_screenshot", "path": "/path/to/output.png"}
→ {"ok": true, "data": {"path": "/abs/path.png", "width": 1280, "height": 720}}
```

### GetLogTail
```json
{"action": "get_log_tail", "lines": 50}
→ {"ok": true, "data": {"entries": [
    {"timestamp": "2024-01-15T10:30:00", "level": "INFO", "message": "..."}
  ]}}
```

### WaitFrames
```json
{"action": "wait_frames", "count": 5}
→ {"ok": true, "data": {"frames_waited": 5}}
```

## Directory Structure

```
crates/
  longhorn-test-client/
    src/
      lib.rs           # Main exports
      client.rs        # EditorClient implementation
      commands.rs      # Typed command builders
      responses.rs     # Response types
      error.rs         # Error types
    Cargo.toml

tests/
  fixtures/
    empty_project/     # Minimal valid project
    complex_scene/     # Many entities, hierarchies
    broken_script/     # Intentionally bad script
  common/
    mod.rs             # TestHarness, helpers
  integration/
    mod.rs
    state_tests.rs     # Play/pause/stop transitions
    entity_tests.rs    # CRUD operations
    asset_tests.rs     # Loading, textures
    ui_tests.rs        # Panel focus, clicks
  e2e/
    mod.rs
    workflow_tests.rs  # Full editing workflows
```

## Example Tests

```rust
#[test]
fn test_play_pause_stop_cycle() {
    let harness = TestHarness::start().unwrap();
    let client = harness.client();

    let state = client.get_state().unwrap();
    assert_eq!(state.mode, EditorMode::Scene);

    client.play().unwrap();
    client.wait_frames(2).unwrap();
    let state = client.get_state().unwrap();
    assert_eq!(state.mode, EditorMode::Play);

    client.pause().unwrap();
    assert!(client.get_state().unwrap().paused);

    client.stop().unwrap();
    assert_eq!(client.get_state().unwrap().mode, EditorMode::Scene);
}

#[test]
fn test_entity_creation_and_modification() {
    let harness = TestHarness::start().unwrap();
    let client = harness.client();

    let initial_count = client.get_entities().unwrap().len();
    let id = client.create_entity("TestSprite").unwrap();
    assert_eq!(client.get_entities().unwrap().len(), initial_count + 1);

    client.set_property(id, "Transform", "position.x", 150.0).unwrap();
    let entity = client.get_entity(id).unwrap();
    assert_eq!(entity.transform.position.x, 150.0);

    harness.screenshot("after_move").unwrap();

    client.delete_entity(id).unwrap();
    assert_eq!(client.get_entities().unwrap().len(), initial_count);
}
```

## Implementation Order

1. Create `longhorn-test-client` crate with basic EditorClient
2. Add new remote commands to editor (take_screenshot, get_log_tail, wait_frames)
3. Build TestHarness for process management
4. Create test fixtures
5. Write integration tests for each area
6. Add E2E workflow tests
