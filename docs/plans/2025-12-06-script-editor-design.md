# Script Editor Design

A dockable Script Editor panel for editing TypeScript files with syntax highlighting and parse error indicators.

## Overview

```
┌─────────────────────────────────────────────┐
│ Inspector Panel                             │
│ ┌─────────────────────────────────────────┐ │
│ │ Script Component                    [⋮] │ │  ← Kebab menu with "Edit"
│ │   path: PlayerController.ts             │ │
│ └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
                    │
                    ▼ (click Edit)
┌─────────────────────────────────────────────┐
│ Script Editor Panel                         │
│ ┌─────────────────────────────────────────┐ │
│ │ PlayerController.ts ●                   │ │  ← Tab with unsaved indicator
│ ├─────────────────────────────────────────┤ │
│ │ 1  │ export default class PlayerCon...  │ │
│ │ 2  │   speed = 100;                     │ │
│ │ 3 ●│   onStart(self: any) {             │ │  ← Red dot = error
│ │ 4  │   }                                │ │
│ └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

## Key Decisions

- **Dockable panel** alongside Hierarchy, Inspector, Viewport, Console
- **Kebab menu (⋮)** on each component in Inspector; Script component has "Edit" action
- **egui_code_editor crate** for text editing, line numbers, and syntax highlighting
- **Manual save** (Cmd/Ctrl+S) with unsaved dot indicator in tab
- **Gutter markers** for parse errors (red dot), hover shows error message
- **Reuse existing TypeScript compiler** for parse error detection

## Components

### ScriptEditorState

```rust
pub struct ScriptEditorState {
    /// Currently open script path
    pub open_file: Option<PathBuf>,
    /// Current editor content
    pub content: String,
    /// Content at last save (for dirty detection)
    original_content: String,
    /// Parse errors with line numbers
    pub errors: Vec<ScriptError>,
}

pub struct ScriptError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl ScriptEditorState {
    pub fn is_dirty(&self) -> bool {
        self.content != self.original_content
    }

    pub fn open(&mut self, path: PathBuf) -> io::Result<()>;
    pub fn save(&mut self) -> io::Result<()>;
    pub fn reparse(&mut self);  // Updates self.errors
}
```

### Kebab Menu

Generic component for Inspector:

```rust
pub struct KebabMenu<'a> {
    actions: &'a [(&'a str, EditorAction)],
}
```

- Renders ⋮ button in component header
- Shows dropdown with actions on click
- Script component actions: `[("Edit", EditorAction::OpenScriptEditor(path))]`
- Extensible for other components (Reset, Remove, etc.)

### Script Editor Panel

```rust
pub fn script_editor_panel(ui: &mut egui::Ui, state: &mut ScriptEditorState) {
    // Header with filename and dirty indicator
    // Code editor with syntax highlighting
    // Custom gutter rendering for error markers
    // Hover tooltips for errors
}
```

## Error Detection

Extend existing `TypeScriptCompiler` to return parse errors:

```rust
pub struct CompileResult {
    pub script: Option<CompiledScript>,
    pub errors: Vec<ScriptError>,
}

impl TypeScriptCompiler {
    pub fn compile_with_diagnostics(&self, source: &str) -> CompileResult;
}
```

- Reparse on content change (debounced ~300ms)
- Collect syntax errors with line/column info
- Display in gutter with hover tooltips

## File Changes

### New Files

| File | Purpose |
|------|---------|
| `src/panels/script_editor.rs` | Script Editor panel UI |
| `src/script_editor_state.rs` | Editor state management |
| `src/components/kebab_menu.rs` | Reusable kebab menu component |

### Modified Files

| File | Changes |
|------|---------|
| `Cargo.toml` | Add `egui_code_editor` dependency |
| `src/lib.rs` | Export new modules |
| `src/editor.rs` | Add `ScriptEditorState`, handle `OpenScriptEditor` action |
| `src/docking.rs` | Register Script Editor as dockable panel |
| `src/panels/inspector.rs` | Add kebab menus to component headers |
| `src/panels/mod.rs` | Export script_editor module |
| `longhorn-scripting/src/compiler.rs` | Add `compile_with_diagnostics()` |

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Cmd/Ctrl+S | Save current file (when panel focused) |

## Syntax Highlighting

`egui_code_editor` provides:
- TypeScript/JavaScript token highlighting
- Line numbers
- Text selection, copy/paste, undo/redo

Customization:
- Match v1 dark theme colors
- Custom gutter rendering for error markers
