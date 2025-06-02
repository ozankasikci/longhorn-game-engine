//! UI components for the mobile game engine
//! 
//! This crate provides UI-specific components for canvas rendering and layout.

use serde::{Serialize, Deserialize};
use engine_ecs_core::{Component, ComponentV2};

// Canvas Component for UI rendering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Canvas {
    pub render_mode: CanvasRenderMode,  // How the canvas is rendered
    pub sorting_layer: i32,             // Global sorting layer
    pub order_in_layer: i32,            // Order within the sorting layer
    pub pixel_perfect: bool,            // Snap to pixel boundaries
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CanvasRenderMode {
    WorldSpace,                         // Rendered in 3D world space
    ScreenSpaceOverlay,                 // Rendered as overlay on top of everything
    ScreenSpaceCamera,                  // Rendered relative to a specific camera
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            render_mode: CanvasRenderMode::WorldSpace,
            sorting_layer: 0,
            order_in_layer: 0,
            pixel_perfect: true,
        }
    }
}

// Component trait implementations
impl Component for Canvas {}
impl ComponentV2 for Canvas {}

// Name component - for identifying objects (shared utility component)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Name {
    pub name: String,
}

// Component trait implementations
impl Component for Name {}
impl ComponentV2 for Name {}

impl Name {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_canvas_default() {
        let canvas = Canvas::default();
        assert_eq!(canvas.render_mode, CanvasRenderMode::WorldSpace);
        assert_eq!(canvas.sorting_layer, 0);
        assert_eq!(canvas.order_in_layer, 0);
        assert!(canvas.pixel_perfect);
    }
    
    #[test]
    fn test_name_creation() {
        let name = Name::new("Test Object");
        assert_eq!(name.name, "Test Object");
        
        let name2 = Name::new(String::from("Another Object"));
        assert_eq!(name2.name, "Another Object");
    }
}