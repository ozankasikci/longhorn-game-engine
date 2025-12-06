# Remote Control Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a Unix domain socket server that allows AI agents to control and inspect the editor remotely.

**Architecture:** Background thread runs socket server on `/tmp/longhorn-editor.sock`. Commands flow through mpsc channels to main event loop. Single client, synchronous request/response, newline-delimited JSON protocol.

**Tech Stack:** Rust std library (UnixListener, mpsc channels), serde_json for serialization.

---

## Task 1: Create Remote Command Types

Define the command and response types with JSON serialization.

**Files:**
- Create: `crates/longhorn-editor/src/remote.rs`

**Step 1: Create the remote module with types**

```rust
// crates/longhorn-editor/src/remote.rs
use serde::{Deserialize, Serialize};

/// Commands that can be sent to the editor via remote control
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum RemoteCommand {
    // Playback
    Play,
    Pause,
    Resume,
    Stop,

    // State queries
    GetState,
    GetEntities,

    // Entity manipulation
    SelectEntity { id: u64 },
    CreateEntity { name: String },
    DeleteEntity { id: u64 },
    SetProperty {
        entity: u64,
        component: String,
        field: String,
        value: serde_json::Value,
    },

    // UI
    ToggleConsole,

    // Project
    LoadProject { path: String },

    // Utility
    Ping,
}

/// Information about an entity (minimal)
#[derive(Debug, Clone, Serialize)]
pub struct EntityInfo {
    pub id: u64,
    pub name: String,
}

/// Response data variants
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ResponseData {
    State {
        mode: String,
        paused: bool,
        entity_count: usize,
        selected_entity: Option<u64>,
    },
    Entities(Vec<EntityInfo>),
    Created { id: u64 },
}

/// Response sent back to the client
#[derive(Debug, Clone, Serialize)]
pub struct RemoteResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ResponseData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl RemoteResponse {
    pub fn ok() -> Self {
        Self { ok: true, data: None, error: None }
    }

    pub fn with_data(data: ResponseData) -> Self {
        Self { ok: true, data: Some(data), error: None }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self { ok: false, data: None, error: Some(message.into()) }
    }
}

/// A command with its response channel
pub struct PendingCommand {
    pub command: RemoteCommand,
    pub response_tx: std::sync::mpsc::Sender<RemoteResponse>,
}
```

**Step 2: Run cargo check**

Run: `cargo check -p longhorn-editor`
Expected: Fails because module not in lib.rs yet

**Step 3: Commit**

```bash
git add crates/longhorn-editor/src/remote.rs
git commit -m "feat(editor): add remote command and response types"
```

---

## Task 2: Export Remote Module

Add the remote module to the library exports.

**Files:**
- Modify: `crates/longhorn-editor/src/lib.rs`

**Step 1: Add module and export**

Add after line 7 (`mod console;`):

```rust
mod remote;
```

Add after line 15 (`pub use console::*;`):

```rust
pub use remote::*;
```

**Step 2: Add serde_json dependency**

In `crates/longhorn-editor/Cargo.toml`, add:

```toml
serde_json = { workspace = true }
```

**Step 3: Run cargo check**

Run: `cargo check -p longhorn-editor`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add crates/longhorn-editor/src/lib.rs crates/longhorn-editor/Cargo.toml
git commit -m "feat(editor): export remote module"
```

---

## Task 3: Create Socket Server

Implement the background thread that listens for connections.

**Files:**
- Create: `crates/longhorn-editor/src/remote_server.rs`

**Step 1: Create the server module**

```rust
// crates/longhorn-editor/src/remote_server.rs
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::path::Path;

use crate::remote::{PendingCommand, RemoteCommand, RemoteResponse};

const SOCKET_PATH: &str = "/tmp/longhorn-editor.sock";

/// Handle for the remote control server
pub struct RemoteServer {
    /// Receiver for incoming commands (polled by main loop)
    pub command_rx: Receiver<PendingCommand>,
    /// Thread handle
    _thread: thread::JoinHandle<()>,
}

