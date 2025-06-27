//! Test utilities for scripting tests

use engine_component_traits::Component;

/// Simple transform component for testing
#[derive(Debug, Clone, PartialEq)]
pub struct TestTransform {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

impl Default for TestTransform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

impl Component for TestTransform {}

/// Simple test component
#[derive(Debug, Clone, PartialEq)]
pub struct TestComponent {
    pub value: i32,
}

impl Component for TestComponent {}