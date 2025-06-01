# Mobile Game Engine

A modular, mobile-first game engine written in Rust.

## 🏗️ Architecture

This engine is built with a clean, modular architecture using Rust workspaces:

### Core Systems
- **`engine-core`** - Core data structures, ECS, math utilities, and time management
- **`engine-graphics`** - Rendering system using wgpu for cross-platform graphics
- **`engine-audio`** - Audio system with mixing, effects, and streaming
- **`engine-physics`** - 2D/3D physics simulation using Rapier
- **`engine-input`** - Cross-platform input handling (keyboard, mouse, touch, gamepad)
- **`engine-assets`** - Asset loading, caching, and management
- **`engine-scripting`** - Scripting system for runtime behavior
- **`engine-ui`** - Immediate-mode UI system for in-game interfaces
- **`engine-platform`** - Platform abstraction layer for mobile/desktop/web
- **`engine-runtime`** - Main runtime that orchestrates all systems

### Tools
- **`engine-editor`** - Unity-style game editor built with GTK4

## 🚀 Quick Start

### Running the Editor
```bash
cargo run --bin unity-editor
```

### Building All Crates
```bash
cargo build --workspace
```

### Running Tests
```bash
cargo test --workspace
```

## 📱 Platform Support

- **Desktop**: Windows, macOS, Linux
- **Mobile**: iOS, Android (planned)
- **Web**: WebAssembly (planned)

## 🎯 Features

- **Unity-Style Editor**: Professional game editor with panels for Hierarchy, Inspector, Scene View, Console, etc.
- **Cross-Platform Rendering**: Modern graphics using wgpu
- **Entity Component System**: High-performance ECS architecture
- **Physics Simulation**: 2D and 3D physics with Rapier
- **Asset Pipeline**: Efficient asset loading and management
- **Mobile Optimized**: Designed specifically for mobile game development

## 📁 Project Structure

```
mobile-game-engine/
├── crates/                    # Engine crates
│   ├── engine-core/           # Core systems
│   ├── engine-graphics/       # Rendering
│   ├── engine-audio/          # Audio
│   ├── engine-physics/        # Physics
│   ├── engine-input/          # Input handling
│   ├── engine-assets/         # Asset management
│   ├── engine-scripting/      # Scripting
│   ├── engine-ui/             # UI system
│   ├── engine-platform/       # Platform abstraction
│   ├── engine-editor/         # Game editor
│   └── engine-runtime/        # Main runtime
│
├── examples/                  # Example projects
├── docs/                      # Documentation
├── tools/                     # Development tools
└── experiments/               # Experimental code
```

## 🛠️ Development

### Prerequisites
- Rust 1.70+
- Platform-specific dependencies:
  - **Linux**: `sudo apt install libgtk-4-dev libadwaita-1-dev`
  - **macOS**: `brew install gtk4 libadwaita`
  - **Windows**: Use vcpkg or MSYS2

### Editor Features
- **Hierarchy Panel**: Scene object management
- **Inspector Panel**: Component editing with Transform, Mesh Renderer, Colliders
- **Scene View**: Interactive 3D scene editing
- **Game View**: Play mode preview
- **Console**: Logging and debugging
- **Project Browser**: Asset management
- **Animation Panel**: Timeline and keyframe editing
- **Audio Mixer**: Sound mixing controls

## 📄 License

MIT OR Apache-2.0