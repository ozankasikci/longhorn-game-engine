# Longhorn Engine Examples

This directory contains example game projects demonstrating various features of the Longhorn game engine.

## Examples

### hello-world

A minimal Longhorn game project demonstrating the basic structure.

**Features:**
- Minimal `game.json` configuration
- Simple entry point (`src/main.ts`)
- Empty assets folder structure

**Purpose:** Quick reference for the absolute minimum needed to create a Longhorn game.

**Run:**
```bash
cargo run --bin longhorn-editor
# Then File → Open Game → select examples/hello-world
```

### test-project

A full-featured example project with multiple entities, scripts, and assets.

**Features:**
- Complete game configuration
- TypeScript script (`PlayerController.ts`)
- Sprite assets
- Entity hierarchy (Player, Enemy)
- Asset registry

**Purpose:**
- Comprehensive example of engine features
- Used for editor development and testing
- Auto-loaded when running the editor

**Run:**
```bash
cargo run --bin longhorn-editor
# Automatically loads on startup
```

## Creating Your Own Game

Use `hello-world` as a starting template:

```bash
cp -r examples/hello-world my-game
cd my-game
# Edit game.json with your game's name
# Add your TypeScript code to src/
```

Then open it in the editor:
```bash
cargo run --bin longhorn-editor
# File → Open Game → select my-game
```
