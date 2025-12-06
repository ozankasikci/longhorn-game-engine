# Remote Control Design

## Overview

Unix domain socket server that allows AI agents to control and inspect the editor remotely. Enables automated testing, debugging, and AI-assisted development workflows.

## Architecture

```
AI Agent ──JSON──▶ Unix Socket ──Command──▶ Channel ──▶ Main Loop
                                                          │
                                                    Process command
                                                          │
AI Agent ◀──JSON── Unix Socket ◀──Response── Channel ◀───┘
```

**Components:**
- **Socket Server Thread** — Background thread listening on `/tmp/longhorn-editor.sock`
- **Command Channel** — `std::sync::mpsc` channel passing commands to main event loop
- **Response Channel** — Return channel for responses back to socket thread
- **Command Handler** — Processes commands in main loop, has access to Editor and Engine

**Threading model:**
- Main thread: winit event loop, rendering, command processing
- Socket thread: accept connection, read JSON, send to channel, wait for response
- Single client connection at a time
- Commands processed synchronously between frames (no concurrent mutation)

## Protocol

**Request format:** One JSON object per line (newline-delimited)

```json
{"action": "play"}
{"action": "get_state"}
{"action": "select_entity", "id": 123}
```

**Response format:**

```json
{"ok": true}
{"ok": true, "data": {"mode": "Play", "paused": false}}
{"ok": false, "error": "Entity not found"}
```

## Command Set

### Playback
- `play` — Enter Play mode
- `pause` — Pause the game
- `resume` — Resume from pause
- `stop` — Stop and return to Scene mode

### State Queries
- `get_state` — Returns `{mode, paused, entity_count, selected_entity}`
- `get_entities` — Returns list of `{id, name}`

### Entity Manipulation
- `select_entity {id}` — Select entity in editor
- `create_entity {name}` — Create new entity, returns `{id}`
- `delete_entity {id}` — Delete entity
- `set_property {entity, component, field, value}` — Modify component field

### UI Commands
- `toggle_console` — Open/close console panel

### Project Commands
- `load_project {path}` — Load a game project

### Utility
- `ping` — Connection test, returns `{ok: true}`

## Implementation

### New Files
- `crates/longhorn-editor/src/remote.rs` — Command/response types
- `crates/longhorn-editor/src/remote_server.rs` — Socket server thread

### Modified Files
- `crates/longhorn-editor/src/editor.rs` — Add `process_command()` method
- `crates/longhorn-editor/src/lib.rs` — Export remote modules
- `editor/src/main.rs` — Spawn server, poll command channel

### Types

```rust
pub enum RemoteCommand {
    Play, Pause, Resume, Stop,
    GetState, GetEntities,
    SelectEntity { id: u64 },
    CreateEntity { name: String },
    DeleteEntity { id: u64 },
    SetProperty { entity: u64, component: String, field: String, value: serde_json::Value },
    ToggleConsole,
    LoadProject { path: String },
    Ping,
}

pub enum RemoteResponse {
    Ok,
    State { mode: String, paused: bool, entity_count: usize, selected_entity: Option<u64> },
    Entities { list: Vec<EntityInfo> },
    Created { id: u64 },
    Error { message: String },
}
```

## Usage

```bash
# Connect and send commands
echo '{"action": "ping"}' | nc -U /tmp/longhorn-editor.sock

# Get current state
echo '{"action": "get_state"}' | nc -U /tmp/longhorn-editor.sock

# Play the game
echo '{"action": "play"}' | nc -U /tmp/longhorn-editor.sock

# List entities
echo '{"action": "get_entities"}' | nc -U /tmp/longhorn-editor.sock
```

## Socket Location

Fixed path: `/tmp/longhorn-editor.sock`

Deleted on startup if stale, deleted on clean shutdown.
