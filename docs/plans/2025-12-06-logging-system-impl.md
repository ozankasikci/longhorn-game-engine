# Logging System Implementation Plan

> **Note:** This implementation plan references Deno Core. The implementation uses **rquickjs (QuickJS)** instead.

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a console panel for script output and session-based file logging for debugging.

**Architecture:** Scripts call `console.log/warn/error` which writes to a shared buffer (displayed in UI) and to the file logger. Engine/editor logs go to file only. Console panel is a collapsible bottom drawer toggled via toolbar button.

**Tech Stack:** Rust, egui, tracing + tracing-subscriber for file logging, rquickjs for script runtime.

---

## Task 1: Add ScriptConsole Buffer

Create a shared buffer that holds console entries from scripts.

**Files:**
- Create: `crates/longhorn-editor/src/console.rs`

**Step 1: Create the console module**

```rust
// crates/longhorn-editor/src/console.rs
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Log level for console entries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleLevel {
    Log,
    Warn,
    Error,
}

/// A single console entry
#[derive(Debug, Clone)]
pub struct ConsoleEntry {
    pub level: ConsoleLevel,
    pub message: String,
    pub timestamp: Instant,
}

/// Maximum number of console entries to retain
const MAX_ENTRIES: usize = 1000;

/// Shared console buffer for script output
#[derive(Clone)]
pub struct ScriptConsole {
    entries: Arc<Mutex<Vec<ConsoleEntry>>>,
}

impl ScriptConsole {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a log entry
    pub fn log(&self, message: String) {
        self.push(ConsoleLevel::Log, message);
    }

    /// Add a warning entry
    pub fn warn(&self, message: String) {
        self.push(ConsoleLevel::Warn, message);
    }

    /// Add an error entry
    pub fn error(&self, message: String) {
        self.push(ConsoleLevel::Error, message);
    }

    fn push(&self, level: ConsoleLevel, message: String) {
        let mut entries = self.entries.lock().unwrap();

        // Drop oldest entries if at capacity
        if entries.len() >= MAX_ENTRIES {
            entries.remove(0);
        }

        entries.push(ConsoleEntry {
            level,
            message,
            timestamp: Instant::now(),
        });

        // Also log to file via log crate
        match level {
            ConsoleLevel::Log => log::info!(target: "script", "{}", entries.last().unwrap().message),
            ConsoleLevel::Warn => log::warn!(target: "script", "{}", entries.last().unwrap().message),
            ConsoleLevel::Error => log::error!(target: "script", "{}", entries.last().unwrap().message),
        }
    }

    /// Get all entries (for UI display)
    pub fn entries(&self) -> Vec<ConsoleEntry> {
        self.entries.lock().unwrap().clone()
    }

    /// Clear all entries
    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.lock().unwrap().is_empty()
    }
}

impl Default for ScriptConsole {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: Run `cargo check` to verify it compiles**

Run: `cargo check -p longhorn-editor`
Expected: Compilation succeeds (file not yet integrated)

**Step 3: Commit**

```bash
git add crates/longhorn-editor/src/console.rs
git commit -m "feat(editor): add ScriptConsole buffer for script output"
```

---

## Task 2: Create ConsolePanel UI

Create the console panel that displays script output in a collapsible bottom drawer.

**Files:**
- Create: `crates/longhorn-editor/src/panels/console.rs`
- Modify: `crates/longhorn-editor/src/panels/mod.rs`

**Step 1: Create the console panel**

```rust
// crates/longhorn-editor/src/panels/console.rs
use egui::{Color32, RichText, ScrollArea, Ui};
use crate::console::{ConsoleLevel, ScriptConsole};

/// Console panel showing script output
pub struct ConsolePanel {
    /// Whether auto-scroll is enabled
    auto_scroll: bool,
}

impl ConsolePanel {
    pub fn new() -> Self {
        Self {
            auto_scroll: true,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, console: &ScriptConsole) {
        // Header row with title and clear button
        ui.horizontal(|ui| {
            ui.heading("Console");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Clear").clicked() {
                    console.clear();
                }
            });
        });

        ui.separator();

        // Scrollable log area
        let entries = console.entries();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(self.auto_scroll)
            .show(ui, |ui| {
                for entry in &entries {
                    let (prefix, color) = match entry.level {
                        ConsoleLevel::Log => ("", Color32::GRAY),
                        ConsoleLevel::Warn => ("⚠ ", Color32::YELLOW),
                        ConsoleLevel::Error => ("✖ ", Color32::from_rgb(255, 100, 100)),
                    };

                    ui.label(RichText::new(format!("{}{}", prefix, entry.message)).color(color));
                }

                if entries.is_empty() {
                    ui.label(RichText::new("No console output").color(Color32::DARK_GRAY));
                }
            });
    }
}

