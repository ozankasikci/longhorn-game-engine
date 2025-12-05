use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Touch event types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TouchEvent {
    Start { x: f32, y: f32 },
    Move { x: f32, y: f32 },
    End { x: f32, y: f32 },
}

impl TouchEvent {
    pub fn position(&self) -> Vec2 {
        match self {
            TouchEvent::Start { x, y } => Vec2::new(*x, *y),
            TouchEvent::Move { x, y } => Vec2::new(*x, *y),
            TouchEvent::End { x, y } => Vec2::new(*x, *y),
        }
    }

    pub fn is_start(&self) -> bool {
        matches!(self, TouchEvent::Start { .. })
    }

    pub fn is_move(&self) -> bool {
        matches!(self, TouchEvent::Move { .. })
    }

    pub fn is_end(&self) -> bool {
        matches!(self, TouchEvent::End { .. })
    }
}