impl RemoteServer {
    /// Start the remote control server in a background thread
    pub fn start() -> std::io::Result<Self> {
        // Remove stale socket file
        let socket_path = Path::new(SOCKET_PATH);
        if socket_path.exists() {
            std::fs::remove_file(socket_path)?;
        }

        // Create listener
        let listener = UnixListener::bind(SOCKET_PATH)?;
        listener.set_nonblocking(false)?;

        log::info!("Remote control server listening on {}", SOCKET_PATH);

        // Channel for commands
        let (command_tx, command_rx) = mpsc::channel();

        // Spawn server thread
        let thread = thread::spawn(move || {
            Self::server_loop(listener, command_tx);
        });

        Ok(Self {
            command_rx,
            _thread: thread,
        })
    }

    fn server_loop(listener: UnixListener, command_tx: Sender<PendingCommand>) {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = Self::handle_connection(stream, &command_tx) {
                        log::warn!("Connection error: {}", e);
                    }
                }
                Err(e) => {
                    log::error!("Accept error: {}", e);
                    break;
                }
            }
        }
    }

    fn handle_connection(
        stream: UnixStream,
        command_tx: &Sender<PendingCommand>,
    ) -> std::io::Result<()> {
        log::debug!("Remote client connected");

        let mut reader = BufReader::new(stream.try_clone()?);
        let mut writer = stream;

        loop {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line)?;

            if bytes_read == 0 {
                log::debug!("Remote client disconnected");
                break;
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            log::debug!("Remote command: {}", line);

            // Parse command
            let response = match serde_json::from_str::<RemoteCommand>(line) {
                Ok(command) => {
                    // Create response channel
                    let (response_tx, response_rx) = mpsc::channel();

                    // Send command to main loop
                    if command_tx.send(PendingCommand { command, response_tx }).is_err() {
                        RemoteResponse::error("Editor shutting down")
                    } else {
                        // Wait for response
                        match response_rx.recv() {
                            Ok(resp) => resp,
                            Err(_) => RemoteResponse::error("No response from editor"),
                        }
                    }
                }
                Err(e) => RemoteResponse::error(format!("Invalid command: {}", e)),
            };

            // Send response
            let response_json = serde_json::to_string(&response).unwrap();
            log::debug!("Remote response: {}", response_json);
            writeln!(writer, "{}", response_json)?;
            writer.flush()?;
        }

        Ok(())
    }
}

impl Drop for RemoteServer {
    fn drop(&mut self) {
        // Clean up socket file
        let _ = std::fs::remove_file(SOCKET_PATH);
        log::info!("Remote control server stopped");
    }
}
```

**Step 2: Add to lib.rs**

In `crates/longhorn-editor/src/lib.rs`, add after `mod remote;`:

```rust
mod remote_server;
```

And add export after `pub use remote::*;`:

```rust
pub use remote_server::*;
```

**Step 3: Run cargo check**

Run: `cargo check -p longhorn-editor`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add crates/longhorn-editor/src/remote_server.rs crates/longhorn-editor/src/lib.rs
git commit -m "feat(editor): add remote control socket server"
```

---

## Task 4: Add Command Handler to Editor

Add a method to Editor that processes remote commands.

**Files:**
- Modify: `crates/longhorn-editor/src/editor.rs`

**Step 1: Add imports at top of file**

Add to existing imports:

```rust
use crate::remote::{RemoteCommand, RemoteResponse, ResponseData, EntityInfo};
use longhorn_core::{Name, Transform, Sprite, World, EntityHandle};
```

**Step 2: Add process_remote_command method to Editor impl**

Add this method after `console()`:

