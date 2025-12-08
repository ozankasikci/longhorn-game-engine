//! 2D Transform Gizmo System
//!
//! Provides interactive gizmos for transforming entities in the editor viewport.

mod interaction;
mod renderer;
mod types;

pub use renderer::{draw_gizmo, GizmoConfig};
pub use interaction::{hit_test_gizmo, update_transform_from_drag};
pub use types::{GizmoHandle, GizmoMode, GizmoState};
