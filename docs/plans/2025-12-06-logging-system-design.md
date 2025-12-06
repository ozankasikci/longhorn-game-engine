# Logging System Design

## Overview

Two-part logging system for the Longhorn editor:

1. **Console Panel** — Collapsible bottom drawer showing script output only (`console.log/warn/error`)
2. **File Logger** — Session-based log file capturing everything (engine, editor, scripts)

## Script API

JS-style console object available in Rhai scripts:

```javascript
console.log("Player spawned at", x, y);
console.warn("Health low:", health);
console.error("Failed to load asset:", path);
```

## Architecture

```
Scripts ──console.log()──▶ ScriptConsole (buffer) ──▶ ConsolePanel (UI)
                              │
                              ▼
                          File Logger (via tracing)

Engine/Editor ──log::info!()──▶ File Logger only
```

### ScriptConsole

Shared buffer between script runtime and UI:

```rust
pub struct ConsoleEntry {
    pub level: ConsoleLevel,  // Log, Warn, Error
    pub message: String,
    pub timestamp: Instant,
}

pub enum ConsoleLevel {
    Log,
    Warn,
    Error,
}
```

- Thread-safe: `Arc<Mutex<Vec<ConsoleEntry>>>`
- Max 1000 entries (oldest dropped when full)
- Registered as `console` global in Rhai runtime

## File Logger

### Location

`<project>/logs/editor-YYYY-MM-DD-HHMMSS.log`

New file per editor session. Created when project opens.

### Format

```
[2025-12-06 14:32:15.123] [INFO] [longhorn_editor::toolbar] Play mode entered
[2025-12-06 14:32:15.456] [DEBUG] [longhorn_rimecraft::engine] Frame tick: 16ms
[2025-12-06 14:32:16.789] [LOG] [script] Player spawned at 100, 200
[2025-12-06 14:32:17.012] [WARN] [script] Health low: 10
```

### Implementation

Replace `env_logger` with `tracing` + `tracing-subscriber`:
- File appender layer for log file
- Stderr layer for development
- `tracing-log` compatibility for existing `log::info!()` calls

## Console Panel UI

### Layout

- Bottom drawer, collapsed by default
- ~150-200px height when open, resizable
- Toolbar button to toggle visibility

### Elements

```
┌─────────────────────────────────────────────────────────┐
│ Console                                        [Clear]  │
├─────────────────────────────────────────────────────────┤
│ Player spawned at 100, 200                              │
│ ⚠ Health low: 10                                        │
│ ✖ Failed to load asset: missing.png                     │
└─────────────────────────────────────────────────────────┘
```

### Behavior

- Colors: log=default, warn=yellow, error=red
- Auto-scroll to bottom (unless user scrolled up)
- Clear button empties buffer

## Implementation Plan

### New Files

1. `crates/longhorn-editor/src/panels/console.rs` — ConsolePanel UI
2. `crates/longhorn-editor/src/console.rs` — ScriptConsole buffer + Rhai bindings
3. `crates/longhorn-editor/src/logging.rs` — File logger setup

### Modified Files

1. `crates/longhorn-editor/src/editor.rs` — Add panel, toggle state, toolbar button
2. `crates/longhorn-editor/src/lib.rs` — Export new modules
3. `crates/longhorn-rimecraft/src/script_runtime.rs` — Register console global
4. `editor/src/main.rs` — Replace env_logger with tracing
5. `Cargo.toml` — Add tracing dependencies

### Dependencies

```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"
tracing-log = "0.2"
```