```rust
    /// Process a remote command and return a response
    pub fn process_remote_command(
        &mut self,
        command: RemoteCommand,
        engine: &mut Engine,
    ) -> RemoteResponse {
        match command {
            RemoteCommand::Ping => {
                RemoteResponse::ok()
            }

            RemoteCommand::Play => {
                self.handle_toolbar_action(crate::ToolbarAction::Play, engine);
                RemoteResponse::ok()
            }

            RemoteCommand::Pause => {
                self.handle_toolbar_action(crate::ToolbarAction::Pause, engine);
                RemoteResponse::ok()
            }

            RemoteCommand::Resume => {
                self.handle_toolbar_action(crate::ToolbarAction::Resume, engine);
                RemoteResponse::ok()
            }

            RemoteCommand::Stop => {
                self.handle_toolbar_action(crate::ToolbarAction::Stop, engine);
                RemoteResponse::ok()
            }

            RemoteCommand::ToggleConsole => {
                self.console_open = !self.console_open;
                RemoteResponse::ok()
            }

            RemoteCommand::GetState => {
                let selected = self.state.selected_entity
                    .map(|e| e.id() as u64);
                RemoteResponse::with_data(ResponseData::State {
                    mode: format!("{:?}", self.state.mode),
                    paused: self.state.paused,
                    entity_count: engine.world().len(),
                    selected_entity: selected,
                })
            }

            RemoteCommand::GetEntities => {
                let entities: Vec<EntityInfo> = engine.world().inner().iter()
                    .map(|entity_ref| {
                        let entity = entity_ref.entity();
                        let handle = EntityHandle::new(entity);
                        let name = engine.world().get::<Name>(handle)
                            .ok()
                            .map(|n| n.0.clone())
                            .unwrap_or_else(|| format!("Entity {}", entity.id()));
                        EntityInfo {
                            id: entity.id() as u64,
                            name,
                        }
                    })
                    .collect();
                RemoteResponse::with_data(ResponseData::Entities(entities))
            }

            RemoteCommand::SelectEntity { id } => {
                // Find entity by ID
                let found = engine.world().inner().iter()
                    .find(|e| e.entity().id() as u64 == id)
                    .map(|e| e.entity());

                match found {
                    Some(entity) => {
                        self.state.select(Some(entity));
                        RemoteResponse::ok()
                    }
                    None => RemoteResponse::error(format!("Entity not found: {}", id)),
                }
            }

            RemoteCommand::CreateEntity { name } => {
                let entity = engine.world_mut()
                    .spawn()
                    .with(Name::new(&name))
                    .with(Transform::default())
                    .build();
                let id = entity.id().to_bits().get();
                log::info!("Created entity '{}' with id {}", name, id);
                RemoteResponse::with_data(ResponseData::Created { id })
            }

            RemoteCommand::DeleteEntity { id } => {
                use longhorn_core::EntityId;
                match EntityId::from_bits(id) {
                    Some(entity_id) => {
                        let handle = EntityHandle::new(entity_id);
                        if engine.world_mut().despawn(handle).is_ok() {
                            // Deselect if this was selected
                            if self.state.selected_entity.map(|e| e.id() as u64) == Some(id) {
                                self.state.select(None);
                            }
                            log::info!("Deleted entity {}", id);
                            RemoteResponse::ok()
                        } else {
                            RemoteResponse::error(format!("Entity not found: {}", id))
                        }
                    }
                    None => RemoteResponse::error(format!("Invalid entity id: {}", id)),
                }
            }

            RemoteCommand::SetProperty { entity, component, field, value } => {
                Self::set_entity_property(engine.world_mut(), entity, &component, &field, value)
            }

            RemoteCommand::LoadProject { path } => {
                match engine.load_game(&path) {
                    Ok(()) => {
                        log::info!("Loaded project: {}", path);
                        RemoteResponse::ok()
                    }
                    Err(e) => RemoteResponse::error(format!("Failed to load project: {}", e)),
                }
            }
        }
    }

    fn set_entity_property(
        world: &mut World,
        entity_id: u64,
        component: &str,
        field: &str,
        value: serde_json::Value,
    ) -> RemoteResponse {
        use longhorn_core::EntityId;

        let entity_id = match EntityId::from_bits(entity_id) {
            Some(id) => id,
            None => return RemoteResponse::error(format!("Invalid entity id: {}", entity_id)),
        };
        let handle = EntityHandle::new(entity_id);

        match component {
            "Transform" => {
                let mut transform = match world.get::<Transform>(handle) {
                    Ok(t) => (*t).clone(),
                    Err(_) => return RemoteResponse::error("Entity has no Transform"),
                };

                match field {
                    "position.x" => {
                        if let Some(v) = value.as_f64() {
                            transform.position.x = v as f32;
                        }
                    }
                    "position.y" => {
                        if let Some(v) = value.as_f64() {
                            transform.position.y = v as f32;
                        }
                    }
                    "rotation" => {
                        if let Some(v) = value.as_f64() {
                            transform.rotation = v as f32;
                        }
                    }
                    "scale.x" => {
                        if let Some(v) = value.as_f64() {
                            transform.scale.x = v as f32;
                        }
                    }
                    "scale.y" => {
                        if let Some(v) = value.as_f64() {
                            transform.scale.y = v as f32;
                        }
                    }
                    _ => return RemoteResponse::error(format!("Unknown field: {}", field)),
                }

                if world.set(handle, transform).is_err() {
                    return RemoteResponse::error("Failed to set Transform");
                }
                RemoteResponse::ok()
            }
            "Name" => {
                if field == "name" || field == "0" {
                    if let Some(s) = value.as_str() {
                        if world.set(handle, Name::new(s)).is_err() {
                            return RemoteResponse::error("Failed to set Name");
                        }
                        return RemoteResponse::ok();
                    }
                }
                RemoteResponse::error(format!("Invalid Name field: {}", field))
            }
            _ => RemoteResponse::error(format!("Unknown component: {}", component)),
        }
    }
```