impl Default for ConsolePanel {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: Add console to panels/mod.rs**

In `crates/longhorn-editor/src/panels/mod.rs`, add:

```rust
mod scene_tree;
mod inspector;
mod viewport;
mod console;

pub use scene_tree::*;
pub use inspector::*;
pub use viewport::*;
pub use console::*;
```

**Step 3: Run `cargo check` to verify**

Run: `cargo check -p longhorn-editor`
Expected: Compilation succeeds

**Step 4: Commit**

```bash
git add crates/longhorn-editor/src/panels/console.rs crates/longhorn-editor/src/panels/mod.rs
git commit -m "feat(editor): add ConsolePanel UI component"
```

---

## Task 3: Export Console Module from Library

Export the console module from the longhorn-editor library.

**Files:**
- Modify: `crates/longhorn-editor/src/lib.rs`

**Step 1: Add console module and export**

In `crates/longhorn-editor/src/lib.rs`, add:

```rust
mod state;
mod toolbar;
mod snapshot;
mod viewport_renderer;
mod panels;
mod editor;
mod console;

pub use state::*;
pub use toolbar::*;
pub use snapshot::*;
pub use viewport_renderer::*;
pub use panels::*;
pub use editor::*;
pub use console::*;
```

**Step 2: Run `cargo check`**

Run: `cargo check -p longhorn-editor`
Expected: Compilation succeeds

**Step 3: Commit**

```bash
git add crates/longhorn-editor/src/lib.rs
git commit -m "feat(editor): export console module"
```

---

## Task 4: Integrate Console into Editor

Add the console panel to the Editor struct and wire up the toolbar button.

**Files:**
- Modify: `crates/longhorn-editor/src/editor.rs`
- Modify: `crates/longhorn-editor/src/toolbar.rs`

**Step 1: Add console state to Editor struct**

In `crates/longhorn-editor/src/editor.rs`, update imports and struct:

```rust
use egui::Context;
use longhorn_engine::Engine;
use crate::{EditorState, EditorMode, SceneTreePanel, InspectorPanel, ViewportPanel, Toolbar, ToolbarAction, SceneSnapshot, ConsolePanel, ScriptConsole};

pub struct Editor {
    state: EditorState,
    scene_tree: SceneTreePanel,
    inspector: InspectorPanel,
    viewport: ViewportPanel,
    toolbar: Toolbar,
    scene_snapshot: Option<SceneSnapshot>,
    console_panel: ConsolePanel,
    console: ScriptConsole,
    console_open: bool,
}
```

**Step 2: Initialize console in Editor::new()**

```rust
impl Editor {
    pub fn new() -> Self {
        Self {
            state: EditorState::new(),
            scene_tree: SceneTreePanel::new(),
            inspector: InspectorPanel::new(),
            viewport: ViewportPanel::new(),
            toolbar: Toolbar::new(),
            scene_snapshot: None,
            console_panel: ConsolePanel::new(),
            console: ScriptConsole::new(),
            console_open: false,
        }
    }

    // Add getter for ScriptConsole (needed by script runtime)
    pub fn console(&self) -> &ScriptConsole {
        &self.console
    }
```

**Step 3: Update toolbar.rs to add Console button and action**

In `crates/longhorn-editor/src/toolbar.rs`, add `ToggleConsole` action:

```rust
/// Actions that can be triggered from the toolbar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    None,
    Play,
    Pause,
    Resume,
    Stop,
    ToggleConsole,
}
```

And add the button in `show()` after the mode indicator:

```rust
            ui.separator();

            // Mode indicator
            let mode_text = match (state.mode, state.paused) {
                (EditorMode::Scene, _) => "Scene Mode",
                (EditorMode::Play, false) => "Playing",
                (EditorMode::Play, true) => "Paused",
            };
            ui.label(mode_text);

            // Spacer to push console button to the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Console").clicked() {
                    action = ToolbarAction::ToggleConsole;
                }
            });
```

**Step 4: Handle ToggleConsole in Editor::show()**

In `Editor::show()`, add handling for the toggle action and render the console panel:

```rust
    pub fn show(&mut self, ctx: &Context, engine: &mut Engine, viewport_texture: Option<egui::TextureId>) -> bool {
        let mut should_exit = false;
        let mut toolbar_action = ToolbarAction::None;

        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            // ... existing menu code ...
        });

        // Toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            toolbar_action = self.toolbar.show(ui, &self.state);
        });

        // Handle toolbar action
        match toolbar_action {
            ToolbarAction::ToggleConsole => {
                self.console_open = !self.console_open;
            }
            _ => self.handle_toolbar_action(toolbar_action, engine),
        }

        // Console panel (bottom, collapsible)
        if self.console_open {
            egui::TopBottomPanel::bottom("console")
                .resizable(true)
                .default_height(150.0)
                .min_height(100.0)
                .max_height(400.0)
                .show(ctx, |ui| {
                    self.console_panel.show(ui, &self.console);
                });
        }

        // Left panel - Scene Tree
        // ... rest of existing code ...
```

**Step 5: Run `cargo check`**

Run: `cargo check -p longhorn-editor`
Expected: Compilation succeeds

**Step 6: Commit**

```bash
git add crates/longhorn-editor/src/editor.rs crates/longhorn-editor/src/toolbar.rs
git commit -m "feat(editor): integrate console panel with toolbar toggle"
```

---

## Task 5: Wire ScriptConsole to Script Runtime

Connect the console buffer to the Deno script runtime so `console.log()` from scripts writes to the buffer.

**Files:**
- Modify: `crates/longhorn-scripting/src/ops.rs`
- Modify: `crates/longhorn-scripting/src/js_runtime.rs`
- Modify: `crates/longhorn-scripting/src/runtime.rs`
- Modify: `crates/longhorn-scripting/src/lib.rs`
- Modify: `crates/longhorn-scripting/Cargo.toml`

**Step 1: Add longhorn-editor dependency to longhorn-scripting**

In `crates/longhorn-scripting/Cargo.toml`, add:

```toml
longhorn-editor = { workspace = true, optional = true }

[features]
default = []
editor = ["longhorn-editor"]
```

**Note:** We'll use a different approach - pass a callback instead of direct dependency to avoid circular deps.

**Step 1 (revised): Create ConsoleCallback type in ops.rs**

In `crates/longhorn-scripting/src/ops.rs`, add a type-erased callback:

```rust
use std::sync::Arc;

/// Callback for console output (type-erased to avoid editor dependency)
pub type ConsoleCallback = Arc<dyn Fn(&str, &str) + Send + Sync>;

/// Thread-local console callback
thread_local! {
    static CONSOLE_CALLBACK: std::cell::RefCell<Option<ConsoleCallback>> = std::cell::RefCell::new(None);
}

/// Set the console callback for the current thread
pub fn set_console_callback(callback: Option<ConsoleCallback>) {
    CONSOLE_CALLBACK.with(|cb| {
        *cb.borrow_mut() = callback;
    });
}

/// Get the console callback for the current thread
fn get_console_callback() -> Option<ConsoleCallback> {
    CONSOLE_CALLBACK.with(|cb| cb.borrow().clone())
}
```

**Step 2: Update op_log to use callback**

In `crates/longhorn-scripting/src/ops.rs`, update `op_log`:

```rust
#[op2(fast)]
pub fn op_log(#[string] level: String, #[string] message: String) {
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
}
```

**Step 3: Export set_console_callback from lib.rs**

In `crates/longhorn-scripting/src/lib.rs`:

```rust
mod compiler;
mod js_runtime;
mod ops;
mod runtime;

pub use compiler::*;
pub use js_runtime::*;
pub use ops::{set_console_callback, ConsoleCallback};
pub use runtime::*;

/// Embedded bootstrap JavaScript code
pub const BOOTSTRAP_JS: &str = include_str!("bootstrap.js");
```

**Step 4: Run `cargo check`**

Run: `cargo check -p longhorn-scripting`
Expected: Compilation succeeds

**Step 5: Commit**

```bash
git add crates/longhorn-scripting/src/ops.rs crates/longhorn-scripting/src/lib.rs
git commit -m "feat(scripting): add console callback for editor integration"
```

---

## Task 6: Connect Editor Console to Script Runtime

Wire the Editor's ScriptConsole to the script runtime callback.

**Files:**
- Modify: `crates/longhorn-editor/src/editor.rs`
- Modify: `crates/longhorn-editor/Cargo.toml`

**Step 1: Add longhorn-scripting dependency to longhorn-editor**

In `crates/longhorn-editor/Cargo.toml`, add:

```toml
longhorn-scripting = { workspace = true }
```

**Step 2: Set up console callback in Editor**

In `crates/longhorn-editor/src/editor.rs`, update imports:

```rust
use longhorn_scripting::set_console_callback;
use std::sync::Arc;
```

Add a method to set up the callback:

```rust
impl Editor {
    pub fn new() -> Self {
        let console = ScriptConsole::new();

        // Set up console callback for script runtime
        let console_clone = console.clone();
        set_console_callback(Some(Arc::new(move |level: &str, message: &str| {
            match level {
                "error" => console_clone.error(message.to_string()),
                "warn" => console_clone.warn(message.to_string()),
                _ => console_clone.log(message.to_string()),
            }
        })));

        Self {
            state: EditorState::new(),
            scene_tree: SceneTreePanel::new(),
            inspector: InspectorPanel::new(),
            viewport: ViewportPanel::new(),
            toolbar: Toolbar::new(),
            scene_snapshot: None,
            console_panel: ConsolePanel::new(),
            console,
            console_open: false,
        }
    }
```

**Step 3: Run `cargo check`**

Run: `cargo check -p longhorn-editor`
Expected: Compilation succeeds

**Step 4: Commit**

```bash
git add crates/longhorn-editor/Cargo.toml crates/longhorn-editor/src/editor.rs
git commit -m "feat(editor): wire console to script runtime callback"
```

---

## Task 7: Add File Logging with Tracing

Set up session-based file logging using tracing.

**Files:**
- Modify: `editor/Cargo.toml`
- Modify: `editor/src/main.rs`

**Step 1: Add tracing dependencies to editor/Cargo.toml**

```toml
# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"
tracing-log = "0.2"
```

Remove `env_logger` from dependencies.

**Step 2: Set up tracing in main.rs**

Replace `env_logger::init()` with tracing setup:

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use std::path::PathBuf;

fn setup_logging(project_path: Option<&PathBuf>) {
    // Create env filter (respects RUST_LOG env var)
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Console layer (stderr)
    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false);

