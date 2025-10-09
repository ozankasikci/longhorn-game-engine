# Engine Runtime Core

Core runtime components for the Longhorn Game Engine, providing the foundation for standalone game execution with proper game loop architecture.

## Features

- Fixed timestep game loop with accumulator pattern
- Variable rendering with interpolation support
- Platform-independent windowing via winit
- Death spiral prevention for performance stability
- Application trait for game implementation

## Architecture

This crate implements the core game loop following industry best practices:

1. **Fixed Timestep Updates**: Game logic runs at consistent intervals (default 60Hz)
2. **Variable Rendering**: Graphics render as fast as possible with interpolation
3. **Accumulator Pattern**: Handles varying frame times gracefully
4. **Event-Driven**: Processes input and system events efficiently

## Usage

```rust
use engine_runtime_core::{GameLoop, Application};

struct MyGame;

impl Application for MyGame {
    fn update(&mut self, delta_time: Duration) {
        // Game logic here
    }
    
    fn render(&mut self, interpolation: f32) {
        // Rendering here
    }
}

let mut game_loop = GameLoop::new();
game_loop.run(MyGame)?;
```