**Step 3: Run cargo check**

Run: `cargo check -p longhorn-editor`
Expected: Compiles (may have warnings about unused imports)

**Step 4: Commit**

```bash
git add crates/longhorn-editor/src/editor.rs
git commit -m "feat(editor): add remote command handler"
```

---

## Task 5: Integrate Server into Main Loop

Start the server and poll for commands in the event loop.

**Files:**
- Modify: `editor/src/main.rs`

**Step 1: Add import**

Add to imports at top:

```rust
use longhorn_editor::RemoteServer;
```

**Step 2: Add remote_server field to EditorApp**

In the `EditorApp` struct, add:

```rust
    remote_server: Option<RemoteServer>,
```

**Step 3: Initialize server in EditorApp::new()**

At the end of `EditorApp::new()`, before the `Self { ... }` block, add:

```rust
        // Start remote control server
        let remote_server = match RemoteServer::start() {
            Ok(server) => Some(server),
            Err(e) => {
                log::warn!("Failed to start remote server: {}", e);
                None
            }
        };
```

And add to the struct initialization:

```rust
            remote_server,
```

**Step 4: Poll commands in render()**

At the start of the `render()` method, after the initial `let Some(...)` checks, add:

```rust
        // Process remote commands
        if let Some(ref server) = self.remote_server {
            while let Ok(pending) = server.command_rx.try_recv() {
                let response = self.editor.process_remote_command(pending.command, &mut self.engine);
                let _ = pending.response_tx.send(response);
            }
        }
```

**Step 5: Run cargo check**

Run: `cargo check -p editor`
Expected: Compiles successfully

**Step 6: Commit**

```bash
git add editor/src/main.rs
git commit -m "feat(editor): integrate remote server into main loop"
```

---

## Task 6: Test End-to-End

Verify the remote control system works.

**Step 1: Build and run editor**

Run: `cargo run -p editor`
Expected: Editor opens, log shows "Remote control server listening on /tmp/longhorn-editor.sock"

**Step 2: Test ping command**

In another terminal:
```bash
echo '{"action": "ping"}' | nc -U /tmp/longhorn-editor.sock
```
Expected: `{"ok":true}`

**Step 3: Test get_state**

```bash
echo '{"action": "get_state"}' | nc -U /tmp/longhorn-editor.sock
```
Expected: `{"ok":true,"data":{"mode":"Scene","paused":false,"entity_count":2,"selected_entity":null}}`

**Step 4: Test get_entities**

```bash
echo '{"action": "get_entities"}' | nc -U /tmp/longhorn-editor.sock
```
Expected: `{"ok":true,"data":[{"id":...,"name":"Player"},{"id":...,"name":"Enemy"}]}`

**Step 5: Test play/stop**

```bash
echo '{"action": "play"}' | nc -U /tmp/longhorn-editor.sock
echo '{"action": "get_state"}' | nc -U /tmp/longhorn-editor.sock
echo '{"action": "stop"}' | nc -U /tmp/longhorn-editor.sock
```
Expected: Mode changes to "Play" then back to "Scene"

**Step 6: Test create_entity**

```bash
echo '{"action": "create_entity", "name": "TestEntity"}' | nc -U /tmp/longhorn-editor.sock
echo '{"action": "get_entities"}' | nc -U /tmp/longhorn-editor.sock
```
Expected: New entity created and appears in list

**Step 7: Commit any fixes**

```bash
git add -A
git commit -m "fix(editor): address remote control integration issues"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Remote command types | `remote.rs` |
| 2 | Export remote module | `lib.rs`, `Cargo.toml` |
| 3 | Socket server thread | `remote_server.rs` |
| 4 | Command handler | `editor.rs` |
| 5 | Main loop integration | `main.rs` |
| 6 | End-to-end testing | - |