    // Build subscriber
    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer);

    // Add file layer if project path provided
    if let Some(path) = project_path {
        let logs_dir = path.join("logs");
        std::fs::create_dir_all(&logs_dir).ok();

        let file_appender = RollingFileAppender::builder()
            .rotation(Rotation::NEVER)
            .filename_prefix("editor")
            .filename_suffix("log")
            .build(&logs_dir)
            .expect("Failed to create log file");

        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(file_appender)
            .with_ansi(false)
            .with_target(true);

        subscriber.with(file_layer).init();
    } else {
        subscriber.init();
    }

    // Bridge log crate to tracing
    tracing_log::LogTracer::init().ok();
}

fn main() {
    // Initial setup without file logging
    setup_logging(None);

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = EditorApp::new();
    event_loop.run_app(&mut app).unwrap();
}
```

**Step 3: Run `cargo check`**

Run: `cargo check -p editor`
Expected: Compilation succeeds

**Step 4: Commit**

```bash
git add editor/Cargo.toml editor/src/main.rs
git commit -m "feat(editor): add tracing-based file logging"
```

---

## Task 8: Test End-to-End

Verify the complete logging system works.

**Step 1: Run the editor**

Run: `cargo run -p editor`
Expected: Editor opens without errors

**Step 2: Click Console button in toolbar**

Expected: Console panel appears at bottom of window

**Step 3: Click Play to run scripts**

Expected: Any `console.log()` calls from scripts appear in the console panel

**Step 4: Verify log file created**

Check `test_project/logs/` directory for session log file.

**Step 5: Click Clear button**

Expected: Console entries are cleared

**Step 6: Commit any fixes**

```bash
git add -A
git commit -m "fix(editor): address integration issues"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | ScriptConsole buffer | `console.rs` |
| 2 | ConsolePanel UI | `panels/console.rs`, `panels/mod.rs` |
| 3 | Export console module | `lib.rs` |
| 4 | Integrate into Editor | `editor.rs`, `toolbar.rs` |
| 5 | Console callback for scripts | `ops.rs`, scripting `lib.rs` |
| 6 | Wire editor to callback | `editor.rs`, editor `Cargo.toml` |
| 7 | File logging with tracing | `main.rs`, editor `Cargo.toml` |
| 8 | End-to-end test | - |
